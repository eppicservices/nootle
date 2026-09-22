pub mod bin_resolve;
pub mod anthropic;
pub mod bedrock;
pub mod claude_agent;
pub mod codex;
pub mod codex_cli;
pub mod google;
pub mod groq;
pub mod ollama;
pub mod openai;
pub mod openrouter;
pub mod types;

pub use anthropic::AnthropicProvider;
pub use bedrock::BedrockProvider;
pub use claude_agent::ClaudeAgentProvider;
pub use codex::CodexProvider;
pub use codex_cli::CodexCliProvider;
pub use google::GoogleProvider;
pub use groq::GroqProvider;
pub use ollama::OllamaProvider;
pub use openai::OpenAiProvider;
pub use openrouter::OpenRouterProvider;
pub use types::{ChatMessage, LlmProvider, ModelInfo};

pub struct LlmRegistry {
    providers: Vec<Box<dyn LlmProvider>>,
}

impl Default for LlmRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmRegistry {
    pub fn new() -> Self {
        Self { providers: vec![] }
    }

    pub fn register(&mut self, provider: Box<dyn LlmProvider>) {
        self.providers.push(provider);
    }

    pub fn unregister(&mut self, name: &str) {
        self.providers.retain(|p| p.provider_name() != name);
    }

    pub fn get_provider(&self, name: &str) -> Option<&dyn LlmProvider> {
        self.providers
            .iter()
            .find(|p| p.provider_name() == name)
            .map(|p| p.as_ref())
    }

    pub fn all_models(&self) -> Vec<ModelInfo> {
        self.providers
            .iter()
            .flat_map(|p| p.available_models())
            .collect()
    }

    pub fn provider_names(&self) -> Vec<String> {
        self.providers
            .iter()
            .map(|p| p.provider_name().to_string())
            .collect()
    }
}
