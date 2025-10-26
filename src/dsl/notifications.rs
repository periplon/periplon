//! Notification Delivery System
//!
//! This module provides asynchronous notification delivery with support for multiple channels,
//! retry logic, variable interpolation, and MCP integration.
//!
//! # Architecture
//!
//! - `NotificationManager`: Coordinates notification delivery across channels
//! - `NotificationSender` trait: Common interface for all notification channels
//! - Channel-specific implementations: NtfySender, SlackSender, etc.
//! - Error handling with comprehensive error types
//! - Async/concurrent delivery with tokio
//! - Retry logic with exponential backoff
//! - Variable interpolation from workflow context
//!
//! # Example
//!
//! ```no_run
//! use periplon_sdk::dsl::notifications::{NotificationManager, NotificationContext};
//! use periplon_sdk::dsl::schema::{NotificationSpec, NotificationChannel};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let manager = NotificationManager::new();
//! let context = NotificationContext::default();
//!
//! let spec = NotificationSpec::Simple("Task completed".to_string());
//! manager.send(&spec, &context).await?;
//! # Ok(())
//! # }
//! ```

use crate::dsl::schema::{
    DiscordEmbed, FileNotificationFormat, NotificationChannel, NotificationSpec, RetryConfig,
    SlackAttachment, SlackMethod,
};
use async_trait::async_trait;
use chrono;
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;
use tokio::time::sleep;

// ============================================================================
// Error Types
// ============================================================================

/// Errors that can occur during notification delivery
#[derive(Debug, Error)]
pub enum NotificationError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// MCP tool invocation failed
    #[error("MCP tool invocation failed: {0}")]
    McpError(String),

    /// Variable interpolation failed
    #[error("Variable interpolation failed: {0}")]
    InterpolationError(String),

    /// Channel configuration invalid
    #[error("Invalid channel configuration: {0}")]
    InvalidConfiguration(String),

    /// Serialization failed
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// IO error (for file-based notifications)
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Retry exhausted
    #[error("All retry attempts exhausted for {channel}: {last_error}")]
    RetryExhausted {
        channel: String,
        attempts: u32,
        last_error: String,
    },

    /// Channel not supported
    #[error("Notification channel not supported: {0}")]
    UnsupportedChannel(String),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
}

pub type NotificationResult<T> = Result<T, NotificationError>;

// ============================================================================
// Notification Context
// ============================================================================

/// Context for variable interpolation in notifications
#[derive(Debug, Clone, Default)]
pub struct NotificationContext {
    /// Workflow-level variables
    pub workflow_vars: HashMap<String, String>,
    /// Task-level variables
    pub task_vars: HashMap<String, String>,
    /// Agent-level variables
    pub agent_vars: HashMap<String, String>,
    /// Secrets (for secure credential access)
    pub secrets: HashMap<String, String>,
    /// Metadata (task name, status, duration, etc.)
    pub metadata: HashMap<String, String>,
}

impl NotificationContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a workflow variable
    pub fn with_workflow_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.workflow_vars.insert(key.into(), value.into());
        self
    }

    /// Add a task variable
    pub fn with_task_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.task_vars.insert(key.into(), value.into());
        self
    }

    /// Add an agent variable
    pub fn with_agent_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.agent_vars.insert(key.into(), value.into());
        self
    }

    /// Add a secret
    pub fn with_secret(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.secrets.insert(key.into(), value.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Interpolate variables in a string
    /// Supports: ${workflow.var}, ${task.var}, ${agent.var}, ${secret.name}, ${metadata.key}
    pub fn interpolate(&self, template: &str) -> NotificationResult<String> {
        let mut result = template.to_string();

        // Interpolate workflow variables
        for (key, value) in &self.workflow_vars {
            let pattern = format!("${{workflow.{}}}", key);
            result = result.replace(&pattern, value);
        }

        // Interpolate task variables
        for (key, value) in &self.task_vars {
            let pattern = format!("${{task.{}}}", key);
            result = result.replace(&pattern, value);
        }

        // Interpolate agent variables
        for (key, value) in &self.agent_vars {
            let pattern = format!("${{agent.{}}}", key);
            result = result.replace(&pattern, value);
        }

        // Interpolate secrets
        for (key, value) in &self.secrets {
            let pattern = format!("${{secret.{}}}", key);
            result = result.replace(&pattern, value);
        }

        // Interpolate metadata
        for (key, value) in &self.metadata {
            let pattern = format!("${{metadata.{}}}", key);
            result = result.replace(&pattern, value);
        }

        // Check for unresolved variables
        if result.contains("${") {
            return Err(NotificationError::InterpolationError(format!(
                "Unresolved variables in template: {}",
                result
            )));
        }

        Ok(result)
    }
}

// ============================================================================
// Notification Sender Trait
// ============================================================================

/// Trait for notification channel senders
#[async_trait]
pub trait NotificationSender: Send + Sync {
    /// Send a notification through this channel
    async fn send(
        &self,
        message: &str,
        channel: &NotificationChannel,
        context: &NotificationContext,
    ) -> NotificationResult<()>;

    /// Get the channel name for logging
    fn channel_name(&self) -> &str;

    /// Check if this sender supports retry
    fn supports_retry(&self) -> bool {
        true
    }
}

// ============================================================================
// Ntfy Sender (MCP Integration)
// ============================================================================

/// Ntfy notification sender using MCP integration
pub struct NtfySender {
    http_client: reqwest::Client,
}

impl NtfySender {
    /// Create a new Ntfy sender
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
        }
    }

    /// Send notification via ntfy MCP tool
    #[allow(clippy::too_many_arguments)]
    async fn send_via_mcp(
        &self,
        message: &str,
        server: &str,
        topic: &str,
        title: Option<&str>,
        priority: Option<u8>,
        tags: &[String],
        click_url: Option<&str>,
        attach_url: Option<&str>,
        markdown: bool,
        auth_token: Option<&str>,
    ) -> NotificationResult<()> {
        // Note: This is a placeholder for MCP integration
        // In actual implementation, this would call the MCP tool via the SDK
        // For now, we'll use direct HTTP as fallback

        log::debug!(
            "Sending ntfy notification to topic '{}' on server '{}'",
            topic,
            server
        );

        let mut headers = reqwest::header::HeaderMap::new();

        if let Some(title_val) = title {
            headers.insert(
                "Title",
                title_val.parse().map_err(|e| {
                    NotificationError::InvalidConfiguration(format!("Invalid title: {}", e))
                })?,
            );
        }

        if let Some(priority_val) = priority {
            headers.insert("Priority", priority_val.to_string().parse().unwrap());
        }

        if !tags.is_empty() {
            headers.insert("Tags", tags.join(",").parse().unwrap());
        }

        if let Some(url) = click_url {
            headers.insert("Click", url.parse().unwrap());
        }

        if let Some(url) = attach_url {
            headers.insert("Attach", url.parse().unwrap());
        }

        if markdown {
            headers.insert("Markdown", "yes".parse().unwrap());
        }

        if let Some(token) = auth_token {
            headers.insert(
                "Authorization",
                format!("Bearer {}", token).parse().unwrap(),
            );
        }

        let url = format!("{}/{}", server.trim_end_matches('/'), topic);

        let response = self
            .http_client
            .post(&url)
            .headers(headers)
            .body(message.to_string())
            .send()
            .await?;

        if response.status().is_success() {
            log::info!("Successfully sent ntfy notification to topic '{}'", topic);
            Ok(())
        } else {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            Err(NotificationError::InvalidConfiguration(format!(
                "Ntfy request failed - HTTP {}: {}",
                status, error_body
            )))
        }
    }
}

impl Default for NtfySender {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NotificationSender for NtfySender {
    async fn send(
        &self,
        message: &str,
        channel: &NotificationChannel,
        context: &NotificationContext,
    ) -> NotificationResult<()> {
        match channel {
            NotificationChannel::Ntfy {
                server,
                topic,
                title,
                priority,
                tags,
                click_url,
                attach_url,
                markdown,
                auth_token,
            } => {
                let interpolated_message = context.interpolate(message)?;
                let interpolated_topic = context.interpolate(topic)?;
                let interpolated_title =
                    title.as_ref().map(|t| context.interpolate(t)).transpose()?;
                let interpolated_click = click_url
                    .as_ref()
                    .map(|u| context.interpolate(u))
                    .transpose()?;
                let interpolated_attach = attach_url
                    .as_ref()
                    .map(|u| context.interpolate(u))
                    .transpose()?;
                let interpolated_auth = auth_token
                    .as_ref()
                    .map(|t| context.interpolate(t))
                    .transpose()?;

                self.send_via_mcp(
                    &interpolated_message,
                    server,
                    &interpolated_topic,
                    interpolated_title.as_deref(),
                    *priority,
                    tags,
                    interpolated_click.as_deref(),
                    interpolated_attach.as_deref(),
                    *markdown,
                    interpolated_auth.as_deref(),
                )
                .await
            }
            _ => Err(NotificationError::UnsupportedChannel(
                "NtfySender only supports Ntfy channels".to_string(),
            )),
        }
    }

    fn channel_name(&self) -> &str {
        "ntfy"
    }
}

// ============================================================================
// Slack Sender (Webhook)
// ============================================================================

/// Slack notification sender using webhooks
pub struct SlackSender {
    http_client: reqwest::Client,
}

impl SlackSender {
    /// Create a new Slack sender
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
        }
    }

    /// Send notification via Slack webhook
    async fn send_webhook(
        &self,
        message: &str,
        webhook_url: &str,
        attachments: &[SlackAttachment],
    ) -> NotificationResult<()> {
        log::debug!("Sending Slack webhook notification");

        let mut payload = json!({
            "text": message,
        });

        if !attachments.is_empty() {
            let attachments_json: Vec<serde_json::Value> = attachments
                .iter()
                .map(|att| {
                    let mut obj = json!({
                        "text": att.text,
                    });
                    if let Some(color) = &att.color {
                        obj["color"] = json!(color);
                    }
                    if !att.fields.is_empty() {
                        obj["fields"] = json!(att
                            .fields
                            .iter()
                            .map(|f| {
                                json!({
                                    "title": f.title,
                                    "value": f.value,
                                    "short": f.short,
                                })
                            })
                            .collect::<Vec<_>>());
                    }
                    obj
                })
                .collect();
            payload["attachments"] = json!(attachments_json);
        }

        let response = self
            .http_client
            .post(webhook_url)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            log::info!("Successfully sent Slack webhook notification");
            Ok(())
        } else {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            Err(NotificationError::InvalidConfiguration(format!(
                "Slack webhook failed - HTTP {}: {}",
                status, error_body
            )))
        }
    }
}

impl Default for SlackSender {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NotificationSender for SlackSender {
    async fn send(
        &self,
        message: &str,
        channel: &NotificationChannel,
        context: &NotificationContext,
    ) -> NotificationResult<()> {
        match channel {
            NotificationChannel::Slack {
                credential,
                channel: _,
                method,
                attachments,
            } => {
                let interpolated_message = context.interpolate(message)?;
                let interpolated_credential = context.interpolate(credential)?;

                match method {
                    SlackMethod::Webhook => {
                        self.send_webhook(
                            &interpolated_message,
                            &interpolated_credential,
                            attachments,
                        )
                        .await
                    }
                    SlackMethod::Bot => {
                        // TODO: Implement Slack Bot API support
                        Err(NotificationError::UnsupportedChannel(
                            "Slack Bot API not yet implemented".to_string(),
                        ))
                    }
                }
            }
            _ => Err(NotificationError::UnsupportedChannel(
                "SlackSender only supports Slack channels".to_string(),
            )),
        }
    }

    fn channel_name(&self) -> &str {
        "slack"
    }
}

// ============================================================================
// Discord Sender (Webhook)
// ============================================================================

/// Discord notification sender using webhooks
pub struct DiscordSender {
    http_client: reqwest::Client,
}

impl DiscordSender {
    /// Create a new Discord sender
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
        }
    }

    /// Send notification via Discord webhook
    async fn send_webhook(
        &self,
        message: &str,
        webhook_url: &str,
        username: Option<&str>,
        avatar_url: Option<&str>,
        tts: bool,
        embed: Option<&DiscordEmbed>,
    ) -> NotificationResult<()> {
        log::debug!("Sending Discord webhook notification");

        let mut payload = json!({
            "content": message,
            "tts": tts,
        });

        if let Some(name) = username {
            payload["username"] = json!(name);
        }

        if let Some(avatar) = avatar_url {
            payload["avatar_url"] = json!(avatar);
        }

        if let Some(embed_data) = embed {
            let mut embed_obj = json!({});

            if let Some(title) = &embed_data.title {
                embed_obj["title"] = json!(title);
            }
            if let Some(desc) = &embed_data.description {
                embed_obj["description"] = json!(desc);
            }
            if let Some(color) = embed_data.color {
                embed_obj["color"] = json!(color);
            }
            if !embed_data.fields.is_empty() {
                embed_obj["fields"] = json!(embed_data
                    .fields
                    .iter()
                    .map(|f| {
                        json!({
                            "name": f.name,
                            "value": f.value,
                            "inline": f.inline,
                        })
                    })
                    .collect::<Vec<_>>());
            }
            if let Some(footer) = &embed_data.footer {
                embed_obj["footer"] = json!({ "text": footer });
            }
            if let Some(timestamp) = &embed_data.timestamp {
                embed_obj["timestamp"] = json!(timestamp);
            }

            payload["embeds"] = json!([embed_obj]);
        }

        let response = self
            .http_client
            .post(webhook_url)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            log::info!("Successfully sent Discord webhook notification");
            Ok(())
        } else {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            Err(NotificationError::InvalidConfiguration(format!(
                "Discord webhook failed - HTTP {}: {}",
                status, error_body
            )))
        }
    }
}

impl Default for DiscordSender {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NotificationSender for DiscordSender {
    async fn send(
        &self,
        message: &str,
        channel: &NotificationChannel,
        context: &NotificationContext,
    ) -> NotificationResult<()> {
        match channel {
            NotificationChannel::Discord {
                webhook_url,
                username,
                avatar_url,
                tts,
                embed,
            } => {
                let interpolated_message = context.interpolate(message)?;
                let interpolated_webhook = context.interpolate(webhook_url)?;

                self.send_webhook(
                    &interpolated_message,
                    &interpolated_webhook,
                    username.as_deref(),
                    avatar_url.as_deref(),
                    *tts,
                    embed.as_ref(),
                )
                .await
            }
            _ => Err(NotificationError::UnsupportedChannel(
                "DiscordSender only supports Discord channels".to_string(),
            )),
        }
    }

    fn channel_name(&self) -> &str {
        "discord"
    }
}

// ============================================================================
// Console Sender
// ============================================================================

/// Console notification sender (stdout)
pub struct ConsoleSender;

impl ConsoleSender {
    /// Create a new console sender
    pub fn new() -> Self {
        Self
    }
}

impl Default for ConsoleSender {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NotificationSender for ConsoleSender {
    async fn send(
        &self,
        message: &str,
        channel: &NotificationChannel,
        context: &NotificationContext,
    ) -> NotificationResult<()> {
        match channel {
            NotificationChannel::Console { colored, timestamp } => {
                let interpolated_message = context.interpolate(message)?;

                let output = if *timestamp {
                    format!(
                        "[{}] {}",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                        interpolated_message
                    )
                } else {
                    interpolated_message
                };

                if *colored {
                    // Use ANSI colors for console output
                    println!("\x1b[32m✓\x1b[0m {}", output);
                } else {
                    println!("✓ {}", output);
                }

                Ok(())
            }
            _ => Err(NotificationError::UnsupportedChannel(
                "ConsoleSender only supports Console channels".to_string(),
            )),
        }
    }

    fn channel_name(&self) -> &str {
        "console"
    }

    fn supports_retry(&self) -> bool {
        false // Console output doesn't need retry
    }
}

// ============================================================================
// File Sender
// ============================================================================

/// File-based notification sender
pub struct FileSender;

impl FileSender {
    /// Create a new file sender
    pub fn new() -> Self {
        Self
    }
}

impl Default for FileSender {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NotificationSender for FileSender {
    async fn send(
        &self,
        message: &str,
        channel: &NotificationChannel,
        context: &NotificationContext,
    ) -> NotificationResult<()> {
        match channel {
            NotificationChannel::File {
                path,
                append,
                timestamp,
                format,
            } => {
                use tokio::fs::OpenOptions;
                use tokio::io::AsyncWriteExt;

                let interpolated_message = context.interpolate(message)?;
                let interpolated_path = context.interpolate(path)?;

                let content = match format {
                    FileNotificationFormat::Text => {
                        if *timestamp {
                            format!(
                                "[{}] {}\n",
                                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                                interpolated_message
                            )
                        } else {
                            format!("{}\n", interpolated_message)
                        }
                    }
                    FileNotificationFormat::Json => {
                        let obj = json!({
                            "message": interpolated_message,
                            "timestamp": chrono::Local::now().to_rfc3339(),
                            "metadata": context.metadata,
                        });
                        format!("{}\n", serde_json::to_string_pretty(&obj)?)
                    }
                    FileNotificationFormat::JsonLines => {
                        let obj = json!({
                            "message": interpolated_message,
                            "timestamp": chrono::Local::now().to_rfc3339(),
                            "metadata": context.metadata,
                        });
                        format!("{}\n", serde_json::to_string(&obj)?)
                    }
                };

                let mut file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .append(*append)
                    .truncate(!*append)
                    .open(&interpolated_path)
                    .await?;

                file.write_all(content.as_bytes()).await?;
                file.flush().await?;

                log::info!(
                    "Successfully wrote notification to file: {}",
                    interpolated_path
                );
                Ok(())
            }
            _ => Err(NotificationError::UnsupportedChannel(
                "FileSender only supports File channels".to_string(),
            )),
        }
    }

    fn channel_name(&self) -> &str {
        "file"
    }

    fn supports_retry(&self) -> bool {
        false // File writes don't typically need retry
    }
}

// ============================================================================
// Placeholder Senders
// ============================================================================

/// Email sender (placeholder for future MCP integration)
pub struct EmailSender;

impl EmailSender {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EmailSender {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NotificationSender for EmailSender {
    async fn send(
        &self,
        _message: &str,
        _channel: &NotificationChannel,
        _context: &NotificationContext,
    ) -> NotificationResult<()> {
        // TODO: Implement email sending via SMTP or MCP integration
        // Requires: lettre crate or MCP email server
        Err(NotificationError::UnsupportedChannel(
            "Email notifications not yet implemented - TODO: Add SMTP support or MCP integration"
                .to_string(),
        ))
    }

    fn channel_name(&self) -> &str {
        "email"
    }
}

/// SMS sender (placeholder for future integration)
pub struct SmsSender;

impl SmsSender {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SmsSender {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NotificationSender for SmsSender {
    async fn send(
        &self,
        _message: &str,
        _channel: &NotificationChannel,
        _context: &NotificationContext,
    ) -> NotificationResult<()> {
        // TODO: Implement SMS sending via Twilio, SNS, or similar service
        Err(NotificationError::UnsupportedChannel(
            "SMS notifications not yet implemented - TODO: Add Twilio/SNS integration".to_string(),
        ))
    }

    fn channel_name(&self) -> &str {
        "sms"
    }
}

/// ElevenLabs voice notification sender (placeholder)
pub struct ElevenLabsSender;

impl ElevenLabsSender {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ElevenLabsSender {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NotificationSender for ElevenLabsSender {
    async fn send(
        &self,
        _message: &str,
        _channel: &NotificationChannel,
        _context: &NotificationContext,
    ) -> NotificationResult<()> {
        // TODO: Implement ElevenLabs TTS integration
        // Requires: ElevenLabs API key and audio playback
        Err(NotificationError::UnsupportedChannel(
            "ElevenLabs voice notifications not yet implemented - TODO: Add ElevenLabs API integration"
                .to_string(),
        ))
    }

    fn channel_name(&self) -> &str {
        "elevenlabs"
    }
}

// ============================================================================
// Notification Manager
// ============================================================================

/// Notification manager coordinates delivery across multiple channels
pub struct NotificationManager {
    senders: HashMap<String, Box<dyn NotificationSender>>,
}

impl NotificationManager {
    /// Create a new notification manager with default senders
    pub fn new() -> Self {
        let mut senders: HashMap<String, Box<dyn NotificationSender>> = HashMap::new();

        senders.insert("ntfy".to_string(), Box::new(NtfySender::new()));
        senders.insert("slack".to_string(), Box::new(SlackSender::new()));
        senders.insert("discord".to_string(), Box::new(DiscordSender::new()));
        senders.insert("console".to_string(), Box::new(ConsoleSender::new()));
        senders.insert("file".to_string(), Box::new(FileSender::new()));
        senders.insert("email".to_string(), Box::new(EmailSender::new()));
        senders.insert("sms".to_string(), Box::new(SmsSender::new()));
        senders.insert("elevenlabs".to_string(), Box::new(ElevenLabsSender::new()));

        Self { senders }
    }

    /// Register a custom sender
    pub fn register_sender(&mut self, name: String, sender: Box<dyn NotificationSender>) {
        self.senders.insert(name, sender);
    }

    /// Check if a sender is registered
    pub fn has_sender(&self, name: &str) -> bool {
        self.senders.contains_key(name)
    }

    /// Send a notification with retry logic
    async fn send_with_retry(
        &self,
        sender: &dyn NotificationSender,
        message: &str,
        channel: &NotificationChannel,
        context: &NotificationContext,
        retry_config: Option<&RetryConfig>,
    ) -> NotificationResult<()> {
        let max_attempts = retry_config.map(|c| c.max_attempts).unwrap_or(1);
        let delay_secs = retry_config.map(|c| c.delay_secs).unwrap_or(1);
        let exponential_backoff = retry_config.map(|c| c.exponential_backoff).unwrap_or(false);

        let mut last_error = None;

        for attempt in 0..max_attempts {
            match sender.send(message, channel, context).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    log::warn!(
                        "Notification attempt {} failed for {}: {}",
                        attempt + 1,
                        sender.channel_name(),
                        e
                    );
                    last_error = Some(e);

                    if attempt < max_attempts - 1 && sender.supports_retry() {
                        let delay = if exponential_backoff {
                            delay_secs * 2_u64.pow(attempt)
                        } else {
                            delay_secs
                        };
                        log::debug!("Retrying in {} seconds...", delay);
                        sleep(Duration::from_secs(delay)).await;
                    }
                }
            }
        }

        Err(NotificationError::RetryExhausted {
            channel: sender.channel_name().to_string(),
            attempts: max_attempts,
            last_error: last_error
                .map(|e| e.to_string())
                .unwrap_or_else(|| "Unknown error".to_string()),
        })
    }

    /// Send a notification
    pub async fn send(
        &self,
        spec: &NotificationSpec,
        context: &NotificationContext,
    ) -> NotificationResult<()> {
        let (message, channels, retry_config) = match spec {
            NotificationSpec::Simple(msg) => {
                // Simple notification - use console as default
                let default_channel = NotificationChannel::Console {
                    colored: true,
                    timestamp: true,
                };
                (msg.as_str(), vec![default_channel], None)
            }
            NotificationSpec::Structured {
                message,
                channels,
                title: _,
                priority: _,
                metadata: _,
            } => {
                // Extract retry config from webhook channels
                let retry = channels.iter().find_map(|ch| {
                    if let NotificationChannel::Webhook { retry, .. } = ch {
                        retry.as_ref()
                    } else {
                        None
                    }
                });
                (message.as_str(), channels.clone(), retry)
            }
        };

        if channels.is_empty() {
            log::warn!("No notification channels specified, using console");
            let default_channel = NotificationChannel::Console {
                colored: true,
                timestamp: true,
            };
            return self
                .send_to_channel(message, &default_channel, context, None)
                .await;
        }

        // Send to all channels
        // TODO: Implement true concurrent delivery with Arc<dyn NotificationSender>
        // For now, send sequentially to avoid borrowing issues
        for channel in channels {
            self.send_to_channel(message, &channel, context, retry_config)
                .await?;
        }

        Ok(())
    }

    /// Send to a specific channel
    async fn send_to_channel(
        &self,
        message: &str,
        channel: &NotificationChannel,
        context: &NotificationContext,
        retry_config: Option<&RetryConfig>,
    ) -> NotificationResult<()> {
        let sender_name = self.get_sender_name(channel);

        let sender = self.senders.get(&sender_name).ok_or_else(|| {
            NotificationError::UnsupportedChannel(format!(
                "No sender registered for channel: {}",
                sender_name
            ))
        })?;

        self.send_with_retry(sender.as_ref(), message, channel, context, retry_config)
            .await
    }

    /// Get the sender name for a channel
    fn get_sender_name(&self, channel: &NotificationChannel) -> String {
        match channel {
            NotificationChannel::Console { .. } => "console",
            NotificationChannel::Email { .. } => "email",
            NotificationChannel::Slack { .. } => "slack",
            NotificationChannel::Discord { .. } => "discord",
            NotificationChannel::Teams { .. } => "teams",
            NotificationChannel::Telegram { .. } => "telegram",
            NotificationChannel::PagerDuty { .. } => "pagerduty",
            NotificationChannel::Webhook { .. } => "webhook",
            NotificationChannel::File { .. } => "file",
            NotificationChannel::Ntfy { .. } => "ntfy",
        }
        .to_string()
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // NotificationContext Tests
    // ========================================================================

    #[test]
    fn test_context_creation() {
        let context = NotificationContext::new();
        assert!(context.workflow_vars.is_empty());
        assert!(context.task_vars.is_empty());
        assert!(context.agent_vars.is_empty());
        assert!(context.secrets.is_empty());
        assert!(context.metadata.is_empty());
    }

    #[test]
    fn test_context_builder_workflow_vars() {
        let context = NotificationContext::new()
            .with_workflow_var("key1", "value1")
            .with_workflow_var("key2", "value2");

        assert_eq!(context.workflow_vars.len(), 2);
        assert_eq!(context.workflow_vars.get("key1").unwrap(), "value1");
        assert_eq!(context.workflow_vars.get("key2").unwrap(), "value2");
    }

    #[test]
    fn test_context_builder_all_scopes() {
        let context = NotificationContext::new()
            .with_workflow_var("w", "workflow")
            .with_task_var("t", "task")
            .with_agent_var("a", "agent")
            .with_secret("s", "secret")
            .with_metadata("m", "meta");

        assert_eq!(context.workflow_vars.len(), 1);
        assert_eq!(context.task_vars.len(), 1);
        assert_eq!(context.agent_vars.len(), 1);
        assert_eq!(context.secrets.len(), 1);
        assert_eq!(context.metadata.len(), 1);
    }

    #[test]
    fn test_context_interpolation_workflow_var() {
        let context = NotificationContext::new().with_workflow_var("project", "my-app");

        let result = context.interpolate("Project: ${workflow.project}").unwrap();
        assert_eq!(result, "Project: my-app");
    }

    #[test]
    fn test_context_interpolation_task_var() {
        let context = NotificationContext::new().with_task_var("name", "build");

        let result = context.interpolate("Task: ${task.name}").unwrap();
        assert_eq!(result, "Task: build");
    }

    #[test]
    fn test_context_interpolation_agent_var() {
        let context = NotificationContext::new().with_agent_var("role", "deployer");

        let result = context.interpolate("Agent role: ${agent.role}").unwrap();
        assert_eq!(result, "Agent role: deployer");
    }

    #[test]
    fn test_context_interpolation_secret() {
        let context = NotificationContext::new().with_secret("api_key", "secret123");

        let result = context.interpolate("Key: ${secret.api_key}").unwrap();
        assert_eq!(result, "Key: secret123");
    }

    #[test]
    fn test_context_interpolation_metadata() {
        let context = NotificationContext::new().with_metadata("status", "success");

        let result = context.interpolate("Status: ${metadata.status}").unwrap();
        assert_eq!(result, "Status: success");
    }

    #[test]
    fn test_context_interpolation_multiple_vars() {
        let context = NotificationContext::new()
            .with_workflow_var("project", "my-app")
            .with_task_var("name", "build")
            .with_metadata("status", "success");

        let template = "Project ${workflow.project} - Task ${task.name} completed with status ${metadata.status}";
        let result = context.interpolate(template).unwrap();

        assert_eq!(
            result,
            "Project my-app - Task build completed with status success"
        );
    }

    #[test]
    fn test_context_interpolation_repeated_vars() {
        let context = NotificationContext::new().with_workflow_var("name", "test");

        let result = context
            .interpolate("${workflow.name} is ${workflow.name}")
            .unwrap();
        assert_eq!(result, "test is test");
    }

    #[test]
    fn test_context_interpolation_no_vars() {
        let context = NotificationContext::new();

        let result = context.interpolate("No variables here").unwrap();
        assert_eq!(result, "No variables here");
    }

    #[test]
    fn test_context_interpolation_unresolved() {
        let context = NotificationContext::new().with_workflow_var("project", "my-app");

        let template = "Project ${workflow.project} - Unknown ${task.missing}";
        let result = context.interpolate(template);

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(NotificationError::InterpolationError(_))
        ));
    }

    #[test]
    fn test_context_interpolation_empty_template() {
        let context = NotificationContext::new();
        let result = context.interpolate("").unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_context_interpolation_special_chars() {
        let context = NotificationContext::new().with_workflow_var("url", "http://example.com");

        let result = context
            .interpolate("URL: ${workflow.url}/path?q=test")
            .unwrap();
        assert_eq!(result, "URL: http://example.com/path?q=test");
    }

    // ========================================================================
    // NotificationSender Tests
    // ========================================================================

    #[tokio::test]
    async fn test_console_sender_basic() {
        let sender = ConsoleSender::new();
        let context = NotificationContext::new();
        let channel = NotificationChannel::Console {
            colored: false,
            timestamp: false,
        };

        let result = sender.send("Test message", &channel, &context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_console_sender_colored() {
        let sender = ConsoleSender::new();
        let context = NotificationContext::new();
        let channel = NotificationChannel::Console {
            colored: true,
            timestamp: false,
        };

        let result = sender.send("Colored message", &channel, &context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_console_sender_with_timestamp() {
        let sender = ConsoleSender::new();
        let context = NotificationContext::new();
        let channel = NotificationChannel::Console {
            colored: false,
            timestamp: true,
        };

        let result = sender.send("Timestamped message", &channel, &context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_console_sender_with_interpolation() {
        let sender = ConsoleSender::new();
        let context = NotificationContext::new().with_workflow_var("name", "TestProject");

        let channel = NotificationChannel::Console {
            colored: false,
            timestamp: false,
        };

        let result = sender
            .send("Project: ${workflow.name}", &channel, &context)
            .await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_console_sender_no_retry() {
        let sender = ConsoleSender::new();
        assert!(!sender.supports_retry());
    }

    #[test]
    fn test_console_sender_channel_name() {
        let sender = ConsoleSender::new();
        assert_eq!(sender.channel_name(), "console");
    }

    #[tokio::test]
    async fn test_console_sender_wrong_channel() {
        let sender = ConsoleSender::new();
        let context = NotificationContext::new();
        let channel = NotificationChannel::File {
            path: "/tmp/test.log".to_string(),
            append: true,
            timestamp: false,
            format: crate::dsl::schema::FileNotificationFormat::Text,
        };

        let result = sender.send("Test", &channel, &context).await;
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(NotificationError::UnsupportedChannel(_))
        ));
    }

    // ========================================================================
    // FileSender Tests
    // ========================================================================

    #[tokio::test]
    async fn test_file_sender_basic() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();

        let sender = FileSender::new();
        let context = NotificationContext::new();
        let channel = NotificationChannel::File {
            path: file_path.clone(),
            append: false,
            timestamp: false,
            format: crate::dsl::schema::FileNotificationFormat::Text,
        };

        let result = sender.send("Test message", &channel, &context).await;
        assert!(result.is_ok());

        let contents = tokio::fs::read_to_string(&file_path).await.unwrap();
        assert!(contents.contains("Test message"));
    }

    #[tokio::test]
    async fn test_file_sender_json_format() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();

        let sender = FileSender::new();
        let context = NotificationContext::new().with_metadata("task", "test-task");
        let channel = NotificationChannel::File {
            path: file_path.clone(),
            append: false,
            timestamp: true,
            format: crate::dsl::schema::FileNotificationFormat::Json,
        };

        let result = sender.send("JSON test", &channel, &context).await;
        assert!(result.is_ok());

        let contents = tokio::fs::read_to_string(&file_path).await.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&contents).unwrap();
        assert_eq!(parsed["message"], "JSON test");
        assert!(parsed["timestamp"].is_string());
    }

    #[tokio::test]
    async fn test_file_sender_jsonlines_format() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();

        let sender = FileSender::new();
        let context = NotificationContext::new();
        let channel = NotificationChannel::File {
            path: file_path.clone(),
            append: false,
            timestamp: false,
            format: crate::dsl::schema::FileNotificationFormat::JsonLines,
        };

        let result = sender.send("JSONL test", &channel, &context).await;
        assert!(result.is_ok());

        let contents = tokio::fs::read_to_string(&file_path).await.unwrap();
        // Should be single-line JSON
        assert!(!contents.trim().contains("\n  "));
    }

    #[tokio::test]
    async fn test_file_sender_append_mode() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();

        let sender = FileSender::new();
        let context = NotificationContext::new();

        // First write
        let channel1 = NotificationChannel::File {
            path: file_path.clone(),
            append: false,
            timestamp: false,
            format: crate::dsl::schema::FileNotificationFormat::Text,
        };
        sender.send("First", &channel1, &context).await.unwrap();

        // Second write (append)
        let channel2 = NotificationChannel::File {
            path: file_path.clone(),
            append: true,
            timestamp: false,
            format: crate::dsl::schema::FileNotificationFormat::Text,
        };
        sender.send("Second", &channel2, &context).await.unwrap();

        let contents = tokio::fs::read_to_string(&file_path).await.unwrap();
        assert!(contents.contains("First"));
        assert!(contents.contains("Second"));
    }

    #[test]
    fn test_file_sender_no_retry() {
        let sender = FileSender::new();
        assert!(!sender.supports_retry());
    }

    #[test]
    fn test_file_sender_channel_name() {
        let sender = FileSender::new();
        assert_eq!(sender.channel_name(), "file");
    }

    // ========================================================================
    // NtfySender Tests
    // ========================================================================

    #[test]
    fn test_ntfy_sender_creation() {
        let sender = NtfySender::new();
        assert_eq!(sender.channel_name(), "ntfy");
    }

    #[test]
    fn test_ntfy_sender_supports_retry() {
        let sender = NtfySender::new();
        assert!(sender.supports_retry());
    }

    #[tokio::test]
    async fn test_ntfy_sender_wrong_channel() {
        let sender = NtfySender::new();
        let context = NotificationContext::new();
        let channel = NotificationChannel::Console {
            colored: false,
            timestamp: false,
        };

        let result = sender.send("Test", &channel, &context).await;
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(NotificationError::UnsupportedChannel(_))
        ));
    }

    // ========================================================================
    // SlackSender Tests
    // ========================================================================

    #[test]
    fn test_slack_sender_creation() {
        let sender = SlackSender::new();
        assert_eq!(sender.channel_name(), "slack");
    }

    #[test]
    fn test_slack_sender_supports_retry() {
        let sender = SlackSender::new();
        assert!(sender.supports_retry());
    }

    #[tokio::test]
    async fn test_slack_sender_wrong_channel() {
        let sender = SlackSender::new();
        let context = NotificationContext::new();
        let channel = NotificationChannel::Console {
            colored: false,
            timestamp: false,
        };

        let result = sender.send("Test", &channel, &context).await;
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(NotificationError::UnsupportedChannel(_))
        ));
    }

    // ========================================================================
    // DiscordSender Tests
    // ========================================================================

    #[test]
    fn test_discord_sender_creation() {
        let sender = DiscordSender::new();
        assert_eq!(sender.channel_name(), "discord");
    }

    #[test]
    fn test_discord_sender_supports_retry() {
        let sender = DiscordSender::new();
        assert!(sender.supports_retry());
    }

    #[tokio::test]
    async fn test_discord_sender_wrong_channel() {
        let sender = DiscordSender::new();
        let context = NotificationContext::new();
        let channel = NotificationChannel::Console {
            colored: false,
            timestamp: false,
        };

        let result = sender.send("Test", &channel, &context).await;
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(NotificationError::UnsupportedChannel(_))
        ));
    }

    // ========================================================================
    // Placeholder Sender Tests
    // ========================================================================

    #[tokio::test]
    async fn test_email_sender_not_implemented() {
        let sender = EmailSender::new();
        let context = NotificationContext::new();
        let channel = NotificationChannel::Console {
            colored: false,
            timestamp: false,
        };

        let result = sender.send("Test", &channel, &context).await;
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(NotificationError::UnsupportedChannel(_))
        ));
    }

    #[tokio::test]
    async fn test_sms_sender_not_implemented() {
        let sender = SmsSender::new();
        let context = NotificationContext::new();
        let channel = NotificationChannel::Console {
            colored: false,
            timestamp: false,
        };

        let result = sender.send("Test", &channel, &context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_elevenlabs_sender_not_implemented() {
        let sender = ElevenLabsSender::new();
        let context = NotificationContext::new();
        let channel = NotificationChannel::Console {
            colored: false,
            timestamp: false,
        };

        let result = sender.send("Test", &channel, &context).await;
        assert!(result.is_err());
    }

    // ========================================================================
    // NotificationManager Tests
    // ========================================================================

    #[test]
    fn test_notification_manager_creation() {
        let manager = NotificationManager::new();
        assert!(manager.senders.contains_key("ntfy"));
        assert!(manager.senders.contains_key("slack"));
        assert!(manager.senders.contains_key("discord"));
        assert!(manager.senders.contains_key("console"));
        assert!(manager.senders.contains_key("file"));
        assert!(manager.senders.contains_key("email"));
        assert!(manager.senders.contains_key("sms"));
        assert!(manager.senders.contains_key("elevenlabs"));
    }

    #[test]
    fn test_notification_manager_has_sender() {
        let manager = NotificationManager::new();
        assert!(manager.has_sender("console"));
        assert!(manager.has_sender("file"));
        assert!(!manager.has_sender("nonexistent"));
    }

    #[tokio::test]
    async fn test_notification_manager_simple() {
        let manager = NotificationManager::new();
        let context = NotificationContext::new();
        let spec = NotificationSpec::Simple("Test notification".to_string());

        let result = manager.send(&spec, &context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_notification_manager_structured() {
        let manager = NotificationManager::new();
        let context = NotificationContext::new();

        let spec = NotificationSpec::Structured {
            message: "Structured test".to_string(),
            channels: vec![NotificationChannel::Console {
                colored: false,
                timestamp: false,
            }],
            title: Some("Test".to_string()),
            priority: None,
            metadata: HashMap::new(),
        };

        let result = manager.send(&spec, &context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_notification_manager_empty_channels() {
        let manager = NotificationManager::new();
        let context = NotificationContext::new();

        let spec = NotificationSpec::Structured {
            message: "No channels".to_string(),
            channels: vec![],
            title: None,
            priority: None,
            metadata: HashMap::new(),
        };

        // Should default to console
        let result = manager.send(&spec, &context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_notification_manager_multiple_channels() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_string_lossy().to_string();

        let manager = NotificationManager::new();
        let context = NotificationContext::new();

        let spec = NotificationSpec::Structured {
            message: "Multi-channel".to_string(),
            channels: vec![
                NotificationChannel::Console {
                    colored: false,
                    timestamp: false,
                },
                NotificationChannel::File {
                    path: file_path.clone(),
                    append: false,
                    timestamp: false,
                    format: crate::dsl::schema::FileNotificationFormat::Text,
                },
            ],
            title: None,
            priority: None,
            metadata: HashMap::new(),
        };

        let result = manager.send(&spec, &context).await;
        assert!(result.is_ok());

        // Verify file was written
        let contents = tokio::fs::read_to_string(&file_path).await.unwrap();
        assert!(contents.contains("Multi-channel"));
    }

    #[test]
    fn test_notification_manager_get_sender_name() {
        let manager = NotificationManager::new();

        let console_channel = NotificationChannel::Console {
            colored: true,
            timestamp: true,
        };
        assert_eq!(manager.get_sender_name(&console_channel), "console");

        let file_channel = NotificationChannel::File {
            path: "/tmp/test.log".to_string(),
            append: true,
            timestamp: false,
            format: crate::dsl::schema::FileNotificationFormat::Text,
        };
        assert_eq!(manager.get_sender_name(&file_channel), "file");
    }

    // ========================================================================
    // Error Type Tests
    // ========================================================================

    #[test]
    fn test_notification_error_display() {
        let err = NotificationError::InterpolationError("test error".to_string());
        assert!(err.to_string().contains("test error"));

        let err = NotificationError::UnsupportedChannel("test_channel".to_string());
        assert!(err.to_string().contains("test_channel"));

        let err = NotificationError::MissingField("test_field".to_string());
        assert!(err.to_string().contains("test_field"));

        let err = NotificationError::RetryExhausted {
            channel: "test_channel".to_string(),
            attempts: 3,
            last_error: "connection failed".to_string(),
        };
        assert!(err.to_string().contains("test_channel"));
        assert!(err.to_string().contains("connection failed"));
    }

    // ========================================================================
    // Schema Serialization Tests
    // ========================================================================

    #[test]
    fn test_notification_spec_simple_serialization() {
        let spec = NotificationSpec::Simple("Test message".to_string());
        let json = serde_json::to_string(&spec).unwrap();
        assert_eq!(json, r#""Test message""#);
    }

    #[test]
    fn test_notification_spec_simple_deserialization() {
        let json = r#""Test message""#;
        let spec: NotificationSpec = serde_json::from_str(json).unwrap();
        match spec {
            NotificationSpec::Simple(msg) => assert_eq!(msg, "Test message"),
            _ => panic!("Expected Simple variant"),
        }
    }

    #[test]
    fn test_notification_spec_structured_serialization() {
        let spec = NotificationSpec::Structured {
            message: "Test".to_string(),
            channels: vec![],
            title: Some("Title".to_string()),
            priority: Some(crate::dsl::schema::NotificationPriority::High),
            metadata: HashMap::new(),
        };
        let json = serde_json::to_value(&spec).unwrap();
        assert_eq!(json["message"], "Test");
        assert_eq!(json["title"], "Title");
        assert_eq!(json["priority"], "high");
    }

    #[test]
    fn test_console_channel_serialization() {
        let channel = NotificationChannel::Console {
            colored: true,
            timestamp: false,
        };
        let json = serde_json::to_value(&channel).unwrap();
        assert_eq!(json["type"], "console");
        assert_eq!(json["colored"], true);
        assert_eq!(json["timestamp"], false);
    }

    #[test]
    fn test_file_channel_serialization() {
        let channel = NotificationChannel::File {
            path: "/tmp/test.log".to_string(),
            append: true,
            timestamp: true,
            format: crate::dsl::schema::FileNotificationFormat::Json,
        };
        let json = serde_json::to_value(&channel).unwrap();
        assert_eq!(json["type"], "file");
        assert_eq!(json["path"], "/tmp/test.log");
        assert_eq!(json["format"], "json");
    }
}
