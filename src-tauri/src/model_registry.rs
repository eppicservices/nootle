use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ModelCategory {
    Transcription,
    Diarization,
    Embedding,
    Denoising,
    VoiceActivity,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelFile {
    /// Name the file is saved as locally (e.g. "encoder.onnx")
    pub local_name: &'static str,
    /// HuggingFace download URL
    pub url: &'static str,
    /// Expected file size in bytes (used for progress calculation)
    pub size_bytes: u64,
    /// SHA-256 hex digest for verification (empty string to skip verification)
    pub sha256: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelVariant {
    pub id: &'static str,
    pub label: &'static str,
    pub files: &'static [ModelFile],
    pub total_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub category: ModelCategory,
    /// Subdirectory name under ~/Library/Application Support/Nootle/models/
    pub dir_name: &'static str,
    pub variants: &'static [ModelVariant],
}

// ── Parakeet TDT 0.6B v3 (Transcription) ──────────────────────────────
// SHA-256 hashes sourced from HuggingFace LFS oid metadata.

const PARAKEET_INT8_FILES: &[ModelFile] = &[
    ModelFile {
        local_name: "encoder.onnx",
        url: "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/main/encoder-model.int8.onnx",
        size_bytes: 652_000_000,
        sha256: "6139d2fa7e1b086097b277c7149725edbab89cc7c7ae64b23c741be4055aff09",
    },
    ModelFile {
        local_name: "decoder.onnx",
        url: "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/main/decoder_joint-model.int8.onnx",
        size_bytes: 18_200_000,
        sha256: "eea7483ee3d1a30375daedc8ed83e3960c91b098812127a0d99d1c8977667a70",
    },
    ModelFile {
        local_name: "vocab.txt",
        url: "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/main/vocab.txt",
        size_bytes: 94_000,
        sha256: "d58544679ea4bc6ac563d1f545eb7d474bd6cfa467f0a6e2c1dc1c7d37e3c35d",
    },
    ModelFile {
        local_name: "nemo128.onnx",
        url: "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/main/nemo128.onnx",
        size_bytes: 140_000,
        sha256: "a9fde1486ebfcc08f328d75ad4610c67835fea58c73ba57e3209a6f6cf019e9f",
    },
];

const PARAKEET_FULL_FILES: &[ModelFile] = &[
    ModelFile {
        local_name: "encoder.onnx",
        url: "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/main/encoder-model.onnx",
        size_bytes: 41_800_000,
        sha256: "98a74b21b4cc0017c1e7030319a4a96f4a9506e50f0708f3a516d02a77c96bb1",
    },
    ModelFile {
        // MUST stay "encoder-model.onnx.data". encoder.onnx keeps its weights in
        // an external data file and refers to it by name in the graph, using the
        // upstream filename. Renaming it to match our local "encoder.onnx" makes
        // ONNX Runtime unable to find the weights, and the encoder then fails to
        // load on every execution provider -- recordings complete with no
        // transcript at all.
        local_name: "encoder-model.onnx.data",
        url: "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/main/encoder-model.onnx.data",
        size_bytes: 2_440_000_000,
        sha256: "9a22d372c51455c34f13405da2520baefb7125bd16981397561423ed32d24f36",
    },
    ModelFile {
        local_name: "decoder.onnx",
        url: "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/main/decoder_joint-model.onnx",
        size_bytes: 72_500_000,
        sha256: "e978ddf6688527182c10fde2eb4b83068421648985ef23f7a86be732be8706c1",
    },
    ModelFile {
        local_name: "vocab.txt",
        url: "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/main/vocab.txt",
        size_bytes: 94_000,
        sha256: "d58544679ea4bc6ac563d1f545eb7d474bd6cfa467f0a6e2c1dc1c7d37e3c35d",
    },
    ModelFile {
        local_name: "nemo128.onnx",
        url: "https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/main/nemo128.onnx",
        size_bytes: 140_000,
        sha256: "a9fde1486ebfcc08f328d75ad4610c67835fea58c73ba57e3209a6f6cf019e9f",
    },
];

const PARAKEET_VARIANTS: &[ModelVariant] = &[
    ModelVariant {
        id: "int8",
        label: "Quantized (≈670 MB) — faster download, slightly less accurate",
        files: PARAKEET_INT8_FILES,
        total_size_bytes: 670_434_000,
    },
    ModelVariant {
        id: "full",
        label: "Full Precision (≈2.5 GB) — best quality, large download",
        files: PARAKEET_FULL_FILES,
        total_size_bytes: 2_554_534_000,
    },
];

// ── Diarization (Segmentation + Embedding) ─────────────────────────────

const DIARIZATION_FILES: &[ModelFile] = &[
    ModelFile {
        local_name: "segmentation.onnx",
        url: "https://huggingface.co/onnx-community/pyannote-segmentation-3.0/resolve/main/onnx/model.onnx",
        size_bytes: 6_000_000,
        sha256: "057ee564753071c0b09b5b611648b50ac188d50846bff5f01e9f7bbf1591ea25",
    },
    ModelFile {
        local_name: "embedding.onnx",
        url: "https://huggingface.co/Wespeaker/wespeaker-voxceleb-resnet34-LM/resolve/main/voxceleb_resnet34_LM.onnx",
        size_bytes: 26_500_000,
        sha256: "7bb2f06e9df17cdf1ef14ee8a15ab08ed28e8d0ef5054ee135741560df2ec068",
    },
];

const DIARIZATION_VARIANTS: &[ModelVariant] = &[ModelVariant {
    id: "default",
    label: "Speaker Identification (≈32 MB)",
    files: DIARIZATION_FILES,
    total_size_bytes: 32_500_000,
}];

// ── Embedding (all-MiniLM-L6-v2) ───────────────────────────────────

const EMBEDDING_FILES: &[ModelFile] = &[
    ModelFile {
        local_name: "model.onnx",
        url: "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/onnx/model.onnx",
        size_bytes: 86_000_000,
        sha256: "6fd5d72fe4589f189f8ebc006442dbb529bb7ce38f8082112682524616046452",
    },
    ModelFile {
        local_name: "vocab.txt",
        url: "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/vocab.txt",
        size_bytes: 232_000,
        sha256: "07eced375cec144d27c900241f3e339478dec958f92fddbc551f295c992038a3",
    },
];

const EMBEDDING_VARIANTS: &[ModelVariant] = &[ModelVariant {
    id: "default",
    label: "Sentence Embeddings (≈86 MB)",
    files: EMBEDDING_FILES,
    total_size_bytes: 86_232_000,
}];

// ── Denoising (DeepFilterNet3) ──────────────────────────────────────

const DENOISE_FILES: &[ModelFile] = &[
    ModelFile {
        local_name: "deepfilternet3.onnx",
        url: "https://huggingface.co/deepfilternet/DeepFilterNet3/resolve/main/DeepFilterNet3_onnx/enc.onnx",
        size_bytes: 1_800_000,
        // TODO(security): gated HuggingFace repo — download with HF token and run `shasum -a 256`
        sha256: "",
    },
    ModelFile {
        local_name: "deepfilternet3_dec.onnx",
        url: "https://huggingface.co/deepfilternet/DeepFilterNet3/resolve/main/DeepFilterNet3_onnx/dec.onnx",
        size_bytes: 1_200_000,
        // TODO(security): gated HuggingFace repo — download with HF token and run `shasum -a 256`
        sha256: "",
    },
];

const DENOISE_VARIANTS: &[ModelVariant] = &[ModelVariant {
    id: "default",
    label: "Noise Cancellation (≈3 MB)",
    files: DENOISE_FILES,
    total_size_bytes: 3_000_000,
}];

// ── VAD (NeMo MarbleNet) ────────────────────────────────────────────

const VAD_FILES: &[ModelFile] = &[ModelFile {
    local_name: "marblenet.onnx",
    url: "https://huggingface.co/nvidia/vad_marblenet/resolve/main/vad_marblenet.onnx",
    size_bytes: 5_000_000,
    // TODO(security): gated HuggingFace repo — download with HF token and run `shasum -a 256`
    sha256: "",
}];

const VAD_VARIANTS: &[ModelVariant] = &[ModelVariant {
    id: "default",
    label: "Voice Activity Detection (≈5 MB)",
    files: VAD_FILES,
    total_size_bytes: 5_000_000,
}];

// ── Full Registry ──────────────────────────────────────────────────────

pub const MODEL_REGISTRY: &[ModelDefinition] = &[
    ModelDefinition {
        id: "parakeet-tdt-0.6b-v3",
        name: "Parakeet TDT 0.6B v3",
        description: "NVIDIA speech-to-text model for English transcription",
        category: ModelCategory::Transcription,
        dir_name: "parakeet-tdt-0.6b-v3",
        variants: PARAKEET_VARIANTS,
    },
    ModelDefinition {
        id: "diarization",
        name: "Speaker Diarization",
        description: "Identifies who is speaking (pyannote segmentation + WeSpeaker LM embedding)",
        category: ModelCategory::Diarization,
        dir_name: "diarization",
        variants: DIARIZATION_VARIANTS,
    },
    ModelDefinition {
        id: "embedding-minilm",
        name: "Sentence Embeddings",
        description: "all-MiniLM-L6-v2 for semantic search across transcripts",
        category: ModelCategory::Embedding,
        dir_name: "embedding-minilm",
        variants: EMBEDDING_VARIANTS,
    },
    ModelDefinition {
        id: "deepfilternet3",
        name: "Noise Cancellation",
        description: "DeepFilterNet3 for real-time audio denoising before transcription",
        category: ModelCategory::Denoising,
        dir_name: "deepfilternet3",
        variants: DENOISE_VARIANTS,
    },
    ModelDefinition {
        id: "vad-marblenet",
        name: "Voice Activity Detection",
        description: "NeMo MarbleNet for detecting speech in audio",
        category: ModelCategory::VoiceActivity,
        dir_name: "vad-marblenet",
        variants: VAD_VARIANTS,
    },
];

/// Look up a model definition by ID.
pub fn get_model(id: &str) -> Option<&'static ModelDefinition> {
    MODEL_REGISTRY.iter().find(|m| m.id == id)
}

/// Look up a specific variant within a model.
pub fn get_variant(
    model_id: &str,
    variant_id: &str,
) -> Option<(&'static ModelDefinition, &'static ModelVariant)> {
    let model = get_model(model_id)?;
    let variant = model.variants.iter().find(|v| v.id == variant_id)?;
    Some((model, variant))
}

/// Get the local directory path for a model.
pub fn model_dir(model: &ModelDefinition) -> std::path::PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("Nootle")
        .join("models")
        .join(model.dir_name)
}

/// Check if all files for a variant are present in a given directory.
fn is_variant_in_dir(dir: &std::path::Path, variant: &ModelVariant) -> bool {
    variant
        .files
        .iter()
        .all(|f| dir.join(f.local_name).exists())
}

/// Check if all files for a variant are present on disk.
pub fn is_variant_downloaded(model: &ModelDefinition, variant: &ModelVariant) -> bool {
    is_variant_in_dir(&model_dir(model), variant)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_five_models() {
        assert_eq!(MODEL_REGISTRY.len(), 5);
    }

    #[test]
    #[test]
    fn external_data_keeps_the_name_the_graph_references() {
        // encoder.onnx names its weights file internally as
        // "encoder-model.onnx.data". Saving it under any other local name
        // silently breaks transcription on every execution provider.
        let (model, variant) = get_variant("parakeet-tdt-0.6b-v3", "full").unwrap();
        let _ = model;
        let data = variant
            .files
            .iter()
            .find(|f| f.url.ends_with(".onnx.data"))
            .expect("fp32 variant must ship an external data file");
        assert_eq!(data.local_name, "encoder-model.onnx.data");
    }

    fn parakeet_has_two_variants() {
        let model = get_model("parakeet-tdt-0.6b-v3").unwrap();
        assert_eq!(model.variants.len(), 2);
        assert_eq!(model.variants[0].id, "int8");
        assert_eq!(model.variants[1].id, "full");
    }

    #[test]
    fn diarization_has_one_variant() {
        let model = get_model("diarization").unwrap();
        assert_eq!(model.variants.len(), 1);
        assert_eq!(model.variants[0].id, "default");
    }

    #[test]
    fn embedding_model_exists() {
        let model = get_model("embedding-minilm").unwrap();
        assert_eq!(model.variants.len(), 1);
        assert_eq!(model.category, ModelCategory::Embedding);
    }

    #[test]
    fn denoise_model_exists() {
        let model = get_model("deepfilternet3").unwrap();
        assert_eq!(model.variants.len(), 1);
        assert_eq!(model.category, ModelCategory::Denoising);
    }

    #[test]
    fn vad_model_exists() {
        let model = get_model("vad-marblenet").unwrap();
        assert_eq!(model.variants.len(), 1);
        assert_eq!(model.category, ModelCategory::VoiceActivity);
    }

    #[test]
    fn get_variant_works() {
        let (model, variant) = get_variant("parakeet-tdt-0.6b-v3", "int8").unwrap();
        assert_eq!(model.id, "parakeet-tdt-0.6b-v3");
        assert_eq!(variant.id, "int8");
        assert_eq!(variant.files.len(), 4);
    }

    #[test]
    fn get_variant_returns_none_for_bad_id() {
        assert!(get_variant("nonexistent", "int8").is_none());
        assert!(get_variant("parakeet-tdt-0.6b-v3", "nonexistent").is_none());
    }

    #[test]
    fn model_dir_contains_nootle() {
        let model = get_model("parakeet-tdt-0.6b-v3").unwrap();
        let dir = model_dir(model);
        assert!(dir.to_str().unwrap().contains("Nootle"));
        assert!(dir.to_str().unwrap().contains("parakeet-tdt-0.6b-v3"));
    }

    #[test]
    fn variant_not_downloaded_in_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let model = get_model("parakeet-tdt-0.6b-v3").unwrap();
        let variant = &model.variants[0];
        assert!(!is_variant_in_dir(tmp.path(), variant));
    }

    #[test]
    fn all_urls_are_huggingface_resolve() {
        for model in MODEL_REGISTRY {
            for variant in model.variants {
                for file in variant.files {
                    assert!(
                        file.url.starts_with("https://huggingface.co/"),
                        "URL should be HuggingFace: {}",
                        file.url
                    );
                    assert!(
                        file.url.contains("/resolve/main/"),
                        "URL should use /resolve/main/ for direct download: {}",
                        file.url
                    );
                }
            }
        }
    }
}
