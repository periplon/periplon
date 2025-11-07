//! HTTP-based LLM Client Implementation
//!
//! Unified client supporting Ollama, OpenAI, Anthropic, and Google APIs

use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::time::Duration;

use crate::domain::Provider;
use crate::ports::secondary::{LlmClient, LlmError, LlmRequest, LlmResponse, TokenUsage};

/// HTTP-based LLM client supporting multiple providers
pub struct HttpLlmClient {
    client: Client,
}

impl HttpLlmClient {
    /// Create a new HTTP LLM client
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(300)) // Default 5 minute timeout
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// Get API key for provider
    fn get_api_key(&self, request: &LlmRequest) -> Result<Option<String>, LlmError> {
        // Check if API key provided in request
        if let Some(key) = &request.api_key {
            return Ok(Some(key.clone()));
        }

        // Try environment variable
        if let Some(env_var) = request.provider.api_key_env_var() {
            if let Ok(key) = std::env::var(env_var) {
                return Ok(Some(key));
            }
            // Ollama doesn't require API key
            if request.provider == Provider::Ollama {
                return Ok(None);
            }
            return Err(LlmError::MissingApiKey(
                request.provider.clone(),
                env_var.to_string(),
            ));
        }

        Ok(None)
    }

    /// Get endpoint URL for provider
    fn get_endpoint(&self, request: &LlmRequest) -> String {
        request
            .endpoint
            .clone()
            .or_else(|| request.provider.default_endpoint().map(String::from))
            .expect("Provider should have default endpoint")
    }

    /// Execute Ollama request
    async fn execute_ollama(&self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
        let endpoint = self.get_endpoint(&request);
        let url = format!("{}/api/generate", endpoint);

        let mut body = json!({
            "model": request.model,
            "prompt": request.prompt,
            "stream": false,
        });

        // Add system prompt if provided
        if let Some(system) = &request.system_prompt {
            body["system"] = json!(system);
        }

        // Add options
        let mut options = serde_json::Map::new();
        if let Some(temp) = request.temperature {
            options.insert("temperature".to_string(), json!(temp));
        }
        if let Some(max_tokens) = request.max_tokens {
            options.insert("num_predict".to_string(), json!(max_tokens));
        }
        if let Some(top_p) = request.top_p {
            options.insert("top_p".to_string(), json!(top_p));
        }
        if let Some(top_k) = request.top_k {
            options.insert("top_k".to_string(), json!(top_k));
        }
        if !request.stop.is_empty() {
            options.insert("stop".to_string(), json!(request.stop));
        }

        if !options.is_empty() {
            body["options"] = json!(options);
        }

        // Add extra params
        for (key, value) in &request.extra_params {
            body[key] = value.clone();
        }

        let mut http_request = self.client.post(&url).json(&body);

        if let Some(timeout) = request.timeout_secs {
            http_request = http_request.timeout(Duration::from_secs(timeout));
        }

        let response = http_request
            .send()
            .await
            .map_err(|e| LlmError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Check for model not found
            if error_text.contains("model") && error_text.contains("not found") {
                return Err(LlmError::ModelNotFound(request.model));
            }

            return Err(LlmError::ApiError(format!(
                "Ollama API error ({}): {}",
                status, error_text
            )));
        }

        let raw: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;

        let content = raw["response"]
            .as_str()
            .ok_or_else(|| LlmError::ParseError("Missing 'response' field".to_string()))?
            .to_string();

        let usage = if let Some(prompt_tokens) = raw["prompt_eval_count"].as_u64() {
            let completion_tokens = raw["eval_count"].as_u64().unwrap_or(0);
            Some(TokenUsage {
                prompt_tokens: prompt_tokens as u32,
                completion_tokens: completion_tokens as u32,
                total_tokens: (prompt_tokens + completion_tokens) as u32,
            })
        } else {
            None
        };

        Ok(LlmResponse {
            content,
            model: request.model,
            provider: Provider::Ollama,
            usage,
            finish_reason: Some(raw["done_reason"].as_str().unwrap_or("stop").to_string()),
            raw_response: Some(raw),
        })
    }

    /// Execute OpenAI request
    async fn execute_openai(&self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
        let api_key = self.get_api_key(&request)?.ok_or_else(|| {
            LlmError::MissingApiKey(Provider::OpenAI, "OPENAI_API_KEY".to_string())
        })?;

        let endpoint = self.get_endpoint(&request);
        let url = format!("{}/chat/completions", endpoint);

        let mut messages = Vec::new();

        // Add system message if provided
        if let Some(system) = &request.system_prompt {
            messages.push(json!({
                "role": "system",
                "content": system
            }));
        }

        // Add user message
        messages.push(json!({
            "role": "user",
            "content": request.prompt
        }));

        let mut body = json!({
            "model": request.model,
            "messages": messages,
        });

        // Add optional parameters
        if let Some(temp) = request.temperature {
            body["temperature"] = json!(temp);
        }
        if let Some(max_tokens) = request.max_tokens {
            body["max_tokens"] = json!(max_tokens);
        }
        if let Some(top_p) = request.top_p {
            body["top_p"] = json!(top_p);
        }
        if !request.stop.is_empty() {
            body["stop"] = json!(request.stop);
        }

        // Add extra params
        for (key, value) in &request.extra_params {
            body[key] = value.clone();
        }

        let mut http_request = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body);

        if let Some(timeout) = request.timeout_secs {
            http_request = http_request.timeout(Duration::from_secs(timeout));
        }

        let response = http_request
            .send()
            .await
            .map_err(|e| LlmError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LlmError::ApiError(format!(
                "OpenAI API error ({}): {}",
                status, error_text
            )));
        }

        let raw: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;

        let content = raw["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| LlmError::ParseError("Missing content in response".to_string()))?
            .to_string();

        let usage = raw["usage"].as_object().map(|usage_obj| TokenUsage {
            prompt_tokens: usage_obj["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: usage_obj["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: usage_obj["total_tokens"].as_u64().unwrap_or(0) as u32,
        });

        let finish_reason = raw["choices"][0]["finish_reason"]
            .as_str()
            .map(String::from);

        Ok(LlmResponse {
            content,
            model: raw["model"].as_str().unwrap_or(&request.model).to_string(),
            provider: Provider::OpenAI,
            usage,
            finish_reason,
            raw_response: Some(raw),
        })
    }

    /// Execute Anthropic request
    async fn execute_anthropic(&self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
        let api_key = self.get_api_key(&request)?.ok_or_else(|| {
            LlmError::MissingApiKey(Provider::Anthropic, "ANTHROPIC_API_KEY".to_string())
        })?;

        let endpoint = self.get_endpoint(&request);
        let url = format!("{}/messages", endpoint);

        let mut body = json!({
            "model": request.model,
            "messages": [{
                "role": "user",
                "content": request.prompt
            }],
            "max_tokens": request.max_tokens.unwrap_or(4096),
        });

        // Add system prompt if provided
        if let Some(system) = &request.system_prompt {
            body["system"] = json!(system);
        }

        // Add optional parameters
        if let Some(temp) = request.temperature {
            body["temperature"] = json!(temp);
        }
        if let Some(top_p) = request.top_p {
            body["top_p"] = json!(top_p);
        }
        if let Some(top_k) = request.top_k {
            body["top_k"] = json!(top_k);
        }
        if !request.stop.is_empty() {
            body["stop_sequences"] = json!(request.stop);
        }

        // Add extra params
        for (key, value) in &request.extra_params {
            body[key] = value.clone();
        }

        let mut http_request = self
            .client
            .post(&url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body);

        if let Some(timeout) = request.timeout_secs {
            http_request = http_request.timeout(Duration::from_secs(timeout));
        }

        let response = http_request
            .send()
            .await
            .map_err(|e| LlmError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LlmError::ApiError(format!(
                "Anthropic API error ({}): {}",
                status, error_text
            )));
        }

        let raw: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;

        let content = raw["content"][0]["text"]
            .as_str()
            .ok_or_else(|| LlmError::ParseError("Missing content in response".to_string()))?
            .to_string();

        let usage = raw["usage"].as_object().map(|usage_obj| TokenUsage {
            prompt_tokens: usage_obj["input_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: usage_obj["output_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: (usage_obj["input_tokens"].as_u64().unwrap_or(0)
                + usage_obj["output_tokens"].as_u64().unwrap_or(0))
                as u32,
        });

        let finish_reason = raw["stop_reason"].as_str().map(String::from);

        Ok(LlmResponse {
            content,
            model: raw["model"].as_str().unwrap_or(&request.model).to_string(),
            provider: Provider::Anthropic,
            usage,
            finish_reason,
            raw_response: Some(raw),
        })
    }

    /// Execute Google Gemini request
    async fn execute_google(&self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
        let api_key = self.get_api_key(&request)?.ok_or_else(|| {
            LlmError::MissingApiKey(Provider::Google, "GOOGLE_API_KEY".to_string())
        })?;

        let endpoint = self.get_endpoint(&request);
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            endpoint, request.model, api_key
        );

        let parts = vec![json!({
            "text": request.prompt
        })];

        let mut body = json!({
            "contents": [{
                "parts": parts
            }]
        });

        // Add generation config
        let mut generation_config = serde_json::Map::new();
        if let Some(temp) = request.temperature {
            generation_config.insert("temperature".to_string(), json!(temp));
        }
        if let Some(max_tokens) = request.max_tokens {
            generation_config.insert("maxOutputTokens".to_string(), json!(max_tokens));
        }
        if let Some(top_p) = request.top_p {
            generation_config.insert("topP".to_string(), json!(top_p));
        }
        if let Some(top_k) = request.top_k {
            generation_config.insert("topK".to_string(), json!(top_k));
        }
        if !request.stop.is_empty() {
            generation_config.insert("stopSequences".to_string(), json!(request.stop));
        }

        if !generation_config.is_empty() {
            body["generationConfig"] = json!(generation_config);
        }

        // Add system instruction if provided
        if let Some(system) = &request.system_prompt {
            body["systemInstruction"] = json!({
                "parts": [{
                    "text": system
                }]
            });
        }

        // Add extra params
        for (key, value) in &request.extra_params {
            body[key] = value.clone();
        }

        let mut http_request = self.client.post(&url).json(&body);

        if let Some(timeout) = request.timeout_secs {
            http_request = http_request.timeout(Duration::from_secs(timeout));
        }

        let response = http_request
            .send()
            .await
            .map_err(|e| LlmError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LlmError::ApiError(format!(
                "Google API error ({}): {}",
                status, error_text
            )));
        }

        let raw: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;

        let content = raw["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or_else(|| LlmError::ParseError("Missing content in response".to_string()))?
            .to_string();

        let usage = raw["usageMetadata"]
            .as_object()
            .map(|usage_obj| TokenUsage {
                prompt_tokens: usage_obj["promptTokenCount"].as_u64().unwrap_or(0) as u32,
                completion_tokens: usage_obj["candidatesTokenCount"].as_u64().unwrap_or(0) as u32,
                total_tokens: usage_obj["totalTokenCount"].as_u64().unwrap_or(0) as u32,
            });

        let finish_reason = raw["candidates"][0]["finishReason"]
            .as_str()
            .map(String::from);

        Ok(LlmResponse {
            content,
            model: request.model,
            provider: Provider::Google,
            usage,
            finish_reason,
            raw_response: Some(raw),
        })
    }
}

impl Default for HttpLlmClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmClient for HttpLlmClient {
    async fn execute(&self, request: LlmRequest) -> Result<LlmResponse, LlmError> {
        match request.provider {
            Provider::Ollama => self.execute_ollama(request).await,
            Provider::OpenAI => self.execute_openai(request).await,
            Provider::Anthropic => self.execute_anthropic(request).await,
            Provider::Google => self.execute_google(request).await,
            _ => Err(LlmError::UnsupportedProvider(request.provider)),
        }
    }

    fn supports_provider(&self, provider: &Provider) -> bool {
        matches!(
            provider,
            Provider::Ollama | Provider::OpenAI | Provider::Anthropic | Provider::Google
        )
    }

    fn name(&self) -> &str {
        "HttpLlmClient"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = HttpLlmClient::new();
        assert_eq!(client.name(), "HttpLlmClient");
    }

    #[test]
    fn test_supports_provider() {
        let client = HttpLlmClient::new();
        assert!(client.supports_provider(&Provider::Ollama));
        assert!(client.supports_provider(&Provider::OpenAI));
        assert!(client.supports_provider(&Provider::Anthropic));
        assert!(client.supports_provider(&Provider::Google));
        assert!(!client.supports_provider(&Provider::Claude));
        assert!(!client.supports_provider(&Provider::Codex));
    }
}
