# Notification System Guide

Comprehensive guide to the DSL notification system for workflow alerts and monitoring.

## Table of Contents

- [Overview](#overview)
- [Type System](#type-system)
- [Quick Start](#quick-start)
- [Notification Channels](#notification-channels)
  - [Console](#console)
  - [File](#file)
  - [Ntfy.sh](#ntfysh)
  - [Slack](#slack)
  - [Discord](#discord)
  - [Telegram](#telegram)
  - [Email](#email)
  - [Webhook](#webhook)
- [Configuration](#configuration)
- [Secret Management](#secret-management)
- [Variable Interpolation](#variable-interpolation)
- [Error Handling & Retry](#error-handling--retry)
- [MCP Integration](#mcp-integration)
- [Security Best Practices](#security-best-practices)
- [Troubleshooting](#troubleshooting)
- [Examples](#examples)

## Overview

The notification system enables workflows to send real-time alerts through multiple channels. Key features include:

- **Multiple channels**: Console, file, Ntfy.sh, Slack, Discord, Telegram, email, webhooks
- **Variable interpolation**: Dynamic message content from workflow context
- **Secret management**: Secure credential handling
- **Retry logic**: Configurable retry with exponential backoff
- **Priority levels**: Low, normal, high, critical
- **MCP integration**: Native support for MCP notification servers
- **Multi-channel broadcasting**: Send to multiple channels simultaneously

## Type System

The notification system is built on strongly-typed Rust enums for compile-time safety.

### NotificationSpec

The root notification type supports two variants:

```rust
pub enum NotificationSpec {
    /// Simple string notification (uses workflow default channels)
    Simple(String),

    /// Structured notification with full configuration
    Structured {
        message: String,
        channels: Vec<NotificationChannel>,
        title: Option<String>,
        priority: Option<NotificationPriority>,
        metadata: HashMap<String, String>,
    },
}
```

### NotificationChannel

All supported notification channels:

```rust
pub enum NotificationChannel {
    Console { colored: bool, timestamp: bool },
    File { path: String, append: bool, timestamp: bool, format: FileNotificationFormat },
    Ntfy { server: String, topic: String, title: Option<String>, priority: Option<u8>, tags: Vec<String>, click_url: Option<String>, attach_url: Option<String>, markdown: bool, auth_token: Option<String> },
    Slack { credential: String, channel: String, method: SlackMethod, attachments: Vec<SlackAttachment> },
    Discord { webhook_url: String, username: Option<String>, avatar_url: Option<String>, tts: bool, embed: Option<DiscordEmbed> },
    Telegram { bot_token: String, chat_id: String, parse_mode: Option<String>, disable_preview: bool, silent: bool },
    Email { to: Vec<String>, cc: Vec<String>, bcc: Vec<String>, subject: String, smtp: SmtpConfig },
    Webhook { url: String, method: String, headers: HashMap<String, String>, auth: Option<WebhookAuth>, body_template: String, timeout_secs: u64, retry: Option<RetryConfig> },
}
```

### NotificationPriority

Priority levels for routing and urgency:

```rust
pub enum NotificationPriority {
    Low,
    Normal,
    High,
    Critical,
}
```

## Quick Start

### Simple Notification

Use the simplest form for quick console notifications:

```yaml
tasks:
  build:
    description: "Build the project"
    agent: build_agent
    on_complete:
      notify: "Build completed successfully!"
```

### Structured Notification

For more control, use structured notifications:

```yaml
tasks:
  build:
    description: "Build the project"
    agent: build_agent
    on_complete:
      notify:
        message: "Build completed for ${workflow.project_name}"
        title: "Build Status"
        priority: high
        channels:
          - type: console
            colored: true
            timestamp: true
          - type: file
            path: "/tmp/build.log"
            append: true
            format: json
```

### Workflow-Level Defaults

Configure default notification behavior at the workflow level:

```yaml
notifications:
  notify_on_start: true
  notify_on_completion: true
  notify_on_failure: true
  default_channels:
    - type: console
      colored: true
      timestamp: true
    - type: ntfy
      server: "https://ntfy.sh"
      topic: "my-workflow"
      priority: 3
```

## Notification Channels

### Console

Send notifications to stdout with optional colors and timestamps.

```yaml
channels:
  - type: console
    colored: true      # ANSI color codes (default: true)
    timestamp: true    # Prepend timestamp (default: false)
```

**Use cases:**
- Development and debugging
- CI/CD pipeline logs
- Terminal-based workflows

**Features:**
- ANSI color support (green checkmark)
- ISO 8601 timestamps
- No retry needed

### File

Write notifications to files in various formats.

```yaml
channels:
  - type: file
    path: "/var/log/workflow.log"
    append: true          # Append to file (default: false)
    timestamp: true       # Add timestamp to each entry
    format: text          # text, json, or jsonlines
```

**Formats:**
- `text`: Plain text with optional timestamps
- `json`: Pretty-printed JSON with metadata
- `jsonlines`: Single-line JSON (JSONL format)

**Use cases:**
- Audit trails
- Log aggregation
- Persistent notification history

**Example JSON output:**
```json
{
  "message": "Task completed",
  "timestamp": "2025-01-20T12:00:00Z",
  "metadata": {
    "task": "build",
    "status": "success"
  }
}
```

### Ntfy.sh

Push notifications to mobile devices via ntfy.sh.

```yaml
channels:
  - type: ntfy
    server: "https://ntfy.sh"              # Server URL
    topic: "my-workflow-alerts"            # Topic name
    title: "Workflow Alert"                # Optional title
    priority: 4                            # 1-5 (1=min, 5=max)
    tags: ["warning", "computer"]          # Emoji tags
    click_url: "https://dashboard.com"     # Click action URL
    attach_url: "https://example.com/log"  # Attachment URL
    markdown: true                         # Enable markdown
    auth_token: "${secret.ntfy_token}"    # Optional authentication
```

**Priority levels:**
- `1`: Min priority
- `2`: Low priority
- `3`: Default priority
- `4`: High priority
- `5`: Max priority (urgent)

**Tags:**
Use emoji shortcodes: `white_check_mark`, `warning`, `fire`, `rocket`, etc.

**MCP integration:**
When using the ntfy MCP server, notifications can also be sent via MCP tools (see [MCP Integration](#mcp-integration)).

**Use cases:**
- Critical alerts
- Mobile push notifications
- Real-time monitoring

### Slack

Send messages to Slack channels via webhooks or bot API.

```yaml
channels:
  - type: slack
    credential: "${secret.slack_webhook}"  # Webhook URL or bot token
    channel: "#notifications"              # Channel name
    method: webhook                        # webhook or bot
    attachments:
      - text: "Build completed successfully"
        color: "good"                      # good, warning, danger, or hex
        fields:
          - title: "Project"
            value: "${workflow.project_name}"
            short: true
          - title: "Status"
            value: "Success"
            short: true
```

**Methods:**
- `webhook`: Incoming webhooks (simpler, no bot required)
- `bot`: Bot API (more features, requires bot token)

**Colors:**
- `good`: Green
- `warning`: Yellow
- `danger`: Red
- Hex color: `#36a64f`

**Use cases:**
- Team notifications
- Build status updates
- Deployment alerts

### Discord

Send messages to Discord channels via webhooks.

```yaml
channels:
  - type: discord
    webhook_url: "${secret.discord_webhook}"
    username: "Workflow Bot"               # Optional bot name
    avatar_url: "https://..."              # Optional avatar
    tts: false                             # Text-to-speech
    embed:
      title: "Workflow Notification"
      description: "Build completed successfully"
      color: 5763719                       # Decimal color
      fields:
        - name: "Status"
          value: "Success"
          inline: true
        - name: "Duration"
          value: "5m 30s"
          inline: true
      footer: "Workflow Engine"
      timestamp: "2025-01-20T12:00:00Z"
```

**Color conversion:**
Convert hex to decimal: `0x57F287` → `5763719`

**Use cases:**
- Community notifications
- Gaming community alerts
- Developer team updates

### Telegram

Send messages via Telegram bot API.

```yaml
channels:
  - type: telegram
    bot_token: "${secret.telegram_token}"
    chat_id: "@channel_name"              # Or numeric ID
    parse_mode: Markdown                  # Markdown, HTML, or none
    disable_preview: false                # Disable link previews
    silent: false                         # Silent notification
```

**Parse modes:**
- `Markdown`: Telegram markdown syntax
- `HTML`: HTML formatting
- None: Plain text

**Use cases:**
- Personal notifications
- Small team alerts
- Bot-based automation

### Email

Send email notifications via SMTP.

```yaml
channels:
  - type: email
    to: ["admin@example.com", "team@example.com"]
    cc: ["manager@example.com"]
    bcc: ["audit@example.com"]
    subject: "Workflow Alert: ${workflow.project_name}"
    smtp:
      host: "smtp.gmail.com"
      port: 587
      username: "notifications@example.com"
      password: "${secret.smtp_password}"
      from: "notifications@example.com"
      use_tls: true
```

**Common SMTP providers:**
- Gmail: `smtp.gmail.com:587` (TLS)
- Outlook: `smtp.office365.com:587` (TLS)
- SendGrid: `smtp.sendgrid.net:587` (TLS)
- Amazon SES: `email-smtp.us-east-1.amazonaws.com:587` (TLS)

**Use cases:**
- Formal notifications
- Audit reports
- Executive summaries

### Webhook

Send notifications to custom HTTP endpoints.

```yaml
channels:
  - type: webhook
    url: "https://api.example.com/notifications"
    method: POST                           # GET, POST, PUT, etc.
    headers:
      Content-Type: "application/json"
      X-API-Key: "${secret.api_key}"
    auth:
      type: bearer                         # bearer, basic, or none
      token: "${secret.webhook_token}"
    body_template: |
      {
        "project": "${workflow.project_name}",
        "message": "{{message}}",
        "timestamp": "{{timestamp}}",
        "priority": "{{priority}}"
      }
    timeout_secs: 30
    retry:
      max_attempts: 3
      delay_secs: 5
      exponential_backoff: true
```

**Template variables:**
- `{{message}}`: Notification message
- `{{timestamp}}`: ISO 8601 timestamp
- `{{priority}}`: Priority level

**Use cases:**
- Custom integrations
- Internal APIs
- Third-party services

## Configuration

### Workflow-Level Defaults

Define default notification behavior for all tasks:

```yaml
notifications:
  notify_on_start: true              # Notify when workflow starts
  notify_on_completion: true         # Notify when workflow completes
  notify_on_failure: true            # Notify on task/workflow failure
  notify_on_workflow_completion: true # Notify when entire workflow finishes
  default_channels:
    - type: console
      colored: true
      timestamp: true
```

### Task-Level Notifications

Override defaults for specific tasks:

```yaml
tasks:
  critical_task:
    description: "Critical deployment task"
    agent: deploy_agent
    on_complete:
      notify:
        message: "Deployment completed"
        priority: critical
        channels:
          - type: ntfy
            priority: 5
          - type: slack
            channel: "#alerts"
          - type: email
            to: ["oncall@example.com"]
    on_error:
      notify:
        message: "CRITICAL: Deployment failed!"
        priority: critical
```

### Priority Levels

```yaml
priority: low       # Non-urgent information
priority: normal    # Standard notifications (default)
priority: high      # Important updates
priority: critical  # Urgent alerts requiring immediate attention
```

## Secret Management

Store sensitive credentials securely using the `secrets` system.

### Environment Variables

```yaml
secrets:
  slack_webhook:
    source:
      type: env
      var: SLACK_WEBHOOK_URL
    description: "Slack webhook URL"

  ntfy_token:
    source:
      type: env
      var: NTFY_AUTH_TOKEN
    description: "Ntfy.sh authentication token"
```

### File-Based Secrets

```yaml
secrets:
  api_key:
    source:
      type: file
      path: "/etc/secrets/api-key.txt"
    description: "API authentication key"
```

### Using Secrets

Reference secrets in notifications using variable interpolation:

```yaml
channels:
  - type: slack
    credential: "${secret.slack_webhook}"

  - type: ntfy
    auth_token: "${secret.ntfy_token}"

  - type: email
    smtp:
      password: "${secret.smtp_password}"
```

**Security:**
- Secrets are never logged or displayed
- Only available during workflow execution
- Not included in state checkpoints

## Variable Interpolation

Inject dynamic values into notification messages using variable syntax.

### Syntax

```yaml
message: "Build completed for ${workflow.project_name} in ${workflow.environment}"
```

### Variable Scopes

**Workflow variables:**
```yaml
${workflow.variable_name}
```

**Task variables:**
```yaml
${task.variable_name}
```

**Agent variables:**
```yaml
${agent.variable_name}
```

**Secrets:**
```yaml
${secret.secret_name}
```

**Metadata:**
```yaml
${metadata.key}
```

### Example

```yaml
inputs:
  project_name:
    type: string
    default: "MyProject"

tasks:
  build:
    on_complete:
      notify:
        message: |
          **Build Complete**

          Project: ${workflow.project_name}
          Task: ${task.name}
          Agent: ${agent.role}
          Status: ${metadata.status}
          Duration: ${metadata.duration}
```

### Escaping

To include literal `${...}` in messages, the system will error on unresolved variables. Ensure all referenced variables exist.

## Error Handling & Retry

Configure retry logic for transient failures.

### Retry Configuration

```yaml
channels:
  - type: webhook
    url: "https://api.example.com/notify"
    retry:
      max_attempts: 3              # Total attempts (default: 1)
      delay_secs: 5                # Delay between retries
      exponential_backoff: true    # Double delay each retry
```

**Exponential backoff:**
- Attempt 1: 0s delay
- Attempt 2: 5s delay
- Attempt 3: 10s delay
- Attempt 4: 20s delay

### Task-Level Retry

```yaml
tasks:
  deploy:
    on_error:
      retry: 3
      retry_delay_secs: 10
      exponential_backoff: true
```

### Fallback Channels

Use multiple channels with automatic fallback:

```yaml
channels:
  # Primary channel (tried first)
  - type: slack
    credential: "${secret.slack_webhook}"

  # Fallback to file if Slack fails
  - type: file
    path: "/tmp/notifications.log"
    append: true
```

## MCP Integration

Integrate with Model Context Protocol (MCP) notification servers.

### MCP Server Configuration

```yaml
mcp_servers:
  ntfy:
    type: stdio
    command: "npx"
    args: ["-y", "@jmca/ntfy-mcp-server"]
    env:
      NTFY_DEFAULT_SERVER: "https://ntfy.sh"
      NTFY_DEFAULT_TOPIC: "my-workflow"
```

### Using MCP Tools in Tasks

Send notifications via MCP tools directly:

```yaml
tasks:
  notify_via_mcp:
    description: "Send notification via MCP tool"
    mcp_tool:
      server: "ntfy"
      tool: "notify_ntfy"
      parameters:
        topic: "alerts"
        message: "MCP notification test"
        title: "Test"
        priority: "4"
        tags: ["test", "mcp"]
```

### Native vs MCP Channels

**Native channels** (recommended):
- Defined in workflow YAML
- Automatic retry handling
- Variable interpolation
- Integrated error handling

**MCP tools** (advanced):
- Direct tool invocation
- Full MCP capabilities
- Requires MCP server setup

Both approaches can coexist in the same workflow.

## Security Best Practices

### 1. Never Hardcode Credentials

❌ **Bad:**
```yaml
channels:
  - type: slack
    credential: "https://hooks.slack.com/services/T00/B00/XXX"
```

✅ **Good:**
```yaml
secrets:
  slack_webhook:
    source:
      type: env
      var: SLACK_WEBHOOK_URL

channels:
  - type: slack
    credential: "${secret.slack_webhook}"
```

### 2. Use Environment Variables

```bash
export SLACK_WEBHOOK_URL="https://hooks.slack.com/services/..."
export NTFY_AUTH_TOKEN="tk_..."
export SMTP_PASSWORD="..."
```

### 3. Restrict File Permissions

```bash
chmod 600 /etc/secrets/api-key.txt
```

### 4. Use TLS/SSL

Always use encrypted connections:
- HTTPS for webhooks
- TLS for SMTP (port 587 or 465)
- WSS for WebSocket connections

### 5. Rotate Credentials

Regularly rotate tokens and passwords, especially:
- API keys
- Webhook URLs
- Bot tokens
- SMTP passwords

### 6. Audit Logging

Enable file-based notifications for audit trails:

```yaml
notifications:
  default_channels:
    - type: file
      path: "/var/log/workflow-audit.log"
      append: true
      timestamp: true
      format: jsonlines
```

### 7. Validate Webhook URLs

Ensure webhook URLs are from trusted domains:
- Verify HTTPS
- Check certificate validity
- Use authentication headers

## Troubleshooting

### Common Issues

#### 1. Variable Not Resolved

**Error:**
```
Variable interpolation failed: Unresolved variables in template: ${workflow.missing}
```

**Solution:**
Ensure the variable is defined in workflow inputs:

```yaml
inputs:
  missing:
    type: string
    default: "value"
```

#### 2. Ntfy Notification Not Received

**Checklist:**
- Verify topic name matches
- Check server URL (default: `https://ntfy.sh`)
- Ensure priority is 1-5
- Verify auth token if using authentication
- Test with ntfy.sh web interface

**Debug:**
```yaml
channels:
  - type: console
    colored: true
  - type: ntfy
    server: "https://ntfy.sh"
    topic: "test-topic"
```

#### 3. Slack Webhook Fails

**Common causes:**
- Invalid webhook URL
- Webhook revoked/expired
- Malformed payload
- Network connectivity

**Solution:**
Test webhook manually:
```bash
curl -X POST "${SLACK_WEBHOOK_URL}" \
  -H 'Content-Type: application/json' \
  -d '{"text":"Test message"}'
```

#### 4. Email Not Sent

**Checklist:**
- Verify SMTP credentials
- Check firewall rules (port 587/465)
- Enable "Less secure apps" (Gmail)
- Use app-specific passwords (Gmail, Outlook)
- Verify TLS settings

**Debug SMTP:**
```bash
telnet smtp.gmail.com 587
```

#### 5. Retry Exhausted

**Error:**
```
All retry attempts exhausted for slack: HTTP 500
```

**Solutions:**
- Increase max_attempts
- Add exponential backoff
- Add fallback channels
- Check service status

### Debug Mode

Enable verbose logging:

```yaml
notifications:
  default_channels:
    - type: console
      colored: true
      timestamp: true
    - type: file
      path: "/tmp/notification-debug.log"
      append: true
      format: jsonlines
```

### Testing Notifications

Create a test workflow:

```yaml
name: "Notification Test"
version: "1.0.0"

agents:
  test_agent:
    description: "Test agent"
    tools: [Read]

tasks:
  test_notification:
    description: "Test all notification channels"
    agent: test_agent
    on_complete:
      notify:
        message: "Test notification from workflow"
        channels:
          - type: console
          - type: file
            path: "/tmp/test.log"
```

## Examples

### Multi-Channel Build Notification

```yaml
tasks:
  build:
    description: "Build project"
    agent: build_agent
    on_complete:
      notify:
        message: |
          **Build Successful**

          Project: ${workflow.project_name}
          Branch: ${workflow.git_branch}
          Commit: ${workflow.git_commit}
          Duration: ${metadata.duration}
        title: "Build Complete"
        priority: normal
        channels:
          - type: console
            colored: true
          - type: ntfy
            server: "https://ntfy.sh"
            topic: "builds"
            priority: 3
            tags: ["white_check_mark", "package"]
          - type: slack
            credential: "${secret.slack_webhook}"
            channel: "#builds"
            attachments:
              - text: "Build completed successfully"
                color: "good"
          - type: file
            path: "/var/log/builds.log"
            append: true
            format: jsonlines
```

### Critical Alert

```yaml
tasks:
  health_check:
    description: "Check system health"
    agent: monitor_agent
    on_error:
      notify:
        message: |
          🚨 CRITICAL ALERT 🚨

          Health check failed for ${workflow.service_name}
          Environment: ${workflow.environment}

          Immediate action required!
        title: "CRITICAL: Health Check Failed"
        priority: critical
        channels:
          - type: ntfy
            priority: 5
            tags: ["rotating_light", "fire", "sos"]
          - type: slack
            channel: "#alerts"
            attachments:
              - color: "danger"
          - type: email
            to: ["oncall@example.com"]
            subject: "CRITICAL: ${workflow.service_name} health check failed"
```

### Conditional Notification

```yaml
tasks:
  deploy:
    description: "Deploy to production"
    agent: deploy_agent
    condition:
      type: task_status
      task: tests
      status: completed
    on_complete:
      notify:
        message: "Deployment to ${workflow.environment} completed"
        priority: high
```

### Summary Report

```yaml
tasks:
  send_summary:
    description: "Send workflow summary"
    agent: report_agent
    depends_on:
      - build
      - test
      - deploy
    on_complete:
      notify:
        message: |
          **Workflow Summary**

          Project: ${workflow.project_name}
          Environment: ${workflow.environment}

          Tasks Completed:
          ✅ Build
          ✅ Tests
          ✅ Deployment

          Duration: ${metadata.total_duration}
          Cost: ${metadata.total_cost}
        channels:
          - type: email
            to: ["team@example.com"]
            subject: "Workflow Summary: ${workflow.project_name}"
          - type: file
            path: "/reports/workflow-summary.json"
            format: json
```

---

For more examples, see:
- `examples/notification_ntfy_demo.yaml` - Ntfy.sh integration examples
- `examples/notification_multi_channel.yaml` - Multi-channel configuration
- `examples/notification_error_scenarios.yaml` - Error handling patterns

For MCP integration details, see the [MCP Integration Guide](mcp_integration.md).
