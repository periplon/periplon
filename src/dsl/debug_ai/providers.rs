//! AI provider abstraction
use super::config::AiConfig;
use crate::error::{Error, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// AI provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiProviderType {
    /// Ollama (local)
    Ollama,

    /// OpenAI API
    OpenAi,

    /// Anthropic API
    Anthropic,

    /// Google Gemini API
    Google,
}

/// AI response
#[derive(Debug, Clone)]
pub struct AiResponse {
    /// Generated text
    pub text: String,

    /// Tokens used (if available)
    pub tokens_used: Option<u32>,

    /// Provider metadata
    pub metadata: serde_json::Value,
}

/// AI provider trait
#[async_trait]
pub trait AiProvider: Send + Sync {
    /// Generate text from prompt
    async fn generate(&self, prompt: &str) -> Result<AiResponse>;

    /// Generate with system prompt
    async fn generate_with_system(&self, system: &str, prompt: &str) -> Result<AiResponse>;

    /// Get provider name
    fn name(&self) -> &str;
}

/// Create provider from configuration
pub fn create_provider(config: &AiConfig) -> Result<Box<dyn AiProvider>> {
    match config.provider {
        AiProviderType::Ollama => Ok(Box::new(OllamaProvider::new(config)?)),
        AiProviderType::OpenAi => Ok(Box::new(OpenAiProvider::new(config)?)),
        AiProviderType::Anthropic => Ok(Box::new(AnthropicProvider::new(config)?)),
        AiProviderType::Google => Ok(Box::new(GoogleProvider::new(config)?)),
    }
}

/// Ollama provider
struct OllamaProvider {
    config: AiConfig,
    client: reqwest::Client,
}

impl OllamaProvider {
    fn new(config: &AiConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            client: reqwest::Client::new(),
        })
    }

    fn endpoint(&self) -> String {
        self.config
            .endpoint
            .clone()
            .unwrap_or_else(|| "http://localhost:11434".to_string())
    }
}

#[async_trait]
impl AiProvider for OllamaProvider {
    async fn generate(&self, prompt: &str) -> Result<AiResponse> {
        self.generate_with_system("", prompt).await
    }

    async fn generate_with_system(&self, system: &str, prompt: &str) -> Result<AiResponse> {
        let url = format!("{}/api/generate", self.endpoint());

        let mut body = serde_json::json!({
            "model": self.config.model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": self.config.temperature,
                "num_predict": self.config.max_tokens,
            }
        });

        if !system.is_empty() {
            body["system"] = serde_json::json!(system);
        }

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::InvalidInput(format!("HTTP error: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::InvalidInput(format!(
                "Ollama request failed: {}",
                response.status()
            )));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::InvalidInput(e.to_string()))?;

        let text = data["response"]
            .as_str()
            .ok_or_else(|| Error::InvalidInput("No response from Ollama".to_string()))?
            .to_string();

        Ok(AiResponse {
            text,
            tokens_used: None,
            metadata: data,
        })
    }

    fn name(&self) -> &str {
        "Ollama"
    }
}

/// OpenAI provider
struct OpenAiProvider {
    config: AiConfig,
    client: reqwest::Client,
}

impl OpenAiProvider {
    fn new(config: &AiConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            client: reqwest::Client::new(),
        })
    }

    fn endpoint(&self) -> String {
        self.config
            .endpoint
            .clone()
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string())
    }

    fn api_key(&self) -> Result<String> {
        self.config
            .api_key
            .clone()
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
            .ok_or_else(|| Error::InvalidInput("OpenAI API key not found".to_string()))
    }
}

#[async_trait]
impl AiProvider for OpenAiProvider {
    async fn generate(&self, prompt: &str) -> Result<AiResponse> {
        self.generate_with_system("You are a helpful assistant.", prompt)
            .await
    }

    async fn generate_with_system(&self, system: &str, prompt: &str) -> Result<AiResponse> {
        let url = format!("{}/chat/completions", self.endpoint());
        let api_key = self.api_key()?;

        let body = serde_json::json!({
            "model": self.config.model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": prompt}
            ],
            "temperature": self.config.temperature,
            "max_tokens": self.config.max_tokens,
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::InvalidInput(format!("HTTP error: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::InvalidInput(format!(
                "OpenAI request failed: {}",
                response.status()
            )));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::InvalidInput(e.to_string()))?;

        let text = data["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| Error::InvalidInput("No response from OpenAI".to_string()))?
            .to_string();

        let tokens = data["usage"]["total_tokens"].as_u64().map(|t| t as u32);

        Ok(AiResponse {
            text,
            tokens_used: tokens,
            metadata: data,
        })
    }

    fn name(&self) -> &str {
        "OpenAI"
    }
}

/// Anthropic provider
struct AnthropicProvider {
    config: AiConfig,
    client: reqwest::Client,
}

impl AnthropicProvider {
    fn new(config: &AiConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            client: reqwest::Client::new(),
        })
    }

    fn endpoint(&self) -> String {
        self.config
            .endpoint
            .clone()
            .unwrap_or_else(|| "https://api.anthropic.com/v1".to_string())
    }

    fn api_key(&self) -> Result<String> {
        self.config
            .api_key
            .clone()
            .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
            .ok_or_else(|| Error::InvalidInput("Anthropic API key not found".to_string()))
    }
}

#[async_trait]
impl AiProvider for AnthropicProvider {
    async fn generate(&self, prompt: &str) -> Result<AiResponse> {
        self.generate_with_system("You are a helpful assistant.", prompt)
            .await
    }

    async fn generate_with_system(&self, system: &str, prompt: &str) -> Result<AiResponse> {
        let url = format!("{}/messages", self.endpoint());
        let api_key = self.api_key()?;

        let body = serde_json::json!({
            "model": self.config.model,
            "max_tokens": self.config.max_tokens,
            "system": system,
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "temperature": self.config.temperature,
        });

        let response = self
            .client
            .post(&url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::InvalidInput(format!("HTTP error: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::InvalidInput(format!(
                "Anthropic request failed: {}",
                response.status()
            )));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::InvalidInput(e.to_string()))?;

        let text = data["content"][0]["text"]
            .as_str()
            .ok_or_else(|| Error::InvalidInput("No response from Anthropic".to_string()))?
            .to_string();

        let tokens = data["usage"]["output_tokens"].as_u64().map(|t| t as u32);

        Ok(AiResponse {
            text,
            tokens_used: tokens,
            metadata: data,
        })
    }

    fn name(&self) -> &str {
        "Anthropic"
    }
}

/// Google Gemini provider
struct GoogleProvider {
    config: AiConfig,
    client: reqwest::Client,
}

impl GoogleProvider {
    fn new(config: &AiConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
            client: reqwest::Client::new(),
        })
    }

    fn endpoint(&self) -> String {
        self.config
            .endpoint
            .clone()
            .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string())
    }

    fn api_key(&self) -> Result<String> {
        self.config
            .api_key
            .clone()
            .or_else(|| std::env::var("GOOGLE_API_KEY").ok())
            .ok_or_else(|| Error::InvalidInput("Google API key not found".to_string()))
    }
}

#[async_trait]
impl AiProvider for GoogleProvider {
    async fn generate(&self, prompt: &str) -> Result<AiResponse> {
        self.generate_with_system("", prompt).await
    }

    async fn generate_with_system(&self, system: &str, prompt: &str) -> Result<AiResponse> {
        let api_key = self.api_key()?;
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.endpoint(),
            self.config.model,
            api_key
        );

        let full_prompt = if system.is_empty() {
            prompt.to_string()
        } else {
            format!("{}\n\n{}", system, prompt)
        };

        let body = serde_json::json!({
            "contents": [{
                "parts": [{"text": full_prompt}]
            }],
            "generationConfig": {
                "temperature": self.config.temperature,
                "maxOutputTokens": self.config.max_tokens,
            }
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::InvalidInput(format!("HTTP error: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::InvalidInput(format!(
                "Google request failed: {}",
                response.status()
            )));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::InvalidInput(e.to_string()))?;

        let text = data["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or_else(|| Error::InvalidInput("No response from Google".to_string()))?
            .to_string();

        Ok(AiResponse {
            text,
            tokens_used: None,
            metadata: data,
        })
    }

    fn name(&self) -> &str {
        "Google Gemini"
    }
}
