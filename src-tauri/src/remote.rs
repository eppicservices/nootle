//! Remote control via the `nootle://` URL scheme.
//!
//! Nootle's recording commands are Tauri commands, which means they are
//! reachable only from its own webview. Nothing outside the app can start a
//! recording -- `nootle-cli` is query-only, and there is no listener.
//!
//! That is the gap this closes. An external meeting detector (mic activity plus
//! WebRTC media flow, which is far more reliable than the process-presence
//! check in `detection.rs`) can now drive recording:
//!
//!     open "nootle://record/start?title=Staff%20sync"
//!     open "nootle://record/stop"
//!     open "nootle://record/toggle?title=Ad-hoc%20call"
//!
//! Design notes:
//!
//! * Every action is idempotent in the direction that matters. Starting while
//!   already recording is a no-op, not an error, because a detector that fires
//!   twice must not interrupt a meeting in progress.
//! * Results come back as events (`remote-control-result`) rather than being
//!   swallowed, so the UI can surface a failure and the log has a record.
//! * Unknown paths are logged and ignored. A malformed URL must never panic the
//!   app or leave a half-open recording.

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use url::Url;

use crate::commands::{self, DbState, EmbeddingState, LlmState, RecordingState};

pub const SCHEME: &str = "nootle";

#[derive(Debug, Clone, Serialize)]
pub struct RemoteResult {
    pub action: String,
    pub ok: bool,
    pub message: String,
    /// Present when the action started or stopped a meeting.
    pub meeting_id: Option<String>,
}

impl RemoteResult {
    fn ok(action: &str, message: impl Into<String>, meeting_id: Option<String>) -> Self {
        Self {
            action: action.to_string(),
            ok: true,
            message: message.into(),
            meeting_id,
        }
    }

    fn err(action: &str, message: impl Into<String>) -> Self {
        Self {
            action: action.to_string(),
            ok: false,
            message: message.into(),
            meeting_id: None,
        }
    }
}

/// A default title good enough to identify the meeting later.
///
/// The detector normally supplies one from the calendar. When it cannot -- an
/// unscheduled call -- a timestamp beats "Untitled" for finding it again.
fn fallback_title() -> String {
    format!("Meeting {}", chrono::Local::now().format("%Y-%m-%d %H:%M"))
}

fn query_value(url: &Url, key: &str) -> Option<String> {
    url.query_pairs()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.into_owned())
        .filter(|v| !v.trim().is_empty())
}

/// The action part of the URL, tolerating both `nootle://record/start` and
/// `nootle:///record/start`. In the first form the host carries "record" and
/// the path carries "/start"; in the second the host is empty.
fn action_of(url: &Url) -> String {
    let host = url.host_str().unwrap_or("").to_string();
    let path = url.path().trim_matches('/').to_string();
    match (host.is_empty(), path.is_empty()) {
        (true, true) => String::new(),
        (true, false) => path,
        (false, true) => host,
        (false, false) => format!("{host}/{path}"),
    }
}

async fn is_recording(app: &AppHandle) -> bool {
    let recording = app.state::<RecordingState>();
    let guard = recording.lock().await;
    guard.is_some()
}

async fn do_start(app: &AppHandle, title: Option<String>) -> RemoteResult {
    if is_recording(app).await {
        // Not an error: a detector that fires twice must not disturb a meeting
        // that is already being captured.
        return RemoteResult::ok("start", "Already recording", None);
    }

    let title = title.unwrap_or_else(fallback_title);
    let db = app.state::<DbState>();
    let llm = app.state::<LlmState>();
    let recording = app.state::<RecordingState>();
    let embedding = app.state::<EmbeddingState>();

    match commands::start_recording(
        app.clone(),
        db,
        llm,
        recording,
        embedding,
        title.clone(),
        None,
        None,
    )
    .await
    {
        Ok(meeting) => {
            tracing::info!("remote: started recording '{title}' ({})", meeting.id);
            RemoteResult::ok("start", format!("Recording '{title}'"), Some(meeting.id))
        }
        Err(e) => {
            tracing::error!("remote: start failed: {e}");
            RemoteResult::err("start", e)
        }
    }
}

async fn do_stop(app: &AppHandle) -> RemoteResult {
    if !is_recording(app).await {
        return RemoteResult::ok("stop", "Not recording", None);
    }

    let db = app.state::<DbState>();
    let llm = app.state::<LlmState>();
    let recording = app.state::<RecordingState>();

    match commands::stop_recording(app.clone(), db, llm, recording).await {
        Ok(meeting) => {
            tracing::info!("remote: stopped recording ({})", meeting.id);
            RemoteResult::ok("stop", "Recording stopped", Some(meeting.id))
        }
        Err(e) => {
            tracing::error!("remote: stop failed: {e}");
            RemoteResult::err("stop", e)
        }
    }
}

/// Emit a result and return it, so early exits still surface in the UI.
fn emit(app: &AppHandle, result: RemoteResult) -> RemoteResult {
    let _ = app.emit("remote-control-result", &result);
    result
}

/// Re-run summarisation and extraction over an existing transcript.
async fn reprocess(app: &AppHandle, meeting_id: String) -> RemoteResult {
    let db = app.state::<DbState>();
    let llm = app.state::<LlmState>();

    let (provider, model) = {
        let registry = llm.read().await;
        let names = registry.provider_names();
        let preferred = db
            .get_setting("summarization_provider")
            .unwrap_or(None)
            .filter(|p| names.iter().any(|n| n == p));
        let Some(provider) = preferred.or_else(|| names.first().cloned()) else {
            return RemoteResult::err("reprocess", "no LLM provider available");
        };
        let models = registry.all_models();
        let Some(model) = models.iter().find(|m| m.provider == provider).map(|m| m.id.clone())
        else {
            return RemoteResult::err("reprocess", format!("no model for provider {provider}"));
        };
        (provider, model)
    };

    tracing::info!("remote: reprocessing {meeting_id} with {provider}/{model}");

    let templates = db.list_templates().unwrap_or_default();
    let mut summaries = 0usize;
    for template in templates.iter().filter(|t| t.is_auto_run) {
        let registry = llm.read().await;
        match crate::summarization::summarize_meeting(
            &db, &registry, &meeting_id, &template.id, &provider, &model,
        )
        .await
        {
            Ok(_) => summaries += 1,
            Err(e) => tracing::warn!("reprocess: template {} failed: {e}", template.id),
        }
    }

    let extracted = {
        let registry = llm.read().await;
        crate::extraction::extract_insights(&db, &registry, &meeting_id, &provider, &model).await
    };
    if let Err(e) = extracted {
        return RemoteResult::err("reprocess", format!("extraction failed: {e}"));
    }

    let _ = db.update_meeting_status(&meeting_id, "summarized");
    RemoteResult::ok(
        "reprocess",
        format!("{summaries} summary/summaries + insights regenerated via {provider}"),
        Some(meeting_id),
    )
}

/// Handle one `nootle://` URL. Never panics, never blocks the caller.
pub async fn handle_url(app: AppHandle, raw: String) {
    let url = match Url::parse(&raw) {
        Ok(u) => u,
        Err(e) => {
            tracing::warn!("remote: unparseable url {raw:?}: {e}");
            return;
        }
    };

    if url.scheme() != SCHEME {
        tracing::warn!("remote: ignoring non-{SCHEME} url {raw:?}");
        return;
    }

    let action = action_of(&url);
    let title = query_value(&url, "title");

    let result = match action.as_str() {
        "record/start" | "record" => do_start(&app, title).await,
        "record/stop" => do_stop(&app).await,
        "record/toggle" => {
            if is_recording(&app).await {
                do_stop(&app).await
            } else {
                do_start(&app, title).await
            }
        }
        // macOS only lists an app under Privacy > Screen Recording once it has
        // actually asked. Without this there is nothing to toggle, and system
        // audio -- everyone else's voice -- is silently never captured.
        "permissions/screen" => {
            let granted = crate::permissions::request_screen_recording();
            tracing::info!("remote: screen recording request -> granted={granted}");
            RemoteResult::ok(
                "permissions/screen",
                if granted {
                    "Screen recording granted"
                } else {
                    "Screen recording not granted; enable Nootle under \
                     Privacy & Security > Screen Recording, then restart it"
                },
                None,
            )
        }
        // Recover a meeting whose transcript survived but whose LLM pass did
        // not. That happened for real: launched from a nootle:// URL the app
        // had no provider on PATH, so a 41-minute meeting transcribed fine and
        // then produced no summary, no insights and no export -- with nothing
        // to re-run it short of recording the meeting again.
        //
        //   nootle://meeting/reprocess?id=<meeting id>   (omit id for the latest)
        "meeting/reprocess" => {
            let meeting_id = match query_value(&url, "id") {
                Some(id) => id,
                None => {
                    let db = app.state::<DbState>();
                    match db.list_meetings(None, false) {
                        Ok(list) if !list.is_empty() => list[0].id.clone(),
                        Ok(_) => {
                            emit(&app, RemoteResult::err("reprocess", "no meetings"));
                            return;
                        }
                        Err(e) => {
                            emit(&app, RemoteResult::err("reprocess", e.to_string()));
                            return;
                        }
                    }
                }
            };
            reprocess(&app, meeting_id).await
        }
        "permissions/status" => {
            let mic = crate::permissions::check_microphone();
            let screen = crate::permissions::check_screen_recording();
            RemoteResult::ok(
                "permissions/status",
                format!("microphone={mic} screen_recording={screen}"),
                None,
            )
        }
        "record/status" => {
            let recording = is_recording(&app).await;
            RemoteResult::ok(
                "status",
                if recording { "Recording" } else { "Idle" },
                None,
            )
        }
        other => {
            tracing::warn!("remote: unknown action {other:?} from {raw:?}");
            RemoteResult::err("unknown", format!("Unknown action: {other}"))
        }
    };

    // Surface the outcome instead of swallowing it, so a failed remote start is
    // visible in the UI rather than looking like nothing happened.
    let _ = app.emit("remote-control-result", &result);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(raw: &str) -> Url {
        Url::parse(raw).expect("valid url")
    }

    #[test]
    fn action_from_host_and_path() {
        assert_eq!(action_of(&parse("nootle://record/start")), "record/start");
        assert_eq!(action_of(&parse("nootle://record/stop")), "record/stop");
        assert_eq!(action_of(&parse("nootle://record")), "record");
    }

    #[test]
    fn action_tolerates_triple_slash() {
        // `open` and some launchers normalise nootle://record/start into
        // nootle:///record/start, which puts everything in the path.
        assert_eq!(action_of(&parse("nootle:///record/start")), "record/start");
        assert_eq!(action_of(&parse("nootle:///record/stop")), "record/stop");
    }

    #[test]
    fn title_is_decoded() {
        let url = parse("nootle://record/start?title=Staff%20sync");
        assert_eq!(query_value(&url, "title").as_deref(), Some("Staff sync"));
    }

    #[test]
    fn blank_title_is_treated_as_absent() {
        assert_eq!(query_value(&parse("nootle://record/start?title="), "title"), None);
        assert_eq!(
            query_value(&parse("nootle://record/start?title=%20%20"), "title"),
            None
        );
    }

    #[test]
    fn missing_title_is_absent() {
        assert_eq!(query_value(&parse("nootle://record/start"), "title"), None);
    }

    #[test]
    fn fallback_title_is_identifiable() {
        let title = fallback_title();
        assert!(title.starts_with("Meeting "));
        assert!(title.len() > "Meeting ".len());
    }
}
