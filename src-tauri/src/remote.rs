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
