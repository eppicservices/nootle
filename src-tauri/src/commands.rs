use crate::audio::{run_audio_capture, validate_audio_devices, RecordingSession};
use crate::db::*;
use crate::diarization::DiarizationEngine;
use crate::extraction;
use crate::llm::{ChatMessage, LlmRegistry};
use crate::model_download::{self, DownloadManager};
use crate::model_registry;
use crate::summarization;
use crate::transcription::{self, TranscriptionEngine};
use std::sync::Arc;
use tauri::{Emitter, State};
use tokio::sync::Mutex as TokioMutex;

fn truncate_at_word_boundary(text: &str, max_chars: usize, suffix: &str) -> String {
    if text.chars().count() <= max_chars {
        return text.trim().to_string();
    }
    let prefix: String = text.chars().take(max_chars).collect();
    match prefix.rfind(' ') {
        Some(pos) => format!("{}{suffix}", &prefix[..pos]),
        None => format!("{prefix}{suffix}"),
    }
}

pub type DbState = Arc<Database>;
pub type RecordingState = Arc<TokioMutex<Option<RecordingSession>>>;
pub type LlmState = Arc<tokio::sync::RwLock<LlmRegistry>>;
pub type DetectorState = Arc<std::sync::Mutex<crate::detection::MeetingDetector>>;
pub type DownloadManagerState = Arc<TokioMutex<DownloadManager>>;
pub type EmbeddingState = Arc<TokioMutex<Option<crate::embedding::EmbeddingEngine>>>;

const ALLOWED_PROVIDERS: &[&str] = &[
    "openai",
    "anthropic",
    "google",
    "groq",
    "openrouter",
    "bedrock",
    "codex",
    "claude-agent",
    "linear",
    "asana",
];

fn validate_provider(provider: &str) -> Result<(), String> {
    if ALLOWED_PROVIDERS.contains(&provider) {
        Ok(())
    } else {
        Err(format!("Invalid provider: {}", provider))
    }
}

#[tauri::command]
pub fn list_meetings(
    db: State<'_, DbState>,
    search: Option<String>,
    include_archived: Option<bool>,
) -> Result<Vec<Meeting>, String> {
    db.list_meetings(search.as_deref(), include_archived.unwrap_or(false))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_meeting(db: State<'_, DbState>, id: String) -> Result<Meeting, String> {
    db.get_meeting(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_meeting(db: State<'_, DbState>, id: String) -> Result<(), String> {
    // Get meeting to find audio path before deleting
    let audio_path = db.get_meeting(&id).ok().and_then(|m| m.audio_path.clone());

    // Delete vec0 embeddings (not cascade-aware)
    let _ = db.delete_meeting_chunks(&id);

    db.delete_meeting(&id).map_err(|e| e.to_string())?;

    if let Some(path) = audio_path {
        let _ = std::fs::remove_file(&path);
    }

    Ok(())
}

#[tauri::command]
pub fn update_meeting_status(
    db: State<'_, DbState>,
    id: String,
    status: String,
) -> Result<(), String> {
    const VALID_STATUSES: &[&str] = &["recording", "transcribing", "summarized", "archived"];
    if !VALID_STATUSES.contains(&status.as_str()) {
        return Err(format!("Invalid meeting status: {}", status));
    }
    db.update_meeting_status(&id, &status)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_meeting_title(
    db: State<'_, DbState>,
    id: String,
    title: String,
) -> Result<(), String> {
    db.update_meeting_title(&id, &title)
        .map_err(|e| e.to_string())
}

/// Changes which summary template a meeting will be summarized with. Pass
/// `None` to clear it and fall back to the auto-run templates.
#[tauri::command]
pub fn update_meeting_template(
    db: State<'_, DbState>,
    id: String,
    template_id: Option<String>,
) -> Result<(), String> {
    db.update_meeting_template(&id, template_id.as_deref())
        .map_err(|e| e.to_string())
}

fn validate_hex_color(color: &str) -> Result<(), String> {
    if color.len() == 7
        && color.starts_with('#')
        && color[1..].chars().all(|ch| ch.is_ascii_hexdigit())
    {
        Ok(())
    } else {
        Err(format!("Invalid hex color: {}", color))
    }
}

fn get_linear_api_key(db: &Database) -> Result<String, String> {
    // Check legacy linear_settings table first
    if let Some(key) = db
        .get_linear_setting("api_key")
        .map_err(|e| e.to_string())?
        .filter(|k| !k.is_empty())
    {
        return Ok(key);
    }
    // Fall back to integrations table
    if let Some(integration) = db
        .get_integration_by_type("linear")
        .map_err(|e| e.to_string())?
    {
        if let Ok(creds) = serde_json::from_str::<serde_json::Value>(&integration.credentials_json)
        {
            if let Some(key) = creds.get("api_key").and_then(|v| v.as_str()) {
                if !key.is_empty() {
                    return Ok(key.to_string());
                }
            }
        }
    }
    Err("Linear API key not configured".to_string())
}

#[tauri::command]
pub fn create_label(
    db: State<'_, DbState>,
    name: String,
    color: String,
    icon: Option<String>,
) -> Result<Label, String> {
    validate_hex_color(&color)?;
    db.create_label(&name, &color, icon.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_labels(db: State<'_, DbState>) -> Result<Vec<Label>, String> {
    db.list_labels().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_label(
    db: State<'_, DbState>,
    id: String,
    name: String,
    color: String,
    icon: Option<String>,
) -> Result<Label, String> {
    validate_hex_color(&color)?;
    db.update_label(&id, &name, &color, icon.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_label(db: State<'_, DbState>, id: String) -> Result<(), String> {
    db.delete_label(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_meeting_label(
    db: State<'_, DbState>,
    meeting_id: String,
    label_id: String,
) -> Result<(), String> {
    db.add_meeting_label(&meeting_id, &label_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_meeting_label(
    db: State<'_, DbState>,
    meeting_id: String,
    label_id: String,
) -> Result<(), String> {
    db.remove_meeting_label(&meeting_id, &label_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_meeting_labels(
    db: State<'_, DbState>,
    meeting_id: String,
) -> Result<Vec<Label>, String> {
    db.get_meeting_labels(&meeting_id)
        .map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
pub struct MeetingLabelEntry {
    pub meeting_id: String,
    pub label: Label,
}

#[tauri::command]
pub fn get_all_meeting_labels(db: State<'_, DbState>) -> Result<Vec<MeetingLabelEntry>, String> {
    db.get_all_meeting_labels()
        .map(|entries| {
            entries
                .into_iter()
                .map(|(meeting_id, label)| MeetingLabelEntry { meeting_id, label })
                .collect()
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_scratch_note(
    db: State<'_, DbState>,
    meeting_id: String,
    content: String,
    timestamp_ms: i64,
) -> Result<ScratchNote, String> {
    db.add_scratch_note(&meeting_id, &content, timestamp_ms)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_scratch_notes(
    db: State<'_, DbState>,
    meeting_id: String,
) -> Result<Vec<ScratchNote>, String> {
    db.get_scratch_notes(&meeting_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_scratch_note(db: State<'_, DbState>, id: String) -> Result<(), String> {
    db.delete_scratch_note(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_transcript(
    db: State<'_, DbState>,
    meeting_id: String,
) -> Result<Vec<TranscriptSegment>, String> {
    db.get_transcript(&meeting_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_transcripts(
    db: State<'_, DbState>,
    query: String,
) -> Result<Vec<TranscriptSearchResult>, String> {
    db.search_transcripts(&query).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_recipe(
    db: State<'_, DbState>,
    name: String,
    description: String,
    slash_command: String,
    prompt_template: String,
    output_format: String,
) -> Result<Recipe, String> {
    db.create_recipe(NewRecipe {
        name,
        description,
        slash_command,
        prompt_template,
        output_format,
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_recipes(db: State<'_, DbState>) -> Result<Vec<Recipe>, String> {
    db.list_recipes().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_recipe(
    db: State<'_, DbState>,
    id: String,
    name: String,
    description: String,
    slash_command: String,
    prompt_template: String,
    output_format: String,
) -> Result<Recipe, String> {
    db.update_recipe(
        &id,
        &name,
        &description,
        &slash_command,
        &prompt_template,
        &output_format,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_recipe(db: State<'_, DbState>, id: String) -> Result<(), String> {
    db.delete_recipe(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn run_recipe(
    db: State<'_, DbState>,
    llm: State<'_, LlmState>,
    meeting_id: String,
    recipe_id: String,
    provider: String,
    model: String,
) -> Result<String, String> {
    let llm = llm.read().await;
    summarization::run_recipe(&db, &llm, &meeting_id, &recipe_id, &provider, &model)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn create_template(
    db: State<'_, DbState>,
    name: String,
    description: String,
    sections: String,
    auto_apply_rules: String,
    prompt: String,
    is_favorite: bool,
    is_auto_run: bool,
) -> Result<Template, String> {
    db.create_template(NewTemplate {
        name,
        description,
        sections,
        auto_apply_rules,
        prompt,
        is_favorite,
        is_auto_run,
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_templates(db: State<'_, DbState>) -> Result<Vec<Template>, String> {
    db.list_templates().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_template(db: State<'_, DbState>, id: String) -> Result<(), String> {
    db.delete_template(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_template(db: State<'_, DbState>, params: UpdateTemplate) -> Result<Template, String> {
    db.update_template(&params).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_summaries(db: State<'_, DbState>, meeting_id: String) -> Result<Vec<Summary>, String> {
    db.get_summaries_for_meeting(&meeting_id)
        .map_err(|e| e.to_string())
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub async fn start_recording(
    app: tauri::AppHandle,
    db: State<'_, DbState>,
    llm: State<'_, LlmState>,
    recording: State<'_, RecordingState>,
    embedding_state: State<'_, EmbeddingState>,
    title: String,
    calendar_event_id: Option<String>,
    template_id: Option<String>,
) -> Result<Meeting, String> {
    let mut session_lock = recording.lock().await;
    if session_lock.is_some() {
        return Err("Already recording".to_string());
    }

    // Check microphone permission before creating any resources
    let mic_status = crate::permissions::check_microphone();
    if mic_status == "denied" {
        return Err(
            "Microphone access denied. Please grant microphone permission in System Settings."
                .to_string(),
        );
    }
    if mic_status == "undetermined" {
        let granted = crate::permissions::request_microphone().await;
        if !granted {
            return Err(
                "Microphone access is required to record. Please grant permission and try again."
                    .to_string(),
            );
        }
    }

    let meeting = db
        .create_meeting(NewMeeting {
            title,
            calendar_event_id,
            template_id,
        })
        .map_err(|e| e.to_string())?;

    // Create recording session — rollback meeting if this fails
    let recordings_dir = dirs::data_dir()
        .ok_or_else(|| "Could not determine data directory".to_string())?
        .join("Nootle")
        .join("recordings");
    let mut session = match RecordingSession::new(&recordings_dir, &meeting.id, 16000) {
        Ok(s) => s,
        Err(e) => {
            let _ = db.delete_meeting(&meeting.id);
            return Err(e.to_string());
        }
    };

    // Initialize audio capture on this thread to fail early if there are permission or device issues
    let audio_tx = match session.take_audio_tx() {
        Some(tx) => tx,
        None => {
            let _ = db.delete_meeting(&meeting.id);
            return Err("Failed to initialize audio channel".to_string());
        }
    };

    if let Err(e) = validate_audio_devices() {
        let _ = db.delete_meeting(&meeting.id);
        return Err(format!("Audio device validation failed: {e}"));
    }

    session.start();

    let denoise_enabled = db
        .get_setting("denoise_enabled")
        .unwrap_or(None)
        .map(|v| v != "false")
        .unwrap_or(true); // default on

    // Spawn audio capture loop on a dedicated thread
    {
        let is_active = session.is_active_flag();
        let audio_path = session.audio_path().to_path_buf();

        let handle = std::thread::spawn(move || {
            // Create denoise engine inside the thread (Session may not be Send)
            let mut denoise_engine =
                if denoise_enabled && crate::denoise::DenoiseEngine::is_available() {
                    match crate::denoise::DenoiseEngine::load() {
                        Ok(e) => {
                            tracing::info!("Denoising enabled");
                            Some(e)
                        }
                        Err(e) => {
                            tracing::warn!("Failed to load denoise engine: {e}");
                            None
                        }
                    }
                } else {
                    None
                };

            if let Err(e) =
                run_audio_capture(audio_tx, is_active, audio_path, denoise_engine.as_mut())
            {
                tracing::error!("Audio capture failed: {e}");
            }
        });
        session.set_capture_handle(handle);
    }

    // Spawn live transcription pipeline if models are available
    if let Some(audio_rx) = session.take_audio_rx() {
        let db_clone = db.inner().clone();
        let llm_clone = llm.inner().clone();
        let embedding_clone = embedding_state.inner().clone();
        let meeting_id = meeting.id.clone();
        let app_handle = app.clone();

        tokio::spawn(async move {
            run_transcription_pipeline(
                audio_rx,
                db_clone,
                llm_clone,
                embedding_clone,
                meeting_id,
                app_handle,
            )
            .await;
        });
    }

    *session_lock = Some(session);

    Ok(meeting)
}

/// Background task: consume audio chunks, transcribe, diarize, persist, and emit events.
async fn run_transcription_pipeline(
    mut audio_rx: tokio::sync::mpsc::Receiver<Vec<f32>>,
    db: Arc<Database>,
    llm_state: LlmState,
    embedding_state: Arc<TokioMutex<Option<crate::embedding::EmbeddingEngine>>>,
    meeting_id: String,
    app: tauri::AppHandle,
) {
    // Try to load transcription engine
    tracing::info!("[DIAG] Transcription pipeline started, loading engine...");
    let mut transcription_engine = match TranscriptionEngine::load() {
        Ok(e) => {
            tracing::info!("[DIAG] Transcription engine loaded successfully");
            Some(e)
        }
        Err(err) => {
            tracing::error!("[DIAG] Transcription engine FAILED to load: {err:#}");
            None
        }
    };

    // Notify frontend whether live transcription is available
    if transcription_engine.is_some() {
        let _ = app.emit(
            "transcription-status",
            serde_json::json!({ "available": true }),
        );
    } else {
        let reason = match TranscriptionEngine::check_status() {
            transcription::ModelStatus::NotDownloaded => {
                "Transcription model not downloaded. Recording audio only.".to_string()
            }
            _ => {
                "Transcription model failed to load. The model files may be corrupted — try deleting and re-downloading in Settings.".to_string()
            }
        };
        let _ = app.emit(
            "transcription-status",
            serde_json::json!({
                "available": false,
                "reason": reason,
            }),
        );
    }

    // Try to load diarization engine
    let mut diarization_engine = match DiarizationEngine::load() {
        Ok(e) => Some(e),
        Err(err) => {
            tracing::info!("Diarization models not available, skipping: {err}");
            None
        }
    };

    let mut offset_samples: u64 = 0;
    let sample_rate: u64 = 16000;

    let mut chunk_count: u64 = 0;
    let mut segment_count: u64 = 0;
    tracing::info!(
        "[DIAG] Entering audio chunk loop, engine={}, diarization={}",
        transcription_engine.is_some(),
        diarization_engine.is_some()
    );

    while let Some(chunk) = audio_rx.recv().await {
        let offset_ms = offset_samples * 1000 / sample_rate;
        let chunk_len = chunk.len() as u64;
        chunk_count += 1;

        // Log every chunk received
        let rms = (chunk.iter().map(|s| s * s).sum::<f32>() / chunk.len() as f32).sqrt();
        tracing::info!(
            "[DIAG] Chunk #{chunk_count}: {chunk_len} samples, offset={offset_ms}ms, RMS={rms:.6}"
        );

        // Run transcription
        if let Some(ref mut engine) = transcription_engine {
            match engine.transcribe(&chunk, offset_ms) {
                Ok(segments) => {
                    tracing::info!(
                        "[DIAG] Chunk #{chunk_count}: transcribe() returned {} segments",
                        segments.len()
                    );
                    segment_count += segments.len() as u64;
                    for seg in &segments {
                        tracing::info!("[DIAG] Segment: {:?}", seg.text);

                        // Run diarization to get speaker label
                        let speaker = if let Some(ref mut diar) = diarization_engine {
                            match diar.diarize(&chunk, offset_ms) {
                                Ok(diar_segs) => diar_segs
                                    .first()
                                    .map(|s| s.speaker_id.clone())
                                    .unwrap_or_else(|| "Speaker".to_string()),
                                Err(_) => "Speaker".to_string(),
                            }
                        } else {
                            "Speaker".to_string()
                        };

                        match db.create_transcript_segment(NewTranscriptSegment {
                            meeting_id: meeting_id.clone(),
                            speaker_label: speaker,
                            text: seg.text.clone(),
                            start_ms: i64::try_from(seg.start_ms).unwrap_or(i64::MAX),
                            end_ms: i64::try_from(seg.end_ms).unwrap_or(i64::MAX),
                            confidence: 0.9,
                        }) {
                            Ok(_) => tracing::info!("[DIAG] DB insert succeeded"),
                            Err(e) => tracing::error!("[DIAG] DB insert FAILED: {e}"),
                        }
                    }

                    // Emit updated transcript to frontend
                    match db.get_transcript(&meeting_id) {
                        Ok(all_segments) => {
                            tracing::info!(
                                "[DIAG] Emitting transcript-update with {} segments",
                                all_segments.len()
                            );
                            match app.emit("transcript-update", &all_segments) {
                                Ok(_) => tracing::info!("[DIAG] Emit succeeded"),
                                Err(e) => tracing::error!("[DIAG] Emit FAILED: {e}"),
                            }
                        }
                        Err(e) => {
                            tracing::error!("[DIAG] get_transcript FAILED: {e}");
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("[DIAG] Chunk #{chunk_count}: transcribe() FAILED: {e:#}");
                }
            }
        } else if chunk_count == 1 {
            tracing::warn!("[DIAG] No transcription engine — chunks will not be transcribed");
        }

        offset_samples += chunk_len;
    }

    tracing::info!("[DIAG] Audio channel closed after {chunk_count} chunks");

    // Auto-generate title from transcript content using LLM
    if let Ok(segments) = db.get_transcript(&meeting_id) {
        let full_text: String = segments
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        if !full_text.trim().is_empty() {
            // Take up to ~500 chars of transcript for the LLM to summarize
            let snippet = truncate_at_word_boundary(&full_text, 500, "");

            let llm = llm_state.read().await;
            let fallback_title = || -> String { truncate_at_word_boundary(&full_text, 60, "...") };
            let title = if let Some(model) = llm.all_models().first().cloned() {
                if let Some(provider) = llm.get_provider(&model.provider) {
                    let prompt = format!(
                        "Generate a short, descriptive title (max 8 words) for this meeting based on the transcript below. \
                         Return ONLY the title, nothing else. No quotes, no punctuation at the end.\n\n{snippet}"
                    );
                    let messages = vec![crate::llm::ChatMessage {
                        role: "user".into(),
                        content: prompt,
                    }];
                    match provider.chat(messages, &model.id).await {
                        Ok(resp) => {
                            let t = resp.trim().trim_matches('"').trim().to_string();
                            if t.is_empty() || t.len() > 100 {
                                fallback_title()
                            } else {
                                t
                            }
                        }
                        Err(e) => {
                            tracing::warn!("LLM title generation failed, using fallback: {e}");
                            fallback_title()
                        }
                    }
                } else {
                    fallback_title()
                }
            } else {
                fallback_title()
            };
            drop(llm);

            if let Err(e) = db.update_meeting_title(&meeting_id, &title) {
                tracing::warn!("Failed to auto-generate title: {e}");
            } else if let Ok(meeting) = db.get_meeting(&meeting_id) {
                let _ = app.emit("meeting-updated", &meeting);
            }
        }
    }

    // Run auto-run templates (summaries) if any are configured
    {
        let llm = llm_state.read().await;
        if let Some(first_model) = llm.all_models().first().cloned() {
            match summarization::run_auto_templates(
                &db,
                &llm,
                &meeting_id,
                &first_model.provider,
                &first_model.id,
            )
            .await
            {
                Ok(summaries) if !summaries.is_empty() => {
                    tracing::info!(
                        "Auto-run produced {} summaries for {meeting_id}",
                        summaries.len()
                    );
                    let _ = app.emit("summaries-updated", &meeting_id);
                }
                Ok(_) => tracing::info!("No auto-run templates configured"),
                Err(e) => tracing::warn!("Auto-run templates failed: {e}"),
            }
        } else {
            tracing::info!("No LLM providers configured, skipping auto-run prompts");
        }
    }

    // Mark meeting as done transcribing only if transcription produced segments
    if segment_count > 0 {
        if let Err(e) = db.update_meeting_status(&meeting_id, "summarized") {
            tracing::warn!("Failed to update meeting status to summarized: {e}");
        }
    } else {
        tracing::info!(
            "No transcript segments produced for {meeting_id}, not marking as summarized"
        );
    }

    // After transcription completes, embed the meeting
    let mut engine_lock = embedding_state.lock().await;
    if let Some(ref mut engine) = *engine_lock {
        match crate::chunking::embed_meeting(&db, engine, &meeting_id) {
            Ok(count) => tracing::info!("Embedded {count} chunks for meeting {meeting_id}"),
            Err(e) => tracing::warn!("Failed to embed meeting {meeting_id}: {e}"),
        }
    }
}

#[tauri::command]
pub async fn stop_recording(
    app: tauri::AppHandle,
    db: State<'_, DbState>,
    llm: State<'_, LlmState>,
    recording: State<'_, RecordingState>,
) -> Result<Meeting, String> {
    let mut session = {
        let mut session_lock = recording.lock().await;
        session_lock
            .take()
            .ok_or_else(|| "Not recording".to_string())?
    };

    let meeting_id = session.meeting_id().to_string();
    let audio_path = session.audio_path().to_string_lossy().to_string();
    session.stop();

    // Wait for the capture thread to finish writing the WAV file (outside the lock)
    if let Some(handle) = session.take_capture_handle() {
        match tokio::task::spawn_blocking(move || handle.join()).await {
            Ok(Ok(())) => tracing::debug!("Audio capture thread joined cleanly"),
            Ok(Err(_)) => tracing::error!("Audio capture thread panicked"),
            Err(e) => tracing::error!("Failed to join audio capture thread: {e}"),
        }
    }

    // Finalize meeting with end time and audio path
    let end_time = chrono::Utc::now().to_rfc3339();
    db.finalize_meeting(&meeting_id, &end_time, Some(&audio_path), "transcribing")
        .map_err(|e| e.to_string())?;

    // Auto-extract insights if an LLM provider is configured
    {
        let db_clone = db.inner().clone();
        let llm_clone = llm.inner().clone();
        let mid = meeting_id.clone();
        tokio::spawn(async move {
            let registry = llm_clone.read().await;
            let providers = registry.provider_names();
            // Honour an explicit choice before falling back to registration
            // order. Order is a fragile default: which provider summarises your
            // meetings then depends on which ones happen to be installed, and
            // standing up a new one silently moves your transcripts to a
            // different vendor. Set `summarization_provider` to pin it.
            let preferred = db_clone
                .get_setting("summarization_provider")
                .unwrap_or(None)
                .filter(|p| providers.iter().any(|name| name == p));
            let chosen = preferred.clone().or_else(|| providers.first().cloned());
            if let Some(provider_name) = chosen.as_ref() {
                let models = registry.all_models();
                let provider_models: Vec<_> = models
                    .iter()
                    .filter(|m| &m.provider == provider_name)
                    .collect();
                if let Some(model) = provider_models.first() {
                    if let Err(e) = crate::extraction::extract_insights(
                        &db_clone,
                        &registry,
                        &mid,
                        provider_name,
                        &model.id,
                    )
                    .await
                    {
                        tracing::warn!("Auto-extraction failed: {e}");
                    }

                    // Workflows opted in with `"auto_run": true` fire once
                    // insights exist, so an export lands without the user
                    // opening the app and clicking per meeting.
                    //
                    // Opt-in, never all-enabled-workflows: the same mechanism
                    // drives Slack, Linear, GitHub and Notion, and silently
                    // messaging a channel because a meeting ended is not a
                    // surprise anyone wants.
                    let provider_owned = provider_name.clone();
                    let model_owned = model.id.clone();
                    drop(registry);
                    run_auto_workflows(
                        &db_clone,
                        &llm_clone,
                        &mid,
                        &provider_owned,
                        &model_owned,
                    )
                    .await;
                }
            }
        });
    }

    {
        let db_analytics = db.inner().clone();
        let app_analytics = app.clone();
        let meeting_id_analytics = meeting_id.clone();

        tauri::async_runtime::spawn(async move {
            if let Err(e) = (|| -> std::result::Result<(), Box<dyn std::error::Error>> {
                let speaker_analytics = crate::analytics::compute_speaker_analytics(
                    &db_analytics,
                    &meeting_id_analytics,
                )?;
                db_analytics.save_speaker_analytics(&meeting_id_analytics, &speaker_analytics)?;

                let transcripts = db_analytics.get_transcript(&meeting_id_analytics)?;
                let texts: Vec<String> = transcripts.iter().map(|t| t.text.clone()).collect();
                let engagement = crate::analytics::compute_engagement(
                    &meeting_id_analytics,
                    &speaker_analytics,
                    &texts,
                );
                db_analytics.save_engagement(&engagement)?;

                let _ = app_analytics.emit("analytics-ready", &meeting_id_analytics);
                tracing::info!("Analytics computed for meeting {}", meeting_id_analytics);
                Ok(())
            })() {
                tracing::warn!("Failed to compute analytics: {e}");
            }
        });
    }

    // Return the updated meeting
    db.get_meeting(&meeting_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn is_recording(recording: State<'_, RecordingState>) -> Result<bool, String> {
    let session = recording.lock().await;
    Ok(session.is_some())
}

/// Read an audio file and return it as base64-encoded WAV data.
#[tauri::command]
pub async fn get_audio_data(
    db: State<'_, DbState>,
    meeting_id: String,
) -> Result<Option<String>, String> {
    let meeting = db.get_meeting(&meeting_id).map_err(|e| e.to_string())?;
    let audio_path = match meeting.audio_path {
        Some(p) if !p.is_empty() => p,
        _ => return Ok(None),
    };
    let path = std::path::Path::new(&audio_path);
    if !path.exists() {
        return Ok(None);
    }
    // Validate that the audio path is within the expected recordings directory
    let recordings_dir = dirs::data_dir()
        .ok_or_else(|| "Could not determine data directory".to_string())?
        .join("Nootle")
        .join("recordings");
    let canonical = std::fs::canonicalize(path).map_err(|_| "Invalid audio path".to_string())?;
    if !canonical.starts_with(&recordings_dir) {
        return Err("Audio path outside recordings directory".to_string());
    }
    let data = std::fs::read(&canonical).map_err(|e| format!("Failed to read audio file: {e}"))?;
    use base64::Engine;
    Ok(Some(
        base64::engine::general_purpose::STANDARD.encode(&data),
    ))
}

#[tauri::command]
pub async fn store_api_key(
    db: State<'_, DbState>,
    llm: State<'_, LlmState>,
    provider: String,
    key: String,
) -> Result<(), String> {
    validate_provider(&provider)?;
    // Linear keys are stored in the database alongside other Linear settings
    if provider == "linear" {
        db.set_linear_setting("api_key", &key)
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    db.store_api_key(&provider, &key)
        .map_err(|e| e.to_string())?;

    // Hot-reload: register the provider in the LLM registry
    let mut registry = llm.write().await;
    registry.unregister(&provider);
    let new_provider: Box<dyn crate::llm::LlmProvider> = match provider.as_str() {
        "openai" => Box::new(crate::llm::OpenAiProvider::new(key)),
        "anthropic" => Box::new(crate::llm::AnthropicProvider::new(key)),
        "google" => Box::new(crate::llm::GoogleProvider::new(key)),
        "groq" => Box::new(crate::llm::GroqProvider::new(key)),
        "openrouter" => Box::new(crate::llm::OpenRouterProvider::new(key)),
        "bedrock" => Box::new(crate::llm::BedrockProvider::new(key)),
        "codex" => Box::new(crate::llm::CodexProvider::new(key)),
        _ => return Ok(()),
    };
    registry.register(new_provider);

    Ok(())
}

#[tauri::command]
pub fn has_api_key(db: State<'_, DbState>, provider: String) -> Result<bool, String> {
    validate_provider(&provider)?;
    if provider == "linear" {
        return db
            .get_linear_setting("api_key")
            .map(|opt| opt.is_some())
            .map_err(|e| e.to_string());
    }
    db.get_api_key(&provider)
        .map(|opt| opt.is_some())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_api_key(
    db: State<'_, DbState>,
    llm: State<'_, LlmState>,
    provider: String,
) -> Result<(), String> {
    validate_provider(&provider)?;
    if provider == "linear" {
        db.delete_linear_setting("api_key")
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    db.delete_api_key(&provider).map_err(|e| e.to_string())?;

    // Hot-reload: remove the provider from the LLM registry
    let mut registry = llm.write().await;
    registry.unregister(&provider);

    Ok(())
}

#[tauri::command]
pub fn list_stored_providers(db: State<'_, DbState>) -> Result<Vec<String>, String> {
    let mut providers = db.list_api_key_providers().map_err(|e| e.to_string())?;
    // Check if Linear API key is stored in the database
    if let Ok(Some(key)) = db.get_linear_setting("api_key") {
        if !key.is_empty() {
            providers.push("linear".to_string());
        }
    }
    Ok(providers)
}

#[tauri::command]
pub async fn generate_summary(
    db: State<'_, DbState>,
    llm: State<'_, LlmState>,
    meeting_id: String,
    template_id: String,
    provider: String,
    model: String,
) -> Result<Summary, String> {
    let llm = llm.read().await;
    summarization::summarize_meeting(&db, &llm, &meeting_id, &template_id, &provider, &model)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn chat_with_meeting(
    db: State<'_, DbState>,
    llm: State<'_, LlmState>,
    meeting_id: String,
    message: String,
    history: Vec<ChatMessage>,
    provider: String,
    model: String,
) -> Result<String, String> {
    let llm = llm.read().await;
    summarization::chat_with_transcript(
        &db,
        &llm,
        &meeting_id,
        &message,
        history,
        &provider,
        &model,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_llm_models(
    llm: State<'_, LlmState>,
) -> Result<Vec<crate::llm::ModelInfo>, String> {
    let llm = llm.read().await;
    Ok(llm.all_models())
}

#[tauri::command]
pub async fn list_llm_providers(llm: State<'_, LlmState>) -> Result<Vec<String>, String> {
    let llm = llm.read().await;
    Ok(llm.provider_names())
}

#[tauri::command]
pub fn check_permissions() -> Result<crate::permissions::PermissionStatus, String> {
    Ok(crate::permissions::check_all())
}

#[tauri::command]
pub async fn request_microphone_permission() -> Result<bool, String> {
    Ok(crate::permissions::request_microphone().await)
}

#[tauri::command]
pub fn request_screen_recording_permission() -> Result<bool, String> {
    Ok(crate::permissions::request_screen_recording())
}

#[tauri::command]
pub async fn request_calendar_permission() -> Result<bool, String> {
    Ok(crate::permissions::request_calendar().await)
}

#[tauri::command]
pub fn seed_default_prompts(db: State<'_, DbState>) -> Result<(), String> {
    let defaults = vec![
        ("Meeting Summary", "Summarize this meeting transcript. Include: key decisions, action items with owners, and main discussion points. Be concise.", true, true),
        ("Action Items", "Extract all action items from this meeting transcript. Format each as: - [Owner]: [Task] (deadline if mentioned).", true, true),
        ("Key Decisions", "List all decisions made during this meeting. For each, note: the decision, who made it, and any context.", true, false),
        ("Follow-up Questions", "Based on this meeting, what open questions remain unanswered? What topics need follow-up?", false, false),
        ("TL;DR", "Give a 2-3 sentence TL;DR of this meeting. What was it about and what was the outcome?", true, false),
    ];

    let existing = db.list_templates().map_err(|e| e.to_string())?;
    for (name, content, is_favorite, is_auto_run) in defaults {
        if existing.iter().any(|t| t.name == name) {
            continue;
        }
        db.create_template(NewTemplate {
            name: name.to_string(),
            description: String::new(),
            sections: "[]".to_string(),
            auto_apply_rules: "{}".to_string(),
            prompt: content.to_string(),
            is_favorite,
            is_auto_run,
        })
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn list_linear_teams(
    db: State<'_, DbState>,
) -> Result<Vec<crate::linear::LinearTeam>, String> {
    let api_key = get_linear_api_key(&db)?;
    crate::linear::list_teams(&api_key)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_linear_projects(
    db: State<'_, DbState>,
    team_id: String,
) -> Result<Vec<crate::linear::LinearProject>, String> {
    let api_key = get_linear_api_key(&db)?;
    crate::linear::list_projects(&api_key, &team_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_linear_ticket(
    db: State<'_, DbState>,
    llm: State<'_, LlmState>,
    summary_id: String,
    team_id: String,
    project_id: Option<String>,
    provider: String,
    model: String,
) -> Result<crate::db::LinearTicket, String> {
    // Fetch summary from DB
    let summary = db.get_summary(&summary_id).map_err(|e| e.to_string())?;
    let meeting = db
        .get_meeting(&summary.meeting_id)
        .map_err(|e| e.to_string())?;

    // Use LLM to generate ticket title and description
    let llm_registry = llm.read().await;
    let llm_provider = llm_registry
        .get_provider(&provider)
        .ok_or_else(|| format!("Provider '{}' not found", provider))?;

    let messages = vec![
        crate::llm::ChatMessage {
            role: "system".into(),
            content: "You are a helpful assistant that creates Linear tickets from meeting summaries. Return valid JSON with exactly two fields: \"title\" (concise, under 80 characters) and \"description\" (markdown-formatted, structured with sections as appropriate). Return ONLY the JSON object, no other text.".into(),
        },
        crate::llm::ChatMessage {
            role: "user".into(),
            content: format!("Meeting: {}\n\nSummary:\n{}", meeting.title, summary.content),
        },
    ];

    let llm_response = llm_provider
        .chat(messages, &model)
        .await
        .map_err(|e| e.to_string())?;

    // Parse LLM response, fallback to raw content
    let (title, description) = match serde_json::from_str::<serde_json::Value>(&llm_response) {
        Ok(json) => {
            let title = json
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or(&meeting.title)
                .to_string();
            let desc = json
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or(&summary.content)
                .to_string();
            (title, desc)
        }
        Err(_) => (meeting.title.clone(), summary.content.clone()),
    };

    let api_key = get_linear_api_key(&db)?;

    let issue = crate::linear::create_issue(
        &api_key,
        &team_id,
        project_id.as_deref(),
        &title,
        &description,
    )
    .await
    .map_err(|e| e.to_string())?;

    let ticket = db
        .create_linear_ticket(NewLinearTicket {
            summary_id: &summary_id,
            meeting_id: &summary.meeting_id,
            linear_issue_id: &issue.id,
            linear_issue_url: &issue.url,
            linear_identifier: &issue.identifier,
            title: &issue.title,
            team_id: &team_id,
            project_id: project_id.as_deref(),
        })
        .map_err(|e| e.to_string())?;

    Ok(ticket)
}

#[tauri::command]
pub async fn create_ticket_from_action_item(
    db: State<'_, DbState>,
    llm: State<'_, LlmState>,
    action_item_id: String,
    team_id: String,
    project_id: Option<String>,
    provider: String,
    model: String,
) -> Result<serde_json::Value, String> {
    let item = db
        .get_insight_by_action_item(&action_item_id)
        .map_err(|e| e.to_string())?;

    if let Some(ref existing) = item.linear_ticket_id {
        return Err(format!("Action item already has a ticket: {}", existing));
    }

    let meeting_title = item.meeting_title.as_deref().unwrap_or("Meeting");

    // Use LLM to generate ticket title and description
    let llm_registry = llm.read().await;
    let llm_provider = llm_registry
        .get_provider(&provider)
        .ok_or_else(|| format!("Provider '{}' not found", provider))?;

    let messages = vec![
        crate::llm::ChatMessage {
            role: "system".into(),
            content: "You are a helpful assistant that creates Linear tickets from meeting action items. Return valid JSON with exactly two fields: \"title\" (concise, under 80 characters) and \"description\" (markdown-formatted with context). Return ONLY the JSON object, no other text.".into(),
        },
        crate::llm::ChatMessage {
            role: "user".into(),
            content: format!(
                "Meeting: {}\n\nAction Item: {}\nAssignee: {}\nDue Date: {}\nContext: {}",
                meeting_title,
                item.content,
                item.assignee.as_deref().unwrap_or("Unassigned"),
                item.due_date.as_deref().unwrap_or("None"),
                item.context.as_deref().unwrap_or("None"),
            ),
        },
    ];

    let llm_response = llm_provider
        .chat(messages, &model)
        .await
        .map_err(|e| e.to_string())?;

    let cleaned = crate::extraction::strip_code_fences_pub(&llm_response);
    let (title, description) = match serde_json::from_str::<serde_json::Value>(cleaned) {
        Ok(json) => {
            let title = json
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or(&item.content)
                .to_string();
            let desc = json
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or(&item.content)
                .to_string();
            (title, desc)
        }
        Err(_) => (item.content.clone(), item.content.clone()),
    };

    let api_key = get_linear_api_key(&db)?;

    let issue = crate::linear::create_issue(
        &api_key,
        &team_id,
        project_id.as_deref(),
        &title,
        &description,
    )
    .await
    .map_err(|e| e.to_string())?;

    // Update the action item with the ticket ID
    db.set_action_item_linear_ticket(&action_item_id, &issue.identifier)
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "identifier": issue.identifier,
        "url": issue.url,
        "title": issue.title,
    }))
}

#[tauri::command]
pub fn get_linear_tickets(
    db: State<'_, DbState>,
    meeting_id: String,
) -> Result<Vec<crate::db::LinearTicket>, String> {
    db.get_linear_tickets(&meeting_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_linear_setting(db: State<'_, DbState>, key: String) -> Result<Option<String>, String> {
    db.get_linear_setting(&key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_linear_setting(
    db: State<'_, DbState>,
    key: String,
    value: String,
) -> Result<(), String> {
    const ALLOWED_KEYS: &[&str] = &["default_team_id", "default_project_id", "api_key"];
    if !ALLOWED_KEYS.contains(&key.as_str()) {
        return Err(format!("Invalid setting key: {}", key));
    }
    db.set_linear_setting(&key, &value)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_available_models() -> Result<serde_json::Value, String> {
    serde_json::to_value(model_registry::MODEL_REGISTRY).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_downloaded_models() -> Result<Vec<model_download::ModelOnDiskStatus>, String> {
    Ok(model_download::get_all_model_status())
}

#[tauri::command]
pub async fn download_model(
    app: tauri::AppHandle,
    download_mgr: State<'_, DownloadManagerState>,
    model_id: String,
    variant_id: String,
) -> Result<(), String> {
    let (model, variant) = model_registry::get_variant(&model_id, &variant_id)
        .ok_or_else(|| format!("Unknown model/variant: {model_id}/{variant_id}"))?;

    let cancel = {
        let mut mgr = download_mgr.lock().await;
        mgr.new_token()
    };

    let result = model_download::download_variant(app, model, variant, cancel).await;

    {
        let mut mgr = download_mgr.lock().await;
        mgr.clear_token();
    }

    result
}

#[tauri::command]
pub async fn cancel_download(download_mgr: State<'_, DownloadManagerState>) -> Result<(), String> {
    let mut mgr = download_mgr.lock().await;
    mgr.cancel();
    Ok(())
}

#[tauri::command]
pub async fn delete_model(model_id: String) -> Result<(), String> {
    let model =
        model_registry::get_model(&model_id).ok_or_else(|| format!("Unknown model: {model_id}"))?;
    model_download::delete_model_files(model)
}

/// Shared RAG helper: embed query, search chunks, call LLM with retrieved context.
#[allow(clippy::too_many_arguments)]
async fn rag_chat(
    db: &Database,
    llm: &LlmRegistry,
    embedding_state: &EmbeddingState,
    message: &str,
    history: Vec<ChatMessage>,
    provider: &str,
    model: &str,
    label_ids: &[String],
    date_from: Option<&str>,
    date_to: Option<&str>,
) -> Result<(String, Vec<serde_json::Value>), String> {
    let mut engine_lock = embedding_state.lock().await;
    let engine = engine_lock
        .as_mut()
        .ok_or_else(|| "Embedding model not loaded. Please download it first.".to_string())?;

    let query_embedding = engine
        .embed(message)
        .map_err(|e| format!("Failed to embed query: {e}"))?;
    drop(engine_lock);

    let results = db
        .search_similar_chunks(&query_embedding, 10, label_ids, date_from, date_to)
        .map_err(|e| e.to_string())?;

    if results.is_empty() {
        return Ok((
            "I couldn't find any relevant transcript passages for your question. Try adjusting your filters or asking a different question.".into(),
            Vec::new(),
        ));
    }

    let mut context_parts = Vec::new();
    let mut sources = Vec::new();
    for result in &results {
        let timestamp = crate::summarization::format_ms(result.start_ms);
        context_parts.push(format!(
            "---\n[Meeting: \"{}\", {}]\n{}\n",
            result.meeting_title, timestamp, result.chunk_text
        ));
        sources.push(serde_json::json!({
            "meeting_id": result.meeting_id,
            "meeting_title": result.meeting_title,
            "start_ms": result.start_ms,
            "end_ms": result.end_ms,
        }));
    }

    let system_prompt = format!(
        "You are Nootle, an AI assistant that answers questions about the user's meetings.\n\n\
         Below are relevant excerpts from the user's meeting transcripts. Each excerpt \
         includes the meeting title and timestamp. Use ONLY these excerpts to answer.\n\
         When you reference information, cite the source as [Meeting Title, timestamp].\n\n\
         {}\n\
         Answer the user's question based on these excerpts. Be concise.",
        context_parts.join("\n")
    );

    let mut messages = vec![ChatMessage {
        role: "system".into(),
        content: system_prompt,
    }];
    messages.extend(history);
    messages.push(ChatMessage {
        role: "user".into(),
        content: message.to_string(),
    });

    let llm_provider = llm
        .get_provider(provider)
        .ok_or_else(|| format!("Provider '{}' not found", provider))?;
    let response = llm_provider
        .chat(messages, model)
        .await
        .map_err(|e| e.to_string())?;

    Ok((response, sources))
}

#[tauri::command]
pub async fn embed_meeting_cmd(
    db: State<'_, DbState>,
    embedding_state: State<'_, EmbeddingState>,
    meeting_id: String,
) -> Result<usize, String> {
    let mut engine_lock = embedding_state.lock().await;
    let engine = engine_lock
        .as_mut()
        .ok_or_else(|| "Embedding model not loaded".to_string())?;
    crate::chunking::embed_meeting(&db, engine, &meeting_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn embed_all_meetings(
    app: tauri::AppHandle,
    db: State<'_, DbState>,
    embedding_state: State<'_, EmbeddingState>,
) -> Result<(), String> {
    let meetings = db.list_meetings(None, false).map_err(|e| e.to_string())?;
    let total = meetings.len();

    let mut engine_lock = embedding_state.lock().await;
    let engine = engine_lock
        .as_mut()
        .ok_or_else(|| "Embedding model not loaded".to_string())?;

    for (i, meeting) in meetings.iter().enumerate() {
        match crate::chunking::embed_meeting(&db, engine, &meeting.id) {
            Ok(_) => {}
            Err(e) => tracing::warn!("Failed to embed meeting {}: {e}", meeting.id),
        }
        let _ = app.emit(
            "embedding-progress",
            serde_json::json!({
                "current": i + 1,
                "total": total,
            }),
        );
    }
    Ok(())
}

#[tauri::command]
pub async fn get_embedding_status(
    db: State<'_, DbState>,
    embedding_state: State<'_, EmbeddingState>,
) -> Result<serde_json::Value, String> {
    let (embedded, total) = db.get_embedding_status().map_err(|e| e.to_string())?;
    let engine_lock = embedding_state.lock().await;
    let model_available = engine_lock.is_some();
    Ok(serde_json::json!({
        "embedded": embedded,
        "total": total,
        "model_available": model_available,
    }))
}

#[tauri::command]
pub fn list_insight_types(db: State<'_, DbState>) -> Result<Vec<crate::db::InsightType>, String> {
    db.list_insight_types().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_insight_type(
    db: State<'_, DbState>,
    name: String,
    slug: String,
    description: Option<String>,
    extraction_prompt: String,
    icon: String,
    has_action_fields: bool,
) -> Result<crate::db::InsightType, String> {
    db.create_insight_type(
        &name,
        &slug,
        description.as_deref(),
        &extraction_prompt,
        &icon,
        has_action_fields,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_insight_type(
    db: State<'_, DbState>,
    id: String,
    name: String,
    description: Option<String>,
    extraction_prompt: String,
    icon: String,
    has_action_fields: bool,
) -> Result<crate::db::InsightType, String> {
    db.update_insight_type(
        &id,
        &name,
        description.as_deref(),
        &extraction_prompt,
        &icon,
        has_action_fields,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_insight_type(db: State<'_, DbState>, id: String) -> Result<(), String> {
    db.delete_insight_type(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_insights(
    db: State<'_, DbState>,
    meeting_id: String,
) -> Result<Vec<crate::db::InsightWithActionItem>, String> {
    db.get_insights_for_meeting(&meeting_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_all_insights(
    db: State<'_, DbState>,
    insight_type: Option<String>,
    status: Option<String>,
    search: Option<String>,
) -> Result<Vec<crate::db::InsightWithActionItem>, String> {
    db.get_all_insights(
        insight_type.as_deref(),
        status.as_deref(),
        search.as_deref(),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn extract_meeting_insights(
    app: tauri::AppHandle,
    db: State<'_, DbState>,
    llm: State<'_, LlmState>,
    meeting_id: String,
    provider: String,
    model: String,
) -> Result<(), String> {
    tracing::info!("[DIAG] extract_meeting_insights called: meeting={meeting_id}, provider={provider}, model={model}");
    let llm = llm.read().await;
    let available = llm.provider_names();
    tracing::info!("[DIAG] Available providers: {available:?}");
    match extraction::extract_insights(&db, &llm, &meeting_id, &provider, &model).await {
        Ok(()) => {
            tracing::info!("[DIAG] Extraction succeeded for {meeting_id}");
            let _ = app.emit("insights-updated", &meeting_id);
            Ok(())
        }
        Err(e) => {
            tracing::error!("[DIAG] Extraction FAILED for {meeting_id}: {e:#}");
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn re_extract_meeting_insights(
    app: tauri::AppHandle,
    db: State<'_, DbState>,
    llm: State<'_, LlmState>,
    meeting_id: String,
    provider: String,
    model: String,
) -> Result<(), String> {
    let llm = llm.read().await;
    extraction::re_extract_insights(&db, &llm, &meeting_id, &provider, &model)
        .await
        .map_err(|e| e.to_string())?;
    let _ = app.emit("insights-updated", &meeting_id);
    Ok(())
}

#[tauri::command]
pub fn update_action_item_status(
    db: State<'_, DbState>,
    id: String,
    status: String,
) -> Result<(), String> {
    const VALID_STATUSES: &[&str] = &["open", "done", "cancelled"];
    if !VALID_STATUSES.contains(&status.as_str()) {
        return Err(format!("Invalid action item status: {status}"));
    }
    db.update_action_item_status(&id, &status)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_action_item(
    db: State<'_, DbState>,
    id: String,
    assignee: Option<String>,
    due_date: Option<String>,
) -> Result<(), String> {
    db.update_action_item(&id, assignee.as_deref(), due_date.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_exe_path() -> Result<String, String> {
    std::env::current_exe()
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_app_setting(
    db: State<'_, DbState>,
    key: String,
) -> Result<Option<String>, String> {
    db.get_setting(&key).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_app_setting(
    db: State<'_, DbState>,
    key: String,
    value: String,
) -> Result<(), String> {
    const ALLOWED_SETTING_KEYS: &[&str] = &["denoise_enabled", "detection_enabled"];
    if !ALLOWED_SETTING_KEYS.contains(&key.as_str()) {
        return Err(format!("Invalid setting key: {key}"));
    }
    db.set_setting(&key, &value).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_chat_conversation(db: State<'_, DbState>) -> Result<ChatConversation, String> {
    db.create_chat_conversation().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_chat_conversations(db: State<'_, DbState>) -> Result<Vec<ChatConversation>, String> {
    db.list_chat_conversations().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_chat_conversation(db: State<'_, DbState>, id: String) -> Result<(), String> {
    db.delete_chat_conversation(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_chat_messages(
    db: State<'_, DbState>,
    conversation_id: String,
) -> Result<Vec<crate::db::ChatMessage>, String> {
    db.list_chat_messages(&conversation_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn send_chat_message(
    db: State<'_, DbState>,
    llm: State<'_, LlmState>,
    embedding_state: State<'_, EmbeddingState>,
    conversation_id: String,
    message: String,
    provider: String,
    model: String,
    label_ids: Vec<String>,
    date_from: Option<String>,
    date_to: Option<String>,
) -> Result<serde_json::Value, String> {
    db.create_chat_message(&conversation_id, "user", &message, None)
        .map_err(|e| e.to_string())?;

    let db_messages = db
        .list_chat_messages(&conversation_id)
        .map_err(|e| e.to_string())?;

    // Build history for LLM (excluding the last user message we just added — rag_chat appends it)
    let history: Vec<ChatMessage> = db_messages
        .iter()
        .rev()
        .skip(1) // skip the user msg we just saved
        .rev()
        .map(|m| ChatMessage {
            role: m.role.clone(),
            content: m.content.clone(),
        })
        .collect();

    let llm = llm.read().await;
    let (response, sources) = rag_chat(
        &db,
        &llm,
        &embedding_state,
        &message,
        history,
        &provider,
        &model,
        &label_ids,
        date_from.as_deref(),
        date_to.as_deref(),
    )
    .await?;

    let sources_json = serde_json::to_string(&sources).ok();
    db.create_chat_message(
        &conversation_id,
        "assistant",
        &response,
        sources_json.as_deref(),
    )
    .map_err(|e| e.to_string())?;

    // Auto-title on first message using LLM
    if db_messages.len() <= 1 {
        let fallback = if message.chars().count() > 50 {
            truncate_at_word_boundary(&message, 47, "...")
        } else {
            message.clone()
        };

        let title = if let Some(llm_provider) = llm.get_provider(&provider) {
            let prompt = format!(
                "Generate a short title (max 6 words) for a conversation that starts with this message. \
                 Return ONLY the title, nothing else. No quotes, no punctuation at the end.\n\n{message}"
            );
            let title_messages = vec![crate::llm::ChatMessage {
                role: "user".into(),
                content: prompt,
            }];
            match llm_provider.chat(title_messages, &model).await {
                Ok(resp) => {
                    let t = resp.trim().trim_matches('"').trim().to_string();
                    if t.is_empty() || t.len() > 100 {
                        fallback
                    } else {
                        t
                    }
                }
                Err(_) => fallback,
            }
        } else {
            fallback
        };

        let _ = db.update_chat_conversation_title(&conversation_id, &title);
    }

    db.touch_chat_conversation(&conversation_id)
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "response": response,
        "sources": sources
    }))
}

#[tauri::command]
pub fn update_chat_conversation_title(
    db: State<'_, DbState>,
    id: String,
    title: String,
) -> Result<(), String> {
    db.update_chat_conversation_title(&id, &title)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_meeting_notes(
    db: State<'_, DbState>,
    id: String,
    raw_notes: String,
) -> Result<(), String> {
    db.update_meeting_notes(&id, &raw_notes)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_enriched_notes(
    db: State<'_, DbState>,
    id: String,
    enriched_notes: String,
) -> Result<(), String> {
    db.update_meeting_enriched_notes(&id, &enriched_notes)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn enrich_meeting_notes(
    db: State<'_, DbState>,
    llm: State<'_, LlmState>,
    meeting_id: String,
    provider: String,
    model: String,
) -> Result<String, String> {
    validate_provider(&provider)?;

    let meeting = db.get_meeting(&meeting_id).map_err(|e| e.to_string())?;
    let raw_notes = meeting
        .raw_notes
        .ok_or_else(|| "No notes to enrich".to_string())?;

    // Get transcript text
    let segments = db.get_transcript(&meeting_id).map_err(|e| e.to_string())?;
    let transcript_text: String = segments
        .iter()
        .map(|s| format!("{}: {}", s.speaker_label, s.text))
        .collect::<Vec<_>>()
        .join("\n");

    let system_prompt = format!(
        "You are enriching meeting notes using the full transcript into a single merged document. \
         Output in markdown format. Keep the user's original notes as the backbone structure. \
         Wrap the user's original note content in [[highlight]]...[[/highlight]] markers so it stands out. \
         Expand each note point with details, context, and supporting information from the transcript. \
         The result should read as one cohesive document — not two separate sections. \
         Maintain the same topic order as the original notes.\n\n\
         TRANSCRIPT:\n{}\n\n\
         USER'S NOTES:\n{}",
        transcript_text, raw_notes
    );

    let messages = vec![
        ChatMessage {
            role: "system".into(),
            content: system_prompt,
        },
        ChatMessage {
            role: "user".into(),
            content: "Enrich my notes into a single merged markdown document. Highlight my original notes with [[highlight]]...[[/highlight]] markers and weave in transcript details around them.".into(),
        },
    ];

    let llm = llm.read().await;
    let llm_provider = llm
        .get_provider(&provider)
        .ok_or_else(|| format!("Provider '{}' not found", provider))?;
    let enriched = llm_provider
        .chat(messages, &model)
        .await
        .map_err(|e| e.to_string())?;

    db.update_meeting_enriched_notes(&meeting_id, &enriched)
        .map_err(|e| e.to_string())?;

    Ok(enriched)
}

#[tauri::command]
pub async fn compute_meeting_analytics(
    db: State<'_, DbState>,
    meeting_id: String,
) -> Result<(), String> {
    let speaker_analytics =
        crate::analytics::compute_speaker_analytics(&db, &meeting_id).map_err(|e| e.to_string())?;

    db.save_speaker_analytics(&meeting_id, &speaker_analytics)
        .map_err(|e| e.to_string())?;

    let transcripts = db.get_transcript(&meeting_id).map_err(|e| e.to_string())?;
    let texts: Vec<String> = transcripts.iter().map(|t| t.text.clone()).collect();
    let engagement = crate::analytics::compute_engagement(&meeting_id, &speaker_analytics, &texts);

    db.save_engagement(&engagement).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn compute_meeting_sentiment(
    db: State<'_, DbState>,
    llm: State<'_, LlmState>,
    meeting_id: String,
    provider: String,
    model: String,
) -> Result<(), String> {
    let registry = llm.read().await;
    let segments =
        crate::analytics::analyze_sentiment(&db, &registry, &meeting_id, &provider, &model)
            .await
            .map_err(|e| e.to_string())?;

    db.save_sentiment_segments(&meeting_id, &segments)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_meeting_analytics(
    db: State<'_, DbState>,
    meeting_id: String,
) -> Result<serde_json::Value, String> {
    let speakers = db
        .get_speaker_analytics(&meeting_id)
        .map_err(|e| e.to_string())?;
    let sentiment = db
        .get_sentiment_segments(&meeting_id)
        .map_err(|e| e.to_string())?;
    let engagement = db.get_engagement(&meeting_id).map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "speakers": speakers,
        "sentiment": sentiment,
        "engagement": engagement,
    }))
}

#[tauri::command]
pub fn create_integration(
    db: State<'_, DbState>,
    integration_type: String,
    name: String,
    credentials_json: String,
) -> Result<crate::db::Integration, String> {
    db.create_integration(&integration_type, &name, &credentials_json)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_integrations(db: State<'_, DbState>) -> Result<Vec<crate::db::Integration>, String> {
    db.list_integrations_safe().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_integration(
    db: State<'_, DbState>,
    id: String,
    name: String,
    credentials_json: String,
) -> Result<crate::db::Integration, String> {
    db.update_integration(&id, &name, &credentials_json)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_integration(db: State<'_, DbState>, id: String) -> Result<(), String> {
    db.delete_integration(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_workflow(
    db: State<'_, DbState>,
    name: String,
    description: Option<String>,
    icon: Option<String>,
    integration_id: String,
    action_type: String,
    config_json: String,
) -> Result<crate::db::Workflow, String> {
    db.create_workflow(
        &name,
        description.as_deref(),
        icon.as_deref(),
        &integration_id,
        &action_type,
        &config_json,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_workflows(db: State<'_, DbState>) -> Result<Vec<crate::db::Workflow>, String> {
    db.list_workflows().map_err(|e| e.to_string())
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub fn update_workflow(
    db: State<'_, DbState>,
    id: String,
    name: String,
    description: Option<String>,
    icon: Option<String>,
    integration_id: String,
    action_type: String,
    config_json: String,
    is_enabled: bool,
) -> Result<crate::db::Workflow, String> {
    db.update_workflow(
        &id,
        &name,
        description.as_deref(),
        icon.as_deref(),
        &integration_id,
        &action_type,
        &config_json,
        is_enabled,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_workflow(db: State<'_, DbState>, id: String) -> Result<(), String> {
    db.delete_workflow(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_workflow_runs(
    db: State<'_, DbState>,
    meeting_id: String,
) -> Result<Vec<crate::db::WorkflowRun>, String> {
    db.list_workflow_runs_for_meeting(&meeting_id)
        .map_err(|e| e.to_string())
}

/// How long to wait for the summary before exporting without one.
const SUMMARY_WAIT_ATTEMPTS: u32 = 30;
const SUMMARY_WAIT_INTERVAL_SECS: u64 = 4;

/// Run every enabled workflow that opted into `auto_run` for this meeting.
///
/// Failures are logged and never propagated: a broken export must not take down
/// the recording that produced it.
async fn run_auto_workflows(
    db: &DbState,
    llm: &LlmState,
    meeting_id: &str,
    provider: &str,
    model: &str,
) {
    let workflows = match db.list_workflows() {
        Ok(w) => w,
        Err(e) => {
            tracing::warn!("Auto-workflows: could not list workflows: {e}");
            return;
        }
    };

    let has_auto = workflows.iter().any(|w| {
        w.is_enabled
            && serde_json::from_str::<serde_json::Value>(&w.config_json)
                .ok()
                .and_then(|c| c.get("auto_run").and_then(|v| v.as_bool()))
                == Some(true)
    });
    if !has_auto {
        return;
    }

    // Summaries are produced by the transcription pipeline, which runs
    // concurrently with extraction -- so by the time we get here the summary
    // may not exist yet, and the export would write "No summary available".
    // Wait for it rather than racing, but never block forever: a meeting whose
    // summary genuinely fails should still export its action items.
    for attempt in 0..SUMMARY_WAIT_ATTEMPTS {
        match db.get_summaries_for_meeting(meeting_id) {
            Ok(s) if !s.is_empty() => break,
            Ok(_) => {
                if attempt + 1 == SUMMARY_WAIT_ATTEMPTS {
                    tracing::warn!(
                        "Auto-workflows: no summary for {meeting_id} after waiting; exporting anyway"
                    );
                }
                tokio::time::sleep(std::time::Duration::from_secs(SUMMARY_WAIT_INTERVAL_SECS)).await;
            }
            Err(e) => {
                tracing::warn!("Auto-workflows: could not read summaries: {e}");
                break;
            }
        }
    }

    for workflow in workflows.into_iter().filter(|w| w.is_enabled) {
        let config: serde_json::Value =
            serde_json::from_str(&workflow.config_json).unwrap_or_else(|_| serde_json::json!({}));
        if config.get("auto_run").and_then(|v| v.as_bool()) != Some(true) {
            continue;
        }

        tracing::info!("Auto-workflow '{}' running for meeting {meeting_id}", workflow.name);
        match run_workflow_inner(
            db,
            llm,
            meeting_id.to_string(),
            workflow.id.clone(),
            Some(provider.to_string()),
            Some(model.to_string()),
        )
        .await
        {
            Ok(_) => tracing::info!("Auto-workflow '{}' succeeded", workflow.name),
            Err(e) => tracing::warn!("Auto-workflow '{}' failed: {e}", workflow.name),
        }
    }
}

#[tauri::command]
pub async fn run_workflow(
    db: State<'_, DbState>,
    llm: State<'_, LlmState>,
    meeting_id: String,
    workflow_id: String,
    llm_provider: Option<String>,
    llm_model: Option<String>,
) -> Result<crate::db::WorkflowRun, String> {
    run_workflow_inner(
        db.inner(),
        llm.inner(),
        meeting_id,
        workflow_id,
        llm_provider,
        llm_model,
    )
    .await
}

/// The body of `run_workflow`, callable from background tasks that hold the
/// state Arcs directly rather than Tauri `State` guards.
pub async fn run_workflow_inner(
    db: &DbState,
    llm: &LlmState,
    meeting_id: String,
    workflow_id: String,
    llm_provider: Option<String>,
    llm_model: Option<String>,
) -> Result<crate::db::WorkflowRun, String> {
    let workflow = db.get_workflow(&workflow_id).map_err(|e| e.to_string())?;
    let integration = db
        .get_integration(&workflow.integration_id)
        .map_err(|e| e.to_string())?;
    let meeting = db.get_meeting(&meeting_id).map_err(|e| e.to_string())?;

    let summaries = db
        .get_summaries_for_meeting(&meeting_id)
        .map_err(|e| e.to_string())?;
    let summary_text = summaries.first().map(|s| s.content.clone());

    // If the workflow has a source template configured, use that template's
    // summary. Auto-generate one if it doesn't exist yet (requires LLM).
    let workflow_config: serde_json::Value =
        serde_json::from_str(&workflow.config_json).unwrap_or_else(|_| serde_json::json!({}));
    let configured_template_id = workflow_config
        .get("template_id")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty());

    let template_summary = if let Some(template_id) = configured_template_id {
        match summaries
            .iter()
            .find(|s| s.template_id.as_deref() == Some(template_id))
        {
            Some(existing) => Some(existing.content.clone()),
            None => match (llm_provider.as_deref(), llm_model.as_deref()) {
                (Some(provider), Some(model)) => {
                    let llm_registry = llm.read().await;
                    crate::summarization::summarize_meeting(
                        &db,
                        &llm_registry,
                        &meeting_id,
                        template_id,
                        provider,
                        model,
                    )
                    .await
                    .ok()
                    .map(|s| s.content)
                }
                _ => None,
            },
        }
    } else {
        None
    };

    let insights = db
        .get_insights_for_meeting(&meeting_id)
        .map_err(|e| e.to_string())?;
    let action_items: Vec<crate::workflows::ActionItemContext> = insights
        .iter()
        .filter(|i| i.insight_type == "action_item")
        .map(|i| crate::workflows::ActionItemContext {
            content: i.content.clone(),
            assignee: i.assignee.clone(),
            due_date: i.due_date.clone(),
            context: i.context.clone(),
        })
        .collect();

    let context = crate::workflows::WorkflowContext {
        meeting_title: meeting.title.clone(),
        meeting_date: meeting.start_time.clone(),
        summary: summary_text,
        template_summary,
        action_items,
    };

    let run = db
        .create_workflow_run(&meeting_id, &workflow_id)
        .map_err(|e| e.to_string())?;

    db.update_workflow_run_status(&run.id, "running", None, None)
        .map_err(|e| e.to_string())?;

    let llm_registry = llm.read().await;
    match crate::workflows::execute_workflow(
        &workflow,
        &integration,
        &context,
        Some(&llm_registry),
        llm_provider.as_deref(),
        llm_model.as_deref(),
    )
    .await
    {
        Ok(result) => {
            let result_json = serde_json::to_string(&result).unwrap_or_default();
            db.update_workflow_run_status(&run.id, "completed", Some(&result_json), None)
                .map_err(|e| e.to_string())?;
        }
        Err(e) => {
            db.update_workflow_run_status(&run.id, "failed", None, Some(&e))
                .map_err(|e| e.to_string())?;
        }
    }

    db.get_workflow_run(&run.id).map_err(|e| e.to_string())
}
