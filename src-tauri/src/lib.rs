pub mod analytics;
pub mod audio;
pub mod chunking;
pub mod commands;
pub mod db;
pub mod denoise;
pub mod detection;
pub mod diarization;
pub mod embedding;
pub mod error;
pub mod extraction;
pub mod linear;
pub mod llm;
pub mod mcp;
pub mod model_download;
pub mod model_registry;
pub mod permissions;
pub mod summarization;
pub mod transcription;
pub mod vad;
pub mod workflows;

use commands::{DetectorState, DownloadManagerState, EmbeddingState, LlmState, RecordingState};
use detection::MeetingDetector;
use llm::{CodexCliProvider, LlmRegistry, OllamaProvider};
use model_download::DownloadManager;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tokio::sync::Mutex as TokioMutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let app_dir = dirs::data_dir()
        .expect("Could not determine data directory")
        .join("Nootle");
    std::fs::create_dir_all(&app_dir).unwrap();
    let db_path = app_dir.join("nootle.db");
    let db = std::sync::Arc::new(db::Database::new(db_path.to_str().unwrap()).unwrap());

    // Fix any meetings stuck in "recording" or "transcribing" from a previous crash
    match db.cleanup_stale_recordings() {
        Ok(0) => {}
        Ok(n) => tracing::info!("Cleaned up {n} stale recording(s) from previous session"),
        Err(e) => tracing::warn!("Failed to clean up stale recordings: {e}"),
    }

    let recording_state: RecordingState = Arc::new(TokioMutex::new(None));

    // Initialize LLM registry with available providers
    let mut llm_registry = LlmRegistry::new();

    // Register Ollama only if it's reachable locally
    if std::net::TcpStream::connect_timeout(
        &"127.0.0.1:11434".parse().unwrap(),
        std::time::Duration::from_millis(500),
    )
    .is_ok()
    {
        llm_registry.register(Box::new(OllamaProvider::new()));
    }

    // Register Codex CLI (ChatGPT subscription) only if the `codex` binary is installed
    if CodexCliProvider::is_available() {
        llm_registry.register(Box::new(CodexCliProvider::new()));
    }

    // Auth is delegated to the user's existing Claude subscription, so no API key.
    if let Some(provider) = llm::ClaudeAgentProvider::detect() {
        tracing::info!("Claude Agent SDK detected at {}", provider.binary_path());
        llm_registry.register(Box::new(provider));
    }

    // Register providers with stored API keys.
    if let Ok(Some(key)) = db.get_api_key("openai") {
        llm_registry.register(Box::new(llm::OpenAiProvider::new(key)));
    }
    if let Ok(Some(key)) = db.get_api_key("anthropic") {
        llm_registry.register(Box::new(llm::AnthropicProvider::new(key)));
    }
    if let Ok(Some(key)) = db.get_api_key("google") {
        llm_registry.register(Box::new(llm::GoogleProvider::new(key)));
    }
    if let Ok(Some(key)) = db.get_api_key("groq") {
        llm_registry.register(Box::new(llm::GroqProvider::new(key)));
    }
    if let Ok(Some(key)) = db.get_api_key("openrouter") {
        llm_registry.register(Box::new(llm::OpenRouterProvider::new(key)));
    }
    if let Ok(Some(key)) = db.get_api_key("bedrock") {
        llm_registry.register(Box::new(llm::BedrockProvider::new(key)));
    }
    if let Ok(Some(key)) = db.get_api_key("codex") {
        llm_registry.register(Box::new(llm::CodexProvider::new(key)));
    }

    let llm_state: LlmState = Arc::new(tokio::sync::RwLock::new(llm_registry));

    let detector = Arc::new(std::sync::Mutex::new(MeetingDetector::new()));
    let detector_state: DetectorState = detector.clone();
    let download_manager: DownloadManagerState = Arc::new(TokioMutex::new(DownloadManager::new()));

    let embedding_engine = if crate::embedding::EmbeddingEngine::is_available() {
        match crate::embedding::EmbeddingEngine::load() {
            Ok(e) => Some(e),
            Err(err) => {
                tracing::warn!("Failed to load embedding engine: {err}");
                None
            }
        }
    } else {
        None
    };
    let embedding_state: EmbeddingState = Arc::new(TokioMutex::new(embedding_engine));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .menu(|handle| {
            use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder};
            let app_menu = SubmenuBuilder::new(handle, "Nootle")
                .about(None)
                .separator()
                .services()
                .separator()
                .hide()
                .hide_others()
                .show_all()
                .separator()
                .quit()
                .build()?;
            let edit_menu = SubmenuBuilder::new(handle, "Edit")
                .undo()
                .redo()
                .separator()
                .cut()
                .copy()
                .paste()
                .select_all()
                .build()?;
            let window_menu = SubmenuBuilder::new(handle, "Window")
                .minimize()
                .build()?;
            let help_menu = SubmenuBuilder::new(handle, "Help")
                .item(
                    &MenuItemBuilder::with_id(
                        "check-for-updates",
                        "Check for Updates\u{2026}",
                    )
                    .build(handle)?,
                )
                .build()?;
            MenuBuilder::new(handle)
                .item(&app_menu)
                .item(&edit_menu)
                .item(&window_menu)
                .item(&help_menu)
                .build()
        })
        .on_menu_event(|app, event| {
            if event.id().as_ref() == "check-for-updates" {
                let handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    use tauri_plugin_dialog::DialogExt;
                    use tauri_plugin_updater::UpdaterExt;

                    let updater = match handle.updater() {
                        Ok(u) => u,
                        Err(e) => {
                            log::warn!("Failed to initialize updater: {e}");
                            handle
                                .dialog()
                                .message("Could not check for updates.")
                                .title("Update Error")
                                .kind(tauri_plugin_dialog::MessageDialogKind::Error)
                                .blocking_show();
                            return;
                        }
                    };

                    match updater.check().await {
                        Ok(Some(update)) => {
                            let msg = format!("Version {} is available.", update.version);
                            let should_open = handle
                                .dialog()
                                .message(msg)
                                .title("Update Available")
                                .kind(tauri_plugin_dialog::MessageDialogKind::Info)
                                .buttons(tauri_plugin_dialog::MessageDialogButtons::OkCancelCustom(
                                    "Download".to_string(),
                                    "Later".to_string(),
                                ))
                                .blocking_show();
                            if should_open {
                                use tauri_plugin_opener::OpenerExt;
                                if let Err(e) = handle.opener().open_url(
                                    "https://github.com/michellemayes/nootle/releases/latest",
                                    None::<&str>,
                                ) {
                                    log::warn!("Failed to open releases URL: {e}");
                                }
                            }
                        }
                        Ok(None) => {
                            handle
                                .dialog()
                                .message("You're running the latest version.")
                                .title("No Updates Available")
                                .kind(tauri_plugin_dialog::MessageDialogKind::Info)
                                .blocking_show();
                        }
                        Err(e) => {
                            log::warn!("Update check failed: {e}");
                            handle
                                .dialog()
                                .message("Could not check for updates. Please check your internet connection.")
                                .title("Update Error")
                                .kind(tauri_plugin_dialog::MessageDialogKind::Error)
                                .blocking_show();
                        }
                    }
                });
            }
        })
        .manage(db)
        .manage(recording_state)
        .manage(llm_state)
        .manage(detector_state)
        .manage(download_manager)
        .manage(embedding_state)
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let update_handle = app_handle.clone();
            let detector = detector.clone();
            let db_for_detection = app.state::<Arc<db::Database>>().inner().clone();

            // Spawn polling task for meeting detection
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

                    let detection_enabled = db_for_detection
                        .get_setting("detection_enabled")
                        .unwrap_or(None)
                        .map(|v| v == "true")
                        .unwrap_or(false);

                    if !detection_enabled {
                        continue;
                    }

                    let (newly_detected, should_notify) = {
                        let mut d = detector.lock().unwrap();
                        let newly_detected = d.check();
                        let has_app = d.has_active_meeting_app();

                        // Use process detection as speech proxy for now.
                        // Full VAD with audio sampling will be added later.
                        let has_speech = has_app;

                        let should_notify = d.should_notify(has_app, has_speech);
                        d.reset_session();
                        (newly_detected, should_notify)
                    };

                    if should_notify {
                        let _ = app_handle.emit("meeting-detected-notify", serde_json::json!({
                            "title": "Meeting Detected",
                            "body": "It looks like you're in a meeting. Start recording?",
                        }));
                    }

                    for meeting in newly_detected {
                        let _ = app_handle.emit("meeting-detected", &meeting);
                    }
                }
            });

            // Background update check
            tauri::async_runtime::spawn(async move {
                use tauri_plugin_updater::UpdaterExt;
                if let Ok(updater) = update_handle.updater() {
                    if let Ok(Some(update)) = updater.check().await {
                        let _ = update_handle.emit("update-available", &update.version);
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_meetings,
            commands::get_meeting,
            commands::delete_meeting,
            commands::update_meeting_status,
            commands::update_meeting_title,
            commands::update_meeting_template,
            commands::create_label,
            commands::list_labels,
            commands::update_label,
            commands::delete_label,
            commands::add_meeting_label,
            commands::remove_meeting_label,
            commands::get_meeting_labels,
            commands::get_all_meeting_labels,
            commands::add_scratch_note,
            commands::get_scratch_notes,
            commands::delete_scratch_note,
            commands::get_transcript,
            commands::search_transcripts,
            commands::create_recipe,
            commands::list_recipes,
            commands::update_recipe,
            commands::delete_recipe,
            commands::run_recipe,
            commands::create_template,
            commands::list_templates,
            commands::delete_template,
            commands::update_template,
            commands::get_summaries,
            commands::start_recording,
            commands::stop_recording,
            commands::is_recording,
            commands::get_audio_data,
            commands::store_api_key,
            commands::has_api_key,
            commands::delete_api_key,
            commands::list_stored_providers,
            commands::generate_summary,
            commands::chat_with_meeting,
            commands::list_llm_models,
            commands::list_llm_providers,
            commands::seed_default_prompts,
            commands::list_linear_teams,
            commands::list_linear_projects,
            commands::create_linear_ticket,
            commands::create_ticket_from_action_item,
            commands::get_linear_tickets,
            commands::get_linear_setting,
            commands::set_linear_setting,
            commands::get_available_models,
            commands::get_downloaded_models,
            commands::download_model,
            commands::cancel_download,
            commands::delete_model,
            commands::embed_meeting_cmd,
            commands::embed_all_meetings,
            commands::get_embedding_status,
            commands::list_insight_types,
            commands::create_insight_type,
            commands::update_insight_type,
            commands::delete_insight_type,
            commands::get_insights,
            commands::get_all_insights,
            commands::extract_meeting_insights,
            commands::re_extract_meeting_insights,
            commands::update_action_item_status,
            commands::update_action_item,
            commands::get_exe_path,
            commands::check_permissions,
            commands::request_microphone_permission,
            commands::request_screen_recording_permission,
            commands::request_calendar_permission,
            commands::get_app_setting,
            commands::set_app_setting,
            commands::create_chat_conversation,
            commands::list_chat_conversations,
            commands::delete_chat_conversation,
            commands::list_chat_messages,
            commands::send_chat_message,
            commands::update_chat_conversation_title,
            commands::save_meeting_notes,
            commands::save_enriched_notes,
            commands::enrich_meeting_notes,
            commands::compute_meeting_analytics,
            commands::compute_meeting_sentiment,
            commands::get_meeting_analytics,
            commands::create_integration,
            commands::list_integrations,
            commands::update_integration,
            commands::delete_integration,
            commands::create_workflow,
            commands::list_workflows,
            commands::update_workflow,
            commands::delete_workflow,
            commands::list_workflow_runs,
            commands::run_workflow,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
