//! LLM Client Port
//!
//! Secondary port for direct LLM API interactions. This port defines the
//! interface for calling LLM providers (Ollama, OpenAI, Anthropic, Google)
//! without going through CLI subprocesses.

use crate::domain::Provider;
use async_trait::async_trait;
use std::collections::HashMap;

/// LLM request configuration
#[derive(Debug, Clone)]
pub struct LlmRequest {
    /// LLM provider
    pub provider: Provider,
    /// Model name
    pub model: String,
    /// User prompt/query
    pub prompt: String,
    /// System prompt (optional)
    pub system_prompt: Option<String>,
    /// API endpoint URL (optional, uses provider default if not specified)
    pub endpoint: Option<String>,
    /// API key (optional, will try environment variable if not provided)
    pub api_key: Option<String>,
    /// Temperature for sampling
    pub temperature: Option<f64>,
    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,
    /// Top-p nucleus sampling
    pub top_p: Option<f64>,
    /// Top-k sampling
    pub top_k: Option<u32>,
    /// Stop sequences
    pub stop: Vec<String>,
    /// Timeout in seconds
    pub timeout_secs: Option<u64>,
    /// Additional provider-specific parameters
    pub extra_params: HashMap<String, serde_json::Value>,
}

/// LLM response
#[derive(Debug, Clone)]
pub struct LlmResponse {
    /// Generated text content
    pub content: String,
    /// Model used for generation
    pub model: String,
    /// Provider
    pub provider: Provider,
    /// Token usage information (if available)
    pub usage: Option<TokenUsage>,
    /// Finish reason (e.g., "stop", "length", "content_filter")
    pub finish_reason: Option<String>,
    /// Raw response from provider (for debugging)
    pub raw_response: Option<serde_json::Value>,
}

/// Token usage information
#[derive(Debug, Clone)]
pub struct TokenUsage {
    /// Input tokens consumed
    pub prompt_tokens: u32,
    /// Output tokens generated
    pub completion_tokens: u32,
    /// Total tokens (prompt + completion)
    pub total_tokens: u32,
}

/// LLM client error
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    /// Provider not supported
    #[error("Provider {0:?} is not supported for direct API calls")]
    UnsupportedProvider(Provider),

    /// API key not found
    #[error("API key not found for provider {0:?}. Set {1} environment variable or provide api_key in configuration")]
    MissingApiKey(Provider, String),

    /// HTTP request error
    #[error("HTTP request failed: {0}")]
    HttpError(String),

    /// API error response
    #[error("API error: {0}")]
    ApiError(String),

    /// JSON parsing error
    #[error("Failed to parse response: {0}")]
    ParseError(String),

    /// Timeout error
    #[error("Request timed out after {0} seconds")]
    Timeout(u64),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Model not found (for Ollama)
    #[error("Model {0} not found. Run 'ollama pull {0}' to download it")]
    ModelNotFound(String),
}

/// LLM client trait for direct API interactions
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Execute an LLM request
    async fn execute(&self, request: LlmRequest) -> Result<LlmResponse, LlmError>;

    /// Check if the provider is supported by this client
    fn supports_provider(&self, provider: &Provider) -> bool;

    /// Get the name of this client implementation
    fn name(&self) -> &str;
}
