// Parakeet v3 transcription engine via ONNX Runtime with CoreML EP.
//
// Pipeline: audio -> nemo128 preprocessor -> FastConformer encoder -> TDT decoder -> text
//
// Model files are downloaded on first use and cached in the app data dir.
// Three ONNX models are needed:
//   1. nemo128.onnx - NeMo mel spectrogram preprocessor (audio -> features)
//   2. encoder.onnx - FastConformer acoustic encoder
//   3. decoder.onnx - TDT (Token-and-Duration Transducer) decoder

use anyhow::{anyhow, Context};
use ndarray::Array2;
use ort::session::Session;
use ort::value::{Tensor, ValueType};
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use std::f32::consts::PI;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

const SAMPLE_RATE: u32 = 16000;
const MODEL_DIR_NAME: &str = "parakeet-tdt-0.6b-v3";

// Mel spectrogram parameters matching NeMo's AudioToMelSpectrogramPreprocessor
const N_FFT: usize = 512;
const HOP_LENGTH: usize = 160; // 10ms at 16kHz
const WIN_LENGTH: usize = 400; // 25ms at 16kHz
const N_MELS: usize = 128; // Parakeet 0.6B uses 128 mel bins
const F_MIN: f32 = 0.0;
const F_MAX: f32 = 8000.0; // Nyquist for 16kHz
const PREEMPHASIS: f32 = 0.97;
const LOG_ZERO_GUARD: f32 = 1e-10;
const MAX_TOKENS_PER_STEP: usize = 10;

/// A single transcription segment with timing.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TranscriptionSegment {
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub speaker_id: Option<String>,
}

/// Status of model download/preparation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ModelStatus {
    NotDownloaded,
    Downloading { progress: f32 },
    Ready,
    Error(String),
}

/// Transcription engine using Parakeet v3 ONNX models with CoreML acceleration.
pub struct TranscriptionEngine {
    /// NeMo mel spectrogram preprocessor (nemo128.onnx). When present, used
    /// instead of the Rust mel computation to guarantee feature compatibility.
    preprocessor: Option<Session>,
    encoder: Session,
    decoder: Session,
    #[allow(dead_code)]
    model_dir: PathBuf,
    vocab: Vec<String>,
    mel_filterbank: Array2<f32>,
    blank_idx: usize,
    vocab_size: usize,
    /// LSTM state shape [layers, 1, hidden] for decoder input_states_1
    state1_dims: [usize; 3],
    /// LSTM state shape [layers, 1, hidden] for decoder input_states_2
    state2_dims: [usize; 3],
}

/// Build an ONNX session, preferring CoreML but degrading to CPU.
///
/// CoreML cannot resolve a model whose weights live in an external data file
/// (`model.onnx` plus `model.onnx.data`); it fails with
/// `initializer.cc:45 !model_path.empty() was false`. The Parakeet encoder is
/// exactly that shape -- 41MB of graph plus 2.4GB of external weights -- so
/// hard-requiring CoreML made the entire transcription engine fail to load,
/// and recordings completed with no transcript at all.
///
/// Accelerate when we can, stay working when we cannot.
fn build_session(path: &Path, label: &str) -> anyhow::Result<Session> {
    let coreml = Session::builder()
        .and_then(|b| {
            b.with_execution_providers([
                ort::execution_providers::CoreMLExecutionProvider::default().build()
            ])
        })
        .and_then(|b| b.commit_from_file(path));

    match coreml {
        Ok(session) => {
            tracing::info!("{label}: loaded with CoreML acceleration");
            Ok(session)
        }
        Err(e) => {
            tracing::warn!("{label}: CoreML unavailable for this model ({e}); using CPU");
            let session = Session::builder()?
                .commit_from_file(path)
                .with_context(|| format!("Failed to load {label} model on CPU"))?;
            tracing::info!("{label}: loaded on CPU");
            Ok(session)
        }
    }
}

impl TranscriptionEngine {
    /// Get the directory where models are stored.
    pub fn model_dir() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Nootle")
            .join("models")
            .join(MODEL_DIR_NAME)
    }

    /// Check if models are downloaded and ready in a given directory.
    fn check_status_in(dir: &Path) -> ModelStatus {
        let encoder_path = dir.join("encoder.onnx");
        let decoder_path = dir.join("decoder.onnx");
        let vocab_path = dir.join("vocab.txt");

        if encoder_path.exists() && decoder_path.exists() && vocab_path.exists() {
            ModelStatus::Ready
        } else {
            ModelStatus::NotDownloaded
        }
    }

    /// Check if models are downloaded and ready.
    pub fn check_status() -> ModelStatus {
        Self::check_status_in(&Self::model_dir())
    }

    /// Load the transcription engine from pre-downloaded model files.
    pub fn load() -> anyhow::Result<Self> {
        let model_dir = Self::model_dir();

        if !model_dir.exists() {
            return Err(anyhow!(
                "Models not found. Please download the Parakeet TDT model first."
            ));
        }

        let preprocessor_path = model_dir.join("nemo128.onnx");
        let encoder_path = model_dir.join("encoder.onnx");
        let decoder_path = model_dir.join("decoder.onnx");
        let vocab_path = model_dir.join("vocab.txt");

        // Load ONNX preprocessor (nemo128.onnx) — CPU is fine for this tiny model
        let preprocessor = if preprocessor_path.exists() {
            match Session::builder()?.commit_from_file(&preprocessor_path) {
                Ok(session) => {
                    tracing::info!("Loaded nemo128 preprocessor (ONNX mel spectrogram)");
                    Some(session)
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to load nemo128 preprocessor, falling back to Rust mel: {e}"
                    );
                    None
                }
            }
        } else {
            tracing::info!(
                "nemo128.onnx not found, using Rust mel spectrogram (may reduce accuracy)"
            );
            None
        };

        // Load ONNX sessions with CoreML EP for Apple Silicon acceleration
        let encoder = build_session(&encoder_path, "encoder")?;
        let decoder = build_session(&decoder_path, "decoder")?;

        // Load vocabulary (format: "token id" per line — extract just the token)
        let vocab_text =
            std::fs::read_to_string(&vocab_path).context("Failed to load vocabulary file")?;
        let vocab: Vec<String> = vocab_text
            .lines()
            .map(|line| match line.rsplit_once(' ') {
                Some((token, _id)) => token.to_string(),
                None => line.to_string(),
            })
            .collect();

        // Find the blank token index ("<blk>" in Parakeet TDT vocab)
        let blank_idx = vocab
            .iter()
            .position(|t| t == "<blk>")
            .ok_or_else(|| anyhow!("Vocab does not contain <blk> blank token"))?;
        let vocab_size = vocab.len();

        // Extract LSTM state dimensions from decoder input metadata
        let state1_dims = extract_state_dims(&decoder, "input_states_1")?;
        let state2_dims = extract_state_dims(&decoder, "input_states_2")?;

        let mel_filterbank = create_mel_filterbank(N_FFT, N_MELS, SAMPLE_RATE, F_MIN, F_MAX);

        // Log model input/output names for debugging
        let encoder_inputs: Vec<_> = encoder
            .inputs()
            .iter()
            .map(|i| i.name().to_string())
            .collect();
        let encoder_outputs: Vec<_> = encoder
            .outputs()
            .iter()
            .map(|o| o.name().to_string())
            .collect();
        let decoder_inputs: Vec<_> = decoder
            .inputs()
            .iter()
            .map(|i| i.name().to_string())
            .collect();
        let decoder_outputs: Vec<_> = decoder
            .outputs()
            .iter()
            .map(|o| o.name().to_string())
            .collect();
        tracing::info!(
            vocab_size,
            blank_idx,
            ?state1_dims,
            ?state2_dims,
            ?encoder_inputs,
            ?encoder_outputs,
            ?decoder_inputs,
            ?decoder_outputs,
            "Transcription engine loaded with CoreML acceleration"
        );

        Ok(Self {
            preprocessor,
            encoder,
            decoder,
            model_dir,
            vocab,
            mel_filterbank,
            blank_idx,
            vocab_size,
            state1_dims,
            state2_dims,
        })
    }

    /// Compute mel spectrogram from raw audio samples (16kHz mono f32).
    ///
    /// Pipeline: preemphasis -> frame + Hann window -> FFT -> power spectrum
    ///           -> mel filterbank -> log -> per-feature normalization
    fn compute_mel_spectrogram(&self, audio: &[f32]) -> Array2<f32> {
        // 1. Preemphasis: y[n] = x[n] - 0.97 * x[n-1]
        let audio = apply_preemphasis(audio, PREEMPHASIS);

        let n_frames = if audio.len() >= WIN_LENGTH {
            (audio.len() - WIN_LENGTH) / HOP_LENGTH + 1
        } else {
            return Array2::zeros((N_MELS, 1));
        };

        // Precompute Hann window
        let hann: Vec<f32> = (0..WIN_LENGTH)
            .map(|i| 0.5 * (1.0 - (2.0 * PI * i as f32 / WIN_LENGTH as f32).cos()))
            .collect();

        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(N_FFT);

        // Number of unique FFT bins (real signal symmetry)
        let n_freq = N_FFT / 2 + 1;
        let mut mel = Array2::zeros((N_MELS, n_frames));

        for frame_idx in 0..n_frames {
            let start = frame_idx * HOP_LENGTH;

            // 2. Window the frame and zero-pad to N_FFT
            let mut fft_buf: Vec<Complex<f32>> = vec![Complex::new(0.0, 0.0); N_FFT];
            for i in 0..WIN_LENGTH {
                if start + i < audio.len() {
                    fft_buf[i] = Complex::new(audio[start + i] * hann[i], 0.0);
                }
            }

            // 3. FFT
            fft.process(&mut fft_buf);

            // 4. Power spectrum: |FFT[k]|^2 / N_FFT
            let power: Vec<f32> = fft_buf[..n_freq]
                .iter()
                .map(|c| (c.re * c.re + c.im * c.im) / N_FFT as f32)
                .collect();

            // 5. Apply mel filterbank and log
            for mel_idx in 0..N_MELS {
                let mut energy: f32 = 0.0;
                for (k, &p) in power.iter().enumerate().take(n_freq) {
                    energy += self.mel_filterbank[[mel_idx, k]] * p;
                }
                mel[[mel_idx, frame_idx]] = (energy + LOG_ZERO_GUARD).ln();
            }
        }

        // 6. Per-feature normalization (mean=0, std=1 per mel bin)
        for mel_idx in 0..N_MELS {
            let mut sum = 0.0f64;
            let mut sum_sq = 0.0f64;
            for frame_idx in 0..n_frames {
                let v = mel[[mel_idx, frame_idx]] as f64;
                sum += v;
                sum_sq += v * v;
            }
            let mean = sum / n_frames as f64;
            let variance = (sum_sq / n_frames as f64) - mean * mean;
            let std = variance.max(1e-10).sqrt();

            for frame_idx in 0..n_frames {
                mel[[mel_idx, frame_idx]] =
                    ((mel[[mel_idx, frame_idx]] as f64 - mean) / std) as f32;
            }
        }

        mel
    }

    /// Run the ONNX nemo128 preprocessor: raw audio [1, samples] → mel [1, 128, T].
    /// Returns (mel_data, n_mels, n_frames).
    fn preprocess_audio(
        preprocessor: &mut Session,
        audio: &[f32],
    ) -> anyhow::Result<(Vec<f32>, usize, usize)> {
        let n_samples = audio.len();
        let audio_tensor = Tensor::from_array(([1_usize, n_samples], audio.to_vec()))?;
        let length_tensor = Tensor::from_array(([1_usize], vec![n_samples as i64]))?;

        let prep_inputs: Vec<_> = preprocessor
            .inputs()
            .iter()
            .map(|i| i.name().to_string())
            .collect();

        let outputs = preprocessor.run(ort::inputs![
            prep_inputs[0].as_str() => audio_tensor,
            prep_inputs[1].as_str() => length_tensor,
        ])?;

        let (feat_shape, feat_data) = outputs[0]
            .try_extract_tensor::<f32>()
            .context("Failed to extract preprocessor features")?;
        let feat_dims: Vec<usize> = feat_shape.iter().map(|&d| d as usize).collect();
        let n_mels = feat_dims[1];
        let n_frames = feat_dims[2];

        Ok((feat_data.to_vec(), n_mels, n_frames))
    }

    /// Transcribe a chunk of audio (16kHz mono f32 samples).
    ///
    /// Returns transcription segments with timing information.
    pub fn transcribe(
        &mut self,
        audio: &[f32],
        offset_ms: u64,
    ) -> anyhow::Result<Vec<TranscriptionSegment>> {
        if audio.is_empty() {
            return Ok(vec![]);
        }

        // 1. Compute mel spectrogram — prefer ONNX preprocessor for accuracy
        let (mel_data, n_mels, n_frames) = if let Some(ref mut preprocessor) = self.preprocessor {
            Self::preprocess_audio(preprocessor, audio)?
        } else {
            let mel = self.compute_mel_spectrogram(audio);
            let (nm, nf) = mel.dim();
            (mel.into_raw_vec_and_offset().0, nm, nf)
        };
        tracing::info!(
            "[DIAG] Mel: n_mels={n_mels}, n_frames={n_frames}, audio_len={}",
            audio.len()
        );

        // 2. Run encoder: mel [1, n_mels, n_frames] -> encoded [1, D, T]
        let mel_tensor = Tensor::from_array(([1, n_mels, n_frames], mel_data))?;
        let length_tensor = Tensor::from_array(([1_usize], vec![n_frames as i64]))?;

        let encoder_outputs = self.encoder.run(ort::inputs![
            "audio_signal" => mel_tensor,
            "length" => length_tensor,
        ])?;

        // 3. Extract encoder output — shape is [1, D, T] (features first, time second)
        //    The ONNX FastConformer encoder outputs (batch, features=1024, time).
        let (enc_shape, enc_data) = encoder_outputs[0]
            .try_extract_tensor::<f32>()
            .context("Failed to extract encoder output")?;
        let enc_dims: Vec<usize> = enc_shape.iter().map(|&d| d as usize).collect();
        let d_model = enc_dims[1]; // feature dimension (1024)
        let t_enc = enc_dims[2]; // time frames
        tracing::info!(
            "[DIAG] Encoder output: enc_dims={enc_dims:?}, d_model={d_model}, t_enc={t_enc}"
        );

        // Transpose from [D, T] to [T, D] so frame t is a contiguous slice at t * d_model.
        // Source layout (row-major, ignoring batch): element [d, t] = data[d * t_enc + t]
        let enc_src: Vec<f32> = enc_data.to_vec();
        let mut enc_flat = vec![0.0f32; d_model * t_enc];
        for d in 0..d_model {
            for t in 0..t_enc {
                enc_flat[t * d_model + d] = enc_src[d * t_enc + t];
            }
        }

        // Get encoded length (clamp to actual tensor size)
        let (_, enc_len_data) = encoder_outputs[1]
            .try_extract_tensor::<i64>()
            .context("Failed to extract encoded lengths")?;
        let encoded_len = (enc_len_data[0] as usize).min(t_enc);
        tracing::info!(
            "[DIAG] Encoded length={encoded_len}, state1_dims={:?}, state2_dims={:?}",
            self.state1_dims,
            self.state2_dims
        );

        // 4. TDT greedy search: call decoder step-by-step over encoder frames
        let mut state1 =
            vec![0.0f32; self.state1_dims[0] * self.state1_dims[1] * self.state1_dims[2]];
        let mut state2 =
            vec![0.0f32; self.state2_dims[0] * self.state2_dims[1] * self.state2_dims[2]];

        let mut tokens: Vec<usize> = Vec::new();
        let mut t: usize = 0;
        let mut emitted_tokens: usize = 0;

        while t < encoded_len {
            // Encoder frame t → shape [1, D, 1] (features-first for decoder)
            let frame_start = t * d_model;
            let frame_data: Vec<f32> = enc_flat[frame_start..frame_start + d_model].to_vec();
            let encoder_frame = Tensor::from_array(([1, d_model, 1], frame_data))?;

            // Previous token (or blank if nothing emitted yet)
            let prev_token = tokens.last().copied().unwrap_or(self.blank_idx) as i32;
            let targets = Tensor::from_array(([1_usize, 1], vec![prev_token]))?;
            let target_length = Tensor::from_array(([1_usize], vec![1_i32]))?;

            let state1_tensor = Tensor::from_array((self.state1_dims, state1.clone()))?;
            let state2_tensor = Tensor::from_array((self.state2_dims, state2.clone()))?;

            let decoder_out = self.decoder.run(ort::inputs![
                "encoder_outputs" => encoder_frame,
                "targets" => targets,
                "target_length" => target_length,
                "input_states_1" => state1_tensor,
                "input_states_2" => state2_tensor,
            ])?;

            // Output 0 = logits (float), outputs 1 & 2 = new LSTM states
            let (logit_shape, output_data) = decoder_out[0]
                .try_extract_tensor::<f32>()
                .context("Failed to extract decoder logits")?;
            let logits: Vec<f32> = output_data.to_vec();

            if t == 0 {
                let logit_dims: Vec<usize> = logit_shape.iter().map(|&d| d as usize).collect();
                tracing::info!(
                    "[DIAG] Decoder output shape: {logit_dims:?}, total logits={}, vocab_size={}",
                    logits.len(),
                    self.vocab_size
                );
            }

            // TDT: split into vocab logits and duration logits
            if logits.len() < self.vocab_size {
                tracing::error!(
                    "[DIAG] Decoder output too small: {} < vocab_size {}",
                    logits.len(),
                    self.vocab_size
                );
                return Err(anyhow!(
                    "Decoder output size {} smaller than vocab size {}",
                    logits.len(),
                    self.vocab_size
                ));
            }
            let vocab_logits = &logits[..self.vocab_size];
            let duration_logits = &logits[self.vocab_size..];

            let token = vocab_logits
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(idx, _)| idx)
                .unwrap_or(self.blank_idx);

            let step: usize = if duration_logits.is_empty() {
                0
            } else {
                duration_logits
                    .iter()
                    .enumerate()
                    .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(idx, _)| idx)
                    .unwrap_or(0)
            };

            if token != self.blank_idx {
                // Update LSTM states only on non-blank emission
                // Decoder outputs: [0]=logits, [1]=prednet_lengths, [2]=output_states_1, [3]=output_states_2
                let (_, s1) = decoder_out[2]
                    .try_extract_tensor::<f32>()
                    .context("Failed to extract output_states_1")?;
                state1 = s1.to_vec();
                let (_, s2) = decoder_out[3]
                    .try_extract_tensor::<f32>()
                    .context("Failed to extract output_states_2")?;
                state2 = s2.to_vec();

                tokens.push(token);
                emitted_tokens += 1;
            }

            if step > 0 {
                t += step;
                emitted_tokens = 0;
            } else if token == self.blank_idx || emitted_tokens >= MAX_TOKENS_PER_STEP {
                t += 1;
                emitted_tokens = 0;
            }
        }

        // 5. Convert token IDs to text
        tracing::info!(
            "[DIAG] TDT decode: {}/{encoded_len} frames processed, {} tokens emitted, blank_idx={}",
            t,
            tokens.len(),
            self.blank_idx
        );
        if !tokens.is_empty() {
            let sample: Vec<_> = tokens
                .iter()
                .take(10)
                .map(|&id| {
                    if id < self.vocab.len() {
                        self.vocab[id].clone()
                    } else {
                        format!("?{id}")
                    }
                })
                .collect();
            tracing::info!("[DIAG] First tokens: {sample:?}");
        }
        let mut text = String::new();
        for &id in &tokens {
            if id < self.vocab.len() {
                let token = &self.vocab[id];
                if token == "<unk>"
                    || token == "<pad>"
                    || token == "<blk>"
                    || token == "<s>"
                    || token == "</s>"
                    || token.starts_with("<|")
                {
                    continue;
                }
                // Handle SentencePiece underscore prefix (word boundary)
                if let Some(stripped) = token.strip_prefix('\u{2581}') {
                    if !text.is_empty() {
                        text.push(' ');
                    }
                    text.push_str(stripped);
                } else {
                    text.push_str(token);
                }
            }
        }

        let text = text.trim().to_string();
        tracing::info!("[DIAG] Final text ({} chars): {:?}", text.len(), text);
        if text.is_empty() {
            return Ok(vec![]);
        }

        let duration_ms = (audio.len() as u64 * 1000) / SAMPLE_RATE as u64;

        Ok(vec![TranscriptionSegment {
            text,
            start_ms: offset_ms,
            end_ms: offset_ms + duration_ms,
            speaker_id: None,
        }])
    }
}

// ── Decoder state helpers ────────────────────────────────────────────────────

/// Extract LSTM state dimensions [layers, 1, hidden] from a decoder input by name.
fn extract_state_dims(session: &Session, name: &str) -> anyhow::Result<[usize; 3]> {
    let input = session
        .inputs()
        .iter()
        .find(|i| i.name() == name)
        .ok_or_else(|| anyhow!("Decoder missing input '{name}'"))?;

    match input.dtype() {
        ValueType::Tensor { shape, .. } => {
            if shape.len() != 3 {
                return Err(anyhow!(
                    "Expected 3-D shape for '{name}', got {}-D",
                    shape.len()
                ));
            }
            // Dynamic dims come as -1; use 1 as fallback but warn
            let dims = [
                shape[0].max(1) as usize,
                1, // batch dim is always 1
                shape[2].max(1) as usize,
            ];
            if shape[0] < 0 || shape[2] < 0 {
                tracing::warn!(
                    ?shape,
                    ?dims,
                    "State '{name}' has dynamic dims — using fallback. \
                     This may cause incorrect LSTM state sizes."
                );
            }
            Ok(dims)
        }
        other => Err(anyhow!("Expected Tensor type for '{name}', got {other:?}")),
    }
}

// ── Mel spectrogram helpers ──────────────────────────────────────────────────

/// Apply preemphasis filter: y[n] = x[n] - coeff * x[n-1]
fn apply_preemphasis(audio: &[f32], coeff: f32) -> Vec<f32> {
    if audio.is_empty() {
        return vec![];
    }
    let mut out = Vec::with_capacity(audio.len());
    out.push(audio[0]);
    for i in 1..audio.len() {
        out.push(audio[i] - coeff * audio[i - 1]);
    }
    out
}

/// Convert frequency in Hz to mel scale.
fn hz_to_mel(hz: f32) -> f32 {
    2595.0 * (1.0 + hz / 700.0).log10()
}

/// Convert mel scale value to Hz.
fn mel_to_hz(mel: f32) -> f32 {
    700.0 * (10.0_f32.powf(mel / 2595.0) - 1.0)
}

/// Create a mel filterbank matrix of shape [n_mels, n_fft/2 + 1].
///
/// Each row is a triangular filter centered on a mel-spaced frequency.
fn create_mel_filterbank(
    n_fft: usize,
    n_mels: usize,
    sample_rate: u32,
    f_min: f32,
    f_max: f32,
) -> Array2<f32> {
    let n_freq = n_fft / 2 + 1;
    let mel_min = hz_to_mel(f_min);
    let mel_max = hz_to_mel(f_max);

    // n_mels + 2 points to define n_mels triangular filters
    let mel_points: Vec<f32> = (0..=(n_mels + 1))
        .map(|i| mel_min + (mel_max - mel_min) * i as f32 / (n_mels + 1) as f32)
        .collect();

    // Convert mel points to FFT bin indices
    let bin_points: Vec<f32> = mel_points
        .iter()
        .map(|&m| mel_to_hz(m) * n_fft as f32 / sample_rate as f32)
        .collect();

    let mut filterbank = Array2::zeros((n_mels, n_freq));

    for m in 0..n_mels {
        let f_left = bin_points[m];
        let f_center = bin_points[m + 1];
        let f_right = bin_points[m + 2];

        for k in 0..n_freq {
            let freq = k as f32;
            if freq >= f_left && freq <= f_center && f_center > f_left {
                filterbank[[m, k]] = (freq - f_left) / (f_center - f_left);
            } else if freq > f_center && freq <= f_right && f_right > f_center {
                filterbank[[m, k]] = (f_right - freq) / (f_right - f_center);
            }
        }
    }

    filterbank
}

// ── Shared engine ────────────────────────────────────────────────────────────

/// Thread-safe handle to the transcription engine for use across async tasks.
pub type SharedTranscriptionEngine = Arc<Mutex<Option<TranscriptionEngine>>>;

/// Initialize the shared transcription engine (lazy - only loads when models exist).
pub fn create_shared_engine() -> SharedTranscriptionEngine {
    Arc::new(Mutex::new(None))
}

/// Try to load the engine if models are available.
pub async fn try_load_engine(engine: &SharedTranscriptionEngine) -> anyhow::Result<()> {
    let mut guard = engine.lock().await;
    if guard.is_some() {
        return Ok(());
    }

    match TranscriptionEngine::check_status() {
        ModelStatus::Ready => {
            let loaded = TranscriptionEngine::load()?;
            *guard = Some(loaded);
            Ok(())
        }
        ModelStatus::NotDownloaded => Err(anyhow!("Models not downloaded yet")),
        ModelStatus::Error(e) => Err(anyhow!("Model error: {}", e)),
        ModelStatus::Downloading { progress } => Err(anyhow!(
            "Models still downloading: {:.0}%",
            progress * 100.0
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_status_not_downloaded() {
        let tmp = tempfile::tempdir().unwrap();
        let status = TranscriptionEngine::check_status_in(tmp.path());
        assert!(matches!(status, ModelStatus::NotDownloaded));
    }

    #[test]
    fn test_model_dir_path() {
        let dir = TranscriptionEngine::model_dir();
        assert!(dir.to_str().unwrap().contains("Nootle"));
        assert!(dir.to_str().unwrap().contains(MODEL_DIR_NAME));
    }

    #[test]
    fn test_shared_engine_creation() {
        let engine = create_shared_engine();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let guard = engine.lock().await;
            assert!(guard.is_none());
        });
    }

    #[test]
    fn test_mel_filterbank_shape() {
        let fb = create_mel_filterbank(N_FFT, N_MELS, SAMPLE_RATE, F_MIN, F_MAX);
        assert_eq!(fb.dim(), (N_MELS, N_FFT / 2 + 1));
    }

    #[test]
    fn test_mel_filterbank_non_negative() {
        let fb = create_mel_filterbank(N_FFT, N_MELS, SAMPLE_RATE, F_MIN, F_MAX);
        for &v in fb.iter() {
            assert!(v >= 0.0, "Filterbank values must be non-negative");
        }
    }

    #[test]
    fn test_mel_filterbank_speech_filters_nonzero() {
        let fb = create_mel_filterbank(N_FFT, N_MELS, SAMPLE_RATE, F_MIN, F_MAX);
        // Skip the lowest filters (< ~50 Hz) which may be all-zero because
        // their triangles are narrower than one FFT bin. Speech energy is
        // above 80 Hz so this is fine.
        let mut nonzero_count = 0;
        for m in 0..N_MELS {
            let sum: f32 = (0..N_FFT / 2 + 1).map(|k| fb[[m, k]]).sum();
            if sum > 0.0 {
                nonzero_count += 1;
            }
        }
        // At least 120 of 128 filters should have nonzero response
        assert!(
            nonzero_count >= 120,
            "Only {nonzero_count}/128 filters have nonzero response"
        );
    }

    #[test]
    fn test_preemphasis() {
        let audio = vec![1.0, 2.0, 3.0, 4.0];
        let out = apply_preemphasis(&audio, 0.97);
        assert_eq!(out.len(), 4);
        assert!((out[0] - 1.0).abs() < 1e-6);
        assert!((out[1] - (2.0 - 0.97)).abs() < 1e-6);
        assert!((out[2] - (3.0 - 0.97 * 2.0)).abs() < 1e-6);
    }

    #[test]
    fn test_hz_mel_roundtrip() {
        for hz in [0.0, 100.0, 440.0, 1000.0, 4000.0, 8000.0] {
            let roundtrip = mel_to_hz(hz_to_mel(hz));
            assert!(
                (roundtrip - hz).abs() < 0.01,
                "Roundtrip failed for {hz}: got {roundtrip}"
            );
        }
    }

    #[test]
    fn test_preemphasis_empty() {
        let out = apply_preemphasis(&[], 0.97);
        assert!(out.is_empty());
    }
}
