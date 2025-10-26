//! Comprehensive Error Handling Tests
//!
//! Tests for retry logic, exponential backoff, and fallback agents

use periplon_sdk::dsl::executor::DSLExecutor;
use periplon_sdk::dsl::schema::{DSLWorkflow, ErrorHandlingSpec, TaskSpec};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Counter for tracking retry attempts
#[derive(Default, Clone)]
struct RetryCounter {
    inner: Arc<Mutex<u32>>,
}

impl RetryCounter {
    fn increment(&self) -> u32 {
        let mut count = self.inner.lock().unwrap();
        *count += 1;
        *count
    }

    fn get(&self) -> u32 {
        *self.inner.lock().unwrap()
    }
}

/// Create a test workflow directory
fn setup_test_workspace(test_name: &str) -> String {
    let workspace_dir = format!("test_results/{}", test_name);
    let _ = fs::remove_dir_all(&workspace_dir);
    fs::create_dir_all(&workspace_dir).unwrap();
    workspace_dir
}

/// Cleanup test workspace
fn cleanup_test_workspace(workspace_dir: &str) {
    let _ = fs::remove_dir_all(workspace_dir);
}

#[tokio::test]
async fn test_basic_retry_mechanism() {
    let workspace_dir = setup_test_workspace("basic_retry");

    // Create a script that fails twice, then succeeds
    let script_path = format!("{}/retry_script.sh", workspace_dir);
    let counter_file = format!("{}/counter.txt", workspace_dir);

    fs::write(
        &script_path,
        format!(
            r#"#!/bin/bash
if [ ! -f "{counter_file}" ]; then
    echo "0" > "{counter_file}"
fi
count=$(cat "{counter_file}")
count=$((count + 1))
echo "$count" > "{counter_file}"
if [ "$count" -lt 3 ]; then
    echo "Attempt $count - failing"
    exit 1
else
    echo "Attempt $count - success"
    exit 0
fi
"#,
            counter_file = counter_file
        ),
    )
    .unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();
    }

    let mut workflow = DSLWorkflow {
        name: "Basic Retry Test".to_string(),
        version: "1.0.0".to_string(),
        dsl_version: "1.0.0".to_string(),
        cwd: Some(workspace_dir.clone()),
        create_cwd: Some(true),
        agents: HashMap::new(),
        tasks: HashMap::new(),
        workflows: HashMap::new(),
        tools: None,
        communication: None,
        mcp_servers: HashMap::new(),
        subflows: HashMap::new(),
        imports: HashMap::new(),
        notifications: None,
        secrets: HashMap::new(),
        inputs: HashMap::new(),
        outputs: HashMap::new(),
        limits: None,
    };

    // Add task with retry configuration
    let task = TaskSpec {
        description: "Task that retries".to_string(),
        command: Some(periplon_sdk::dsl::schema::CommandSpec {
            executable: "bash".to_string(),
            args: vec![script_path.clone()],
            working_dir: None,
            env: HashMap::new(),
            timeout_secs: None,
            capture_stdout: true,
            capture_stderr: true,
        }),
        on_error: Some(ErrorHandlingSpec {
            retry: 3,
            fallback_agent: None,
            retry_delay_secs: 1,
            exponential_backoff: false,
        }),
        ..Default::default()
    };

    workflow.tasks.insert("retry_task".to_string(), task);

    // Execute workflow
    let mut executor = DSLExecutor::new(workflow).expect("Failed to create executor");
    let result = executor.execute().await;

    // Should succeed after retries
    assert!(result.is_ok(), "Workflow should succeed after retries");

    // Check that it took 3 attempts
    let counter_content = fs::read_to_string(&counter_file).unwrap();
    let count: u32 = counter_content.trim().parse().unwrap();
    assert_eq!(count, 3, "Should have taken 3 attempts");

    cleanup_test_workspace(&workspace_dir);
}

#[tokio::test]
async fn test_exponential_backoff() {
    let workspace_dir = setup_test_workspace("exponential_backoff");

    // Create a script that always fails (to test backoff timing)
    let script_path = format!("{}/failing_script.sh", workspace_dir);
    let timestamp_file = format!("{}/timestamps.txt", workspace_dir);

    fs::write(
        &script_path,
        format!(
            r#"#!/bin/bash
date +%s >> "{timestamp_file}"
exit 1
"#,
            timestamp_file = timestamp_file
        ),
    )
    .unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();
    }

    let mut workflow = DSLWorkflow {
        name: "Exponential Backoff Test".to_string(),
        version: "1.0.0".to_string(),
        dsl_version: "1.0.0".to_string(),
        cwd: Some(workspace_dir.clone()),
        create_cwd: Some(true),
        agents: HashMap::new(),
        tasks: HashMap::new(),
        workflows: HashMap::new(),
        tools: None,
        communication: None,
        mcp_servers: HashMap::new(),
        subflows: HashMap::new(),
        imports: HashMap::new(),
        notifications: None,
        secrets: HashMap::new(),
        inputs: HashMap::new(),
        outputs: HashMap::new(),
        limits: None,
    };

    // Add task with exponential backoff
    let task = TaskSpec {
        description: "Task with exponential backoff".to_string(),
        command: Some(periplon_sdk::dsl::schema::CommandSpec {
            executable: "bash".to_string(),
            args: vec![script_path.clone()],
            working_dir: None,
            env: HashMap::new(),
            timeout_secs: None,
            capture_stdout: true,
            capture_stderr: true,
        }),
        on_error: Some(ErrorHandlingSpec {
            retry: 3,
            fallback_agent: None,
            retry_delay_secs: 1,
            exponential_backoff: true,
        }),
        ..Default::default()
    };

    workflow.tasks.insert("backoff_task".to_string(), task);

    // Execute workflow (will fail, but we're testing timing)
    let start_time = Instant::now();
    let mut executor = DSLExecutor::new(workflow).expect("Failed to create executor");
    let _ = executor.execute().await;
    let elapsed = start_time.elapsed();

    // With exponential backoff: 1s, 2s, 4s = 7s total minimum
    // Allow some tolerance for execution time
    assert!(
        elapsed.as_secs() >= 7,
        "Exponential backoff should take at least 7 seconds, took {}s",
        elapsed.as_secs()
    );

    // Verify timestamps show increasing delays
    if Path::new(&timestamp_file).exists() {
        let timestamps_content = fs::read_to_string(&timestamp_file).unwrap();
        let timestamps: Vec<i64> = timestamps_content
            .lines()
            .filter_map(|line| line.trim().parse().ok())
            .collect();

        if timestamps.len() >= 3 {
            let delay1 = timestamps[1] - timestamps[0];
            let delay2 = timestamps[2] - timestamps[1];

            // Second delay should be roughly 2x first delay (exponential)
            // Allow tolerance for execution overhead
            assert!(
                delay2 >= delay1,
                "Delays should be increasing: {}s, {}s",
                delay1,
                delay2
            );
        }
    }

    cleanup_test_workspace(&workspace_dir);
}

#[tokio::test]
async fn test_fallback_agent_success() {
    let workspace_dir = setup_test_workspace("fallback_agent");

    // Create scripts: primary fails, fallback succeeds
    let primary_script = format!("{}/primary.sh", workspace_dir);
    let fallback_script = format!("{}/fallback.sh", workspace_dir);
    let output_file = format!("{}/output.txt", workspace_dir);

    fs::write(
        &primary_script,
        r#"#!/bin/bash
echo "Primary agent failed"
exit 1
"#,
    )
    .unwrap();

    fs::write(
        &fallback_script,
        format!(
            r#"#!/bin/bash
echo "Fallback agent succeeded" > "{output_file}"
exit 0
"#,
            output_file = output_file
        ),
    )
    .unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for script in &[&primary_script, &fallback_script] {
            let mut perms = fs::metadata(script).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(script, perms).unwrap();
        }
    }

    let mut workflow = DSLWorkflow {
        name: "Fallback Agent Test".to_string(),
        version: "1.0.0".to_string(),
        dsl_version: "1.0.0".to_string(),
        cwd: Some(workspace_dir.clone()),
        create_cwd: Some(true),
        agents: HashMap::new(),
        tasks: HashMap::new(),
        workflows: HashMap::new(),
        tools: None,
        communication: None,
        mcp_servers: HashMap::new(),
        subflows: HashMap::new(),
        imports: HashMap::new(),
        notifications: None,
        secrets: HashMap::new(),
        inputs: HashMap::new(),
        outputs: HashMap::new(),
        limits: None,
    };

    // Note: Fallback agents are designed for agent-based tasks, not command tasks
    // This test demonstrates the configuration structure
    // In a real scenario, you would use actual agents with the SDK

    let task = TaskSpec {
        description: "Task with fallback".to_string(),
        command: Some(periplon_sdk::dsl::schema::CommandSpec {
            executable: "bash".to_string(),
            args: vec![primary_script.clone()],
            working_dir: None,
            env: HashMap::new(),
            timeout_secs: None,
            capture_stdout: true,
            capture_stderr: true,
        }),
        on_error: Some(ErrorHandlingSpec {
            retry: 0, // No retries, go straight to fallback
            fallback_agent: Some("fallback_agent".to_string()),
            retry_delay_secs: 1,
            exponential_backoff: false,
        }),
        ..Default::default()
    };

    workflow.tasks.insert("fallback_task".to_string(), task);

    // The fallback mechanism is implemented for agent-based tasks
    // For this test, we verify the configuration structure is correct
    assert!(workflow.tasks.contains_key("fallback_task"));
    let task = workflow.tasks.get("fallback_task").unwrap();
    assert!(task.on_error.is_some());
    let error_handling = task.on_error.as_ref().unwrap();
    assert_eq!(
        error_handling.fallback_agent,
        Some("fallback_agent".to_string())
    );

    cleanup_test_workspace(&workspace_dir);
}

#[tokio::test]
async fn test_retry_with_delay() {
    let workspace_dir = setup_test_workspace("retry_with_delay");

    let script_path = format!("{}/delay_script.sh", workspace_dir);
    let timestamp_file = format!("{}/timestamps.txt", workspace_dir);

    fs::write(
        &script_path,
        format!(
            r#"#!/bin/bash
date +%s >> "{timestamp_file}"
exit 1
"#,
            timestamp_file = timestamp_file
        ),
    )
    .unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();
    }

    let mut workflow = DSLWorkflow {
        name: "Retry Delay Test".to_string(),
        version: "1.0.0".to_string(),
        dsl_version: "1.0.0".to_string(),
        cwd: Some(workspace_dir.clone()),
        create_cwd: Some(true),
        agents: HashMap::new(),
        tasks: HashMap::new(),
        workflows: HashMap::new(),
        tools: None,
        communication: None,
        mcp_servers: HashMap::new(),
        subflows: HashMap::new(),
        imports: HashMap::new(),
        notifications: None,
        secrets: HashMap::new(),
        inputs: HashMap::new(),
        outputs: HashMap::new(),
        limits: None,
    };

    let task = TaskSpec {
        description: "Task with retry delay".to_string(),
        command: Some(periplon_sdk::dsl::schema::CommandSpec {
            executable: "bash".to_string(),
            args: vec![script_path.clone()],
            working_dir: None,
            env: HashMap::new(),
            timeout_secs: None,
            capture_stdout: true,
            capture_stderr: true,
        }),
        on_error: Some(ErrorHandlingSpec {
            retry: 2,
            fallback_agent: None,
            retry_delay_secs: 2,
            exponential_backoff: false,
        }),
        ..Default::default()
    };

    workflow.tasks.insert("delay_task".to_string(), task);

    let start_time = Instant::now();
    let mut executor = DSLExecutor::new(workflow).expect("Failed to create executor");
    let _ = executor.execute().await;
    let elapsed = start_time.elapsed();

    // With 2 retries and 2s delay: 2s + 2s = 4s minimum
    assert!(
        elapsed.as_secs() >= 4,
        "Retry delay should be at least 4 seconds, was {}s",
        elapsed.as_secs()
    );

    cleanup_test_workspace(&workspace_dir);
}

#[tokio::test]
async fn test_error_handling_config_validation() {
    // Test that ErrorHandlingSpec is correctly configured
    let error_spec = ErrorHandlingSpec {
        retry: 3,
        fallback_agent: Some("backup_agent".to_string()),
        retry_delay_secs: 5,
        exponential_backoff: true,
    };

    assert_eq!(error_spec.retry, 3);
    assert_eq!(error_spec.fallback_agent, Some("backup_agent".to_string()));
    assert_eq!(error_spec.retry_delay_secs, 5);
    assert!(error_spec.exponential_backoff);
}

#[tokio::test]
async fn test_retry_exhaustion() {
    let workspace_dir = setup_test_workspace("retry_exhaustion");

    // Create a script that always fails
    let script_path = format!("{}/always_fail.sh", workspace_dir);
    let counter_file = format!("{}/attempts.txt", workspace_dir);

    fs::write(
        &script_path,
        format!(
            r#"#!/bin/bash
if [ ! -f "{counter_file}" ]; then
    echo "0" > "{counter_file}"
fi
count=$(cat "{counter_file}")
count=$((count + 1))
echo "$count" > "{counter_file}"
echo "Attempt $count - failing"
exit 1
"#,
            counter_file = counter_file
        ),
    )
    .unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();
    }

    let mut workflow = DSLWorkflow {
        name: "Retry Exhaustion Test".to_string(),
        version: "1.0.0".to_string(),
        dsl_version: "1.0.0".to_string(),
        cwd: Some(workspace_dir.clone()),
        create_cwd: Some(true),
        agents: HashMap::new(),
        tasks: HashMap::new(),
        workflows: HashMap::new(),
        tools: None,
        communication: None,
        mcp_servers: HashMap::new(),
        subflows: HashMap::new(),
        imports: HashMap::new(),
        notifications: None,
        secrets: HashMap::new(),
        inputs: HashMap::new(),
        outputs: HashMap::new(),
        limits: None,
    };

    let task = TaskSpec {
        description: "Task that exhausts retries".to_string(),
        command: Some(periplon_sdk::dsl::schema::CommandSpec {
            executable: "bash".to_string(),
            args: vec![script_path.clone()],
            working_dir: None,
            env: HashMap::new(),
            timeout_secs: None,
            capture_stdout: true,
            capture_stderr: true,
        }),
        on_error: Some(ErrorHandlingSpec {
            retry: 3,
            fallback_agent: None,
            retry_delay_secs: 1,
            exponential_backoff: false,
        }),
        ..Default::default()
    };

    workflow.tasks.insert("exhaust_task".to_string(), task);

    let mut executor = DSLExecutor::new(workflow).expect("Failed to create executor");
    let result = executor.execute().await;

    // Should fail after all retries
    assert!(
        result.is_err(),
        "Workflow should fail after exhausting retries"
    );

    // Verify it tried the correct number of times (initial + 3 retries = 4 total)
    let counter_content = fs::read_to_string(&counter_file).unwrap();
    let count: u32 = counter_content.trim().parse().unwrap();
    assert_eq!(
        count, 4,
        "Should have 4 total attempts (1 initial + 3 retries)"
    );

    cleanup_test_workspace(&workspace_dir);
}

#[tokio::test]
async fn test_no_retry_on_success() {
    let workspace_dir = setup_test_workspace("no_retry_success");

    // Create a script that succeeds immediately
    let script_path = format!("{}/success_script.sh", workspace_dir);
    let counter_file = format!("{}/success_counter.txt", workspace_dir);

    fs::write(
        &script_path,
        format!(
            r#"#!/bin/bash
if [ ! -f "{counter_file}" ]; then
    echo "0" > "{counter_file}"
fi
count=$(cat "{counter_file}")
count=$((count + 1))
echo "$count" > "{counter_file}"
echo "Success on attempt $count"
exit 0
"#,
            counter_file = counter_file
        ),
    )
    .unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();
    }

    let mut workflow = DSLWorkflow {
        name: "No Retry on Success Test".to_string(),
        version: "1.0.0".to_string(),
        dsl_version: "1.0.0".to_string(),
        cwd: Some(workspace_dir.clone()),
        create_cwd: Some(true),
        agents: HashMap::new(),
        tasks: HashMap::new(),
        workflows: HashMap::new(),
        tools: None,
        communication: None,
        mcp_servers: HashMap::new(),
        subflows: HashMap::new(),
        imports: HashMap::new(),
        notifications: None,
        secrets: HashMap::new(),
        inputs: HashMap::new(),
        outputs: HashMap::new(),
        limits: None,
    };

    let task = TaskSpec {
        description: "Successful task with retry config".to_string(),
        command: Some(periplon_sdk::dsl::schema::CommandSpec {
            executable: "bash".to_string(),
            args: vec![script_path.clone()],
            working_dir: None,
            env: HashMap::new(),
            timeout_secs: None,
            capture_stdout: true,
            capture_stderr: true,
        }),
        on_error: Some(ErrorHandlingSpec {
            retry: 5,
            fallback_agent: None,
            retry_delay_secs: 1,
            exponential_backoff: false,
        }),
        ..Default::default()
    };

    workflow.tasks.insert("success_task".to_string(), task);

    let mut executor = DSLExecutor::new(workflow).expect("Failed to create executor");
    let result = executor.execute().await;

    assert!(result.is_ok(), "Workflow should succeed");

    // Should only execute once (no retries on success)
    let counter_content = fs::read_to_string(&counter_file).unwrap();
    let count: u32 = counter_content.trim().parse().unwrap();
    assert_eq!(count, 1, "Should only execute once on success");

    cleanup_test_workspace(&workspace_dir);
}
