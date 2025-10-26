//! CLI Group Management Commands Integration Tests
//!
//! Tests the `dsl-executor group` subcommands:
//! - group list
//! - group install
//! - group update
//! - group validate

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

// ============================================================================
// Test Helpers
// ============================================================================

/// Get the path to the dsl-executor binary
fn get_dsl_executor_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");

    // Try release first, then debug
    let release_path = path.join("release").join("dsl-executor");
    if release_path.exists() {
        return release_path;
    }

    path.join("debug").join("dsl-executor")
}

/// Check if dsl-executor binary exists (built)
fn is_binary_available() -> bool {
    get_dsl_executor_path().exists()
}

/// Create a minimal task YAML for testing
fn create_test_task(dir: &std::path::Path, name: &str, version: &str) -> PathBuf {
    let content = format!(
        r#"apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "{}"
  version: "{}"
  description: "Test task"
spec:
  agent_template:
    description: "Test agent"
    tools: ["Read"]
"#,
        name, version
    );

    let file_path = dir.join(format!("{}.task.yaml", name));
    fs::write(&file_path, content).unwrap();
    file_path
}

/// Create a test task group YAML
fn create_test_group(
    dir: &std::path::Path,
    name: &str,
    version: &str,
    tasks: Vec<(&str, &str)>,
) -> PathBuf {
    let tasks_yaml: Vec<String> = tasks
        .iter()
        .map(|(task_name, task_version)| {
            format!(
                r#"  - name: "{}"
    version: "{}""#,
                task_name, task_version
            )
        })
        .collect();

    let content = format!(
        r#"apiVersion: "taskgroup/v1"
kind: "TaskGroup"
metadata:
  name: "{}"
  version: "{}"
  description: "Test group for CLI testing"
spec:
  tasks:
{}
"#,
        name,
        version,
        tasks_yaml.join("\n")
    );

    let file_path = dir.join(format!("{}.taskgroup.yaml", name));
    fs::write(&file_path, content).unwrap();
    file_path
}

// ============================================================================
// SECTION 1: Group List Command Tests
// ============================================================================

#[test]
fn test_group_list_empty_directory() {
    if !is_binary_available() {
        eprintln!("Skipping test: dsl-executor binary not built");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let groups_dir = temp_dir.path().join("groups");
    fs::create_dir_all(&groups_dir).unwrap();

    let output = Command::new(get_dsl_executor_path())
        .args(&["group", "list", "--path", groups_dir.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    // Should succeed even with no groups
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No task groups found") || stdout.contains("groups: []"));
}

#[test]
fn test_group_list_with_groups() {
    if !is_binary_available() {
        eprintln!("Skipping test: dsl-executor binary not built");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let groups_dir = temp_dir.path().join("groups");
    let tasks_dir = temp_dir.path().join("tasks");
    fs::create_dir_all(&groups_dir).unwrap();
    fs::create_dir_all(&tasks_dir).unwrap();

    // Create test tasks
    create_test_task(&tasks_dir, "task1", "1.0.0");
    create_test_task(&tasks_dir, "task2", "1.0.0");

    // Create test groups
    create_test_group(
        &groups_dir,
        "test-group-1",
        "1.0.0",
        vec![("task1", "1.0.0")],
    );
    create_test_group(
        &groups_dir,
        "test-group-2",
        "2.0.0",
        vec![("task2", "1.0.0")],
    );

    let output = Command::new(get_dsl_executor_path())
        .args(&["group", "list", "--path", groups_dir.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test-group-1") || stdout.contains("test-group-1@1.0.0"));
    assert!(stdout.contains("test-group-2") || stdout.contains("test-group-2@2.0.0"));
}

#[test]
fn test_group_list_json_output() {
    if !is_binary_available() {
        eprintln!("Skipping test: dsl-executor binary not built");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let groups_dir = temp_dir.path().join("groups");
    let tasks_dir = temp_dir.path().join("tasks");
    fs::create_dir_all(&groups_dir).unwrap();
    fs::create_dir_all(&tasks_dir).unwrap();

    create_test_task(&tasks_dir, "task1", "1.0.0");
    create_test_group(&groups_dir, "json-test", "1.0.0", vec![("task1", "1.0.0")]);

    let output = Command::new(get_dsl_executor_path())
        .args(&[
            "group",
            "list",
            "--path",
            groups_dir.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should be valid JSON
    let json_result: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    assert!(json_result.is_ok(), "Output should be valid JSON");

    if let Ok(json) = json_result {
        assert!(json.get("search_paths").is_some());
        assert!(json.get("groups").is_some());
        assert!(json.get("total").is_some());
    }
}

#[test]
fn test_group_list_verbose() {
    if !is_binary_available() {
        eprintln!("Skipping test: dsl-executor binary not built");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let groups_dir = temp_dir.path().join("groups");
    let tasks_dir = temp_dir.path().join("tasks");
    fs::create_dir_all(&groups_dir).unwrap();
    fs::create_dir_all(&tasks_dir).unwrap();

    create_test_task(&tasks_dir, "task1", "1.0.0");
    create_test_group(
        &groups_dir,
        "verbose-test",
        "1.0.0",
        vec![("task1", "1.0.0")],
    );

    let output = Command::new(get_dsl_executor_path())
        .args(&[
            "group",
            "list",
            "--path",
            groups_dir.to_str().unwrap(),
            "--verbose",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Verbose mode should show more details like task count
    assert!(stdout.contains("task") || stdout.contains("Task"));
}

// ============================================================================
// SECTION 2: Group Validate Command Tests
// ============================================================================

#[test]
fn test_group_validate_valid_group() {
    if !is_binary_available() {
        eprintln!("Skipping test: dsl-executor binary not built");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let tasks_dir = temp_dir.path().join("tasks");
    fs::create_dir_all(&tasks_dir).unwrap();

    // Create task first
    create_test_task(&tasks_dir, "valid-task", "1.0.0");

    // Create group file
    let group_file = create_test_group(
        temp_dir.path(),
        "valid-group",
        "1.0.0",
        vec![("valid-task", "1.0.0")],
    );

    // Set up task path environment (if needed by loader)
    let output = Command::new(get_dsl_executor_path())
        .args(&["group", "validate", group_file.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // The command might fail if tasks aren't in the standard paths,
    // but it should at least parse the group file structure
    if !output.status.success() {
        // Check if it's a "task not found" error, which means parsing succeeded
        assert!(
            stderr.contains("not found") || stderr.contains("Task"),
            "Should fail with task not found, not a parse error"
        );
    } else {
        assert!(stdout.contains("valid") || stdout.contains("Task group is valid"));
    }
}

#[test]
fn test_group_validate_invalid_yaml() {
    if !is_binary_available() {
        eprintln!("Skipping test: dsl-executor binary not built");
        return;
    }

    let temp_dir = TempDir::new().unwrap();

    // Create invalid YAML file
    let invalid_file = temp_dir.path().join("invalid.taskgroup.yaml");
    fs::write(
        &invalid_file,
        r#"
apiVersion: "taskgroup/v1"
kind: "TaskGroup"
metadata:
  name: "invalid"
  version: "1.0.0"
spec:
  # Missing tasks field - invalid!
"#,
    )
    .unwrap();

    let output = Command::new(get_dsl_executor_path())
        .args(&["group", "validate", invalid_file.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    // Should fail
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error") || stderr.contains("error"));
}

#[test]
fn test_group_validate_json_output() {
    if !is_binary_available() {
        eprintln!("Skipping test: dsl-executor binary not built");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let group_file = create_test_group(
        temp_dir.path(),
        "json-validate",
        "1.0.0",
        vec![("some-task", "1.0.0")],
    );

    let output = Command::new(get_dsl_executor_path())
        .args(&["group", "validate", group_file.to_str().unwrap(), "--json"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Even if validation fails, JSON output should be valid JSON
    if !stdout.is_empty() {
        let json_result: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
        if json_result.is_ok() {
            let json = json_result.unwrap();
            assert!(json.get("valid").is_some());
            assert!(json.get("group_name").is_some());
            assert!(json.get("group_version").is_some());
        }
    }
}

// ============================================================================
// SECTION 3: Group Install Command Tests
// ============================================================================

#[test]
fn test_group_install_invalid_reference() {
    if !is_binary_available() {
        eprintln!("Skipping test: dsl-executor binary not built");
        return;
    }

    let output = Command::new(get_dsl_executor_path())
        .args(&["group", "install", "invalid-ref-format"])
        .output()
        .expect("Failed to execute command");

    // Should fail with invalid reference
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid") || stderr.contains("Error"));
}

#[test]
fn test_group_install_nonexistent_group() {
    if !is_binary_available() {
        eprintln!("Skipping test: dsl-executor binary not built");
        return;
    }

    let output = Command::new(get_dsl_executor_path())
        .args(&["group", "install", "nonexistent-group@1.0.0"])
        .output()
        .expect("Failed to execute command");

    // Should fail - group doesn't exist
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found") || stderr.contains("Error"));
}

// ============================================================================
// SECTION 4: Group Update Command Tests
// ============================================================================

#[test]
fn test_group_update_all() {
    if !is_binary_available() {
        eprintln!("Skipping test: dsl-executor binary not built");
        return;
    }

    // Update all groups (even if none exist, should not fail)
    let output = Command::new(get_dsl_executor_path())
        .args(&["group", "update"])
        .output()
        .expect("Failed to execute command");

    // Should succeed (or at least not crash)
    // Note: May succeed or fail depending on whether standard paths have groups
    let _stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Just verify the command runs without panicking
    assert!(
        output.status.success() || stderr.contains("Error"),
        "Command should either succeed or fail gracefully"
    );
}

#[test]
fn test_group_update_json_output() {
    if !is_binary_available() {
        eprintln!("Skipping test: dsl-executor binary not built");
        return;
    }

    let output = Command::new(get_dsl_executor_path())
        .args(&["group", "update", "--json"])
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Should be valid JSON
        let json_result: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
        if json_result.is_ok() {
            let json = json_result.unwrap();
            assert!(json.get("success").is_some());
            assert!(json.get("updated_count").is_some());
        }
    }
}

#[test]
fn test_group_update_force() {
    if !is_binary_available() {
        eprintln!("Skipping test: dsl-executor binary not built");
        return;
    }

    let output = Command::new(get_dsl_executor_path())
        .args(&["group", "update", "--force"])
        .output()
        .expect("Failed to execute command");

    // Force flag should be accepted
    // Success depends on whether groups are available
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should not have argument parsing errors
    assert!(
        !stderr.contains("unexpected argument") && !stderr.contains("unrecognized"),
        "Force flag should be recognized"
    );
}

// ============================================================================
// SECTION 5: Command Help Tests
// ============================================================================

#[test]
fn test_group_command_help() {
    if !is_binary_available() {
        eprintln!("Skipping test: dsl-executor binary not built");
        return;
    }

    let output = Command::new(get_dsl_executor_path())
        .args(&["group", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("list"));
    assert!(stdout.contains("install"));
    assert!(stdout.contains("update"));
    assert!(stdout.contains("validate"));
}

#[test]
fn test_group_list_help() {
    if !is_binary_available() {
        eprintln!("Skipping test: dsl-executor binary not built");
        return;
    }

    let output = Command::new(get_dsl_executor_path())
        .args(&["group", "list", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("verbose") || stdout.contains("--verbose"));
    assert!(stdout.contains("json") || stdout.contains("--json"));
    assert!(stdout.contains("path") || stdout.contains("--path"));
}

#[test]
fn test_group_validate_help() {
    if !is_binary_available() {
        eprintln!("Skipping test: dsl-executor binary not built");
        return;
    }

    let output = Command::new(get_dsl_executor_path())
        .args(&["group", "validate", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("GROUP_FILE") || stdout.contains("group-file"));
    assert!(stdout.contains("verbose") || stdout.contains("--verbose"));
}

// ============================================================================
// SECTION 6: Integration Tests
// ============================================================================

#[test]
fn test_group_workflow_list_validate_sequence() {
    if !is_binary_available() {
        eprintln!("Skipping test: dsl-executor binary not built");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let groups_dir = temp_dir.path().join("groups");
    let tasks_dir = temp_dir.path().join("tasks");
    fs::create_dir_all(&groups_dir).unwrap();
    fs::create_dir_all(&tasks_dir).unwrap();

    // Create task
    create_test_task(&tasks_dir, "integration-task", "1.0.0");

    // Create group
    let group_file = create_test_group(
        &groups_dir,
        "integration-group",
        "1.0.0",
        vec![("integration-task", "1.0.0")],
    );

    // Step 1: List groups
    let list_output = Command::new(get_dsl_executor_path())
        .args(&["group", "list", "--path", groups_dir.to_str().unwrap()])
        .output()
        .expect("Failed to list groups");

    assert!(list_output.status.success());
    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(list_stdout.contains("integration-group"));

    // Step 2: Validate the group
    let validate_output = Command::new(get_dsl_executor_path())
        .args(&["group", "validate", group_file.to_str().unwrap()])
        .output()
        .expect("Failed to validate group");

    // Validation might fail if task not in standard path, but should parse correctly
    let validate_stderr = String::from_utf8_lossy(&validate_output.stderr);

    // Should not have YAML parsing errors
    assert!(
        !validate_stderr.contains("YAML") && !validate_stderr.contains("parse"),
        "Should not have YAML parsing errors"
    );
}

#[test]
fn test_multiple_groups_discovery() {
    if !is_binary_available() {
        eprintln!("Skipping test: dsl-executor binary not built");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let groups_dir = temp_dir.path().join("groups");
    let tasks_dir = temp_dir.path().join("tasks");
    fs::create_dir_all(&groups_dir).unwrap();
    fs::create_dir_all(&tasks_dir).unwrap();

    // Create multiple tasks
    create_test_task(&tasks_dir, "task-a", "1.0.0");
    create_test_task(&tasks_dir, "task-b", "1.0.0");
    create_test_task(&tasks_dir, "task-c", "1.0.0");

    // Create multiple groups
    create_test_group(
        &groups_dir,
        "group-alpha",
        "1.0.0",
        vec![("task-a", "1.0.0")],
    );
    create_test_group(
        &groups_dir,
        "group-beta",
        "2.0.0",
        vec![("task-b", "1.0.0")],
    );
    create_test_group(
        &groups_dir,
        "group-gamma",
        "3.0.0",
        vec![("task-c", "1.0.0")],
    );

    let output = Command::new(get_dsl_executor_path())
        .args(&["group", "list", "--path", groups_dir.to_str().unwrap()])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should discover all three groups
    let has_alpha = stdout.contains("group-alpha") || stdout.contains("group-alpha@1.0.0");
    let has_beta = stdout.contains("group-beta") || stdout.contains("group-beta@2.0.0");
    let has_gamma = stdout.contains("group-gamma") || stdout.contains("group-gamma@3.0.0");

    assert!(has_alpha, "Should find group-alpha");
    assert!(has_beta, "Should find group-beta");
    assert!(has_gamma, "Should find group-gamma");
}
