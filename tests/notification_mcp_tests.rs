//! MCP Integration Tests for Notification System
//!
//! These tests verify notification delivery through the MCP server integration.
//! They test actual notification delivery, error handling, variable interpolation,
//! and concurrent notifications.
//!
//! Prerequisites:
//! - MCP server must be configured and accessible
//! - Environment variable SKIP_MCP_TESTS can be set to skip these tests
//! - ntfy server must be accessible (uses ntfy.sh by default)

use periplon_sdk::dsl::{
    NotificationChannel, NotificationContext, NotificationManager, NotificationSpec,
};
use std::collections::HashMap;
use std::env;

/// Helper to check if MCP tests should be skipped
fn should_skip_mcp_tests() -> bool {
    env::var("SKIP_MCP_TESTS").is_ok()
}

/// Helper to create a unique test topic name
fn create_test_topic(test_name: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("test-{}-{}", test_name, timestamp)
}

#[tokio::test]
async fn test_ntfy_basic_notification() {
    if should_skip_mcp_tests() {
        println!("Skipping MCP test (SKIP_MCP_TESTS is set)");
        return;
    }

    let manager = NotificationManager::new();
    let context = NotificationContext::new();
    let topic = create_test_topic("basic");

    let spec = NotificationSpec::Structured {
        message: "Basic ntfy notification test".to_string(),
        channels: vec![NotificationChannel::Ntfy {
            server: "https://ntfy.sh".to_string(),
            topic: topic.clone(),
            title: Some("Test Notification".to_string()),
            priority: Some(3),
            tags: vec!["test".to_string()],
            click_url: None,
            attach_url: None,
            markdown: false,
            auth_token: None,
        }],
        title: None,
        priority: None,
        metadata: HashMap::new(),
    };

    let result = manager.send(&spec, &context).await;
    assert!(
        result.is_ok(),
        "Basic ntfy notification should succeed: {:?}",
        result
    );
    println!("✓ Basic ntfy notification sent to topic: {}", topic);
}

#[tokio::test]
async fn test_ntfy_with_variable_interpolation() {
    if should_skip_mcp_tests() {
        println!("Skipping MCP test (SKIP_MCP_TESTS is set)");
        return;
    }

    let manager = NotificationManager::new();
    let topic = create_test_topic("interpolation");

    let context = NotificationContext::new()
        .with_workflow_var("workflow_name", "TestWorkflow")
        .with_task_var("task_id", "task-123")
        .with_metadata("status", "success")
        .with_metadata("duration", "45s");

    let message = r#"
Workflow: ${workflow.workflow_name}
Task: ${task.task_id}
Status: ${metadata.status}
Duration: ${metadata.duration}
"#;

    let spec = NotificationSpec::Structured {
        message: message.to_string(),
        channels: vec![NotificationChannel::Ntfy {
            server: "https://ntfy.sh".to_string(),
            topic: topic.clone(),
            title: Some("${workflow.workflow_name} - ${metadata.status}".to_string()),
            priority: Some(4),
            tags: vec!["workflow".to_string(), "test".to_string()],
            click_url: None,
            attach_url: None,
            markdown: true,
            auth_token: None,
        }],
        title: None,
        priority: None,
        metadata: HashMap::new(),
    };

    let result = manager.send(&spec, &context).await;
    assert!(
        result.is_ok(),
        "Ntfy notification with interpolation should succeed: {:?}",
        result
    );
    println!(
        "✓ Ntfy notification with variable interpolation sent to topic: {}",
        topic
    );
}

#[tokio::test]
async fn test_ntfy_with_markdown() {
    if should_skip_mcp_tests() {
        println!("Skipping MCP test (SKIP_MCP_TESTS is set)");
        return;
    }

    let manager = NotificationManager::new();
    let topic = create_test_topic("markdown");
    let context = NotificationContext::new();

    let message = r#"
# Deployment Complete

## Summary
- **Status**: ✅ Success
- **Environment**: Production
- **Version**: 1.2.3

## Details
The deployment was completed successfully with no errors.

### Next Steps
1. Monitor application logs
2. Verify metrics dashboard
3. Update documentation
"#;

    let spec = NotificationSpec::Structured {
        message: message.to_string(),
        channels: vec![NotificationChannel::Ntfy {
            server: "https://ntfy.sh".to_string(),
            topic: topic.clone(),
            title: Some("Deployment Report".to_string()),
            priority: Some(4),
            tags: vec!["deployment".to_string(), "success".to_string()],
            click_url: Some("https://example.com/deployment/123".to_string()),
            attach_url: None,
            markdown: true,
            auth_token: None,
        }],
        title: None,
        priority: None,
        metadata: HashMap::new(),
    };

    let result = manager.send(&spec, &context).await;
    assert!(
        result.is_ok(),
        "Ntfy notification with markdown should succeed: {:?}",
        result
    );
    println!("✓ Ntfy notification with markdown sent to topic: {}", topic);
}

#[tokio::test]
async fn test_ntfy_priority_levels() {
    if should_skip_mcp_tests() {
        println!("Skipping MCP test (SKIP_MCP_TESTS is set)");
        return;
    }

    let manager = NotificationManager::new();
    let context = NotificationContext::new();

    // Test different priority levels
    let priorities = vec![
        (1, "min", "🔵"),
        (3, "default", "⚪"),
        (4, "high", "🟠"),
        (5, "urgent", "🔴"),
    ];

    for (priority, label, emoji) in priorities {
        let topic = create_test_topic(&format!("priority-{}", label));

        let spec = NotificationSpec::Structured {
            message: format!("Priority {} test notification", label),
            channels: vec![NotificationChannel::Ntfy {
                server: "https://ntfy.sh".to_string(),
                topic: topic.clone(),
                title: Some(format!("{} Priority Test", emoji)),
                priority: Some(priority),
                tags: vec!["priority-test".to_string(), label.to_string()],
                click_url: None,
                attach_url: None,
                markdown: false,
                auth_token: None,
            }],
            title: None,
            priority: None,
            metadata: HashMap::new(),
        };

        let result = manager.send(&spec, &context).await;
        assert!(
            result.is_ok(),
            "Ntfy notification with priority {} should succeed: {:?}",
            priority,
            result
        );
        println!(
            "✓ Ntfy notification with priority {} sent to topic: {}",
            priority, topic
        );

        // Small delay to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}

#[tokio::test]
async fn test_ntfy_with_tags() {
    if should_skip_mcp_tests() {
        println!("Skipping MCP test (SKIP_MCP_TESTS is set)");
        return;
    }

    let manager = NotificationManager::new();
    let topic = create_test_topic("tags");
    let context = NotificationContext::new();

    let spec = NotificationSpec::Structured {
        message: "Testing notification with emoji tags".to_string(),
        channels: vec![NotificationChannel::Ntfy {
            server: "https://ntfy.sh".to_string(),
            topic: topic.clone(),
            title: Some("Tag Test".to_string()),
            priority: Some(3),
            tags: vec![
                "warning".to_string(),
                "rocket".to_string(),
                "tada".to_string(),
            ],
            click_url: None,
            attach_url: None,
            markdown: false,
            auth_token: None,
        }],
        title: None,
        priority: None,
        metadata: HashMap::new(),
    };

    let result = manager.send(&spec, &context).await;
    assert!(
        result.is_ok(),
        "Ntfy notification with tags should succeed: {:?}",
        result
    );
    println!("✓ Ntfy notification with tags sent to topic: {}", topic);
}

#[tokio::test]
async fn test_ntfy_concurrent_notifications() {
    if should_skip_mcp_tests() {
        println!("Skipping MCP test (SKIP_MCP_TESTS is set)");
        return;
    }

    let base_topic = create_test_topic("concurrent");

    // Send 5 notifications concurrently
    let mut tasks = vec![];

    for i in 1..=5 {
        let topic = format!("{}-{}", base_topic, i);
        let manager = NotificationManager::new();
        let context =
            NotificationContext::new().with_metadata("notification_id", format!("notif-{}", i));

        let spec = NotificationSpec::Structured {
            message: format!("Concurrent notification #{}", i),
            channels: vec![NotificationChannel::Ntfy {
                server: "https://ntfy.sh".to_string(),
                topic: topic.clone(),
                title: Some(format!("Concurrent Test #{}", i)),
                priority: Some(3),
                tags: vec!["concurrent".to_string(), format!("test-{}", i)],
                click_url: None,
                attach_url: None,
                markdown: false,
                auth_token: None,
            }],
            title: None,
            priority: None,
            metadata: HashMap::new(),
        };

        let task = tokio::spawn(async move {
            let result = manager.send(&spec, &context).await;
            (i, topic, result)
        });

        tasks.push(task);
    }

    // Wait for all tasks to complete
    let results = futures::future::join_all(tasks).await;

    // Verify all succeeded
    for result in results {
        let (i, topic, send_result) = result.unwrap();
        assert!(
            send_result.is_ok(),
            "Concurrent notification {} should succeed: {:?}",
            i,
            send_result
        );
        println!("✓ Concurrent notification {} sent to topic: {}", i, topic);
    }
}

#[tokio::test]
async fn test_ntfy_error_handling_invalid_server() {
    if should_skip_mcp_tests() {
        println!("Skipping MCP test (SKIP_MCP_TESTS is set)");
        return;
    }

    let manager = NotificationManager::new();
    let context = NotificationContext::new();
    let topic = create_test_topic("error");

    let spec = NotificationSpec::Structured {
        message: "This should fail".to_string(),
        channels: vec![NotificationChannel::Ntfy {
            server: "http://invalid-server-that-does-not-exist.local".to_string(),
            topic: topic.clone(),
            title: None,
            priority: None,
            tags: vec![],
            click_url: None,
            attach_url: None,
            markdown: false,
            auth_token: None,
        }],
        title: None,
        priority: None,
        metadata: HashMap::new(),
    };

    let result = manager.send(&spec, &context).await;
    assert!(
        result.is_err(),
        "Ntfy notification to invalid server should fail"
    );
    println!("✓ Error handling for invalid server works correctly");
}

#[tokio::test]
async fn test_ntfy_mixed_channels() {
    if should_skip_mcp_tests() {
        println!("Skipping MCP test (SKIP_MCP_TESTS is set)");
        return;
    }

    use tempfile::NamedTempFile;

    let manager = NotificationManager::new();
    let topic = create_test_topic("mixed");
    let context = NotificationContext::new()
        .with_workflow_var("workflow", "multi-channel-test")
        .with_metadata("timestamp", chrono::Local::now().to_rfc3339());

    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_string_lossy().to_string();

    let message = "Testing notification delivery across multiple channels";

    let spec = NotificationSpec::Structured {
        message: message.to_string(),
        channels: vec![
            NotificationChannel::Console {
                colored: true,
                timestamp: true,
            },
            NotificationChannel::File {
                path: file_path.clone(),
                append: false,
                timestamp: true,
                format: periplon_sdk::dsl::FileNotificationFormat::Json,
            },
            NotificationChannel::Ntfy {
                server: "https://ntfy.sh".to_string(),
                topic: topic.clone(),
                title: Some("Multi-Channel Test".to_string()),
                priority: Some(3),
                tags: vec!["multi-channel".to_string()],
                click_url: None,
                attach_url: None,
                markdown: false,
                auth_token: None,
            },
        ],
        title: None,
        priority: None,
        metadata: HashMap::new(),
    };

    let result = manager.send(&spec, &context).await;
    assert!(
        result.is_ok(),
        "Mixed channel notification should succeed: {:?}",
        result
    );

    // Verify file was written
    let contents = tokio::fs::read_to_string(&file_path).await.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert_eq!(parsed["message"], message);

    println!("✓ Multi-channel notification (console + file + ntfy) sent successfully");
}

#[tokio::test]
async fn test_ntfy_with_click_url() {
    if should_skip_mcp_tests() {
        println!("Skipping MCP test (SKIP_MCP_TESTS is set)");
        return;
    }

    let manager = NotificationManager::new();
    let topic = create_test_topic("clickurl");
    let context = NotificationContext::new();

    let spec = NotificationSpec::Structured {
        message: "Click the notification to view deployment details".to_string(),
        channels: vec![NotificationChannel::Ntfy {
            server: "https://ntfy.sh".to_string(),
            topic: topic.clone(),
            title: Some("Deployment Complete".to_string()),
            priority: Some(4),
            tags: vec!["link".to_string()],
            click_url: Some("https://github.com/example/repo/actions/runs/123".to_string()),
            attach_url: None,
            markdown: false,
            auth_token: None,
        }],
        title: None,
        priority: None,
        metadata: HashMap::new(),
    };

    let result = manager.send(&spec, &context).await;
    assert!(
        result.is_ok(),
        "Ntfy notification with click URL should succeed: {:?}",
        result
    );
    println!(
        "✓ Ntfy notification with click URL sent to topic: {}",
        topic
    );
}

#[tokio::test]
async fn test_ntfy_simple_spec() {
    if should_skip_mcp_tests() {
        println!("Skipping MCP test (SKIP_MCP_TESTS is set)");
        return;
    }

    let manager = NotificationManager::new();
    let context = NotificationContext::new();

    // Simple spec should use console by default, not ntfy
    let spec = NotificationSpec::Simple("Simple test notification".to_string());

    let result = manager.send(&spec, &context).await;
    assert!(result.is_ok(), "Simple notification should succeed");
    println!("✓ Simple notification spec works correctly (uses console)");
}
