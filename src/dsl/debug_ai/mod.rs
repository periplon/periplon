//! AI Integration for Debugging
//!
//! Provides AI-powered features for workflow debugging:
//! - Generate workflow blocks from natural language
//! - Suggest fixes for errors
//! - Analyze workflow structure
//! - Auto-complete workflows

pub mod config;
pub mod generator;
pub mod providers;
pub mod suggestions;

pub use config::{create_default_config, AiConfig};
pub use generator::{generate_task, generate_workflow_block};
pub use providers::{AiProvider, AiProviderType, AiResponse};
pub use suggestions::{analyze_error, explain_workflow, suggest_fix, suggest_improvements};

use crate::error::Result;

/// AI assistant for debugging
pub struct DebugAiAssistant {
    /// Current AI configuration
    config: AiConfig,

    /// Provider instance
    provider: Box<dyn AiProvider>,
}

impl DebugAiAssistant {
    /// Create a new AI assistant with default configuration
    pub fn new() -> Result<Self> {
        let config = create_default_config();
        let provider = providers::create_provider(&config)?;

        Ok(Self { config, provider })
    }

    /// Create AI assistant with specific configuration
    pub fn with_config(config: AiConfig) -> Result<Self> {
        let provider = providers::create_provider(&config)?;

        Ok(Self { config, provider })
    }

    /// Change provider and model on the fly
    pub fn set_provider(&mut self, provider_type: AiProviderType, model: String) -> Result<()> {
        self.config.provider = provider_type;
        self.config.model = model;
        self.provider = providers::create_provider(&self.config)?;
        Ok(())
    }

    /// Set temperature (0.0-2.0)
    pub fn set_temperature(&mut self, temperature: f32) {
        self.config.temperature = temperature;
    }

    /// Set max tokens
    pub fn set_max_tokens(&mut self, max_tokens: u32) {
        self.config.max_tokens = max_tokens;
    }

    /// Set endpoint (None to use provider default)
    pub fn set_endpoint(&mut self, endpoint: Option<String>) -> Result<()> {
        self.config.endpoint = endpoint;
        self.provider = providers::create_provider(&self.config)?;
        Ok(())
    }

    /// Set API key (None to use environment variable)
    pub fn set_api_key(&mut self, api_key: Option<String>) -> Result<()> {
        self.config.api_key = api_key;
        self.provider = providers::create_provider(&self.config)?;
        Ok(())
    }

    /// Generate workflow block from description
    pub async fn generate_block(&self, description: &str) -> Result<String> {
        generator::generate_workflow_block(self.provider.as_ref(), description).await
    }

    /// Suggest fix for an error
    pub async fn suggest_fix(&self, error: &str, context: &str) -> Result<String> {
        suggestions::suggest_fix(self.provider.as_ref(), error, context).await
    }

    /// Analyze workflow and suggest improvements
    pub async fn analyze_workflow(&self, workflow_yaml: &str) -> Result<String> {
        suggestions::suggest_improvements(self.provider.as_ref(), workflow_yaml).await
    }

    /// Explain what a workflow does
    pub async fn explain_workflow(&self, workflow_yaml: &str) -> Result<String> {
        suggestions::explain_workflow(self.provider.as_ref(), workflow_yaml).await
    }

    /// Get current configuration
    pub fn config(&self) -> &AiConfig {
        &self.config
    }
}

impl Default for DebugAiAssistant {
    fn default() -> Self {
        Self::new().expect("Failed to create default AI assistant")
    }
}
