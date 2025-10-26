# Notification Schema Design

## Overview

This document defines the schema design for an extensible, multi-channel notification system for the DSL workflow engine. The design supports multiple notification channels (Email, SMS, Slack, Discord, Ntfy, Elevenlabs, webhooks), conditional delivery, variable interpolation, and integration with the existing secrets management system.

## Design Principles

1. **Extensibility** - Easy to add new notification channels
2. **Flexibility** - Support simple string notifications and complex configurations
3. **Security** - Integrate with secrets management for credentials
4. **Reliability** - Include retry logic and fallback channels
5. **Non-blocking** - Notification failures should not fail tasks by default
6. **Variable support** - Full integration with existing variable interpolation system

## Core Schema Components

### NotificationSpec

The main notification specification that replaces the simple `notify: Option<String>` in `ActionSpec`.

```rust
/// Notification specification with support for multiple channels and conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NotificationSpec {
    /// Simple string notification (backward compatible)
    /// Sends to console output
    Simple(String),

    /// Detailed notification configuration
    Detailed {
        /// Notification message (supports variable interpolation)
        message: String,

        /// Optional title (supports variable interpolation)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        title: Option<String>,

        /// Notification channels to use
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        channels: Vec<NotificationChannel>,

        /// Condition for sending notification
        #[serde(default, skip_serializing_if = "Option::is_none")]
        condition: Option<ConditionSpec>,

        /// Notification priority (affects routing and delivery)
        #[serde(default, skip_serializing_if = "is_default_priority")]
        priority: NotificationPriority,

        /// Retry configuration for failed deliveries
        #[serde(default, skip_serializing_if = "Option::is_none")]
        retry: Option<NotificationRetry>,

        /// Fail task if notification delivery fails (default: false)
        #[serde(default, skip_serializing_if = "is_false")]
        fail_on_error: bool,

        /// Template engine to use (default: simple)
        #[serde(default, skip_serializing_if = "is_default_template_engine")]
        template_engine: TemplateEngine,

        /// Additional metadata to include in notification
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        metadata: HashMap<String, String>,
    },
}
```

### NotificationPriority

```rust
/// Notification priority levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NotificationPriority {
    /// Low priority - can be batched/delayed
    Low,

    /// Normal priority (default)
    Normal,

    /// High priority - immediate delivery
    High,

    /// Critical/urgent - all channels, immediate
    Critical,
}

impl Default for NotificationPriority {
    fn default() -> Self {
        NotificationPriority::Normal
    }
}

fn is_default_priority(priority: &NotificationPriority) -> bool {
    *priority == NotificationPriority::Normal
}
```

### NotificationRetry

```rust
/// Retry configuration for notification delivery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRetry {
    /// Maximum number of retry attempts
    #[serde(default = "default_notification_retries")]
    pub max_attempts: u32,

    /// Initial delay between retries in seconds
    #[serde(default = "default_notification_delay")]
    pub delay_secs: u64,

    /// Use exponential backoff
    #[serde(default, skip_serializing_if = "is_false")]
    pub exponential_backoff: bool,

    /// Timeout for each notification attempt in seconds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
}

fn default_notification_retries() -> u32 {
    3
}

fn default_notification_delay() -> u64 {
    2
}
```

### TemplateEngine

```rust
/// Template engine for notification messages
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TemplateEngine {
    /// Simple variable interpolation (${var})
    Simple,

    /// Handlebars templating (future)
    Handlebars,

    /// Jinja2-like templating (future)
    Jinja,
}

impl Default for TemplateEngine {
    fn default() -> Self {
        TemplateEngine::Simple
    }
}

fn is_default_template_engine(engine: &TemplateEngine) -> bool {
    *engine == TemplateEngine::Simple
}
```

## Notification Channels

### NotificationChannel Enum

```rust
/// Notification channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NotificationChannel {
    /// Console output (stdout)
    Console {
        /// Use colored output
        #[serde(default = "default_true")]
        colored: bool,

        /// Include timestamp
        #[serde(default = "default_true")]
        timestamp: bool,
    },

    /// Email notification
    Email {
        /// Recipient email addresses
        to: Vec<String>,

        /// CC recipients
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        cc: Vec<String>,

        /// BCC recipients
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        bcc: Vec<String>,

        /// Email subject (supports variable interpolation)
        subject: String,

        /// Email body format
        #[serde(default)]
        format: EmailFormat,

        /// SMTP configuration reference (from secrets or config)
        smtp_config: String,

        /// From address (optional, may be in SMTP config)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        from: Option<String>,

        /// Attachments (file paths)
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attachments: Vec<String>,
    },

    /// SMS notification
    Sms {
        /// Phone number(s) to send to (E.164 format)
        to: Vec<String>,

        /// SMS provider configuration
        provider: SmsProvider,

        /// Message (max 160 chars for single SMS)
        /// If longer, will be split or truncated based on provider
        #[serde(skip)]  // Message comes from NotificationSpec.message
        _phantom: (),
    },

    /// Slack notification
    Slack {
        /// Slack webhook URL (can reference secret: ${secret.slack_webhook})
        #[serde(default, skip_serializing_if = "Option::is_none")]
        webhook_url: Option<String>,

        /// Slack channel (requires bot token)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        channel: Option<String>,

        /// Slack bot token (can reference secret: ${secret.slack_token})
        #[serde(default, skip_serializing_if = "Option::is_none")]
        bot_token: Option<String>,

        /// Thread timestamp (for threaded replies)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        thread_ts: Option<String>,

        /// Message format
        #[serde(default)]
        format: SlackFormat,

        /// Slack username to display
        #[serde(default, skip_serializing_if = "Option::is_none")]
        username: Option<String>,

        /// Icon emoji (:ghost:)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        icon_emoji: Option<String>,

        /// Icon URL
        #[serde(default, skip_serializing_if = "Option::is_none")]
        icon_url: Option<String>,

        /// Slack blocks (advanced formatting)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        blocks: Option<Vec<serde_json::Value>>,
    },

    /// Discord notification
    Discord {
        /// Discord webhook URL (can reference secret)
        webhook_url: String,

        /// Username to display
        #[serde(default, skip_serializing_if = "Option::is_none")]
        username: Option<String>,

        /// Avatar URL
        #[serde(default, skip_serializing_if = "Option::is_none")]
        avatar_url: Option<String>,

        /// Text-to-speech
        #[serde(default, skip_serializing_if = "is_false")]
        tts: bool,

        /// Rich embed
        #[serde(default, skip_serializing_if = "Option::is_none")]
        embed: Option<DiscordEmbed>,
    },

    /// Ntfy notification (via MCP or direct)
    Ntfy {
        /// Ntfy topic
        topic: String,

        /// Ntfy server URL (default: ntfy.sh)
        #[serde(default = "default_ntfy_server")]
        server: String,

        /// Priority (1-5 or min/low/default/high/urgent)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        priority: Option<String>,

        /// Tags (emoji shortcodes or text)
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        tags: Vec<String>,

        /// Click action URL
        #[serde(default, skip_serializing_if = "Option::is_none")]
        click: Option<String>,

        /// Attachment URL
        #[serde(default, skip_serializing_if = "Option::is_none")]
        attach: Option<String>,

        /// Action buttons JSON
        #[serde(default, skip_serializing_if = "Option::is_none")]
        actions: Option<String>,

        /// Forward to email
        #[serde(default, skip_serializing_if = "Option::is_none")]
        email: Option<String>,

        /// Phone call
        #[serde(default, skip_serializing_if = "Option::is_none")]
        call: Option<String>,

        /// Scheduled delivery
        #[serde(default, skip_serializing_if = "Option::is_none")]
        delay: Option<String>,

        /// Enable Markdown rendering
        #[serde(default, skip_serializing_if = "is_false")]
        markdown: bool,

        /// Icon URL
        #[serde(default, skip_serializing_if = "Option::is_none")]
        icon: Option<String>,

        /// Authentication (via secrets)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        auth: Option<NtfyAuth>,
    },

    /// Elevenlabs voice notification
    Elevenlabs {
        /// Phone number to call
        phone_number: String,

        /// Elevenlabs API key (can reference secret)
        api_key: String,

        /// Voice ID to use
        voice_id: String,

        /// Agent ID (if using conversational AI)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        agent_id: Option<String>,

        /// Language code (e.g., "en-US")
        #[serde(default, skip_serializing_if = "Option::is_none")]
        language: Option<String>,

        /// Additional parameters
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        parameters: HashMap<String, serde_json::Value>,
    },

    /// Custom webhook
    Webhook {
        /// Webhook URL (supports variable interpolation)
        url: String,

        /// HTTP method
        #[serde(default = "default_webhook_method")]
        method: HttpMethod,

        /// Request headers
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        headers: HashMap<String, String>,

        /// Authentication
        #[serde(default, skip_serializing_if = "Option::is_none")]
        auth: Option<HttpAuth>,

        /// Payload template (JSON)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        payload_template: Option<String>,

        /// Timeout in seconds
        #[serde(default, skip_serializing_if = "Option::is_none")]
        timeout_secs: Option<u64>,

        /// Verify TLS certificates
        #[serde(default = "default_true")]
        verify_tls: bool,
    },

    /// MCP tool invocation (for extensibility)
    McpTool {
        /// MCP server name
        server: String,

        /// Tool name
        tool: String,

        /// Tool parameters (supports variable interpolation)
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        parameters: HashMap<String, serde_json::Value>,

        /// Timeout in seconds
        #[serde(default, skip_serializing_if = "Option::is_none")]
        timeout_secs: Option<u64>,
    },
}

fn default_true() -> bool {
    true
}

fn default_ntfy_server() -> String {
    "https://ntfy.sh".to_string()
}

fn default_webhook_method() -> HttpMethod {
    HttpMethod::Post
}
```

### Supporting Types for Channels

```rust
/// Email format
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EmailFormat {
    /// Plain text
    Text,

    /// HTML
    Html,

    /// Markdown (converted to HTML)
    Markdown,
}

impl Default for EmailFormat {
    fn default() -> Self {
        EmailFormat::Text
    }
}

/// SMS provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SmsProvider {
    /// Twilio
    Twilio {
        /// Account SID (can reference secret)
        account_sid: String,

        /// Auth token (can reference secret)
        auth_token: String,

        /// From phone number
        from: String,
    },

    /// AWS SNS
    AwsSns {
        /// AWS region
        region: String,

        /// AWS access key (can reference secret)
        access_key: String,

        /// AWS secret key (can reference secret)
        secret_key: String,

        /// Sender ID (optional)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        sender_id: Option<String>,
    },

    /// Generic HTTP SMS gateway
    HttpGateway {
        /// Gateway URL template
        url_template: String,

        /// HTTP method
        #[serde(default = "default_webhook_method")]
        method: HttpMethod,

        /// Headers
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        headers: HashMap<String, String>,

        /// Authentication
        #[serde(default, skip_serializing_if = "Option::is_none")]
        auth: Option<HttpAuth>,
    },
}

/// Slack message format
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SlackFormat {
    /// Plain text
    Text,

    /// Slack markdown (mrkdwn)
    Markdown,

    /// Slack blocks (use blocks field)
    Blocks,
}

impl Default for SlackFormat {
    fn default() -> Self {
        SlackFormat::Text
    }
}

/// Discord embed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordEmbed {
    /// Embed title
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// URL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Color (decimal)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<u32>,

    /// Timestamp
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,

    /// Fields
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<DiscordEmbedField>,

    /// Thumbnail
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<DiscordEmbedImage>,

    /// Image
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<DiscordEmbedImage>,

    /// Footer
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub footer: Option<DiscordEmbedFooter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordEmbedField {
    pub name: String,
    pub value: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub inline: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordEmbedImage {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordEmbedFooter {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
}

/// Ntfy authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NtfyAuth {
    /// Bearer token
    Token {
        /// Token value (can reference secret)
        token: String,
    },

    /// Basic authentication
    Basic {
        /// Username
        username: String,

        /// Password (can reference secret)
        password: String,
    },
}
```

## Integration with Existing Systems

### ActionSpec Update

```rust
/// Action specification (updated)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionSpec {
    /// Notification configuration
    /// Supports both simple strings and detailed specifications
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notify: Option<NotificationSpec>,

    /// Future: Other actions (approval, wait, etc.)
}
```

### TaskSpec Integration Points

Notifications can be triggered at multiple points:

```rust
pub struct TaskSpec {
    // ... existing fields ...

    /// Action on task start
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_start: Option<ActionSpec>,

    /// Action on task completion (existing)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_complete: Option<ActionSpec>,

    /// Action on task error (could be added to ErrorHandlingSpec)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_error: Option<ErrorHandlingSpec>,  // Could include ActionSpec
}
```

### Secrets Integration

All credential fields support secret references:

```yaml
secrets:
  slack_webhook:
    source:
      type: env
      var: SLACK_WEBHOOK_URL

  twilio_auth:
    source:
      type: file
      path: /secure/twilio.token

  smtp_password:
    source:
      type: env
      var: SMTP_PASSWORD

tasks:
  notify_completion:
    agent: notifier
    on_complete:
      notify:
        message: "Task completed successfully!"
        channels:
          - type: slack
            webhook_url: "${secret.slack_webhook}"
          - type: email
            to: ["admin@example.com"]
            subject: "Workflow Complete"
            smtp_config: "${secret.smtp_config}"
```

### Variable Interpolation

All message and string fields support variable interpolation:

```yaml
tasks:
  process_data:
    description: "Process data file"
    agent: processor
    inputs:
      filename: "data.csv"
    on_complete:
      notify:
        title: "Processing Complete"
        message: |
          File ${task.inputs.filename} has been processed.
          Total rows: ${task.outputs.row_count}
          Status: ${task.status}
          Duration: ${task.duration_secs}s
        channels:
          - type: email
            to: ["${workflow.admin_email}"]
            subject: "Data Processing Complete - ${task.inputs.filename}"
```

## Example YAML Configurations

### Example 1: Simple Console Notification (Backward Compatible)

```yaml
tasks:
  build:
    agent: builder
    description: "Build the project"
    on_complete:
      notify: "Build completed successfully!"
```

### Example 2: Multi-Channel Notification

```yaml
secrets:
  slack_webhook:
    source:
      type: env
      var: SLACK_WEBHOOK_URL
  admin_email:
    source:
      type: env
      var: ADMIN_EMAIL

tasks:
  deploy:
    agent: deployer
    description: "Deploy to production"
    on_complete:
      notify:
        title: "Deployment Complete"
        message: |
          Production deployment completed successfully!
          Version: ${workflow.version}
          Environment: ${workflow.environment}
          Deployed by: ${workflow.user}
        priority: high
        channels:
          - type: console
            colored: true
            timestamp: true

          - type: slack
            webhook_url: "${secret.slack_webhook}"
            format: markdown
            icon_emoji: ":rocket:"
            username: "Deploy Bot"

          - type: email
            to: ["${secret.admin_email}"]
            subject: "Production Deployment - ${workflow.version}"
            format: html
            smtp_config: "smtp_main"
```

### Example 3: Ntfy Notification with Rich Features

```yaml
secrets:
  ntfy_topic:
    source:
      type: env
      var: NTFY_TOPIC
  admin_phone:
    source:
      type: env
      var: ADMIN_PHONE

tasks:
  critical_check:
    agent: monitor
    description: "Monitor critical service"
    on_error:
      retry: 3
      # Notification on error (could be part of ActionSpec in on_error)
    on_complete:
      notify:
        title: "Critical Check Failed"
        message: "Service monitoring detected an issue requiring immediate attention."
        priority: critical
        channels:
          - type: ntfy
            topic: "${secret.ntfy_topic}"
            server: "https://ntfy.sh"
            priority: "urgent"
            tags: ["warning", "fire"]
            click: "https://dashboard.example.com/alerts"
            email: "${secret.admin_email}"
            call: "${secret.admin_phone}"
            actions: |
              [
                {
                  "action": "view",
                  "label": "View Dashboard",
                  "url": "https://dashboard.example.com"
                },
                {
                  "action": "http",
                  "label": "Acknowledge",
                  "url": "https://api.example.com/ack",
                  "method": "POST"
                }
              ]
            markdown: true
        retry:
          max_attempts: 5
          delay_secs: 10
          exponential_backoff: true
        fail_on_error: true
```

### Example 4: SMS Notification via Twilio

```yaml
secrets:
  twilio_sid:
    source:
      type: env
      var: TWILIO_ACCOUNT_SID
  twilio_token:
    source:
      type: env
      var: TWILIO_AUTH_TOKEN
  twilio_from:
    source:
      type: env
      var: TWILIO_PHONE_NUMBER
  oncall_phone:
    source:
      type: env
      var: ONCALL_PHONE

tasks:
  production_issue:
    agent: detector
    description: "Detect production issues"
    on_error:
      notify:
        title: "Production Alert"
        message: "URGENT: Production issue detected. Check logs immediately."
        priority: critical
        channels:
          - type: sms
            to: ["${secret.oncall_phone}"]
            provider:
              type: twilio
              account_sid: "${secret.twilio_sid}"
              auth_token: "${secret.twilio_token}"
              from: "${secret.twilio_from}"
        retry:
          max_attempts: 3
          delay_secs: 5
        fail_on_error: false
```

### Example 5: Discord Webhook with Embed

```yaml
secrets:
  discord_webhook:
    source:
      type: env
      var: DISCORD_WEBHOOK_URL

tasks:
  test_suite:
    agent: tester
    description: "Run test suite"
    outputs:
      test_results:
        source:
          type: file
          path: "./test-results.json"
    on_complete:
      notify:
        message: "Test suite execution completed"
        channels:
          - type: discord
            webhook_url: "${secret.discord_webhook}"
            username: "Test Bot"
            avatar_url: "https://example.com/bot-avatar.png"
            embed:
              title: "Test Suite Results"
              description: "Automated test execution completed"
              color: 3066993  # Green
              url: "https://ci.example.com/build/${workflow.build_id}"
              fields:
                - name: "Total Tests"
                  value: "${task.outputs.test_results.total}"
                  inline: true
                - name: "Passed"
                  value: "${task.outputs.test_results.passed}"
                  inline: true
                - name: "Failed"
                  value: "${task.outputs.test_results.failed}"
                  inline: true
                - name: "Duration"
                  value: "${task.duration_secs}s"
                  inline: true
              thumbnail:
                url: "https://example.com/test-icon.png"
              footer:
                text: "CI/CD Pipeline"
                icon_url: "https://example.com/ci-icon.png"
              timestamp: "${task.completed_at}"
```

### Example 6: Elevenlabs Voice Call

```yaml
secrets:
  elevenlabs_key:
    source:
      type: env
      var: ELEVENLABS_API_KEY
  emergency_contact:
    source:
      type: env
      var: EMERGENCY_PHONE

tasks:
  emergency_alert:
    agent: monitor
    description: "Monitor for emergency conditions"
    on_error:
      notify:
        title: "Emergency Alert"
        message: |
          Emergency condition detected in the system.
          Immediate action required.
          Issue: ${error.message}
          Time: ${task.error_time}
        priority: critical
        channels:
          - type: elevenlabs
            phone_number: "${secret.emergency_contact}"
            api_key: "${secret.elevenlabs_key}"
            voice_id: "21m00Tcm4TlvDq8ikWAM"  # Rachel voice
            agent_id: "emergency_caller"
            language: "en-US"
        retry:
          max_attempts: 5
          delay_secs: 30
          exponential_backoff: true
        fail_on_error: false
```

### Example 7: Custom Webhook with Template

```yaml
secrets:
  webhook_auth:
    source:
      type: env
      var: WEBHOOK_BEARER_TOKEN

tasks:
  data_sync:
    agent: syncer
    description: "Sync data to external system"
    on_complete:
      notify:
        message: "Data sync completed"
        channels:
          - type: webhook
            url: "https://api.example.com/notifications"
            method: POST
            headers:
              Content-Type: "application/json"
              X-Service: "workflow-engine"
            auth:
              type: bearer
              token: "${secret.webhook_auth}"
            payload_template: |
              {
                "event": "workflow.task.completed",
                "task": {
                  "id": "${task.id}",
                  "name": "${task.description}",
                  "status": "${task.status}",
                  "duration": ${task.duration_secs}
                },
                "workflow": {
                  "id": "${workflow.id}",
                  "name": "${workflow.name}",
                  "version": "${workflow.version}"
                },
                "metadata": {
                  "timestamp": "${task.completed_at}",
                  "user": "${workflow.user}"
                }
              }
            timeout_secs: 30
            verify_tls: true
```

### Example 8: Conditional Notifications

```yaml
tasks:
  quality_check:
    agent: checker
    description: "Run quality checks"
    outputs:
      quality_score:
        source:
          type: state
          key: "quality_score"
    on_complete:
      notify:
        title: "Quality Check Complete"
        message: "Quality score: ${task.outputs.quality_score}"
        # Only notify if quality score is below threshold
        condition:
          type: state_exists
          key: "quality_score"
        channels:
          - type: slack
            webhook_url: "${secret.slack_webhook}"

  critical_failure:
    agent: checker
    on_complete:
      notify:
        title: "Critical Failure Detected"
        message: "Task ${task.id} failed critically"
        # Only notify on specific conditions
        condition:
          and:
            - type: task_status
              task: "quality_check"
              status: failed
            - type: state_equals
              key: "severity"
              value: "critical"
        priority: critical
        channels:
          - type: ntfy
            topic: "alerts"
            priority: "urgent"
          - type: sms
            to: ["${secret.oncall_phone}"]
            provider:
              type: twilio
              account_sid: "${secret.twilio_sid}"
              auth_token: "${secret.twilio_token}"
              from: "${secret.twilio_from}"
```

### Example 9: MCP Tool for Custom Integrations

```yaml
mcp_servers:
  custom_notifier:
    type: stdio
    command: "python"
    args: ["-m", "custom_notification_server"]

tasks:
  custom_notify:
    agent: notifier
    on_complete:
      notify:
        message: "Custom notification via MCP"
        channels:
          - type: mcp_tool
            server: "custom_notifier"
            tool: "send_notification"
            parameters:
              channel: "custom_channel"
              priority: "high"
              metadata:
                task_id: "${task.id}"
                workflow: "${workflow.name}"
            timeout_secs: 60
```

### Example 10: Fallback Channels with Priority

```yaml
secrets:
  primary_webhook:
    source:
      type: env
      var: PRIMARY_WEBHOOK
  slack_webhook:
    source:
      type: env
      var: SLACK_WEBHOOK
  admin_email:
    source:
      type: env
      var: ADMIN_EMAIL

tasks:
  important_task:
    agent: worker
    on_complete:
      notify:
        title: "Important Task Complete"
        message: "Critical workflow step completed"
        priority: high
        # Multiple channels act as fallbacks
        # If webhook fails, Slack is tried, then email, then console
        channels:
          - type: webhook
            url: "${secret.primary_webhook}"
            method: POST
            timeout_secs: 10

          - type: slack
            webhook_url: "${secret.slack_webhook}"

          - type: email
            to: ["${secret.admin_email}"]
            subject: "Important Task Complete"
            smtp_config: "smtp_main"

          - type: console
            colored: true
        retry:
          max_attempts: 2
          delay_secs: 3
          exponential_backoff: false
        fail_on_error: false  # Don't fail task if notification fails
```

## Migration Path

### Phase 1: Backward Compatibility

Existing workflows continue to work:

```yaml
# Old format (still supported)
tasks:
  build:
    on_complete:
      notify: "Build complete"
```

Renders as console output with backward-compatible behavior.

### Phase 2: Opt-in to New Features

Users can gradually adopt new features:

```yaml
# Start using channels
tasks:
  build:
    on_complete:
      notify:
        message: "Build complete"
        channels:
          - type: console
          - type: slack
            webhook_url: "${secret.slack_webhook}"
```

### Phase 3: Advanced Features

Full feature adoption:

```yaml
# Use all advanced features
tasks:
  build:
    on_complete:
      notify:
        title: "Build Status"
        message: "Build ${task.status}"
        priority: high
        condition:
          type: task_status
          task: "build"
          status: completed
        channels:
          - type: slack
            webhook_url: "${secret.slack_webhook}"
            format: markdown
        retry:
          max_attempts: 3
          exponential_backoff: true
```

## Implementation Notes

### Executor Integration

```rust
// In executor.rs, replace simple println! with notification dispatcher

// Old code:
if let Some(notify) = &on_complete.notify {
    println!("Notification: {}", notify);
}

// New code:
if let Some(notify_spec) = &on_complete.notify {
    // Dispatch to notification service
    notification_service
        .send(notify_spec, &notification_context)
        .await?;
}
```

### Notification Context

```rust
pub struct NotificationContext {
    pub workflow_name: String,
    pub workflow_version: String,
    pub task_id: String,
    pub task_description: String,
    pub task_status: String,
    pub task_duration_secs: Option<u64>,
    pub error_message: Option<String>,
    pub variables: VariableContext,
    pub state: Arc<Mutex<Option<WorkflowState>>>,
}
```

### Error Handling

- Notification failures logged but don't fail tasks (unless `fail_on_error: true`)
- Retry logic applied per channel
- Fallback to next channel if primary fails
- All notification errors captured in workflow state

### Testing

- Unit tests for schema parsing
- Integration tests with mock channels
- Optional E2E tests with real services (credentials required)
- Test notification validation and variable interpolation

## Future Enhancements

1. **Notification Templates Library**
   - Predefined templates for common notifications
   - Template inheritance and composition

2. **Notification Batching**
   - Batch multiple notifications into digests
   - Rate limiting per channel

3. **Two-Way Notifications**
   - Response handling (approve/reject workflows)
   - Interactive buttons with callbacks

4. **Notification History**
   - Track all sent notifications in state
   - Query notification delivery status
   - Audit trail

5. **Advanced Templating**
   - Handlebars/Jinja2 support
   - Template functions (formatDate, truncate, etc.)
   - Conditional rendering within templates

6. **Channel Groups**
   - Define channel groups for easier management
   - Route by priority or type
