use crate::db::{Database, NewSummary, Summary, TranscriptSegment};
use crate::llm::{ChatMessage, LlmRegistry};

fn format_transcript(segments: &[TranscriptSegment]) -> String {
    segments
        .iter()
        .map(|s| {
            format!(
                "[{}] {}: {}",
                format_ms(s.start_ms),
                s.speaker_label,
                s.text
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub async fn summarize_meeting(
    db: &Database,
    llm: &LlmRegistry,
    meeting_id: &str,
    template_id: &str,
    provider_name: &str,
    model: &str,
) -> anyhow::Result<Summary> {
    let transcript = db.get_transcript(meeting_id)?;
    if transcript.is_empty() {
        anyhow::bail!("No transcript found for meeting {}", meeting_id);
    }

    let template = db.get_template(template_id)?;

    let transcript_text = format_transcript(&transcript);

    let scratch_notes = db.get_scratch_notes(meeting_id).unwrap_or_default();
    let notes_section = if scratch_notes.is_empty() {
        String::new()
    } else {
        let formatted: Vec<String> = scratch_notes
            .iter()
            .map(|n| format!("- [{}] \"{}\"", format_ms(n.timestamp_ms), n.content))
            .collect();
        format!(
            "\n\nThe user highlighted these moments during the meeting:\n{}\n\nGive extra attention to these highlighted topics in your summary.",
            formatted.join("\n")
        )
    };

    let messages = vec![
        ChatMessage {
            role: "system".into(),
            content: template.prompt,
        },
        ChatMessage {
            role: "user".into(),
            content: format!(
                "Here is the meeting transcript:\n\n{}{}",
                transcript_text, notes_section
            ),
        },
    ];

    let provider = llm
        .get_provider(provider_name)
        .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found", provider_name))?;
    let content = provider.chat(messages, model).await?;

    let summary = db.create_summary(NewSummary {
        meeting_id: meeting_id.to_string(),
        template_id: Some(template_id.to_string()),
        provider: provider_name.to_string(),
        model: model.to_string(),
        content,
    })?;

    Ok(summary)
}

/// Summarizes a finished meeting without user input.
///
/// A template picked for this specific meeting during recording wins; otherwise
/// every template marked auto-run is applied.
pub async fn run_auto_templates(
    db: &Database,
    llm: &LlmRegistry,
    meeting_id: &str,
    provider_name: &str,
    model: &str,
) -> anyhow::Result<Vec<Summary>> {
    let selected = db
        .get_meeting(meeting_id)
        .ok()
        .and_then(|meeting| meeting.template_id)
        .and_then(|id| db.get_template(&id).ok());

    let templates = match selected {
        Some(template) => vec![template],
        None => db.get_auto_run_templates()?,
    };

    let mut summaries = Vec::new();
    for template in templates {
        match summarize_meeting(db, llm, meeting_id, &template.id, provider_name, model).await {
            Ok(summary) => summaries.push(summary),
            Err(e) => tracing::error!("Auto-run template '{}' failed: {}", template.name, e),
        }
    }
    Ok(summaries)
}

pub async fn chat_with_transcript(
    db: &Database,
    llm: &LlmRegistry,
    meeting_id: &str,
    user_message: &str,
    conversation_history: Vec<ChatMessage>,
    provider_name: &str,
    model: &str,
) -> anyhow::Result<String> {
    let transcript = db.get_transcript(meeting_id)?;
    let transcript_text = format_transcript(&transcript);

    let mut messages = vec![ChatMessage {
        role: "system".into(),
        content: format!(
            "You are a helpful assistant that answers questions about a meeting transcript. \
             Here is the full transcript:\n\n{}\n\n\
             Answer the user's questions based on this transcript. \
             Be concise and reference specific parts of the conversation when relevant.",
            transcript_text
        ),
    }];

    messages.extend(conversation_history);

    messages.push(ChatMessage {
        role: "user".into(),
        content: user_message.to_string(),
    });

    let provider = llm
        .get_provider(provider_name)
        .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found", provider_name))?;
    provider.chat(messages, model).await
}

pub async fn run_recipe(
    db: &Database,
    llm: &LlmRegistry,
    meeting_id: &str,
    recipe_id: &str,
    provider_name: &str,
    model: &str,
) -> anyhow::Result<String> {
    let recipe = db.get_recipe(recipe_id)?;
    let meeting = db.get_meeting(meeting_id)?;
    let transcript = db.get_transcript(meeting_id)?;
    let transcript_text = format_transcript(&transcript);

    let mut prompt = recipe.prompt_template.clone();
    prompt = prompt.replace("{{transcript}}", &transcript_text);
    prompt = prompt.replace("{{title}}", &meeting.title);
    prompt = prompt.replace("{{date}}", &meeting.start_time);

    if let Ok(summaries) = db.get_summaries_for_meeting(meeting_id) {
        if let Some(summary) = summaries.first() {
            prompt = prompt.replace("{{summary}}", &summary.content);
        }
    }

    let provider = llm
        .get_provider(provider_name)
        .ok_or_else(|| anyhow::anyhow!("Provider '{}' not found", provider_name))?;
    provider
        .chat(
            vec![
                ChatMessage {
                    role: "system".into(),
                    content: "You are a meeting assistant. Produce the requested output based on the meeting data provided.".into(),
                },
                ChatMessage {
                    role: "user".into(),
                    content: prompt,
                },
            ],
            model,
        )
        .await
}

pub fn format_ms(ms: i64) -> String {
    let total_seconds = ms / 1000;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{:02}:{:02}", minutes, seconds)
}
