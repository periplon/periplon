# Notification System Analysis

## Current Implementation

### ActionSpec.notify (src/dsl/schema.rs:498-504)

The current notification system is minimal:

```rust
/// Action specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionSpec {
    /// Notification message
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notify: Option<String>,
}
```

**Limitations:**
- Simple string-based notification only
- No channel specification (email, SMS, Slack, etc.)
- No configuration options
- No integration with external notification services
- No conditional notifications
- No templating or variable interpolation

### Notification Handling (src/dsl/executor.rs:924-929)

Current implementation simply prints to stdout:

```rust
// Handle on_complete actions
if let Some(on_complete) = &spec.on_complete {
    if let Some(notify) = &on_complete.notify {
        println!("Notification: {}", notify);
    }
}
```

**Issues:**
- Notifications only printed to console
- No integration with notification services
- No error handling for notification failures
- No retry logic
- Only triggered on task completion (not on errors, starts, etc.)

### Usage in Codebase

**Template generation** (src/dsl/template.rs:765, 1612):
- Includes example `notify: "Success message"` in generated templates
- No advanced examples or documentation

**Parser tests** (src/dsl/parser.rs:536-568):
- Basic serialization/deserialization tests
- No validation of notification configuration

**Predefined tasks** (src/dsl/predefined_tasks/discovery.rs:407):
- Reference to "slack-notify" task template
- Not actually implemented

## Available MCP Servers

### Ntfy Integration (MCP Tool Available)

The system has access to `mcp__testjmca__notify_ntfy` tool with capabilities:

```typescript
{
  "topic": string,           // Required: ntfy topic
  "message": string,         // Notification message
  "title": string,           // Optional title
  "priority": string,        // Priority (1-5 or keywords)
  "tags": string[],          // Tags or emoji shortcodes
  "click": string,           // Click action URL
  "attach": string,          // Attachment URL
  "actions": string,         // Action buttons
  "email": string,           // Forward to email
  "call": string,            // Phone number for voice call
  "delay": string,           // Schedule delivery
  "markdown": boolean,       // Enable Markdown
  "icon": string,            // Icon URL
  "server": string,          // Override ntfy server
  "headers": object,         // Custom headers
  "method": string,          // HTTP method
  "timeout_ms": number       // Request timeout
}
```

**Integration Points:**
- Can be invoked via `McpToolSpec` in task execution
- Supports rich notification features
- Server can be overridden for self-hosted instances

### Other MCP Tools Available

1. **mcp__testjmca__geocode** - Location services (not directly notification-related)
2. **mcp__testjmca__openstreetmap_snapshot** - Map generation (could be used for location-based notifications)
3. **mcp__testjmca__generate_image** - AI image generation (could be used for notification attachments)
4. **mcp__testjmca__search_books** - Book search (not notification-related)

## Error Handling Patterns

### Current Error Handling (src/dsl/executor.rs)

**Error propagation:**
```rust
use crate::error::{Error, Result};
```

**Error types used:**
- `Error::InvalidInput` - Primary error type for validation and execution errors
- Errors are recorded in workflow state: `workflow_state.record_task_error(&task_id, &e.to_string())`
- Error hooks can be triggered on failures

**Retry mechanism:**
```rust
let mut error_attempt = 0;
// ... retry logic
if !ErrorRecovery::should_retry(&recovery_strategy, error_attempt) {
    // Handle final failure
}
```

**Error recovery strategies** (src/dsl/hooks.rs:148-224):
- `Retry` - Retry with exponential backoff
- `Skip` - Skip task and continue
- `Fallback` - Use fallback agent
- `Abort` - Abort entire workflow

### Logging Patterns

**Console output via println!/eprintln!:**
- 90+ `println!` statements throughout executor.rs
- Used for progress tracking, status updates, and debugging
- No structured logging framework (no `log::` or `tracing::`)
- Warning messages use `eprintln!`: `eprintln!("Warning: Failed to checkpoint state...")`

**Output patterns:**
```rust
println!("Task completed: {}", task_id);           // Success messages
println!("Task '{}' failed: {}", task_id, e);      // Error messages
println!("Notification: {}", notify);              // Current notification output
eprintln!("Warning: {}", message);                 // Warnings
```

## Integration with Secrets Management

### Current Secrets System (src/dsl/schema.rs:1007-1036)

```rust
pub struct SecretSpec {
    pub source: SecretSource,
    pub description: Option<String>,
}

pub enum SecretSource {
    Env { var: String },
    File { path: String },
    Value { value: String },  // Not recommended for production
}
```

**Usage:**
- Secrets defined at workflow level: `secrets: HashMap<String, SecretSpec>`
- Can be referenced in variables: `${secret.name}`
- Used in HTTP auth, MCP tool parameters, etc.

**Limitations:**
- No encrypted secret storage
- File-based secrets not validated
- No secret rotation support
- No integration with external secret managers (Vault, AWS Secrets Manager, etc.)

## Variable Interpolation System

### Variable Context (src/dsl/variables.rs)

**Scoped variables:**
- `workflow.variable` - Workflow-level variables
- `agent.variable` - Agent-level variables
- `task.variable` - Task-level variables
- `state.key` - Workflow state variables
- `secret.name` - Secret references

**Interpolation syntax:**
- `${scope.variable}` - Explicit scope
- `${variable}` - Implicit scope resolution

**Supported in:**
- Task descriptions
- Input/output specifications
- HTTP URLs, headers, body
- Command arguments
- Script content
- MCP tool parameters

## Recommendations for Notification Enhancement

### High Priority

1. **Expand ActionSpec to NotificationSpec**
   - Support multiple notification channels
   - Add configuration options per channel
   - Enable conditional notifications
   - Support templating and variable interpolation

2. **Implement Notification Executor**
   - Separate notification logic from task execution
   - Add retry logic with exponential backoff
   - Support async notification delivery
   - Handle notification failures gracefully

3. **Integrate with MCP Ntfy Tool**
   - Use existing `mcp__testjmca__notify_ntfy` for immediate functionality
   - Support custom ntfy server configuration
   - Enable rich notification features (priority, tags, actions)

4. **Add Notification Hooks**
   - `on_start` - Notify when task/workflow starts
   - `on_complete` - Notify on successful completion
   - `on_error` - Notify on failures
   - `on_retry` - Notify on retry attempts

### Medium Priority

1. **Multi-Channel Support**
   - Email (via SMTP or service APIs)
   - SMS (via Twilio, AWS SNS)
   - Slack (via webhooks or API)
   - Discord (via webhooks)
   - Custom webhooks

2. **Notification Templates**
   - Jinja2-like templating
   - Access to task context, state, variables
   - Conditional content rendering
   - Support for HTML and Markdown

3. **Notification Batching**
   - Batch multiple notifications
   - Digest mode for frequent updates
   - Rate limiting

### Low Priority

1. **Notification History**
   - Track sent notifications in state
   - Notification delivery status
   - Audit trail

2. **Advanced Features**
   - Two-way notifications (response handling)
   - Voice calls (via Elevenlabs, Twilio)
   - Push notifications (mobile apps)

## Architecture Recommendations

### Hexagonal Architecture Compliance

**New components should follow existing patterns:**

**Domain** (src/dsl/notification/):
- `notification.rs` - Core notification types and logic
- Pure Rust, no external dependencies

**Ports** (src/ports/secondary/):
- `notification_service.rs` - Trait defining notification operations

**Adapters** (src/adapters/secondary/):
- `ntfy_notification.rs` - Ntfy implementation
- `email_notification.rs` - Email implementation
- `slack_notification.rs` - Slack implementation
- `composite_notification.rs` - Multi-channel orchestration

**Integration points:**
- Executor calls notification service after task completion/error
- Notification service dispatches to appropriate adapters
- Adapters use MCP tools, HTTP clients, or SDK clients

### Error Handling Strategy

1. **Non-blocking notifications**
   - Notification failures should not fail tasks
   - Log notification errors but continue execution
   - Optional strict mode for critical notifications

2. **Retry logic**
   - Use existing `ErrorRecovery` patterns
   - Exponential backoff for transient failures
   - Circuit breaker for persistent failures

3. **Fallback channels**
   - Primary channel fails → try secondary
   - Email fails → try Slack → try console

### Testing Strategy

1. **Unit tests**
   - Notification schema parsing
   - Variable interpolation
   - Condition evaluation

2. **Integration tests**
   - Mock notification services
   - Test all channel types
   - Test failure scenarios

3. **End-to-end tests**
   - Real notification delivery (if configured)
   - Optional, skipped in CI if credentials not available
