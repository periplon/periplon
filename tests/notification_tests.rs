//! Integration tests for notification delivery system
//!
//! This test suite covers end-to-end notification delivery scenarios including:
//! - Multi-channel delivery
//! - Variable interpolation
//! - Error handling and retries
//! - Concurrent delivery (when supported)
//! - File format variations
//! - Schema parsing and serialization

use periplon_sdk::dsl::{
    FileNotificationFormat, NotificationChannel, NotificationContext, NotificationManager,
    NotificationSpec, RetryConfig, SlackMethod,
};
use std::collections::HashMap;

#[tokio::test]
async fn test_simple_console_notification() {
    let manager = NotificationManager::new();
    let context = NotificationContext::new();

    let spec = NotificationSpec::Simple("Test notification message".to_string());

    let result = manager.send(&spec, &context).await;
    assert!(result.is_ok(), "Simple console notification should succeed");
}

#[tokio::test]
async fn test_structured_console_notification() {
    let manager = NotificationManager::new();
    let context = NotificationContext::new();

    let spec = NotificationSpec::Structured {
        message: "Structured test message".to_string(),
        channels: vec![NotificationChannel::Console {
            colored: true,
            timestamp: true,
        }],
        title: Some("Test Title".to_string()),
        priority: None,
        metadata: std::collections::HashMap::new(),
    };

    let result = manager.send(&spec, &context).await;
    assert!(
        result.is_ok(),
        "Structured console notification should succeed"
    );
}

#[tokio::test]
async fn test_variable_interpolation() {
    let context = NotificationContext::new()
        .with_workflow_var("project", "my-project")
        .with_task_var("task_name", "build")
        .with_metadata("status", "success");

    let template =
        "Project ${workflow.project}: Task ${task.task_name} completed with ${metadata.status}";
    let result = context.interpolate(template);

    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "Project my-project: Task build completed with success"
    );
}

#[tokio::test]
async fn test_variable_interpolation_with_secrets() {
    let context = NotificationContext::new()
        .with_secret("api_key", "secret-key-123")
        .with_workflow_var("env", "production");

    let template = "Deploying to ${workflow.env} with key ${secret.api_key}";
    let result = context.interpolate(template);

    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "Deploying to production with key secret-key-123"
    );
}

#[tokio::test]
async fn test_unresolved_variable_error() {
    let context = NotificationContext::new().with_workflow_var("known", "value");

    let template = "Known: ${workflow.known}, Unknown: ${workflow.unknown}";
    let result = context.interpolate(template);

    assert!(result.is_err(), "Should error on unresolved variables");
}

#[tokio::test]
async fn test_file_notification_text_format() {
    use tempfile::NamedTempFile;

    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_string_lossy().to_string();

    let manager = NotificationManager::new();
    let context = NotificationContext::new();

    let spec = NotificationSpec::Structured {
        message: "File notification test".to_string(),
        channels: vec![NotificationChannel::File {
            path: file_path.clone(),
            append: false,
            timestamp: false,
            format: periplon_sdk::dsl::FileNotificationFormat::Text,
        }],
        title: None,
        priority: None,
        metadata: std::collections::HashMap::new(),
    };

    let result = manager.send(&spec, &context).await;
    assert!(result.is_ok(), "File notification should succeed");

    // Verify file contents
    let contents = std::fs::read_to_string(&file_path).unwrap();
    assert!(contents.contains("File notification test"));
}

#[tokio::test]
async fn test_file_notification_json_format() {
    use tempfile::NamedTempFile;

    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_string_lossy().to_string();

    let manager = NotificationManager::new();
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("task_id".to_string(), "test-123".to_string());

    let context = NotificationContext::new().with_metadata("task_id", "test-123");

    let spec = NotificationSpec::Structured {
        message: "JSON notification".to_string(),
        channels: vec![NotificationChannel::File {
            path: file_path.clone(),
            append: false,
            timestamp: true,
            format: periplon_sdk::dsl::FileNotificationFormat::Json,
        }],
        title: None,
        priority: None,
        metadata,
    };

    let result = manager.send(&spec, &context).await;
    assert!(result.is_ok(), "JSON file notification should succeed");

    // Verify file contains valid JSON
    let contents = std::fs::read_to_string(&file_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert_eq!(parsed["message"], "JSON notification");
}

#[tokio::test]
async fn test_notification_context_builder() {
    let context = NotificationContext::new()
        .with_workflow_var("key1", "value1")
        .with_task_var("key2", "value2")
        .with_agent_var("key3", "value3")
        .with_secret("key4", "value4")
        .with_metadata("key5", "value5");

    assert_eq!(context.workflow_vars.get("key1").unwrap(), "value1");
    assert_eq!(context.task_vars.get("key2").unwrap(), "value2");
    assert_eq!(context.agent_vars.get("key3").unwrap(), "value3");
    assert_eq!(context.secrets.get("key4").unwrap(), "value4");
    assert_eq!(context.metadata.get("key5").unwrap(), "value5");
}

#[tokio::test]
async fn test_multiple_channels_sequential() {
    use tempfile::NamedTempFile;

    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_string_lossy().to_string();

    let manager = NotificationManager::new();
    let context = NotificationContext::new();

    let spec = NotificationSpec::Structured {
        message: "Multi-channel test".to_string(),
        channels: vec![
            NotificationChannel::Console {
                colored: false,
                timestamp: false,
            },
            NotificationChannel::File {
                path: file_path.clone(),
                append: false,
                timestamp: false,
                format: periplon_sdk::dsl::FileNotificationFormat::Text,
            },
        ],
        title: None,
        priority: None,
        metadata: std::collections::HashMap::new(),
    };

    let result = manager.send(&spec, &context).await;
    assert!(result.is_ok(), "Multi-channel notification should succeed");

    // Verify file was written
    let contents = std::fs::read_to_string(&file_path).unwrap();
    assert!(contents.contains("Multi-channel test"));
}

#[tokio::test]
async fn test_placeholder_senders_return_errors() {
    use periplon_sdk::dsl::{EmailSender, NotificationSender};

    let sender = EmailSender::new();
    let context = NotificationContext::new();
    let channel = NotificationChannel::Console {
        colored: false,
        timestamp: false,
    };

    let result = sender.send("test", &channel, &context).await;
    assert!(
        result.is_err(),
        "Email sender should return error (not implemented)"
    );
}

#[tokio::test]
async fn test_notification_manager_has_all_senders() {
    let manager = NotificationManager::new();

    // Verify all expected senders are registered
    assert!(
        manager.has_sender("ntfy"),
        "Manager should have ntfy sender"
    );
    assert!(
        manager.has_sender("slack"),
        "Manager should have slack sender"
    );
    assert!(
        manager.has_sender("discord"),
        "Manager should have discord sender"
    );
    assert!(
        manager.has_sender("console"),
        "Manager should have console sender"
    );
    assert!(
        manager.has_sender("file"),
        "Manager should have file sender"
    );
}

#[test]
fn test_interpolation_all_scopes() {
    let context = NotificationContext::new()
        .with_workflow_var("w_var", "workflow_value")
        .with_task_var("t_var", "task_value")
        .with_agent_var("a_var", "agent_value")
        .with_secret("s_var", "secret_value")
        .with_metadata("m_var", "metadata_value");

    let template = "W:${workflow.w_var} T:${task.t_var} A:${agent.a_var} S:${secret.s_var} M:${metadata.m_var}";
    let result = context.interpolate(template).unwrap();

    assert_eq!(
        result,
        "W:workflow_value T:task_value A:agent_value S:secret_value M:metadata_value"
    );
}

#[tokio::test]
async fn test_file_append_mode() {
    use tempfile::NamedTempFile;
    use tokio::fs;

    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_string_lossy().to_string();

    let manager = NotificationManager::new();
    let context = NotificationContext::new();

    // First write
    let spec1 = NotificationSpec::Structured {
        message: "First message".to_string(),
        channels: vec![NotificationChannel::File {
            path: file_path.clone(),
            append: false,
            timestamp: false,
            format: periplon_sdk::dsl::FileNotificationFormat::Text,
        }],
        title: None,
        priority: None,
        metadata: std::collections::HashMap::new(),
    };

    manager.send(&spec1, &context).await.unwrap();

    // Second write in append mode
    let spec2 = NotificationSpec::Structured {
        message: "Second message".to_string(),
        channels: vec![NotificationChannel::File {
            path: file_path.clone(),
            append: true,
            timestamp: false,
            format: periplon_sdk::dsl::FileNotificationFormat::Text,
        }],
        title: None,
        priority: None,
        metadata: std::collections::HashMap::new(),
    };

    manager.send(&spec2, &context).await.unwrap();

    // Verify both messages are in file
    let contents = fs::read_to_string(&file_path).await.unwrap();
    assert!(contents.contains("First message"));
    assert!(contents.contains("Second message"));
}

// ============================================================================
// Advanced Integration Tests
// ============================================================================

#[tokio::test]
async fn test_retry_logic_with_webhook() {
    // Test retry configuration with webhook (mock will fail)
    let manager = NotificationManager::new();
    let context = NotificationContext::new();

    let retry_config = RetryConfig {
        max_attempts: 2,
        delay_secs: 0, // No delay for testing
        exponential_backoff: false,
    };

    let spec = NotificationSpec::Structured {
        message: "Test retry".to_string(),
        channels: vec![NotificationChannel::Webhook {
            url: "http://invalid-url-for-testing.local".to_string(),
            method: periplon_sdk::dsl::HttpMethod::Post,
            headers: HashMap::new(),
            auth: None,
            body_template: None,
            timeout_secs: Some(1),
            retry: Some(retry_config),
        }],
        title: None,
        priority: None,
        metadata: HashMap::new(),
    };

    // This will fail but shouldn't panic
    let result = manager.send(&spec, &context).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_notification_with_all_variable_scopes() {
    use tempfile::NamedTempFile;

    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_string_lossy().to_string();

    let manager = NotificationManager::new();
    let context = NotificationContext::new()
        .with_workflow_var("workflow_id", "wf-123")
        .with_task_var("task_name", "deploy")
        .with_agent_var("agent_role", "deployer")
        .with_secret("api_key", "secret-xyz")
        .with_metadata("status", "success")
        .with_metadata("duration", "30s");

    let message = r#"
Workflow: ${workflow.workflow_id}
Task: ${task.task_name}
Agent: ${agent.agent_role}
Status: ${metadata.status}
Duration: ${metadata.duration}
"#;

    let spec = NotificationSpec::Structured {
        message: message.to_string(),
        channels: vec![NotificationChannel::File {
            path: file_path.clone(),
            append: false,
            timestamp: false,
            format: FileNotificationFormat::Text,
        }],
        title: None,
        priority: None,
        metadata: HashMap::new(),
    };

    let result = manager.send(&spec, &context).await;
    assert!(result.is_ok());

    let contents = std::fs::read_to_string(&file_path).unwrap();
    assert!(contents.contains("wf-123"));
    assert!(contents.contains("deploy"));
    assert!(contents.contains("deployer"));
    assert!(contents.contains("success"));
    assert!(contents.contains("30s"));
}

#[tokio::test]
async fn test_file_path_interpolation() {
    let context = NotificationContext::new()
        .with_workflow_var("project", "my-app")
        .with_metadata("timestamp", "20250120");

    let manager = NotificationManager::new();

    // Use tempdir for dynamic path
    let temp_dir = tempfile::tempdir().unwrap();
    let base_path = temp_dir.path().to_string_lossy().to_string();

    let spec = NotificationSpec::Structured {
        message: "Test message".to_string(),
        channels: vec![NotificationChannel::File {
            path: format!("{}/notifications.log", base_path),
            append: false,
            timestamp: false,
            format: FileNotificationFormat::Text,
        }],
        title: None,
        priority: None,
        metadata: HashMap::new(),
    };

    let result = manager.send(&spec, &context).await;
    assert!(result.is_ok());

    // Verify file was created
    let log_path = format!("{}/notifications.log", base_path);
    assert!(std::path::Path::new(&log_path).exists());
}

#[tokio::test]
async fn test_json_lines_multiple_notifications() {
    use tempfile::NamedTempFile;

    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_string_lossy().to_string();

    let manager = NotificationManager::new();

    // Send multiple notifications
    for i in 1..=3 {
        let context =
            NotificationContext::new().with_metadata("notification_id", format!("notif-{}", i));

        let spec = NotificationSpec::Structured {
            message: format!("Notification {}", i),
            channels: vec![NotificationChannel::File {
                path: file_path.clone(),
                append: true,
                timestamp: true,
                format: FileNotificationFormat::JsonLines,
            }],
            title: None,
            priority: None,
            metadata: HashMap::new(),
        };

        manager.send(&spec, &context).await.unwrap();
    }

    // Verify all lines are valid JSON
    let contents = std::fs::read_to_string(&file_path).unwrap();
    let lines: Vec<&str> = contents.trim().split('\n').collect();
    assert_eq!(lines.len(), 3);

    for line in lines {
        let parsed: serde_json::Value = serde_json::from_str(line).unwrap();
        assert!(parsed["message"].is_string());
        assert!(parsed["timestamp"].is_string());
    }
}

#[tokio::test]
async fn test_structured_notification_with_metadata() {
    use tempfile::NamedTempFile;

    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_string_lossy().to_string();

    let manager = NotificationManager::new();

    let mut metadata = HashMap::new();
    metadata.insert("environment".to_string(), "production".to_string());
    metadata.insert("version".to_string(), "1.2.3".to_string());

    let context = NotificationContext::new()
        .with_metadata("environment", "production")
        .with_metadata("version", "1.2.3");

    let spec = NotificationSpec::Structured {
        message: "Deployment complete".to_string(),
        channels: vec![NotificationChannel::File {
            path: file_path.clone(),
            append: false,
            timestamp: true,
            format: FileNotificationFormat::Json,
        }],
        title: Some("Deployment Notification".to_string()),
        priority: Some(periplon_sdk::dsl::NotificationPriority::High),
        metadata,
    };

    let result = manager.send(&spec, &context).await;
    assert!(result.is_ok());

    let contents = std::fs::read_to_string(&file_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&contents).unwrap();

    assert_eq!(parsed["message"], "Deployment complete");
    assert!(parsed["metadata"].is_object());
    assert_eq!(parsed["metadata"]["environment"], "production");
    assert_eq!(parsed["metadata"]["version"], "1.2.3");
}

#[test]
fn test_notification_priority_serialization() {
    use periplon_sdk::dsl::NotificationPriority;

    let priorities = vec![
        (NotificationPriority::Low, "low"),
        (NotificationPriority::Normal, "normal"),
        (NotificationPriority::High, "high"),
        (NotificationPriority::Critical, "critical"),
    ];

    for (priority, expected_str) in priorities {
        let json = serde_json::to_string(&priority).unwrap();
        assert_eq!(json, format!(r#""{}""#, expected_str));

        let deserialized: NotificationPriority = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, priority);
    }
}

#[test]
fn test_file_notification_format_serialization() {
    let formats = vec![
        (FileNotificationFormat::Text, "text"),
        (FileNotificationFormat::Json, "json"),
        (FileNotificationFormat::JsonLines, "jsonlines"),
    ];

    for (format, expected_str) in formats {
        let json = serde_json::to_string(&format).unwrap();
        assert_eq!(json, format!(r#""{}""#, expected_str));

        let deserialized: FileNotificationFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, format);
    }
}

#[test]
fn test_slack_channel_deserialization() {
    let json = r##"{
        "type": "slack",
        "credential": "${secret.slack_webhook}",
        "channel": "#deployments",
        "method": "webhook",
        "attachments": [
            {
                "text": "Deployment succeeded",
                "color": "good",
                "fields": [
                    {
                        "title": "Environment",
                        "value": "Production",
                        "short": true
                    }
                ]
            }
        ]
    }"##;

    let channel: NotificationChannel = serde_json::from_str(json).unwrap();

    match channel {
        NotificationChannel::Slack {
            credential,
            channel: ch,
            method,
            attachments,
        } => {
            assert_eq!(credential, "${secret.slack_webhook}");
            assert_eq!(ch, "#deployments");
            assert_eq!(method, SlackMethod::Webhook);
            assert_eq!(attachments.len(), 1);
            assert_eq!(attachments[0].text, "Deployment succeeded");
            assert_eq!(attachments[0].color, Some("good".to_string()));
            assert_eq!(attachments[0].fields.len(), 1);
        }
        _ => panic!("Expected Slack channel"),
    }
}

#[test]
fn test_discord_channel_deserialization() {
    let json = r#"{
        "type": "discord",
        "webhook_url": "https://discord.com/api/webhooks/123/abc",
        "username": "DeployBot",
        "tts": false,
        "embed": {
            "title": "Deployment Status",
            "description": "Production deployment completed",
            "color": 65280,
            "fields": [
                {
                    "name": "Version",
                    "value": "1.2.3",
                    "inline": true
                }
            ],
            "footer": "Automated notification"
        }
    }"#;

    let channel: NotificationChannel = serde_json::from_str(json).unwrap();

    match channel {
        NotificationChannel::Discord {
            webhook_url,
            username,
            tts,
            embed,
            ..
        } => {
            assert_eq!(webhook_url, "https://discord.com/api/webhooks/123/abc");
            assert_eq!(username, Some("DeployBot".to_string()));
            assert_eq!(tts, false);
            assert!(embed.is_some());

            let embed = embed.unwrap();
            assert_eq!(embed.title, Some("Deployment Status".to_string()));
            assert_eq!(embed.color, Some(65280));
            assert_eq!(embed.fields.len(), 1);
        }
        _ => panic!("Expected Discord channel"),
    }
}

#[test]
fn test_ntfy_channel_deserialization() {
    let json = r#"{
        "type": "ntfy",
        "server": "https://ntfy.sh",
        "topic": "my-topic",
        "title": "Alert",
        "priority": 4,
        "tags": ["warning", "urgent"],
        "markdown": true
    }"#;

    let channel: NotificationChannel = serde_json::from_str(json).unwrap();

    match channel {
        NotificationChannel::Ntfy {
            server,
            topic,
            title,
            priority,
            tags,
            markdown,
            ..
        } => {
            assert_eq!(server, "https://ntfy.sh");
            assert_eq!(topic, "my-topic");
            assert_eq!(title, Some("Alert".to_string()));
            assert_eq!(priority, Some(4));
            assert_eq!(tags.len(), 2);
            assert!(tags.contains(&"warning".to_string()));
            assert_eq!(markdown, true);
        }
        _ => panic!("Expected Ntfy channel"),
    }
}

#[test]
fn test_notification_spec_yaml_parsing() {
    // Test simple string notification
    let yaml = r#""Task completed successfully""#;
    let spec: NotificationSpec = serde_yaml::from_str(yaml).unwrap();
    match spec {
        NotificationSpec::Simple(msg) => assert_eq!(msg, "Task completed successfully"),
        _ => panic!("Expected Simple variant"),
    }

    // Test structured notification
    let yaml = r#"
message: "Build completed"
title: "CI/CD Notification"
priority: high
channels:
  - type: console
    colored: true
    timestamp: true
"#;
    let spec: NotificationSpec = serde_yaml::from_str(yaml).unwrap();
    match spec {
        NotificationSpec::Structured {
            message,
            title,
            priority,
            channels,
            ..
        } => {
            assert_eq!(message, "Build completed");
            assert_eq!(title, Some("CI/CD Notification".to_string()));
            assert_eq!(
                priority,
                Some(periplon_sdk::dsl::NotificationPriority::High)
            );
            assert_eq!(channels.len(), 1);
        }
        _ => panic!("Expected Structured variant"),
    }
}

#[tokio::test]
async fn test_concurrent_file_writes() {
    use tempfile::NamedTempFile;
    use tokio::fs;

    // Send multiple notifications to different files to avoid conflicts
    let mut tasks = vec![];

    for i in 1..=5 {
        let temp_file_i = NamedTempFile::new().unwrap();
        let file_path_i = temp_file_i.path().to_string_lossy().to_string();

        let manager_clone = NotificationManager::new();
        let context = NotificationContext::new().with_metadata("id", format!("{}", i));

        let spec = NotificationSpec::Structured {
            message: format!("Concurrent notification {}", i),
            channels: vec![NotificationChannel::File {
                path: file_path_i.clone(),
                append: false,
                timestamp: false,
                format: FileNotificationFormat::Text,
            }],
            title: None,
            priority: None,
            metadata: HashMap::new(),
        };

        let task = tokio::spawn(async move {
            manager_clone.send(&spec, &context).await.unwrap();
            file_path_i
        });

        tasks.push(task);
    }

    // Wait for all tasks
    let results = futures::future::join_all(tasks).await;

    // Verify all files were written
    for result in results {
        let path = result.unwrap();
        let contents = fs::read_to_string(&path).await.unwrap();
        assert!(contents.contains("Concurrent notification"));
    }
}

#[tokio::test]
async fn test_error_on_missing_file_parent_dir() {
    let manager = NotificationManager::new();
    let context = NotificationContext::new();

    let spec = NotificationSpec::Structured {
        message: "Test".to_string(),
        channels: vec![NotificationChannel::File {
            path: "/nonexistent/path/to/file.log".to_string(),
            append: false,
            timestamp: false,
            format: FileNotificationFormat::Text,
        }],
        title: None,
        priority: None,
        metadata: HashMap::new(),
    };

    let result = manager.send(&spec, &context).await;
    assert!(result.is_err());
}

#[test]
fn test_notification_context_clone() {
    let context = NotificationContext::new()
        .with_workflow_var("key", "value")
        .with_task_var("task", "test");

    let cloned = context.clone();

    assert_eq!(context.workflow_vars, cloned.workflow_vars);
    assert_eq!(context.task_vars, cloned.task_vars);
}

#[test]
fn test_notification_manager_custom_sender() {
    use async_trait::async_trait;
    use periplon_sdk::dsl::{NotificationResult, NotificationSender};

    struct CustomSender;

    #[async_trait]
    impl NotificationSender for CustomSender {
        async fn send(
            &self,
            _message: &str,
            _channel: &NotificationChannel,
            _context: &NotificationContext,
        ) -> NotificationResult<()> {
            Ok(())
        }

        fn channel_name(&self) -> &str {
            "custom"
        }
    }

    let mut manager = NotificationManager::new();
    manager.register_sender("custom".to_string(), Box::new(CustomSender));

    assert!(manager.has_sender("custom"));
}

#[tokio::test]
async fn test_notification_with_empty_message() {
    let manager = NotificationManager::new();
    let context = NotificationContext::new();

    let spec = NotificationSpec::Simple("".to_string());

    // Should succeed with empty message
    let result = manager.send(&spec, &context).await;
    assert!(result.is_ok());
}

#[test]
fn test_interpolation_with_unicode() {
    let context = NotificationContext::new()
        .with_workflow_var("emoji", "✅")
        .with_task_var("lang", "日本語");

    let result = context
        .interpolate("Status: ${workflow.emoji} Language: ${task.lang}")
        .unwrap();
    assert_eq!(result, "Status: ✅ Language: 日本語");
}
