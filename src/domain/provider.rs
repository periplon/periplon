use serde::{Deserialize, Serialize};

/// AI Provider selection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    /// Anthropic Claude (default)
    #[default]
    Claude,
    /// OpenAI Codex
    Codex,
    /// Ollama (local LLM server)
    Ollama,
    /// OpenAI API
    OpenAI,
    /// Anthropic API (direct)
    Anthropic,
    /// Google Gemini API
    Google,
}

impl Provider {
    /// Check if this is a Claude provider
    pub fn is_claude(&self) -> bool {
        matches!(self, Provider::Claude)
    }

    /// Check if this is a Codex provider
    pub fn is_codex(&self) -> bool {
        matches!(self, Provider::Codex)
    }

    /// Check if this provider uses CLI-based execution
    pub fn is_cli_based(&self) -> bool {
        matches!(self, Provider::Claude | Provider::Codex)
    }

    /// Check if this provider uses direct API calls
    pub fn is_api_based(&self) -> bool {
        matches!(
            self,
            Provider::Ollama | Provider::OpenAI | Provider::Anthropic | Provider::Google
        )
    }

    /// Get the CLI binary name for this provider (only for CLI-based providers)
    pub fn cli_binary_name(&self) -> Option<&str> {
        match self {
            Provider::Claude => Some("claude"),
            Provider::Codex => Some("codex"),
            _ => None,
        }
    }

    /// Get the default API endpoint for this provider (only for API-based providers)
    pub fn default_endpoint(&self) -> Option<&str> {
        match self {
            Provider::Ollama => Some("http://localhost:11434"),
            Provider::OpenAI => Some("https://api.openai.com/v1"),
            Provider::Anthropic => Some("https://api.anthropic.com/v1"),
            Provider::Google => Some("https://generativelanguage.googleapis.com/v1beta"),
            _ => None,
        }
    }

    /// Get valid model names for this provider
    pub fn valid_models(&self) -> Vec<&str> {
        match self {
            Provider::Claude => vec![
                "claude-sonnet-4-5",
                "claude-sonnet-4",
                "claude-opus-4",
                "claude-haiku-4",
            ],
            Provider::Codex => vec!["gpt-5-codex", "gpt-5"],
            Provider::Ollama => vec![
                // Common Ollama models
                "llama3.3",
                "llama3.2",
                "llama3.1",
                "qwen2.5",
                "phi4",
                "mistral",
                "mixtral",
                "codellama",
                "deepseek-coder",
                "gemma2",
            ],
            Provider::OpenAI => vec![
                "gpt-4o",
                "gpt-4o-mini",
                "gpt-4-turbo",
                "gpt-4",
                "gpt-3.5-turbo",
                "o1",
                "o1-mini",
            ],
            Provider::Anthropic => vec![
                "claude-3-5-sonnet-20241022",
                "claude-3-5-haiku-20241022",
                "claude-3-opus-20240229",
                "claude-3-sonnet-20240229",
                "claude-3-haiku-20240307",
            ],
            Provider::Google => vec![
                "gemini-2.0-flash-exp",
                "gemini-exp-1206",
                "gemini-2.0-flash-thinking-exp-01-21",
                "gemini-1.5-pro",
                "gemini-1.5-flash",
                "gemini-1.5-flash-8b",
            ],
        }
    }

    /// Validate if a model name is valid for this provider
    /// For Ollama, we accept any model name since users can have custom models
    pub fn is_valid_model(&self, model: &str) -> bool {
        match self {
            Provider::Ollama => true, // Accept any model for Ollama
            _ => self.valid_models().contains(&model),
        }
    }

    /// Get default model for this provider
    pub fn default_model(&self) -> &str {
        match self {
            Provider::Claude => "claude-sonnet-4-5",
            Provider::Codex => "gpt-5-codex",
            Provider::Ollama => "olmo2:13b",
            Provider::OpenAI => "gpt-4o",
            Provider::Anthropic => "claude-3-5-sonnet-20241022",
            Provider::Google => "gemini-2.0-flash-exp",
        }
    }

    /// Get the environment variable name for the API key
    pub fn api_key_env_var(&self) -> Option<&str> {
        match self {
            Provider::OpenAI => Some("OPENAI_API_KEY"),
            Provider::Anthropic => Some("ANTHROPIC_API_KEY"),
            Provider::Google => Some("GOOGLE_API_KEY"),
            Provider::Ollama => None, // Ollama doesn't require API key by default
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_default() {
        assert_eq!(Provider::default(), Provider::Claude);
    }

    #[test]
    fn test_provider_cli_binary_name() {
        assert_eq!(Provider::Claude.cli_binary_name(), Some("claude"));
        assert_eq!(Provider::Codex.cli_binary_name(), Some("codex"));
        assert_eq!(Provider::Ollama.cli_binary_name(), None);
        assert_eq!(Provider::OpenAI.cli_binary_name(), None);
    }

    #[test]
    fn test_provider_is_cli_based() {
        assert!(Provider::Claude.is_cli_based());
        assert!(Provider::Codex.is_cli_based());
        assert!(!Provider::Ollama.is_cli_based());
        assert!(!Provider::OpenAI.is_cli_based());
    }

    #[test]
    fn test_provider_is_api_based() {
        assert!(!Provider::Claude.is_api_based());
        assert!(!Provider::Codex.is_api_based());
        assert!(Provider::Ollama.is_api_based());
        assert!(Provider::OpenAI.is_api_based());
        assert!(Provider::Anthropic.is_api_based());
        assert!(Provider::Google.is_api_based());
    }

    #[test]
    fn test_provider_default_endpoint() {
        assert_eq!(
            Provider::Ollama.default_endpoint(),
            Some("http://localhost:11434")
        );
        assert_eq!(
            Provider::OpenAI.default_endpoint(),
            Some("https://api.openai.com/v1")
        );
        assert_eq!(
            Provider::Anthropic.default_endpoint(),
            Some("https://api.anthropic.com/v1")
        );
        assert_eq!(
            Provider::Google.default_endpoint(),
            Some("https://generativelanguage.googleapis.com/v1beta")
        );
        assert_eq!(Provider::Claude.default_endpoint(), None);
    }

    #[test]
    fn test_provider_api_key_env_var() {
        assert_eq!(Provider::OpenAI.api_key_env_var(), Some("OPENAI_API_KEY"));
        assert_eq!(
            Provider::Anthropic.api_key_env_var(),
            Some("ANTHROPIC_API_KEY")
        );
        assert_eq!(Provider::Google.api_key_env_var(), Some("GOOGLE_API_KEY"));
        assert_eq!(Provider::Ollama.api_key_env_var(), None);
        assert_eq!(Provider::Claude.api_key_env_var(), None);
    }

    #[test]
    fn test_provider_valid_models() {
        let claude_models = Provider::Claude.valid_models();
        assert!(claude_models.contains(&"claude-sonnet-4-5"));

        let codex_models = Provider::Codex.valid_models();
        assert!(codex_models.contains(&"gpt-5-codex"));
        assert!(codex_models.contains(&"gpt-5"));

        let ollama_models = Provider::Ollama.valid_models();
        assert!(ollama_models.contains(&"llama3.3"));

        let openai_models = Provider::OpenAI.valid_models();
        assert!(openai_models.contains(&"gpt-4o"));

        let anthropic_models = Provider::Anthropic.valid_models();
        assert!(anthropic_models.contains(&"claude-3-5-sonnet-20241022"));

        let google_models = Provider::Google.valid_models();
        assert!(google_models.contains(&"gemini-2.0-flash-exp"));
    }

    #[test]
    fn test_provider_is_valid_model() {
        assert!(Provider::Claude.is_valid_model("claude-sonnet-4-5"));
        assert!(!Provider::Claude.is_valid_model("gpt-5-codex"));

        assert!(Provider::Codex.is_valid_model("gpt-5-codex"));
        assert!(Provider::Codex.is_valid_model("gpt-5"));
        assert!(!Provider::Codex.is_valid_model("claude-sonnet-4-5"));

        // Ollama accepts any model
        assert!(Provider::Ollama.is_valid_model("custom-model"));
        assert!(Provider::Ollama.is_valid_model("llama3.3"));

        assert!(Provider::OpenAI.is_valid_model("gpt-4o"));
        assert!(!Provider::OpenAI.is_valid_model("claude-3-opus"));
    }

    #[test]
    fn test_provider_default_model() {
        assert_eq!(Provider::Claude.default_model(), "claude-sonnet-4-5");
        assert_eq!(Provider::Codex.default_model(), "gpt-5-codex");
        assert_eq!(Provider::Ollama.default_model(), "olmo2:13b");
        assert_eq!(Provider::OpenAI.default_model(), "gpt-4o");
        assert_eq!(
            Provider::Anthropic.default_model(),
            "claude-3-5-sonnet-20241022"
        );
        assert_eq!(Provider::Google.default_model(), "gemini-2.0-flash-exp");
    }
}
