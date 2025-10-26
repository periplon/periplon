# Notification System - Quick Start Guide

## 5-Minute Quick Start

### Basic Console Notification

```rust
use claude_agent_sdk::dsl::{NotificationManager, NotificationContext, NotificationSpec};

#[tokio::main]
async fn main() {
    let manager = NotificationManager::new();
    let context = NotificationContext::new();

    let spec = NotificationSpec::Simple("Hello, World!".to_string());
    manager.send(&spec, &context).await.unwrap();
}
```

### With Variables

```rust
let context = NotificationContext::new()
    .with_workflow_var("project", "my-app")
    .with_task_var("status", "success");

let spec = NotificationSpec::Simple(
    "Project ${workflow.project} completed with ${task.status}".to_string()
);

manager.send(&spec, &context).await.unwrap();
// Output: "Project my-app completed with success"
```

### Multiple Channels

```rust
use claude_agent_sdk::dsl::NotificationChannel;

let spec = NotificationSpec::Structured {
    message: "Deployment complete!".to_string(),
    channels: vec![
        NotificationChannel::Console { colored: true, timestamp: true },
        NotificationChannel::Ntfy {
            server: "https://ntfy.sh".to_string(),
            topic: "deployments".to_string(),
            title: Some("CI/CD".to_string()),
            priority: Some(4),
            tags: vec!["rocket".to_string()],
            click_url: None,
            attach_url: None,
            markdown: false,
            auth_token: None,
        },
    ],
    title: Some("Deployment".to_string()),
    priority: None,
    metadata: std::collections::HashMap::new(),
};

manager.send(&spec, &context).await.unwrap();
```

## Available Channels

| Channel | Status | Use Case |
|---------|--------|----------|
| Console | ✅ Ready | Terminal output |
| Ntfy | ✅ Ready | Mobile push notifications |
| Slack | ✅ Ready | Team chat (webhook) |
| Discord | ✅ Ready | Community chat (webhook) |
| File | ✅ Ready | Log files (text/JSON) |
| Email | 🚧 TODO | Email notifications |
| SMS | 🚧 TODO | Text messages |
| ElevenLabs | 🚧 TODO | Voice notifications |

## Variable Scopes

| Syntax | Scope | Example |
|--------|-------|---------|
| `${workflow.var}` | Workflow-level | Project name, version |
| `${task.var}` | Task-level | Task name, status |
| `${agent.var}` | Agent-level | Agent capabilities |
| `${secret.name}` | Secrets | API keys, tokens |
| `${metadata.key}` | Metadata | Duration, timestamps |

## Common Patterns

### Success Notification
```rust
let context = NotificationContext::new()
    .with_metadata("task", "build")
    .with_metadata("duration", "45s");

let spec = NotificationSpec::Simple(
    "✅ ${metadata.task} completed in ${metadata.duration}".to_string()
);
```

### Error Notification
```rust
let context = NotificationContext::new()
    .with_metadata("error", "Connection timeout")
    .with_workflow_var("env", "production");

let spec = NotificationSpec::Structured {
    message: "❌ Error in ${workflow.env}: ${metadata.error}".to_string(),
    channels: vec![/* ... */],
    priority: Some(NotificationPriority::Critical),
    // ...
};
```

### With Secrets
```rust
let context = NotificationContext::new()
    .with_secret("ntfy_token", std::env::var("NTFY_TOKEN").unwrap());

NotificationChannel::Ntfy {
    auth_token: Some("${secret.ntfy_token}".to_string()),
    // ...
}
```

## DSL Workflow Integration

```yaml
tasks:
  deploy:
    description: "Deploy to production"
    agent: "deployer"
    on_complete:
      notify:
        message: "Deployed ${workflow.version} to production"
        channels:
          - type: console
            colored: true
          - type: ntfy
            topic: "deployments"
            priority: 4
```

## Testing

```bash
# Run all notification tests
cargo test --test notification_tests

# Run specific test
cargo test test_variable_interpolation

# With output
cargo test -- --nocapture
```

## Common Issues

### Unresolved Variables
**Error:** `InterpolationError: Unresolved variables in template`

**Fix:** Ensure all referenced variables are in context:
```rust
// Bad
context.interpolate("${workflow.missing}")  // Error!

// Good
let context = context.with_workflow_var("missing", "value");
context.interpolate("${workflow.missing}")  // OK
```

### HTTP Errors
**Error:** `HttpError: Connection refused`

**Fix:** Check network connectivity and URLs:
```rust
// Verify webhook URL is correct
// Check firewall/network settings
// Ensure service is reachable
```

### File Permissions
**Error:** `IoError: Permission denied`

**Fix:** Ensure write permissions for file path:
```bash
chmod 644 /path/to/notification.log
# Or use a writable directory
```

## Need Help?

- 📖 Full docs: `docs/notifications_delivery.md`
- 🧪 Examples: `tests/notification_tests.rs`
- 💻 Source: `src/dsl/notifications.rs`
- 🏗️ Implementation: `NOTIFICATION_DELIVERY_IMPLEMENTATION.md`

## What's Next?

1. Implement email/SMS senders (see TODOs)
2. Add concurrent delivery
3. Add rate limiting
4. Add delivery metrics
5. Create more examples

---

**That's it! Start sending notifications in minutes! 🚀**
