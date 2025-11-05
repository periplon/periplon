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

    /// Get the CLI binary name for this provider
    pub fn cli_binary_name(&self) -> &str {
        match self {
            Provider::Claude => "claude",
            Provider::Codex => "codex",
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
        }
    }

    /// Validate if a model name is valid for this provider
    pub fn is_valid_model(&self, model: &str) -> bool {
        self.valid_models().contains(&model)
    }

    /// Get default model for this provider
    pub fn default_model(&self) -> &str {
        match self {
            Provider::Claude => "claude-sonnet-4-5",
            Provider::Codex => "gpt-5-codex",
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
        assert_eq!(Provider::Claude.cli_binary_name(), "claude");
        assert_eq!(Provider::Codex.cli_binary_name(), "codex");
    }

    #[test]
    fn test_provider_valid_models() {
        let claude_models = Provider::Claude.valid_models();
        assert!(claude_models.contains(&"claude-sonnet-4-5"));

        let codex_models = Provider::Codex.valid_models();
        assert!(codex_models.contains(&"gpt-5-codex"));
        assert!(codex_models.contains(&"gpt-5"));
    }

    #[test]
    fn test_provider_is_valid_model() {
        assert!(Provider::Claude.is_valid_model("claude-sonnet-4-5"));
        assert!(!Provider::Claude.is_valid_model("gpt-5-codex"));

        assert!(Provider::Codex.is_valid_model("gpt-5-codex"));
        assert!(Provider::Codex.is_valid_model("gpt-5"));
        assert!(!Provider::Codex.is_valid_model("claude-sonnet-4-5"));
    }

    #[test]
    fn test_provider_default_model() {
        assert_eq!(Provider::Claude.default_model(), "claude-sonnet-4-5");
        assert_eq!(Provider::Codex.default_model(), "gpt-5-codex");
    }
}
