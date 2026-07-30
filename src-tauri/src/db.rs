use crate::error::{NootleError, Result};
use rusqlite::{ffi::sqlite3_auto_extension, params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sqlite_vec::sqlite3_vec_init;
use std::sync::{Mutex, MutexGuard};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meeting {
    pub id: String,
    pub title: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub audio_path: Option<String>,
    pub status: String,
    pub calendar_event_id: Option<String>,
    pub raw_notes: Option<String>,
    pub enriched_notes: Option<String>,
    pub template_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewMeeting {
    pub title: String,
    pub calendar_event_id: Option<String>,
    pub template_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptSegment {
    pub id: String,
    pub meeting_id: String,
    pub speaker_label: String,
    pub text: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTranscriptSegment {
    pub meeting_id: String,
    pub speaker_label: String,
    pub text: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub id: String,
    pub name: String,
    pub color: String,
    pub icon: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub description: String,
    pub sections: String,
    pub auto_apply_rules: String,
    pub prompt: String,
    pub is_builtin: bool,
    pub is_favorite: bool,
    pub is_auto_run: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTemplate {
    pub name: String,
    pub description: String,
    pub sections: String,
    pub auto_apply_rules: String,
    pub prompt: String,
    pub is_favorite: bool,
    pub is_auto_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub sections: String,
    pub auto_apply_rules: String,
    pub prompt: String,
    pub is_favorite: bool,
    pub is_auto_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub id: String,
    pub meeting_id: String,
    pub template_id: Option<String>,
    pub provider: String,
    pub model: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSummary {
    pub meeting_id: String,
    pub template_id: Option<String>,
    pub provider: String,
    pub model: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearTicket {
    pub id: String,
    pub summary_id: String,
    pub meeting_id: String,
    pub linear_issue_id: String,
    pub linear_issue_url: String,
    pub linear_identifier: String,
    pub title: String,
    pub team_id: String,
    pub project_id: Option<String>,
    pub created_at: String,
}

pub struct NewLinearTicket<'a> {
    pub summary_id: &'a str,
    pub meeting_id: &'a str,
    pub linear_issue_id: &'a str,
    pub linear_issue_url: &'a str,
    pub linear_identifier: &'a str,
    pub title: &'a str,
    pub team_id: &'a str,
    pub project_id: Option<&'a str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptSearchResult {
    pub meeting_id: String,
    pub meeting_title: String,
    pub speaker_label: String,
    pub text: String,
    pub start_ms: i64,
    pub end_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptChunk {
    pub id: String,
    pub meeting_id: String,
    pub chunk_index: i32,
    pub text: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub speaker_labels: String, // JSON array
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkSearchResult {
    pub chunk_id: String,
    pub meeting_id: String,
    pub meeting_title: String,
    pub chunk_text: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub speaker_labels: String,
    pub distance: f64,
}

/// Convert a f32 slice to a little-endian byte vector for sqlite-vec.
fn f32_slice_to_bytes(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|f| f.to_le_bytes()).collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightType {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub extraction_prompt: String,
    pub icon: String,
    pub has_action_fields: bool,
    pub is_builtin: bool,
    pub sort_order: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    pub id: String,
    pub meeting_id: String,
    #[serde(rename = "type")]
    pub insight_type: String,
    pub content: String,
    pub context: Option<String>,
    pub transcript_start_ms: Option<i64>,
    pub transcript_end_ms: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewInsight {
    pub meeting_id: String,
    pub insight_type: String,
    pub content: String,
    pub context: Option<String>,
    pub transcript_start_ms: Option<i64>,
    pub transcript_end_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub id: String,
    pub insight_id: String,
    pub assignee: Option<String>,
    pub due_date: Option<String>,
    pub status: String,
    pub linear_ticket_id: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewActionItem {
    pub insight_id: String,
    pub assignee: Option<String>,
    pub due_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionRun {
    pub id: String,
    pub meeting_id: String,
    pub provider: String,
    pub model: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightWithActionItem {
    pub id: String,
    pub meeting_id: String,
    #[serde(rename = "type")]
    pub insight_type: String,
    pub content: String,
    pub context: Option<String>,
    pub transcript_start_ms: Option<i64>,
    pub transcript_end_ms: Option<i64>,
    pub created_at: String,
    pub action_item_id: Option<String>,
    pub assignee: Option<String>,
    pub due_date: Option<String>,
    pub status: Option<String>,
    pub linear_ticket_id: Option<String>,
    pub action_item_updated_at: Option<String>,
    pub meeting_title: Option<String>,
    pub meeting_start_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatConversation {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
    pub sources_json: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakerAnalytics {
    pub id: String,
    pub meeting_id: String,
    pub speaker_label: String,
    pub talk_time_ms: i64,
    pub turn_count: i64,
    pub interruption_count: i64,
    pub avg_turn_length_ms: i64,
    pub longest_monologue_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentSegment {
    pub id: String,
    pub meeting_id: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub sentiment: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingEngagement {
    pub id: String,
    pub meeting_id: String,
    pub engagement_level: String,
    pub participation_balance: f64,
    pub question_count: i64,
    pub back_and_forth_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScratchNote {
    pub id: String,
    pub meeting_id: String,
    pub content: String,
    pub timestamp_ms: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub description: String,
    pub slash_command: String,
    pub prompt_template: String,
    pub output_format: String,
    pub is_builtin: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewRecipe {
    pub name: String,
    pub description: String,
    pub slash_command: String,
    pub prompt_template: String,
    pub output_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Integration {
    pub id: String,
    pub integration_type: String,
    pub name: String,
    pub credentials_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub integration_id: String,
    pub action_type: String,
    pub config_json: String,
    pub is_enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRun {
    pub id: String,
    pub meeting_id: String,
    pub workflow_id: String,
    pub status: String,
    pub result_json: Option<String>,
    pub error: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub workflow_name: Option<String>,
    pub workflow_icon: Option<String>,
}

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Register sqlite-vec as an auto-extension so every new connection loads it.
    fn ensure_vec_extension() {
        use std::sync::Once;
        static INIT: Once = Once::new();
        INIT.call_once(|| unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute::<
                *const (),
                unsafe extern "C" fn(
                    *mut rusqlite::ffi::sqlite3,
                    *mut *mut i8,
                    *const rusqlite::ffi::sqlite3_api_routines,
                ) -> i32,
            >(sqlite3_vec_init as *const ())));
        });
    }

    fn lock_conn(&self) -> Result<MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|e| NootleError::Other(format!("Database lock poisoned: {e}")))
    }

    pub fn new(path: &str) -> Result<Self> {
        Self::ensure_vec_extension();
        let conn = Connection::open(path)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.initialize()?;
        Ok(db)
    }

    pub fn new_in_memory() -> Result<Self> {
        Self::ensure_vec_extension();
        let conn = Connection::open_in_memory()?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.initialize()?;
        Ok(db)
    }

    fn initialize(&self) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS labels (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                color TEXT NOT NULL DEFAULT '#6366f1',
                icon TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS meeting_labels (
                meeting_id TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
                label_id TEXT NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
                PRIMARY KEY (meeting_id, label_id)
            );

            CREATE TABLE IF NOT EXISTS meetings (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                start_time TEXT NOT NULL,
                end_time TEXT,
                audio_path TEXT,
                status TEXT NOT NULL DEFAULT 'recording',
                calendar_event_id TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS transcripts (
                id TEXT PRIMARY KEY,
                meeting_id TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
                speaker_label TEXT NOT NULL DEFAULT 'Speaker',
                text TEXT NOT NULL,
                start_ms INTEGER NOT NULL,
                end_ms INTEGER NOT NULL,
                confidence REAL NOT NULL DEFAULT 1.0
            );

            CREATE TABLE IF NOT EXISTS summaries (
                id TEXT PRIMARY KEY,
                meeting_id TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
                prompt_id TEXT,
                provider TEXT NOT NULL,
                model TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS linear_tickets (
                id TEXT PRIMARY KEY,
                summary_id TEXT NOT NULL REFERENCES summaries(id) ON DELETE CASCADE,
                meeting_id TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
                linear_issue_id TEXT NOT NULL,
                linear_issue_url TEXT NOT NULL,
                linear_identifier TEXT NOT NULL,
                title TEXT NOT NULL,
                team_id TEXT NOT NULL,
                project_id TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS api_keys (
                provider TEXT PRIMARY KEY,
                key_value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS linear_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS templates (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                sections TEXT NOT NULL DEFAULT '[]',
                auto_apply_rules TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS transcripts_fts USING fts5(
                text, content='transcripts', content_rowid='rowid'
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS summaries_fts USING fts5(
                content, content='summaries', content_rowid='rowid'
            );

            CREATE TRIGGER IF NOT EXISTS transcripts_ai AFTER INSERT ON transcripts BEGIN
                INSERT INTO transcripts_fts(rowid, text) VALUES (new.rowid, new.text);
            END;
            CREATE TRIGGER IF NOT EXISTS transcripts_ad AFTER DELETE ON transcripts BEGIN
                INSERT INTO transcripts_fts(transcripts_fts, rowid, text) VALUES('delete', old.rowid, old.text);
            END;
            CREATE TRIGGER IF NOT EXISTS summaries_ai AFTER INSERT ON summaries BEGIN
                INSERT INTO summaries_fts(rowid, content) VALUES (new.rowid, new.content);
            END;
            CREATE TRIGGER IF NOT EXISTS summaries_ad AFTER DELETE ON summaries BEGIN
                INSERT INTO summaries_fts(summaries_fts, rowid, content) VALUES('delete', old.rowid, old.content);
            END;

            CREATE TABLE IF NOT EXISTS transcript_chunks (
                id TEXT PRIMARY KEY,
                meeting_id TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
                chunk_index INTEGER NOT NULL,
                text TEXT NOT NULL,
                start_ms INTEGER NOT NULL,
                end_ms INTEGER NOT NULL,
                speaker_labels TEXT NOT NULL DEFAULT '[]'
            );

            CREATE TABLE IF NOT EXISTS embedding_config (
                id TEXT PRIMARY KEY,
                model_name TEXT NOT NULL,
                dimensions INTEGER NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            -- Dimension (384) must match the embedding model output (all-MiniLM-L6-v2).
            -- Changing models requires recreating this table and re-embedding all chunks.
            CREATE VIRTUAL TABLE IF NOT EXISTS chunk_embeddings USING vec0(
                chunk_id TEXT PRIMARY KEY,
                embedding float[384]
            );

            CREATE TABLE IF NOT EXISTS insights (
                id TEXT PRIMARY KEY,
                meeting_id TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
                type TEXT NOT NULL,
                content TEXT NOT NULL,
                context TEXT,
                transcript_start_ms INTEGER,
                transcript_end_ms INTEGER,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS action_items (
                id TEXT PRIMARY KEY,
                insight_id TEXT NOT NULL UNIQUE REFERENCES insights(id) ON DELETE CASCADE,
                assignee TEXT,
                due_date TEXT,
                status TEXT NOT NULL DEFAULT 'open',
                linear_ticket_id TEXT,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS extraction_runs (
                id TEXT PRIMARY KEY,
                meeting_id TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
                provider TEXT NOT NULL,
                model TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'running',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS insights_fts USING fts5(
                content, content='insights', content_rowid='rowid'
            );

            CREATE TRIGGER IF NOT EXISTS insights_ai AFTER INSERT ON insights BEGIN
                INSERT INTO insights_fts(rowid, content) VALUES (new.rowid, new.content);
            END;
            CREATE TRIGGER IF NOT EXISTS insights_ad AFTER DELETE ON insights BEGIN
                INSERT INTO insights_fts(insights_fts, rowid, content) VALUES('delete', old.rowid, old.content);
            END;

            CREATE TABLE IF NOT EXISTS insight_types (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                slug TEXT NOT NULL UNIQUE,
                description TEXT,
                extraction_prompt TEXT NOT NULL,
                icon TEXT DEFAULT 'lightbulb',
                has_action_fields INTEGER DEFAULT 0,
                is_builtin INTEGER DEFAULT 0,
                sort_order INTEGER DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS chat_conversations (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL DEFAULT 'New Conversation',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS chat_messages (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL REFERENCES chat_conversations(id) ON DELETE CASCADE,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                sources_json TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS meeting_analytics (
                id TEXT PRIMARY KEY,
                meeting_id TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
                speaker_label TEXT NOT NULL,
                talk_time_ms INTEGER NOT NULL DEFAULT 0,
                turn_count INTEGER NOT NULL DEFAULT 0,
                interruption_count INTEGER NOT NULL DEFAULT 0,
                avg_turn_length_ms INTEGER NOT NULL DEFAULT 0,
                longest_monologue_ms INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS sentiment_segments (
                id TEXT PRIMARY KEY,
                meeting_id TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
                start_ms INTEGER NOT NULL,
                end_ms INTEGER NOT NULL,
                sentiment TEXT NOT NULL,
                score REAL NOT NULL DEFAULT 0.0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS meeting_engagement (
                id TEXT PRIMARY KEY,
                meeting_id TEXT NOT NULL UNIQUE REFERENCES meetings(id) ON DELETE CASCADE,
                engagement_level TEXT NOT NULL DEFAULT 'medium',
                participation_balance REAL NOT NULL DEFAULT 0.0,
                question_count INTEGER NOT NULL DEFAULT 0,
                back_and_forth_ratio REAL NOT NULL DEFAULT 0.0
            );

            CREATE TABLE IF NOT EXISTS scratch_notes (
                id TEXT PRIMARY KEY,
                meeting_id TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
                content TEXT NOT NULL,
                timestamp_ms INTEGER NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS recipes (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                slash_command TEXT NOT NULL UNIQUE,
                prompt_template TEXT NOT NULL,
                output_format TEXT NOT NULL DEFAULT 'markdown',
                is_builtin INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS integrations (
                id TEXT PRIMARY KEY,
                integration_type TEXT NOT NULL,
                name TEXT NOT NULL,
                credentials_json TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS workflows (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                icon TEXT,
                integration_id TEXT NOT NULL REFERENCES integrations(id) ON DELETE CASCADE,
                action_type TEXT NOT NULL,
                config_json TEXT NOT NULL DEFAULT '{}',
                is_enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS workflow_runs (
                id TEXT PRIMARY KEY,
                meeting_id TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
                workflow_id TEXT NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
                status TEXT NOT NULL DEFAULT 'pending',
                result_json TEXT,
                error TEXT,
                started_at TEXT NOT NULL DEFAULT (datetime('now')),
                completed_at TEXT
            );
            ",
        )?;
        Self::seed_default_insight_types(&conn)?;
        Self::refresh_builtin_insight_prompts(&conn)?;

        // Migrations: add notes columns to meetings
        let _ = conn.execute("ALTER TABLE meetings ADD COLUMN raw_notes TEXT", []);
        let _ = conn.execute("ALTER TABLE meetings ADD COLUMN enriched_notes TEXT", []);

        // Migration: add description, prompt, is_builtin to templates (idempotent)
        Self::run_template_migrations(&conn)?;

        // Migration: add template_id to meetings (idempotent)
        let _ = conn.execute(
            "ALTER TABLE meetings ADD COLUMN template_id TEXT REFERENCES templates(id)",
            [],
        );

        Self::seed_builtin_templates(&conn)?;

        // Migration: tags+categories → labels
        let has_labels: bool = conn
            .prepare("SELECT 1 FROM sqlite_master WHERE type='table' AND name='labels'")
            .and_then(|mut s| s.exists([]))
            .unwrap_or(false);

        if !has_labels {
            conn.execute_batch("
                -- Create labels table
                CREATE TABLE labels (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL UNIQUE,
                    color TEXT NOT NULL DEFAULT '#6366f1',
                    icon TEXT,
                    created_at TEXT NOT NULL DEFAULT (datetime('now'))
                );

                CREATE TABLE meeting_labels (
                    meeting_id TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
                    label_id TEXT NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
                    PRIMARY KEY (meeting_id, label_id)
                );

                -- Migrate tags → labels
                INSERT OR IGNORE INTO labels (id, name, color, created_at)
                    SELECT id, name, color, created_at FROM tags;

                -- Migrate categories → labels (with icon)
                INSERT OR IGNORE INTO labels (id, name, color, icon, created_at)
                    SELECT id, name, color, icon, created_at FROM categories;

                -- Migrate meeting_tags → meeting_labels
                INSERT OR IGNORE INTO meeting_labels (meeting_id, label_id)
                    SELECT meeting_id, tag_id FROM meeting_tags;

                -- Migrate category_id assignments → meeting_labels
                INSERT OR IGNORE INTO meeting_labels (meeting_id, label_id)
                    SELECT id, category_id FROM meetings WHERE category_id IS NOT NULL;

                -- Recreate meetings without category_id
                CREATE TABLE meetings_new (
                    id TEXT PRIMARY KEY,
                    title TEXT NOT NULL,
                    start_time TEXT NOT NULL,
                    end_time TEXT,
                    audio_path TEXT,
                    status TEXT NOT NULL DEFAULT 'recording',
                    calendar_event_id TEXT,
                    raw_notes TEXT,
                    enriched_notes TEXT,
                    template_id TEXT,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
                );
                INSERT INTO meetings_new SELECT id, title, start_time, end_time, audio_path, status, calendar_event_id, raw_notes, enriched_notes, template_id, created_at, updated_at FROM meetings;
                DROP TABLE meetings;
                ALTER TABLE meetings_new RENAME TO meetings;

                -- Recreate templates without category_id
                CREATE TABLE templates_new (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    description TEXT NOT NULL DEFAULT '',
                    sections TEXT NOT NULL DEFAULT '[]',
                    auto_apply_rules TEXT NOT NULL DEFAULT '{}',
                    prompt TEXT NOT NULL DEFAULT '',
                    is_builtin INTEGER NOT NULL DEFAULT 0,
                    is_favorite INTEGER NOT NULL DEFAULT 0,
                    is_auto_run INTEGER NOT NULL DEFAULT 0,
                    created_at TEXT NOT NULL DEFAULT (datetime('now'))
                );
                INSERT INTO templates_new SELECT id, name, description, sections, auto_apply_rules, prompt, is_builtin, is_favorite, is_auto_run, created_at FROM templates;
                DROP TABLE templates;
                ALTER TABLE templates_new RENAME TO templates;

                -- Drop old tables
                DROP TABLE IF EXISTS meeting_tags;
                DROP TABLE IF EXISTS tags;
                DROP TABLE IF EXISTS categories;
            ")?;
        }

        Self::seed_builtin_recipes(&conn)?;

        Ok(())
    }

    /// Replace extraction_prompt on built-in insight types if it still matches
    /// an older default. Custom user-edited prompts are left alone.
    fn refresh_builtin_insight_prompts(conn: &rusqlite::Connection) -> Result<()> {
        let updates: &[(&str, &str, &[&str])] = &[
            (
                "action_item",
                "Extract action items — anything someone said they'd do, follow up on, look into, or send. Phrasing is often casual ('I'll handle that', 'let's circle back', 'TODO: ...'). Capture the task even if no assignee or deadline is named. Include who's responsible (assignee) only if explicitly mentioned.",
                &["Extract action items — tasks that someone needs to do. Include who is responsible (assignee) and any deadline mentioned."],
            ),
            (
                "decision",
                "Extract decisions that were agreed upon during the meeting. Include both formal decisions ('we decided X') and de facto choices implied by what people committed to. Each decision should be a clear statement of what was settled.",
                &["Extract clear decisions that were agreed upon. Each decision should be a definitive statement of what was decided."],
            ),
            (
                "key_moment",
                "Extract key moments — important statements, revelations, surprises, disagreements, or turning points in the discussion that are worth remembering.",
                &["Extract key moments — important statements, revelations, or turning points in the discussion that are worth remembering."],
            ),
        ];
        for (slug, new_prompt, old_prompts) in updates {
            for old in *old_prompts {
                conn.execute(
                    "UPDATE insight_types SET extraction_prompt = ?1 WHERE slug = ?2 AND is_builtin = 1 AND extraction_prompt = ?3",
                    params![new_prompt, slug, old],
                )?;
            }
        }
        Ok(())
    }

    fn seed_default_insight_types(conn: &rusqlite::Connection) -> Result<()> {
        let count: i64 =
            conn.query_row("SELECT COUNT(*) FROM insight_types", [], |row| row.get(0))?;
        if count > 0 {
            return Ok(());
        }
        let now = chrono::Utc::now().to_rfc3339();
        let defaults = [
            ("decision", "Decision", "Decisions made during the meeting", "Extract decisions that were agreed upon during the meeting. Include both formal decisions ('we decided X') and de facto choices implied by what people committed to. Each decision should be a clear statement of what was settled.", "lightbulb", false, true, 0),
            ("action_item", "Action Item", "Tasks assigned to team members", "Extract action items — anything someone said they'd do, follow up on, look into, or send. Phrasing is often casual ('I'll handle that', 'let's circle back', 'TODO: ...'). Capture the task even if no assignee or deadline is named. Include who's responsible (assignee) only if explicitly mentioned.", "list-checks", true, true, 1),
            ("key_moment", "Key Moment", "Important moments worth remembering", "Extract key moments — important statements, revelations, surprises, disagreements, or turning points in the discussion that are worth remembering.", "star", false, true, 2),
        ];
        for (slug, name, desc, prompt, icon, has_action, is_builtin, sort) in defaults {
            let id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO insight_types (id, name, slug, description, extraction_prompt, icon, has_action_fields, is_builtin, sort_order, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![id, name, slug, desc, prompt, icon, has_action as i32, is_builtin as i32, sort, now],
            )?;
        }
        Ok(())
    }

    fn run_template_migrations(conn: &rusqlite::Connection) -> Result<()> {
        let mut stmt = conn.prepare("PRAGMA table_info(templates)")?;
        let existing_columns: std::collections::HashSet<String> = stmt
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<std::result::Result<_, _>>()?;

        let migrations: &[(&str, &str)] = &[
            (
                "description",
                "ALTER TABLE templates ADD COLUMN description TEXT NOT NULL DEFAULT ''",
            ),
            (
                "prompt",
                "ALTER TABLE templates ADD COLUMN prompt TEXT NOT NULL DEFAULT ''",
            ),
            (
                "is_builtin",
                "ALTER TABLE templates ADD COLUMN is_builtin INTEGER NOT NULL DEFAULT 0",
            ),
            (
                "is_favorite",
                "ALTER TABLE templates ADD COLUMN is_favorite INTEGER NOT NULL DEFAULT 0",
            ),
            (
                "is_auto_run",
                "ALTER TABLE templates ADD COLUMN is_auto_run INTEGER NOT NULL DEFAULT 0",
            ),
        ];

        for (column, sql) in migrations {
            if !existing_columns.contains(*column) {
                conn.execute(sql, [])?;
            }
        }

        // Migrate existing prompts into templates (one-time migration)
        let has_prompts_table: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='prompts'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map(|c| c > 0)
            .unwrap_or(false);

        if has_prompts_table {
            let prompt_count: i64 = conn
                .query_row("SELECT COUNT(*) FROM prompts", [], |row| row.get(0))
                .unwrap_or(0);

            if prompt_count > 0 {
                let now = chrono::Utc::now().to_rfc3339();
                let mut prompt_stmt = conn
                    .prepare("SELECT id, name, content, is_favorite, is_auto_run FROM prompts")?;
                let prompts: Vec<(String, String, String, i32, i32)> = prompt_stmt
                    .query_map([], |row| {
                        Ok((
                            row.get(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?,
                        ))
                    })?
                    .collect::<std::result::Result<Vec<_>, _>>()?;

                for (id, name, content, is_favorite, is_auto_run) in prompts {
                    // Skip if a template with this name already exists
                    let exists: bool = conn
                        .query_row(
                            "SELECT COUNT(*) FROM templates WHERE name = ?1",
                            params![name],
                            |row| row.get::<_, i64>(0),
                        )
                        .map(|c| c > 0)
                        .unwrap_or(false);
                    if exists {
                        continue;
                    }
                    conn.execute(
                        "INSERT INTO templates (id, name, description, sections, auto_apply_rules, prompt, is_builtin, is_favorite, is_auto_run, created_at)
                         VALUES (?1, ?2, '', '[]', '{}', ?3, 0, ?4, ?5, ?6)",
                        params![id, name, content, is_favorite, is_auto_run, now],
                    )?;
                }
            }
            conn.execute("DROP TABLE IF EXISTS prompts", [])?;
        }

        // Rebuild summaries if its stored schema still has the dangling FK
        // `prompt_id TEXT REFERENCES prompts(id)` — the prompts table was
        // dropped above but SQLite preserves the original CREATE TABLE
        // statement, so any insert hits "no such table: main.prompts".
        let summaries_sql: Option<String> = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type='table' AND name='summaries'",
                [],
                |row| row.get(0),
            )
            .ok();
        if summaries_sql
            .as_deref()
            .map(|s| s.contains("REFERENCES prompts"))
            .unwrap_or(false)
        {
            // Standard SQLite table rebuild: turn foreign_keys off, do the
            // swap inside a transaction, turn foreign_keys back on. PRAGMA
            // foreign_keys cannot run inside a transaction, so it stays
            // outside the BEGIN/COMMIT. The leading DROP recovers from any
            // pre-existing partial state from earlier failed attempts.
            conn.execute_batch("PRAGMA foreign_keys=OFF;")?;
            let rebuild_result = conn.execute_batch(
                "
                BEGIN;
                DROP TABLE IF EXISTS summaries_new;
                CREATE TABLE summaries_new (
                    id TEXT PRIMARY KEY,
                    meeting_id TEXT NOT NULL REFERENCES meetings(id) ON DELETE CASCADE,
                    prompt_id TEXT,
                    provider TEXT NOT NULL,
                    model TEXT NOT NULL,
                    content TEXT NOT NULL,
                    created_at TEXT NOT NULL DEFAULT (datetime('now'))
                );
                INSERT INTO summaries_new (id, meeting_id, prompt_id, provider, model, content, created_at)
                    SELECT id, meeting_id, prompt_id, provider, model, content, created_at
                    FROM summaries
                    WHERE meeting_id IN (SELECT id FROM meetings);
                DROP TABLE summaries;
                ALTER TABLE summaries_new RENAME TO summaries;
                CREATE TRIGGER IF NOT EXISTS summaries_ai AFTER INSERT ON summaries BEGIN
                    INSERT INTO summaries_fts(rowid, content) VALUES (new.rowid, new.content);
                END;
                CREATE TRIGGER IF NOT EXISTS summaries_ad AFTER DELETE ON summaries BEGIN
                    INSERT INTO summaries_fts(summaries_fts, rowid, content) VALUES('delete', old.rowid, old.content);
                END;
                COMMIT;
                ",
            );
            conn.execute_batch("PRAGMA foreign_keys=ON;")?;
            rebuild_result?;
        }

        Ok(())
    }

    fn seed_builtin_templates(conn: &rusqlite::Connection) -> Result<()> {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM templates WHERE is_builtin = 1",
            [],
            |row| row.get(0),
        )?;
        if count > 0 {
            return Ok(());
        }

        let now = chrono::Utc::now().to_rfc3339();
        let builtins: Vec<(&str, &str, &str, &str)> = vec![
            (
                "General",
                "Default for any meeting",
                "Summarize this meeting transcript. Include key discussion points, decisions made, and action items.",
                r#"["Summary","Key Discussion Points","Decisions","Action Items"]"#,
            ),
            (
                "1-on-1",
                "Manager/report check-ins",
                "Summarize this 1-on-1 meeting. Structure as: Updates, Feedback, Blockers, Action Items.",
                r#"["Updates","Feedback","Blockers","Action Items"]"#,
            ),
            (
                "Daily Standup",
                "Quick team sync",
                "Summarize this standup. For each speaker list: What they did, what they'll do, blockers.",
                r#"["Yesterday","Today","Blockers"]"#,
            ),
            (
                "Sprint Retro",
                "What went well/poorly",
                "Summarize this retrospective. Organize into: What Went Well, What Didn't, Action Items for Improvement.",
                r#"["What Went Well","What Didn't Go Well","Action Items for Improvement"]"#,
            ),
            (
                "Customer Call",
                "External stakeholder meeting",
                "Summarize this customer call. Include: Customer requests, commitments made, follow-ups needed, sentiment.",
                r#"["Customer Requests","Commitments Made","Follow-ups","Sentiment"]"#,
            ),
            (
                "User Interview",
                "UX research session",
                "Summarize this user interview. Capture: Key quotes, pain points, feature requests, surprising insights.",
                r#"["Key Quotes","Pain Points","Feature Requests","Surprising Insights"]"#,
            ),
            (
                "Brainstorm",
                "Ideation session",
                "Summarize this brainstorm session. List: Ideas proposed, ideas with most support, next steps to evaluate.",
                r#"["Ideas Proposed","Ideas with Most Support","Next Steps"]"#,
            ),
            (
                "All-Hands",
                "Company-wide meeting",
                "Summarize this all-hands meeting. Cover: Announcements, Q&A highlights, key decisions, action items.",
                r#"["Announcements","Q&A Highlights","Key Decisions","Action Items"]"#,
            ),
        ];

        for (name, description, prompt, sections) in builtins {
            let id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO templates (id, name, description, sections, auto_apply_rules, prompt, is_builtin, created_at)
                 VALUES (?1, ?2, ?3, ?4, '{}', ?5, 1, ?6)",
                params![id, name, description, sections, prompt, now],
            )?;
        }
        Ok(())
    }

    pub fn list_tables(&self) -> Result<Vec<String>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE '%_fts%' AND name NOT LIKE '%_content%' AND name NOT LIKE '%_docsize%' AND name NOT LIKE '%_data%' AND name NOT LIKE '%_idx%' AND name NOT LIKE '%_config%'"
        )?;
        let tables = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(tables)
    }

    pub fn create_meeting(&self, new: NewMeeting) -> Result<Meeting> {
        let conn = self.lock_conn()?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let status = "recording";

        conn.execute(
            "INSERT INTO meetings (id, title, start_time, status, calendar_event_id, template_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![id, new.title, now, status, new.calendar_event_id, new.template_id, now, now],
        )?;

        Ok(Meeting {
            id,
            title: new.title,
            start_time: now.clone(),
            end_time: None,
            audio_path: None,
            status: status.to_string(),
            calendar_event_id: new.calendar_event_id,
            raw_notes: None,
            enriched_notes: None,
            template_id: new.template_id,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn get_meeting(&self, id: &str) -> Result<Meeting> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, start_time, end_time, audio_path, status, calendar_event_id, raw_notes, enriched_notes, template_id, created_at, updated_at
             FROM meetings WHERE id = ?1",
        )?;

        let meeting = stmt
            .query_row(params![id], |row| {
                Ok(Meeting {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    start_time: row.get(2)?,
                    end_time: row.get(3)?,
                    audio_path: row.get(4)?,
                    status: row.get(5)?,
                    calendar_event_id: row.get(6)?,
                    raw_notes: row.get(7)?,
                    enriched_notes: row.get(8)?,
                    template_id: row.get(9)?,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                })
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    NootleError::Other(format!("Meeting not found: {}", id))
                }
                other => NootleError::Database(other),
            })?;

        Ok(meeting)
    }

    pub fn list_meetings(
        &self,
        search: Option<&str>,
        include_archived: bool,
    ) -> Result<Vec<Meeting>> {
        let conn = self.lock_conn()?;

        let mut sql = String::from(
            "SELECT id, title, start_time, end_time, audio_path, status, calendar_event_id, raw_notes, enriched_notes, template_id, created_at, updated_at
             FROM meetings"
        );
        let mut conditions: Vec<String> = Vec::new();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if !include_archived {
            conditions.push("status != 'archived'".to_string());
        }

        if let Some(query) = search {
            conditions.push(format!(
                "title LIKE ?{} ESCAPE '\\'",
                param_values.len() + 1
            ));
            let escaped = query
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_");
            param_values.push(Box::new(format!("%{}%", escaped)));
        }

        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }

        sql.push_str(" ORDER BY start_time DESC");

        let mut stmt = conn.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let meetings = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(Meeting {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    start_time: row.get(2)?,
                    end_time: row.get(3)?,
                    audio_path: row.get(4)?,
                    status: row.get(5)?,
                    calendar_event_id: row.get(6)?,
                    raw_notes: row.get(7)?,
                    enriched_notes: row.get(8)?,
                    template_id: row.get(9)?,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(meetings)
    }

    pub fn update_meeting_status(&self, id: &str, status: &str) -> Result<()> {
        const VALID_STATUSES: &[&str] = &["recording", "transcribing", "summarized", "archived"];
        if !VALID_STATUSES.contains(&status) {
            return Err(NootleError::Other(format!(
                "Invalid meeting status: {}",
                status
            )));
        }
        let conn = self.lock_conn()?;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE meetings SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status, now, id],
        )?;

        Ok(())
    }

    pub fn update_meeting_title(&self, id: &str, title: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE meetings SET title = ?1, updated_at = ?2 WHERE id = ?3",
            params![title, now, id],
        )?;

        Ok(())
    }

    /// Sets (or clears, with `None`) the summary template this meeting should be
    /// summarized with once transcription finishes.
    pub fn update_meeting_template(&self, id: &str, template_id: Option<&str>) -> Result<()> {
        let conn = self.lock_conn()?;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE meetings SET template_id = ?1, updated_at = ?2 WHERE id = ?3",
            params![template_id, now, id],
        )?;

        Ok(())
    }

    pub fn update_meeting_notes(&self, id: &str, raw_notes: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE meetings SET raw_notes = ?1, updated_at = ?2 WHERE id = ?3",
            params![raw_notes, now, id],
        )?;
        Ok(())
    }

    pub fn update_meeting_enriched_notes(&self, id: &str, enriched_notes: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE meetings SET enriched_notes = ?1, updated_at = ?2 WHERE id = ?3",
            params![enriched_notes, now, id],
        )?;
        Ok(())
    }

    pub fn finalize_meeting(
        &self,
        id: &str,
        end_time: &str,
        audio_path: Option<&str>,
        status: &str,
    ) -> Result<()> {
        let conn = self.lock_conn()?;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE meetings SET end_time = ?1, audio_path = ?2, status = ?3, updated_at = ?4 WHERE id = ?5",
            params![end_time, audio_path, status, now, id],
        )?;

        Ok(())
    }

    /// Fix any meetings stuck in "recording" or "transcribing" status from a previous crash.
    /// Sets them to "summarized" with end_time = now so they appear as completed.
    pub fn cleanup_stale_recordings(&self) -> Result<usize> {
        let conn = self.lock_conn()?;
        let now = chrono::Utc::now().to_rfc3339();

        let count = conn.execute(
            "UPDATE meetings SET status = 'summarized', end_time = COALESCE(end_time, ?1), updated_at = ?1 WHERE status IN ('recording', 'transcribing')",
            params![now],
        )?;

        Ok(count)
    }

    pub fn delete_meeting(&self, id: &str) -> Result<()> {
        let conn = self.lock_conn()?;

        conn.execute("DELETE FROM meetings WHERE id = ?1", params![id])?;

        Ok(())
    }

    pub fn create_label(&self, name: &str, color: &str, icon: Option<&str>) -> Result<Label> {
        let conn = self.lock_conn()?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO labels (id, name, color, icon, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, name, color, icon, now],
        )?;

        Ok(Label {
            id,
            name: name.to_string(),
            color: color.to_string(),
            icon: icon.map(|s| s.to_string()),
            created_at: now,
        })
    }

    pub fn list_labels(&self) -> Result<Vec<Label>> {
        let conn = self.lock_conn()?;
        let mut stmt =
            conn.prepare("SELECT id, name, color, icon, created_at FROM labels ORDER BY name ASC")?;

        let labels = stmt
            .query_map([], |row| {
                Ok(Label {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                    icon: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(labels)
    }

    pub fn update_label(
        &self,
        id: &str,
        name: &str,
        color: &str,
        icon: Option<&str>,
    ) -> Result<Label> {
        let conn = self.lock_conn()?;
        conn.execute(
            "UPDATE labels SET name = ?1, color = ?2, icon = ?3 WHERE id = ?4",
            params![name, color, icon, id],
        )?;
        let mut stmt =
            conn.prepare("SELECT id, name, color, icon, created_at FROM labels WHERE id = ?1")?;
        let label = stmt
            .query_row(params![id], |row| {
                Ok(Label {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                    icon: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    NootleError::Other(format!("Label not found: {}", id))
                }
                other => NootleError::Database(other),
            })?;
        Ok(label)
    }

    pub fn delete_label(&self, id: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute("DELETE FROM labels WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn add_meeting_label(&self, meeting_id: &str, label_id: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT OR IGNORE INTO meeting_labels (meeting_id, label_id) VALUES (?1, ?2)",
            params![meeting_id, label_id],
        )?;
        Ok(())
    }

    pub fn remove_meeting_label(&self, meeting_id: &str, label_id: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            "DELETE FROM meeting_labels WHERE meeting_id = ?1 AND label_id = ?2",
            params![meeting_id, label_id],
        )?;
        Ok(())
    }

    pub fn get_meeting_labels(&self, meeting_id: &str) -> Result<Vec<Label>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT l.id, l.name, l.color, l.icon, l.created_at
             FROM labels l
             INNER JOIN meeting_labels ml ON ml.label_id = l.id
             WHERE ml.meeting_id = ?1
             ORDER BY l.name ASC",
        )?;

        let labels = stmt
            .query_map(params![meeting_id], |row| {
                Ok(Label {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                    icon: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(labels)
    }

    pub fn get_all_meeting_labels(&self) -> Result<Vec<(String, Label)>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT ml.meeting_id, l.id, l.name, l.color, l.icon, l.created_at
             FROM meeting_labels ml
             INNER JOIN labels l ON ml.label_id = l.id
             ORDER BY l.name ASC",
        )?;

        let results = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    Label {
                        id: row.get(1)?,
                        name: row.get(2)?,
                        color: row.get(3)?,
                        icon: row.get(4)?,
                        created_at: row.get(5)?,
                    },
                ))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(results)
    }

    pub fn add_scratch_note(
        &self,
        meeting_id: &str,
        content: &str,
        timestamp_ms: i64,
    ) -> Result<ScratchNote> {
        let conn = self.lock_conn()?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO scratch_notes (id, meeting_id, content, timestamp_ms, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, meeting_id, content, timestamp_ms, now],
        )?;

        Ok(ScratchNote {
            id,
            meeting_id: meeting_id.to_string(),
            content: content.to_string(),
            timestamp_ms,
            created_at: now,
        })
    }

    pub fn get_scratch_notes(&self, meeting_id: &str) -> Result<Vec<ScratchNote>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, meeting_id, content, timestamp_ms, created_at
             FROM scratch_notes
             WHERE meeting_id = ?1
             ORDER BY timestamp_ms ASC",
        )?;

        let notes = stmt
            .query_map(params![meeting_id], |row| {
                Ok(ScratchNote {
                    id: row.get(0)?,
                    meeting_id: row.get(1)?,
                    content: row.get(2)?,
                    timestamp_ms: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(notes)
    }

    pub fn delete_scratch_note(&self, id: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute("DELETE FROM scratch_notes WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn seed_builtin_recipes(conn: &rusqlite::Connection) -> Result<()> {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM recipes WHERE is_builtin = 1",
            [],
            |row| row.get(0),
        )?;
        if count > 0 {
            return Ok(());
        }

        let now = chrono::Utc::now().to_rfc3339();
        let builtins: Vec<(&str, &str, &str, &str)> = vec![
            (
                "brief",
                "Write Brief",
                "Turn a brainstorm into a structured brief",
                "Based on this meeting transcript:\n\n{{transcript}}\n\nWrite a structured project brief with: Objective, Background, Scope, Key Requirements, Timeline, and Next Steps.",
            ),
            (
                "followup",
                "Draft Follow-up",
                "Draft a follow-up email",
                "Based on this meeting:\n\n{{transcript}}\n\nDraft a concise follow-up email summarizing what was discussed and listing next steps for each participant.",
            ),
            (
                "tickets",
                "Extract Tickets",
                "Extract work items as tickets",
                "From this meeting transcript:\n\n{{transcript}}\n\nExtract discrete work items as tickets. For each: Title, Description, Assignee (if mentioned), Priority (if implied).",
            ),
            (
                "recap",
                "Meeting Recap",
                "Casual recap for Slack",
                "Create a brief, casual recap of this meeting suitable for posting in a Slack channel:\n\n{{transcript}}",
            ),
            (
                "decisions",
                "List Decisions",
                "Extract all decisions made",
                "From this transcript, extract every decision that was made. For each: What was decided, who made the call, and any conditions or caveats:\n\n{{transcript}}",
            ),
        ];

        for (slash_command, name, description, prompt_template) in builtins {
            let id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO recipes (id, name, description, slash_command, prompt_template, output_format, is_builtin, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, 'markdown', 1, ?6, ?7)",
                params![id, name, description, slash_command, prompt_template, now, now],
            )?;
        }
        Ok(())
    }

    pub fn create_recipe(&self, new: NewRecipe) -> Result<Recipe> {
        let conn = self.lock_conn()?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO recipes (id, name, description, slash_command, prompt_template, output_format, is_builtin, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7, ?8)",
            params![id, new.name, new.description, new.slash_command, new.prompt_template, new.output_format, now, now],
        )?;

        Ok(Recipe {
            id,
            name: new.name,
            description: new.description,
            slash_command: new.slash_command,
            prompt_template: new.prompt_template,
            output_format: new.output_format,
            is_builtin: false,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn list_recipes(&self) -> Result<Vec<Recipe>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, slash_command, prompt_template, output_format, is_builtin, created_at, updated_at
             FROM recipes ORDER BY is_builtin DESC, name ASC",
        )?;

        let recipes = stmt
            .query_map([], |row| {
                Ok(Recipe {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    slash_command: row.get(3)?,
                    prompt_template: row.get(4)?,
                    output_format: row.get(5)?,
                    is_builtin: row.get::<_, i32>(6)? != 0,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(recipes)
    }

    pub fn get_recipe(&self, id: &str) -> Result<Recipe> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, slash_command, prompt_template, output_format, is_builtin, created_at, updated_at
             FROM recipes WHERE id = ?1",
        )?;

        let recipe = stmt
            .query_row(params![id], |row| {
                Ok(Recipe {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    slash_command: row.get(3)?,
                    prompt_template: row.get(4)?,
                    output_format: row.get(5)?,
                    is_builtin: row.get::<_, i32>(6)? != 0,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    NootleError::Other(format!("Recipe not found: {}", id))
                }
                other => NootleError::Database(other),
            })?;

        Ok(recipe)
    }

    pub fn get_recipe_by_command(&self, slash_command: &str) -> Result<Recipe> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, slash_command, prompt_template, output_format, is_builtin, created_at, updated_at
             FROM recipes WHERE slash_command = ?1",
        )?;

        let recipe = stmt
            .query_row(params![slash_command], |row| {
                Ok(Recipe {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    slash_command: row.get(3)?,
                    prompt_template: row.get(4)?,
                    output_format: row.get(5)?,
                    is_builtin: row.get::<_, i32>(6)? != 0,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    NootleError::Other(format!("Recipe not found for command: {}", slash_command))
                }
                other => NootleError::Database(other),
            })?;

        Ok(recipe)
    }

    pub fn update_recipe(
        &self,
        id: &str,
        name: &str,
        description: &str,
        slash_command: &str,
        prompt_template: &str,
        output_format: &str,
    ) -> Result<Recipe> {
        let conn = self.lock_conn()?;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE recipes SET name = ?1, description = ?2, slash_command = ?3, prompt_template = ?4, output_format = ?5, updated_at = ?6 WHERE id = ?7",
            params![name, description, slash_command, prompt_template, output_format, now, id],
        )?;

        let mut stmt = conn.prepare(
            "SELECT id, name, description, slash_command, prompt_template, output_format, is_builtin, created_at, updated_at
             FROM recipes WHERE id = ?1",
        )?;

        let recipe = stmt
            .query_row(params![id], |row| {
                Ok(Recipe {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    slash_command: row.get(3)?,
                    prompt_template: row.get(4)?,
                    output_format: row.get(5)?,
                    is_builtin: row.get::<_, i32>(6)? != 0,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    NootleError::Other(format!("Recipe not found: {}", id))
                }
                other => NootleError::Database(other),
            })?;

        Ok(recipe)
    }

    pub fn delete_recipe(&self, id: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute("DELETE FROM recipes WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn create_transcript_segment(
        &self,
        new: NewTranscriptSegment,
    ) -> Result<TranscriptSegment> {
        let conn = self.lock_conn()?;
        let id = uuid::Uuid::new_v4().to_string();

        conn.execute(
            "INSERT INTO transcripts (id, meeting_id, speaker_label, text, start_ms, end_ms, confidence)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                id,
                new.meeting_id,
                new.speaker_label,
                new.text,
                new.start_ms,
                new.end_ms,
                new.confidence
            ],
        )?;

        Ok(TranscriptSegment {
            id,
            meeting_id: new.meeting_id,
            speaker_label: new.speaker_label,
            text: new.text,
            start_ms: new.start_ms,
            end_ms: new.end_ms,
            confidence: new.confidence,
        })
    }

    pub fn get_transcript(&self, meeting_id: &str) -> Result<Vec<TranscriptSegment>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, meeting_id, speaker_label, text, start_ms, end_ms, confidence
             FROM transcripts WHERE meeting_id = ?1 ORDER BY start_ms ASC",
        )?;

        let segments = stmt
            .query_map(params![meeting_id], |row| {
                Ok(TranscriptSegment {
                    id: row.get(0)?,
                    meeting_id: row.get(1)?,
                    speaker_label: row.get(2)?,
                    text: row.get(3)?,
                    start_ms: row.get(4)?,
                    end_ms: row.get(5)?,
                    confidence: row.get(6)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(segments)
    }

    pub fn create_template(&self, new: NewTemplate) -> Result<Template> {
        let conn = self.lock_conn()?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let is_fav = new.is_favorite as i32;
        let is_auto = new.is_auto_run as i32;

        conn.execute(
            "INSERT INTO templates (id, name, description, sections, auto_apply_rules, prompt, is_builtin, is_favorite, is_auto_run, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7, ?8, ?9)",
            params![
                id,
                new.name,
                new.description,
                new.sections,
                new.auto_apply_rules,
                new.prompt,
                is_fav,
                is_auto,
                now
            ],
        )?;

        Ok(Template {
            id,
            name: new.name,
            description: new.description,
            sections: new.sections,
            auto_apply_rules: new.auto_apply_rules,
            prompt: new.prompt,
            is_builtin: false,
            is_favorite: new.is_favorite,
            is_auto_run: new.is_auto_run,
            created_at: now,
        })
    }

    pub fn list_templates(&self) -> Result<Vec<Template>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, sections, auto_apply_rules, prompt, is_builtin, is_favorite, is_auto_run, created_at
             FROM templates ORDER BY is_builtin DESC, created_at DESC",
        )?;

        let templates = stmt
            .query_map([], |row| {
                Ok(Template {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    sections: row.get(3)?,
                    auto_apply_rules: row.get(4)?,
                    prompt: row.get(5)?,
                    is_builtin: row.get::<_, i32>(6)? != 0,
                    is_favorite: row.get::<_, i32>(7)? != 0,
                    is_auto_run: row.get::<_, i32>(8)? != 0,
                    created_at: row.get(9)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(templates)
    }

    pub fn get_template(&self, id: &str) -> Result<Template> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, sections, auto_apply_rules, prompt, is_builtin, is_favorite, is_auto_run, created_at
             FROM templates WHERE id = ?1",
        )?;

        let template = stmt
            .query_row(params![id], |row| {
                Ok(Template {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    sections: row.get(3)?,
                    auto_apply_rules: row.get(4)?,
                    prompt: row.get(5)?,
                    is_builtin: row.get::<_, i32>(6)? != 0,
                    is_favorite: row.get::<_, i32>(7)? != 0,
                    is_auto_run: row.get::<_, i32>(8)? != 0,
                    created_at: row.get(9)?,
                })
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    NootleError::Other(format!("Template not found: {}", id))
                }
                other => NootleError::Database(other),
            })?;

        Ok(template)
    }

    pub fn get_auto_run_templates(&self) -> Result<Vec<Template>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, sections, auto_apply_rules, prompt, is_builtin, is_favorite, is_auto_run, created_at
             FROM templates WHERE is_auto_run = 1 ORDER BY created_at DESC",
        )?;

        let templates = stmt
            .query_map([], |row| {
                Ok(Template {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    sections: row.get(3)?,
                    auto_apply_rules: row.get(4)?,
                    prompt: row.get(5)?,
                    is_builtin: row.get::<_, i32>(6)? != 0,
                    is_favorite: row.get::<_, i32>(7)? != 0,
                    is_auto_run: row.get::<_, i32>(8)? != 0,
                    created_at: row.get(9)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(templates)
    }

    pub fn delete_template(&self, id: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        let is_builtin: bool = conn
            .query_row(
                "SELECT is_builtin FROM templates WHERE id = ?1",
                params![id],
                |row| row.get::<_, i32>(0).map(|v| v != 0),
            )
            .map_err(|_| NootleError::Other("Template not found".to_string()))?;
        if is_builtin {
            return Err(NootleError::Other(
                "Cannot delete built-in templates".to_string(),
            ));
        }
        conn.execute("DELETE FROM templates WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn update_template(&self, params: &UpdateTemplate) -> Result<Template> {
        let conn = self.lock_conn()?;
        let is_fav = params.is_favorite as i32;
        let is_auto = params.is_auto_run as i32;

        conn.execute(
            "UPDATE templates SET name = ?1, description = ?2, sections = ?3, auto_apply_rules = ?4, prompt = ?5, is_favorite = ?6, is_auto_run = ?7 WHERE id = ?8",
            rusqlite::params![params.name, params.description, params.sections, params.auto_apply_rules, params.prompt, is_fav, is_auto, params.id],
        )?;

        let mut stmt = conn.prepare(
            "SELECT id, name, description, sections, auto_apply_rules, prompt, is_builtin, is_favorite, is_auto_run, created_at
             FROM templates WHERE id = ?1",
        )?;

        let template = stmt
            .query_row(rusqlite::params![params.id], |row| {
                Ok(Template {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    sections: row.get(3)?,
                    auto_apply_rules: row.get(4)?,
                    prompt: row.get(5)?,
                    is_builtin: row.get::<_, i32>(6)? != 0,
                    is_favorite: row.get::<_, i32>(7)? != 0,
                    is_auto_run: row.get::<_, i32>(8)? != 0,
                    created_at: row.get(9)?,
                })
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    NootleError::Other(format!("Template not found: {}", params.id))
                }
                other => NootleError::Database(other),
            })?;

        Ok(template)
    }

    pub fn create_summary(&self, new: NewSummary) -> Result<Summary> {
        let conn = self.lock_conn()?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO summaries (id, meeting_id, prompt_id, provider, model, content, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id, new.meeting_id, new.template_id, new.provider, new.model, new.content, now],
        )?;

        Ok(Summary {
            id,
            meeting_id: new.meeting_id,
            template_id: new.template_id,
            provider: new.provider,
            model: new.model,
            content: new.content,
            created_at: now,
        })
    }

    pub fn get_summaries_for_meeting(&self, meeting_id: &str) -> Result<Vec<Summary>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, meeting_id, prompt_id, provider, model, content, created_at
             FROM summaries WHERE meeting_id = ?1 ORDER BY created_at DESC",
        )?;

        let summaries = stmt
            .query_map(params![meeting_id], |row| {
                Ok(Summary {
                    id: row.get(0)?,
                    meeting_id: row.get(1)?,
                    template_id: row.get(2)?,
                    provider: row.get(3)?,
                    model: row.get(4)?,
                    content: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(summaries)
    }

    pub fn create_linear_ticket(&self, params: NewLinearTicket<'_>) -> Result<LinearTicket> {
        let conn = self.lock_conn()?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO linear_tickets (id, summary_id, meeting_id, linear_issue_id, linear_issue_url, linear_identifier, title, team_id, project_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![id, params.summary_id, params.meeting_id, params.linear_issue_id, params.linear_issue_url, params.linear_identifier, params.title, params.team_id, params.project_id, now],
        )?;

        Ok(LinearTicket {
            id,
            summary_id: params.summary_id.to_string(),
            meeting_id: params.meeting_id.to_string(),
            linear_issue_id: params.linear_issue_id.to_string(),
            linear_issue_url: params.linear_issue_url.to_string(),
            linear_identifier: params.linear_identifier.to_string(),
            title: params.title.to_string(),
            team_id: params.team_id.to_string(),
            project_id: params.project_id.map(|s| s.to_string()),
            created_at: now,
        })
    }

    pub fn get_linear_tickets(&self, meeting_id: &str) -> Result<Vec<LinearTicket>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, summary_id, meeting_id, linear_issue_id, linear_issue_url, linear_identifier, title, team_id, project_id, created_at
             FROM linear_tickets WHERE meeting_id = ?1 ORDER BY created_at DESC",
        )?;

        let tickets = stmt
            .query_map(params![meeting_id], |row| {
                Ok(LinearTicket {
                    id: row.get(0)?,
                    summary_id: row.get(1)?,
                    meeting_id: row.get(2)?,
                    linear_issue_id: row.get(3)?,
                    linear_issue_url: row.get(4)?,
                    linear_identifier: row.get(5)?,
                    title: row.get(6)?,
                    team_id: row.get(7)?,
                    project_id: row.get(8)?,
                    created_at: row.get(9)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(tickets)
    }

    pub fn get_summary(&self, id: &str) -> Result<Summary> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, meeting_id, prompt_id, provider, model, content, created_at
             FROM summaries WHERE id = ?1",
        )?;

        stmt.query_row(params![id], |row| {
            Ok(Summary {
                id: row.get(0)?,
                meeting_id: row.get(1)?,
                template_id: row.get(2)?,
                provider: row.get(3)?,
                model: row.get(4)?,
                content: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                NootleError::Other(format!("Summary not found: {}", id))
            }
            other => NootleError::Database(other),
        })
    }

    // --- API Keys (stored in SQLite) ---

    pub fn store_api_key(&self, provider: &str, key: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT INTO api_keys (provider, key_value) VALUES (?1, ?2)
             ON CONFLICT(provider) DO UPDATE SET key_value = excluded.key_value",
            params![provider, key],
        )?;
        Ok(())
    }

    pub fn get_api_key(&self, provider: &str) -> Result<Option<String>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare("SELECT key_value FROM api_keys WHERE provider = ?1")?;
        match stmt.query_row(params![provider], |row| row.get::<_, String>(0)) {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn delete_api_key(&self, provider: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            "DELETE FROM api_keys WHERE provider = ?1",
            params![provider],
        )?;
        Ok(())
    }

    pub fn list_api_key_providers(&self) -> Result<Vec<String>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare("SELECT provider FROM api_keys ORDER BY provider ASC")?;
        let providers = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(providers)
    }

    // --- Linear Settings ---

    pub fn get_linear_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare("SELECT value FROM linear_settings WHERE key = ?1")?;
        match stmt.query_row(params![key], |row| row.get::<_, String>(0)) {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn set_linear_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT INTO linear_settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn delete_linear_setting(&self, key: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute("DELETE FROM linear_settings WHERE key = ?1", params![key])?;
        Ok(())
    }

    // --- FTS5 Search ---

    pub fn search_transcripts(&self, query: &str) -> Result<Vec<TranscriptSearchResult>> {
        let conn = self.lock_conn()?;
        // Sanitize for FTS5: escape internal double quotes and wrap as phrase
        let safe_query = format!("\"{}\"", query.replace('"', "\"\""));
        let mut stmt = conn.prepare(
            "SELECT t.meeting_id, m.title, t.speaker_label, t.text, t.start_ms, t.end_ms
             FROM transcripts_fts fts
             JOIN transcripts t ON t.rowid = fts.rowid
             JOIN meetings m ON m.id = t.meeting_id
             WHERE transcripts_fts MATCH ?1
             ORDER BY fts.rank",
        )?;

        let results = stmt
            .query_map(params![safe_query], |row| {
                Ok(TranscriptSearchResult {
                    meeting_id: row.get(0)?,
                    meeting_title: row.get(1)?,
                    speaker_label: row.get(2)?,
                    text: row.get(3)?,
                    start_ms: row.get(4)?,
                    end_ms: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(results)
    }

    // --- Transcript Chunks & Embeddings ---

    pub fn insert_chunk(&self, chunk: &TranscriptChunk) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT INTO transcript_chunks (id, meeting_id, chunk_index, text, start_ms, end_ms, speaker_labels)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                chunk.id,
                chunk.meeting_id,
                chunk.chunk_index,
                chunk.text,
                chunk.start_ms,
                chunk.end_ms,
                chunk.speaker_labels
            ],
        )?;
        Ok(())
    }

    pub fn insert_chunk_embedding(&self, chunk_id: &str, embedding: &[f32]) -> Result<()> {
        let conn = self.lock_conn()?;
        let bytes = f32_slice_to_bytes(embedding);
        conn.execute(
            "INSERT INTO chunk_embeddings (chunk_id, embedding) VALUES (?1, ?2)",
            params![chunk_id, bytes],
        )?;
        Ok(())
    }

    pub fn search_similar_chunks(
        &self,
        query_embedding: &[f32],
        limit: usize,
        label_ids: &[String],
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> Result<Vec<ChunkSearchResult>> {
        let conn = self.lock_conn()?;
        let query_bytes = f32_slice_to_bytes(query_embedding);

        // First get KNN results from vec0, then join with metadata and apply filters.
        let mut sql = String::from(
            "SELECT ce.chunk_id, tc.meeting_id, m.title, tc.text, tc.start_ms, tc.end_ms, tc.speaker_labels, ce.distance
             FROM chunk_embeddings ce
             JOIN transcript_chunks tc ON tc.id = ce.chunk_id
             JOIN meetings m ON m.id = tc.meeting_id
             WHERE ce.embedding MATCH ?1
             AND ce.k = ?2",
        );

        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        param_values.push(Box::new(query_bytes));
        // Over-fetch 4x when filters are active so post-filter still yields enough results.
        // With strict filters on a small dataset, the caller may receive fewer than `limit` rows.
        let k = if label_ids.is_empty() && date_from.is_none() && date_to.is_none() {
            limit as i64
        } else {
            (limit as i64) * 4
        };
        param_values.push(Box::new(k));

        if !label_ids.is_empty() {
            let placeholders: Vec<String> = label_ids
                .iter()
                .enumerate()
                .map(|(i, _)| format!("?{}", param_values.len() + i + 1))
                .collect();
            sql.push_str(&format!(
                " AND m.id IN (SELECT meeting_id FROM meeting_labels WHERE label_id IN ({}))",
                placeholders.join(", ")
            ));
            for label_id in label_ids {
                param_values.push(Box::new(label_id.clone()));
            }
        }

        if let Some(from) = date_from {
            sql.push_str(&format!(" AND m.start_time >= ?{}", param_values.len() + 1));
            param_values.push(Box::new(from.to_string()));
        }

        if let Some(to) = date_to {
            sql.push_str(&format!(" AND m.start_time <= ?{}", param_values.len() + 1));
            param_values.push(Box::new(to.to_string()));
        }

        let limit_param_idx = param_values.len() + 1;
        sql.push_str(&format!(" ORDER BY ce.distance LIMIT ?{}", limit_param_idx));
        param_values.push(Box::new(limit as i64));

        let mut stmt = conn.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();

        let results = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(ChunkSearchResult {
                    chunk_id: row.get(0)?,
                    meeting_id: row.get(1)?,
                    meeting_title: row.get(2)?,
                    chunk_text: row.get(3)?,
                    start_ms: row.get(4)?,
                    end_ms: row.get(5)?,
                    speaker_labels: row.get(6)?,
                    distance: row.get(7)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(results)
    }

    pub fn get_embedding_status(&self) -> Result<(u32, u32)> {
        let conn = self.lock_conn()?;

        let embedded_count: u32 = conn.query_row(
            "SELECT COUNT(DISTINCT tc.meeting_id) FROM transcript_chunks tc
             JOIN chunk_embeddings ce ON ce.chunk_id = tc.id",
            [],
            |row| row.get(0),
        )?;

        let total_meetings: u32 = conn.query_row(
            "SELECT COUNT(*) FROM meetings WHERE status IN ('transcribing', 'summarized', 'archived')",
            [],
            |row| row.get(0),
        )?;

        Ok((embedded_count, total_meetings))
    }

    pub fn has_meeting_chunks(&self, meeting_id: &str) -> Result<bool> {
        let conn = self.lock_conn()?;
        let count: u32 = conn.query_row(
            "SELECT COUNT(*) FROM transcript_chunks WHERE meeting_id = ?1",
            params![meeting_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn delete_meeting_chunks(&self, meeting_id: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        // Delete embeddings first (referencing chunks), then chunks
        conn.execute(
            "DELETE FROM chunk_embeddings WHERE chunk_id IN (
                SELECT id FROM transcript_chunks WHERE meeting_id = ?1
            )",
            params![meeting_id],
        )?;
        conn.execute(
            "DELETE FROM transcript_chunks WHERE meeting_id = ?1",
            params![meeting_id],
        )?;
        Ok(())
    }

    // --- Insights ---

    pub fn create_insight(&self, new: NewInsight) -> Result<Insight> {
        let conn = self.lock_conn()?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO insights (id, meeting_id, type, content, context, transcript_start_ms, transcript_end_ms, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![id, new.meeting_id, new.insight_type, new.content, new.context, new.transcript_start_ms, new.transcript_end_ms, now],
        )?;
        Ok(Insight {
            id,
            meeting_id: new.meeting_id,
            insight_type: new.insight_type,
            content: new.content,
            context: new.context,
            transcript_start_ms: new.transcript_start_ms,
            transcript_end_ms: new.transcript_end_ms,
            created_at: now,
        })
    }

    pub fn get_insights_for_meeting(&self, meeting_id: &str) -> Result<Vec<InsightWithActionItem>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT i.id, i.meeting_id, i.type, i.content, i.context, i.transcript_start_ms, i.transcript_end_ms, i.created_at,
                    a.id, a.assignee, a.due_date, a.status, a.linear_ticket_id, a.updated_at,
                    m.title, m.start_time
             FROM insights i
             LEFT JOIN action_items a ON a.insight_id = i.id
             LEFT JOIN meetings m ON m.id = i.meeting_id
             WHERE i.meeting_id = ?1
             ORDER BY i.transcript_start_ms ASC NULLS LAST, i.created_at ASC",
        )?;
        let rows = stmt
            .query_map(params![meeting_id], |row| {
                Ok(InsightWithActionItem {
                    id: row.get(0)?,
                    meeting_id: row.get(1)?,
                    insight_type: row.get(2)?,
                    content: row.get(3)?,
                    context: row.get(4)?,
                    transcript_start_ms: row.get(5)?,
                    transcript_end_ms: row.get(6)?,
                    created_at: row.get(7)?,
                    action_item_id: row.get(8)?,
                    assignee: row.get(9)?,
                    due_date: row.get(10)?,
                    status: row.get(11)?,
                    linear_ticket_id: row.get(12)?,
                    action_item_updated_at: row.get(13)?,
                    meeting_title: row.get(14)?,
                    meeting_start_time: row.get(15)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_all_insights(
        &self,
        insight_type: Option<&str>,
        status: Option<&str>,
        search: Option<&str>,
    ) -> Result<Vec<InsightWithActionItem>> {
        let conn = self.lock_conn()?;
        let mut sql = String::from(
            "SELECT i.id, i.meeting_id, i.type, i.content, i.context, i.transcript_start_ms, i.transcript_end_ms, i.created_at,
                    a.id, a.assignee, a.due_date, a.status, a.linear_ticket_id, a.updated_at,
                    m.title, m.start_time
             FROM insights i
             LEFT JOIN action_items a ON a.insight_id = i.id
             LEFT JOIN meetings m ON m.id = i.meeting_id",
        );
        let mut conditions: Vec<String> = Vec::new();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(t) = insight_type {
            conditions.push(format!("i.type = ?{}", param_values.len() + 1));
            param_values.push(Box::new(t.to_string()));
        }
        if let Some(s) = status {
            conditions.push(format!(
                "(a.status = ?{} OR a.status IS NULL)",
                param_values.len() + 1
            ));
            param_values.push(Box::new(s.to_string()));
        }
        if let Some(q) = search {
            // Sanitize for FTS5: escape internal double quotes and wrap as phrase
            let safe_query = format!("\"{}\"", q.replace('"', "\"\""));
            conditions.push(format!(
                "i.rowid IN (SELECT rowid FROM insights_fts WHERE insights_fts MATCH ?{})",
                param_values.len() + 1
            ));
            param_values.push(Box::new(safe_query));
        }
        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }
        sql.push_str(" ORDER BY CASE WHEN a.status = 'open' THEN 0 ELSE 1 END, a.due_date ASC NULLS LAST, i.created_at DESC");

        let mut stmt = conn.prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let rows = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(InsightWithActionItem {
                    id: row.get(0)?,
                    meeting_id: row.get(1)?,
                    insight_type: row.get(2)?,
                    content: row.get(3)?,
                    context: row.get(4)?,
                    transcript_start_ms: row.get(5)?,
                    transcript_end_ms: row.get(6)?,
                    created_at: row.get(7)?,
                    action_item_id: row.get(8)?,
                    assignee: row.get(9)?,
                    due_date: row.get(10)?,
                    status: row.get(11)?,
                    linear_ticket_id: row.get(12)?,
                    action_item_updated_at: row.get(13)?,
                    meeting_title: row.get(14)?,
                    meeting_start_time: row.get(15)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_insight_by_action_item(
        &self,
        action_item_id: &str,
    ) -> Result<InsightWithActionItem> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT i.id, i.meeting_id, i.type, i.content, i.context, i.transcript_start_ms, i.transcript_end_ms, i.created_at,
                    a.id, a.assignee, a.due_date, a.status, a.linear_ticket_id, a.updated_at,
                    m.title, m.start_time
             FROM action_items a
             JOIN insights i ON i.id = a.insight_id
             LEFT JOIN meetings m ON m.id = i.meeting_id
             WHERE a.id = ?1",
        )?;
        let row = stmt
            .query_row(params![action_item_id], |row| {
                Ok(InsightWithActionItem {
                    id: row.get(0)?,
                    meeting_id: row.get(1)?,
                    insight_type: row.get(2)?,
                    content: row.get(3)?,
                    context: row.get(4)?,
                    transcript_start_ms: row.get(5)?,
                    transcript_end_ms: row.get(6)?,
                    created_at: row.get(7)?,
                    action_item_id: row.get(8)?,
                    assignee: row.get(9)?,
                    due_date: row.get(10)?,
                    status: row.get(11)?,
                    linear_ticket_id: row.get(12)?,
                    action_item_updated_at: row.get(13)?,
                    meeting_title: row.get(14)?,
                    meeting_start_time: row.get(15)?,
                })
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    NootleError::Other(format!("Action item not found: {}", action_item_id))
                }
                other => NootleError::Database(other),
            })?;
        Ok(row)
    }

    pub fn delete_insights_for_meeting(&self, meeting_id: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            "DELETE FROM insights WHERE meeting_id = ?1",
            params![meeting_id],
        )?;
        Ok(())
    }

    pub fn create_action_item(&self, new: NewActionItem) -> Result<ActionItem> {
        let conn = self.lock_conn()?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO action_items (id, insight_id, assignee, due_date, status, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'open', ?5)",
            params![id, new.insight_id, new.assignee, new.due_date, now],
        )?;
        Ok(ActionItem {
            id,
            insight_id: new.insight_id,
            assignee: new.assignee,
            due_date: new.due_date,
            status: "open".to_string(),
            linear_ticket_id: None,
            updated_at: now,
        })
    }

    pub fn update_action_item_status(&self, id: &str, status: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE action_items SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status, now, id],
        )?;
        Ok(())
    }

    pub fn update_action_item(
        &self,
        id: &str,
        assignee: Option<&str>,
        due_date: Option<&str>,
    ) -> Result<()> {
        let conn = self.lock_conn()?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE action_items SET assignee = ?1, due_date = ?2, updated_at = ?3 WHERE id = ?4",
            params![assignee, due_date, now, id],
        )?;
        Ok(())
    }

    pub fn set_action_item_linear_ticket(&self, id: &str, linear_ticket_id: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE action_items SET linear_ticket_id = ?1, updated_at = ?2 WHERE id = ?3",
            params![linear_ticket_id, now, id],
        )?;
        Ok(())
    }

    pub fn create_extraction_run(
        &self,
        meeting_id: &str,
        provider: &str,
        model: &str,
    ) -> Result<ExtractionRun> {
        let conn = self.lock_conn()?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO extraction_runs (id, meeting_id, provider, model, status, created_at)
             VALUES (?1, ?2, ?3, ?4, 'running', ?5)",
            params![id, meeting_id, provider, model, now],
        )?;
        Ok(ExtractionRun {
            id,
            meeting_id: meeting_id.to_string(),
            provider: provider.to_string(),
            model: model.to_string(),
            status: "running".to_string(),
            created_at: now,
        })
    }

    pub fn update_extraction_run_status(&self, id: &str, status: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            "UPDATE extraction_runs SET status = ?1 WHERE id = ?2",
            params![status, id],
        )?;
        Ok(())
    }

    // --- Insight Types ---

    pub fn list_insight_types(&self) -> Result<Vec<InsightType>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, slug, description, extraction_prompt, icon, has_action_fields, is_builtin, sort_order, created_at
             FROM insight_types ORDER BY sort_order ASC, created_at ASC",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok(InsightType {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    slug: row.get(2)?,
                    description: row.get(3)?,
                    extraction_prompt: row.get(4)?,
                    icon: row.get(5)?,
                    has_action_fields: row.get::<_, i32>(6)? != 0,
                    is_builtin: row.get::<_, i32>(7)? != 0,
                    sort_order: row.get(8)?,
                    created_at: row.get(9)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn create_insight_type(
        &self,
        name: &str,
        slug: &str,
        description: Option<&str>,
        extraction_prompt: &str,
        icon: &str,
        has_action_fields: bool,
    ) -> Result<InsightType> {
        let conn = self.lock_conn()?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let max_sort: i32 = conn.query_row(
            "SELECT COALESCE(MAX(sort_order), 0) FROM insight_types",
            [],
            |row| row.get(0),
        )?;
        conn.execute(
            "INSERT INTO insight_types (id, name, slug, description, extraction_prompt, icon, has_action_fields, is_builtin, sort_order, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8, ?9)",
            params![id, name, slug, description, extraction_prompt, icon, has_action_fields as i32, max_sort + 1, now],
        )?;
        Ok(InsightType {
            id,
            name: name.to_string(),
            slug: slug.to_string(),
            description: description.map(|s| s.to_string()),
            extraction_prompt: extraction_prompt.to_string(),
            icon: icon.to_string(),
            has_action_fields,
            is_builtin: false,
            sort_order: max_sort + 1,
            created_at: now,
        })
    }

    pub fn update_insight_type(
        &self,
        id: &str,
        name: &str,
        description: Option<&str>,
        extraction_prompt: &str,
        icon: &str,
        has_action_fields: bool,
    ) -> Result<InsightType> {
        let conn = self.lock_conn()?;
        conn.execute(
            "UPDATE insight_types SET name = ?1, description = ?2, extraction_prompt = ?3, icon = ?4, has_action_fields = ?5 WHERE id = ?6",
            params![name, description, extraction_prompt, icon, has_action_fields as i32, id],
        )?;
        let mut stmt = conn.prepare(
            "SELECT id, name, slug, description, extraction_prompt, icon, has_action_fields, is_builtin, sort_order, created_at
             FROM insight_types WHERE id = ?1",
        )?;
        let it = stmt.query_row(params![id], |row| {
            Ok(InsightType {
                id: row.get(0)?,
                name: row.get(1)?,
                slug: row.get(2)?,
                description: row.get(3)?,
                extraction_prompt: row.get(4)?,
                icon: row.get(5)?,
                has_action_fields: row.get::<_, i32>(6)? != 0,
                is_builtin: row.get::<_, i32>(7)? != 0,
                sort_order: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?;
        Ok(it)
    }

    pub fn delete_insight_type(&self, id: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        let is_builtin: i32 = conn
            .query_row(
                "SELECT is_builtin FROM insight_types WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .map_err(|_| NootleError::Other(format!("Insight type not found: {}", id)))?;
        if is_builtin != 0 {
            return Err(NootleError::Other(
                "Cannot delete built-in insight types".into(),
            ));
        }
        conn.execute("DELETE FROM insight_types WHERE id = ?1", params![id])?;
        Ok(())
    }

    // --- App Settings ---

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare("SELECT value FROM app_settings WHERE key = ?1")?;
        let result = stmt.query_row(params![key], |row| row.get::<_, String>(0));
        match result {
            Ok(val) => Ok(Some(val)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    // --- Chat Conversations ---

    pub fn create_chat_conversation(&self) -> Result<ChatConversation> {
        let conn = self.lock_conn()?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO chat_conversations (id, title, created_at, updated_at) VALUES (?1, 'New Conversation', ?2, ?2)",
            params![id, now],
        )?;
        Ok(ChatConversation {
            id,
            title: "New Conversation".into(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn list_chat_conversations(&self) -> Result<Vec<ChatConversation>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, title, created_at, updated_at FROM chat_conversations ORDER BY updated_at DESC",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok(ChatConversation {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    created_at: row.get(2)?,
                    updated_at: row.get(3)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn update_chat_conversation_title(&self, id: &str, title: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            "UPDATE chat_conversations SET title = ?1 WHERE id = ?2",
            params![title, id],
        )?;
        Ok(())
    }

    pub fn touch_chat_conversation(&self, id: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE chat_conversations SET updated_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        Ok(())
    }

    pub fn delete_chat_conversation(&self, id: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute("DELETE FROM chat_conversations WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn create_chat_message(
        &self,
        conversation_id: &str,
        role: &str,
        content: &str,
        sources_json: Option<&str>,
    ) -> Result<ChatMessage> {
        let conn = self.lock_conn()?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO chat_messages (id, conversation_id, role, content, sources_json, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, conversation_id, role, content, sources_json, now],
        )?;
        Ok(ChatMessage {
            id,
            conversation_id: conversation_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            sources_json: sources_json.map(|s| s.to_string()),
            created_at: now,
        })
    }

    pub fn list_chat_messages(&self, conversation_id: &str) -> Result<Vec<ChatMessage>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, conversation_id, role, content, sources_json, created_at FROM chat_messages WHERE conversation_id = ?1 ORDER BY created_at ASC",
        )?;
        let rows = stmt
            .query_map(params![conversation_id], |row| {
                Ok(ChatMessage {
                    id: row.get(0)?,
                    conversation_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    sources_json: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    // --- Meeting Analytics ---

    pub fn save_speaker_analytics(
        &self,
        meeting_id: &str,
        analytics: &[SpeakerAnalytics],
    ) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            "DELETE FROM meeting_analytics WHERE meeting_id = ?1",
            params![meeting_id],
        )?;
        for a in analytics {
            conn.execute(
                "INSERT INTO meeting_analytics (id, meeting_id, speaker_label, talk_time_ms, turn_count, interruption_count, avg_turn_length_ms, longest_monologue_ms) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![a.id, a.meeting_id, a.speaker_label, a.talk_time_ms, a.turn_count, a.interruption_count, a.avg_turn_length_ms, a.longest_monologue_ms],
            )?;
        }
        Ok(())
    }

    pub fn get_speaker_analytics(&self, meeting_id: &str) -> Result<Vec<SpeakerAnalytics>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, meeting_id, speaker_label, talk_time_ms, turn_count, interruption_count, avg_turn_length_ms, longest_monologue_ms FROM meeting_analytics WHERE meeting_id = ?1"
        )?;
        let rows = stmt
            .query_map(params![meeting_id], |row| {
                Ok(SpeakerAnalytics {
                    id: row.get(0)?,
                    meeting_id: row.get(1)?,
                    speaker_label: row.get(2)?,
                    talk_time_ms: row.get(3)?,
                    turn_count: row.get(4)?,
                    interruption_count: row.get(5)?,
                    avg_turn_length_ms: row.get(6)?,
                    longest_monologue_ms: row.get(7)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn save_sentiment_segments(
        &self,
        meeting_id: &str,
        segments: &[SentimentSegment],
    ) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            "DELETE FROM sentiment_segments WHERE meeting_id = ?1",
            params![meeting_id],
        )?;
        for s in segments {
            conn.execute(
                "INSERT INTO sentiment_segments (id, meeting_id, start_ms, end_ms, sentiment, score) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![s.id, s.meeting_id, s.start_ms, s.end_ms, s.sentiment, s.score],
            )?;
        }
        Ok(())
    }

    pub fn get_sentiment_segments(&self, meeting_id: &str) -> Result<Vec<SentimentSegment>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, meeting_id, start_ms, end_ms, sentiment, score FROM sentiment_segments WHERE meeting_id = ?1 ORDER BY start_ms"
        )?;
        let rows = stmt
            .query_map(params![meeting_id], |row| {
                Ok(SentimentSegment {
                    id: row.get(0)?,
                    meeting_id: row.get(1)?,
                    start_ms: row.get(2)?,
                    end_ms: row.get(3)?,
                    sentiment: row.get(4)?,
                    score: row.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn save_engagement(&self, engagement: &MeetingEngagement) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO meeting_engagement (id, meeting_id, engagement_level, participation_balance, question_count, back_and_forth_ratio) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![engagement.id, engagement.meeting_id, engagement.engagement_level, engagement.participation_balance, engagement.question_count, engagement.back_and_forth_ratio],
        )?;
        Ok(())
    }

    pub fn get_engagement(&self, meeting_id: &str) -> Result<Option<MeetingEngagement>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, meeting_id, engagement_level, participation_balance, question_count, back_and_forth_ratio FROM meeting_engagement WHERE meeting_id = ?1"
        )?;
        let result = stmt.query_row(params![meeting_id], |row| {
            Ok(MeetingEngagement {
                id: row.get(0)?,
                meeting_id: row.get(1)?,
                engagement_level: row.get(2)?,
                participation_balance: row.get(3)?,
                question_count: row.get(4)?,
                back_and_forth_ratio: row.get(5)?,
            })
        });
        match result {
            Ok(e) => Ok(Some(e)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn create_integration(
        &self,
        integration_type: &str,
        name: &str,
        credentials_json: &str,
    ) -> Result<Integration> {
        let conn = self.lock_conn()?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO integrations (id, integration_type, name, credentials_json, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, integration_type, name, credentials_json, now],
        )?;

        Ok(Integration {
            id,
            integration_type: integration_type.to_string(),
            name: name.to_string(),
            credentials_json: credentials_json.to_string(),
            created_at: now,
        })
    }

    pub fn get_integration(&self, id: &str) -> Result<Integration> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, integration_type, name, credentials_json, created_at
             FROM integrations WHERE id = ?1",
        )?;

        let integration = stmt
            .query_row(params![id], |row| {
                Ok(Integration {
                    id: row.get(0)?,
                    integration_type: row.get(1)?,
                    name: row.get(2)?,
                    credentials_json: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    NootleError::Other(format!("Integration not found: {}", id))
                }
                other => NootleError::Database(other),
            })?;

        Ok(integration)
    }

    pub fn list_integrations(&self) -> Result<Vec<Integration>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, integration_type, name, credentials_json, created_at
             FROM integrations ORDER BY created_at DESC",
        )?;

        let integrations = stmt
            .query_map([], |row| {
                Ok(Integration {
                    id: row.get(0)?,
                    integration_type: row.get(1)?,
                    name: row.get(2)?,
                    credentials_json: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(integrations)
    }

    pub fn list_integrations_safe(&self) -> Result<Vec<Integration>> {
        let mut integrations = self.list_integrations()?;
        for i in &mut integrations {
            i.credentials_json = String::new();
        }
        Ok(integrations)
    }

    pub fn update_integration(
        &self,
        id: &str,
        name: &str,
        credentials_json: &str,
    ) -> Result<Integration> {
        let conn = self.lock_conn()?;
        conn.execute(
            "UPDATE integrations SET name = ?1, credentials_json = ?2 WHERE id = ?3",
            params![name, credentials_json, id],
        )?;

        let mut stmt = conn.prepare(
            "SELECT id, integration_type, name, credentials_json, created_at
             FROM integrations WHERE id = ?1",
        )?;

        let integration = stmt
            .query_row(params![id], |row| {
                Ok(Integration {
                    id: row.get(0)?,
                    integration_type: row.get(1)?,
                    name: row.get(2)?,
                    credentials_json: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    NootleError::Other(format!("Integration not found: {}", id))
                }
                other => NootleError::Database(other),
            })?;

        Ok(integration)
    }

    pub fn delete_integration(&self, id: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute(
            "DELETE FROM workflow_runs WHERE workflow_id IN (SELECT id FROM workflows WHERE integration_id = ?1)",
            params![id],
        )?;
        conn.execute(
            "DELETE FROM workflows WHERE integration_id = ?1",
            params![id],
        )?;
        conn.execute("DELETE FROM integrations WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_integration_by_type(&self, integration_type: &str) -> Result<Option<Integration>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, integration_type, name, credentials_json, created_at
             FROM integrations WHERE integration_type = ?1 LIMIT 1",
        )?;

        let integration = stmt
            .query_row(params![integration_type], |row| {
                Ok(Integration {
                    id: row.get(0)?,
                    integration_type: row.get(1)?,
                    name: row.get(2)?,
                    credentials_json: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })
            .optional()?;

        Ok(integration)
    }

    pub fn create_workflow(
        &self,
        name: &str,
        description: Option<&str>,
        icon: Option<&str>,
        integration_id: &str,
        action_type: &str,
        config_json: &str,
    ) -> Result<Workflow> {
        let conn = self.lock_conn()?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO workflows (id, name, description, icon, integration_id, action_type, config_json, is_enabled, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8)",
            params![id, name, description, icon, integration_id, action_type, config_json, now],
        )?;

        Ok(Workflow {
            id,
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            icon: icon.map(|s| s.to_string()),
            integration_id: integration_id.to_string(),
            action_type: action_type.to_string(),
            config_json: config_json.to_string(),
            is_enabled: true,
            created_at: now,
        })
    }

    pub fn get_workflow(&self, id: &str) -> Result<Workflow> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, icon, integration_id, action_type, config_json, is_enabled, created_at
             FROM workflows WHERE id = ?1",
        )?;

        let workflow = stmt
            .query_row(params![id], |row| {
                Ok(Workflow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    icon: row.get(3)?,
                    integration_id: row.get(4)?,
                    action_type: row.get(5)?,
                    config_json: row.get(6)?,
                    is_enabled: row.get::<_, i32>(7)? != 0,
                    created_at: row.get(8)?,
                })
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    NootleError::Other(format!("Workflow not found: {}", id))
                }
                other => NootleError::Database(other),
            })?;

        Ok(workflow)
    }

    pub fn list_workflows(&self) -> Result<Vec<Workflow>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, icon, integration_id, action_type, config_json, is_enabled, created_at
             FROM workflows ORDER BY created_at DESC",
        )?;

        let workflows = stmt
            .query_map([], |row| {
                Ok(Workflow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    icon: row.get(3)?,
                    integration_id: row.get(4)?,
                    action_type: row.get(5)?,
                    config_json: row.get(6)?,
                    is_enabled: row.get::<_, i32>(7)? != 0,
                    created_at: row.get(8)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(workflows)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_workflow(
        &self,
        id: &str,
        name: &str,
        description: Option<&str>,
        icon: Option<&str>,
        integration_id: &str,
        action_type: &str,
        config_json: &str,
        is_enabled: bool,
    ) -> Result<Workflow> {
        let conn = self.lock_conn()?;
        conn.execute(
            "UPDATE workflows SET name = ?1, description = ?2, icon = ?3, integration_id = ?4, action_type = ?5, config_json = ?6, is_enabled = ?7 WHERE id = ?8",
            params![
                name,
                description,
                icon,
                integration_id,
                action_type,
                config_json,
                is_enabled as i32,
                id
            ],
        )?;

        let mut stmt = conn.prepare(
            "SELECT id, name, description, icon, integration_id, action_type, config_json, is_enabled, created_at
             FROM workflows WHERE id = ?1",
        )?;

        let workflow = stmt
            .query_row(params![id], |row| {
                Ok(Workflow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    icon: row.get(3)?,
                    integration_id: row.get(4)?,
                    action_type: row.get(5)?,
                    config_json: row.get(6)?,
                    is_enabled: row.get::<_, i32>(7)? != 0,
                    created_at: row.get(8)?,
                })
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    NootleError::Other(format!("Workflow not found: {}", id))
                }
                other => NootleError::Database(other),
            })?;

        Ok(workflow)
    }

    pub fn delete_workflow(&self, id: &str) -> Result<()> {
        let conn = self.lock_conn()?;
        conn.execute("DELETE FROM workflows WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn create_workflow_run(&self, meeting_id: &str, workflow_id: &str) -> Result<WorkflowRun> {
        let conn = self.lock_conn()?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO workflow_runs (id, meeting_id, workflow_id, status, started_at)
             VALUES (?1, ?2, ?3, 'pending', ?4)",
            params![id, meeting_id, workflow_id, now],
        )?;

        Ok(WorkflowRun {
            id,
            meeting_id: meeting_id.to_string(),
            workflow_id: workflow_id.to_string(),
            status: "pending".to_string(),
            result_json: None,
            error: None,
            started_at: now,
            completed_at: None,
            workflow_name: None,
            workflow_icon: None,
        })
    }

    pub fn get_workflow_run(&self, id: &str) -> Result<WorkflowRun> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT wr.id, wr.meeting_id, wr.workflow_id, wr.status, wr.result_json, wr.error, wr.started_at, wr.completed_at, w.name, w.icon
             FROM workflow_runs wr
             LEFT JOIN workflows w ON w.id = wr.workflow_id
             WHERE wr.id = ?1",
        )?;

        let run = stmt
            .query_row(params![id], |row| {
                Ok(WorkflowRun {
                    id: row.get(0)?,
                    meeting_id: row.get(1)?,
                    workflow_id: row.get(2)?,
                    status: row.get(3)?,
                    result_json: row.get(4)?,
                    error: row.get(5)?,
                    started_at: row.get(6)?,
                    completed_at: row.get(7)?,
                    workflow_name: row.get(8)?,
                    workflow_icon: row.get(9)?,
                })
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    NootleError::Other(format!("WorkflowRun not found: {}", id))
                }
                other => NootleError::Database(other),
            })?;

        Ok(run)
    }

    pub fn update_workflow_run_status(
        &self,
        id: &str,
        status: &str,
        result_json: Option<&str>,
        error: Option<&str>,
    ) -> Result<()> {
        let conn = self.lock_conn()?;
        let completed_at = if status == "completed" || status == "failed" {
            Some(chrono::Utc::now().to_rfc3339())
        } else {
            None
        };

        conn.execute(
            "UPDATE workflow_runs SET status = ?1, result_json = ?2, error = ?3, completed_at = ?4 WHERE id = ?5",
            params![status, result_json, error, completed_at, id],
        )?;

        Ok(())
    }

    pub fn list_workflow_runs_for_meeting(&self, meeting_id: &str) -> Result<Vec<WorkflowRun>> {
        let conn = self.lock_conn()?;
        let mut stmt = conn.prepare(
            "SELECT wr.id, wr.meeting_id, wr.workflow_id, wr.status, wr.result_json, wr.error, wr.started_at, wr.completed_at, w.name, w.icon
             FROM workflow_runs wr
             LEFT JOIN workflows w ON w.id = wr.workflow_id
             WHERE wr.meeting_id = ?1
             ORDER BY wr.started_at DESC",
        )?;

        let runs = stmt
            .query_map(params![meeting_id], |row| {
                Ok(WorkflowRun {
                    id: row.get(0)?,
                    meeting_id: row.get(1)?,
                    workflow_id: row.get(2)?,
                    status: row.get(3)?,
                    result_json: row.get(4)?,
                    error: row.get(5)?,
                    started_at: row.get(6)?,
                    completed_at: row.get(7)?,
                    workflow_name: row.get(8)?,
                    workflow_icon: row.get(9)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(runs)
    }
}
