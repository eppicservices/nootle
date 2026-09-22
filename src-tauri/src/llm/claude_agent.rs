use super::types::{ChatMessage, LlmProvider, ModelInfo};
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

/// Calls the locally installed `claude` CLI (Claude Agent SDK) so the user's
/// existing Claude subscription handles auth — no API key needed.
pub struct ClaudeAgentProvider {
    binary_path: String,
}

const DISALLOWED_TOOLS: &str =
    "Bash Read Write Edit Glob Grep WebFetch WebSearch Task TodoWrite NotebookEdit";

const CHAT_TIMEOUT: Duration = Duration::from_secs(300);

impl ClaudeAgentProvider {
    pub fn detect() -> Option<Self> {
        // `sh -c "command -v claude"` is not enough: a non-login shell inherits
        // the GUI app's bare PATH, so the CLI is invisible when Nootle is
        // launched from Finder or a nootle:// URL rather than a terminal.
        let path = super::bin_resolve::resolve("claude")?;
        Some(Self { binary_path: path })
    }

    pub fn binary_path(&self) -> &str {
        &self.binary_path
    }
}

#[async_trait::async_trait]
impl LlmProvider for ClaudeAgentProvider {
    fn provider_name(&self) -> &str {
        "claude-agent"
    }

    fn available_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                id: "sonnet".into(),
                name: "Claude Sonnet (subscription)".into(),
                provider: "claude-agent".into(),
            },
            ModelInfo {
                id: "opus".into(),
                name: "Claude Opus (subscription)".into(),
                provider: "claude-agent".into(),
            },
            ModelInfo {
                id: "haiku".into(),
                name: "Claude Haiku (subscription)".into(),
                provider: "claude-agent".into(),
            },
        ]
    }

    async fn chat(&self, messages: Vec<ChatMessage>, model: &str) -> anyhow::Result<String> {
        let system = messages
            .iter()
            .find(|m| m.role == "system")
            .map(|m| m.content.clone());

        let prompt = messages
            .iter()
            .filter(|m| m.role != "system")
            .map(|m| {
                let label = match m.role.as_str() {
                    "assistant" => "Assistant",
                    "user" => "User",
                    other => other,
                };
                format!("{label}: {}", m.content)
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        let mut cmd = Command::new(&self.binary_path);
        cmd.arg("-p")
            .arg("--model")
            .arg(model)
            .arg("--output-format")
            .arg("text")
            .arg("--disallowed-tools")
            .arg(DISALLOWED_TOOLS);

        if let Some(sys) = system {
            cmd.arg("--system-prompt").arg(sys);
        }

        cmd.kill_on_drop(true)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            anyhow::anyhow!("Failed to spawn claude CLI at '{}': {e}", self.binary_path)
        })?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(prompt.as_bytes()).await?;
            stdin.shutdown().await?;
        }

        let output = tokio::time::timeout(CHAT_TIMEOUT, child.wait_with_output())
            .await
            .map_err(|_| anyhow::anyhow!("Claude CLI timed out after {:?}", CHAT_TIMEOUT))??;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!(
                "Claude CLI exited with status {}: {}",
                output.status,
                stderr.trim()
            );
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}
