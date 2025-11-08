//! AI configuration
use super::providers::AiProviderType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    /// Provider type
    pub provider: AiProviderType,

    /// Model name
    pub model: String,

    /// API endpoint (optional, uses provider default if not specified)
    pub endpoint: Option<String>,

    /// API key (optional, uses environment variable if not specified)
    pub api_key: Option<String>,

    /// Temperature for generation (0.0-2.0)
    pub temperature: f32,

    /// Maximum tokens to generate
    pub max_tokens: u32,

    /// Additional provider-specific parameters
    pub extra_params: HashMap<String, serde_json::Value>,
}

impl Default for AiConfig {
    fn default() -> Self {
        create_default_config()
    }
}

/// Create default AI configuration
pub fn create_default_config() -> AiConfig {
    AiConfig {
        provider: AiProviderType::Ollama, // Default to local Ollama
        model: "llama3.3".to_string(),
        endpoint: None, // Use provider default
        api_key: None,  // Use environment variable
        temperature: 0.7,
        max_tokens: 2000,
        extra_params: HashMap::new(),
    }
}

/// Create configuration for specific provider
pub fn config_for_provider(provider: AiProviderType, model: Option<String>) -> AiConfig {
    let default_model = match provider {
        AiProviderType::Ollama => "llama3.3",
        AiProviderType::OpenAi => "gpt-4o",
        AiProviderType::Anthropic => "claude-3-5-sonnet-20241022",
        AiProviderType::Google => "gemini-2.0-flash-exp",
    };

    AiConfig {
        provider,
        model: model.unwrap_or_else(|| default_model.to_string()),
        endpoint: None,
        api_key: None,
        temperature: 0.7,
        max_tokens: 2000,
        extra_params: HashMap::new(),
    }
}
