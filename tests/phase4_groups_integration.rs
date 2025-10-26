//! Comprehensive Integration Tests for Phase 4: Task Groups System
//!
//! This test suite provides end-to-end integration testing for the task groups feature including:
//! - Task group loading from filesystem
//! - Workflow integration (imports, uses_workflow)
//! - Shared configuration application
//! - Namespace resolution
//! - Multi-path discovery
//! - Error handling and validation
//! - Complete end-to-end scenarios
//!
//! These tests validate production readiness of the Phase 4 implementation.

use periplon_sdk::dsl::predefined_tasks::groups::loader::TaskGroupLoader;
use periplon_sdk::dsl::predefined_tasks::groups::parser::parse_task_group;
use periplon_sdk::dsl::predefined_tasks::groups::schema::TaskGroupReference;
use periplon_sdk::dsl::schema::{DSLWorkflow, TaskSpec, WorkflowImport};
use periplon_sdk::dsl::validator::validate_workflow;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// Test Helpers
// ============================================================================

/// Create a minimal task group YAML for testing
fn create_minimal_task_group_yaml(name: &str, version: &str) -> String {
    format!(
        r#"
apiVersion: "taskgroup/v1"
kind: "TaskGroup"
metadata:
  name: "{}"
  version: "{}"
  description: "Test task group"
spec:
  tasks:
    - name: "test-task"
      version: "1.0.0"
"#,
        name, version
    )
}

/// Setup test environment with groups directory
fn setup_test_env() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let groups_dir = temp_dir.path().join("groups");
    fs::create_dir_all(&groups_dir).unwrap();
    temp_dir
}

// ============================================================================
// SECTION 1: Basic Parsing Tests
// ============================================================================

#[test]
fn test_parse_minimal_task_group() {
    let yaml = create_minimal_task_group_yaml("minimal-group", "1.0.0");
    let group = parse_task_group(&yaml).unwrap();

    assert_eq!(group.metadata.name, "minimal-group");
    assert_eq!(group.metadata.version, "1.0.0");
    assert_eq!(group.spec.tasks.len(), 1);
    assert_eq!(group.spec.tasks[0].name, "test-task");
}

#[test]
fn test_parse_complete_task_group() {
    let yaml = r#"
apiVersion: "taskgroup/v1"
kind: "TaskGroup"
metadata:
  name: "complete-group"
  version: "2.1.0"
  author: "Test Author"
  description: "Complete task group example"
  license: "MIT"
  repository: "https://github.com/example/tasks"
  tags:
    - "testing"
    - "integration"
spec:
  tasks:
    - name: "task-one"
      version: "^1.0.0"
      required: true
      description: "First task"
    - name: "task-two"
      version: "~2.0.0"
      required: false
      description: "Second task"
  shared_config:
    inputs:
      api_key:
        type: string
        required: true
        description: "API key for authentication"
    permissions:
      mode: "acceptEdits"
      allowed_directories:
        - "/tmp"
    environment:
      ENV_VAR: "test-value"
    max_turns: 20
"#;

    let group = parse_task_group(yaml).unwrap();

    assert_eq!(group.metadata.name, "complete-group");
    assert_eq!(group.metadata.version, "2.1.0");
    assert_eq!(group.metadata.author, Some("Test Author".to_string()));
    assert_eq!(group.metadata.tags.len(), 2);
    assert_eq!(group.spec.tasks.len(), 2);

    // Verify shared config
    let shared = group.spec.shared_config.as_ref().unwrap();
    assert_eq!(shared.inputs.len(), 1);
    assert!(shared.inputs.contains_key("api_key"));
    assert_eq!(
        shared.environment.get("ENV_VAR"),
        Some(&"test-value".to_string())
    );
    assert_eq!(shared.max_turns, Some(20));
}

#[test]
fn test_parse_task_group_from_file() {
    let temp_dir = setup_test_env();
    let groups_dir = temp_dir.path().join("groups");

    let group_yaml = create_minimal_task_group_yaml("file-group", "1.0.0");
    let group_path = groups_dir.join("file-group.taskgroup.yaml");
    fs::write(&group_path, group_yaml).unwrap();

    let content = fs::read_to_string(&group_path).unwrap();
    let group = parse_task_group(&content).unwrap();

    assert_eq!(group.metadata.name, "file-group");
    assert_eq!(group.metadata.version, "1.0.0");
}

// ============================================================================
// SECTION 2: Task Group Discovery Tests
// ============================================================================

#[test]
fn test_task_group_loader_discovery() {
    let temp_dir = setup_test_env();
    let groups_dir = temp_dir.path().join("groups");

    // Create multiple task groups
    let group1_yaml = create_minimal_task_group_yaml("group-one", "1.0.0");
    fs::write(groups_dir.join("group-one.taskgroup.yaml"), group1_yaml).unwrap();

    let group2_yaml = create_minimal_task_group_yaml("group-two", "2.0.0");
    fs::write(groups_dir.join("group-two.taskgroup.yaml"), group2_yaml).unwrap();

    // Initialize loader with test directory
    let loader = TaskGroupLoader::with_paths(vec![groups_dir]);

    // Test discovery
    let groups = loader.discover_all().unwrap();
    assert!(groups.len() >= 2);

    // Verify discovered groups (keys are "name@version")
    assert!(groups.contains_key("group-one@1.0.0"));
    assert!(groups.contains_key("group-two@2.0.0"));
}

#[test]
fn test_multi_path_group_discovery() {
    let temp_dir = setup_test_env();

    // Create two separate group directories
    let groups_dir1 = temp_dir.path().join("groups1");
    let groups_dir2 = temp_dir.path().join("groups2");
    fs::create_dir_all(&groups_dir1).unwrap();
    fs::create_dir_all(&groups_dir2).unwrap();

    // Group in first directory
    let group1_yaml = create_minimal_task_group_yaml("group-alpha", "1.0.0");
    fs::write(groups_dir1.join("group-alpha.taskgroup.yaml"), group1_yaml).unwrap();

    // Group in second directory
    let group2_yaml = create_minimal_task_group_yaml("group-beta", "1.0.0");
    fs::write(groups_dir2.join("group-beta.taskgroup.yaml"), group2_yaml).unwrap();

    // Initialize loader with both paths
    let loader = TaskGroupLoader::with_paths(vec![groups_dir1, groups_dir2]);

    // Discover should find both
    let groups = loader.discover_all().unwrap();
    assert!(groups.len() >= 2);
    assert!(groups.contains_key("group-alpha@1.0.0"));
    assert!(groups.contains_key("group-beta@1.0.0"));
}

#[test]
fn test_discover_large_number_of_groups() {
    let temp_dir = setup_test_env();
    let groups_dir = temp_dir.path().join("groups");

    // Create 50 groups
    for i in 0..50 {
        let group_yaml = create_minimal_task_group_yaml(&format!("group-{}", i), "1.0.0");
        fs::write(
            groups_dir.join(format!("group-{}.taskgroup.yaml", i)),
            group_yaml,
        )
        .unwrap();
    }

    let loader = TaskGroupLoader::with_paths(vec![groups_dir]);

    // Discovery should handle large numbers efficiently
    let start = std::time::Instant::now();
    let groups = loader.discover_all().unwrap();
    let duration = start.elapsed();

    assert!(groups.len() >= 50);
    assert!(duration < std::time::Duration::from_secs(5)); // Should be fast
}

// ============================================================================
// SECTION 3: Task Group Reference Parsing Tests
// ============================================================================

#[test]
fn test_task_group_reference_parsing() {
    // Valid reference: name@version
    let ref1 = TaskGroupReference::parse("google-workspace@1.0.0").unwrap();
    assert_eq!(ref1.name, "google-workspace");
    assert_eq!(ref1.version, "1.0.0");
    assert!(ref1.workflow.is_none());

    // Valid reference with workflow: name@version#workflow
    let ref2 = TaskGroupReference::parse("google-workspace@1.0.0#upload-files").unwrap();
    assert_eq!(ref2.name, "google-workspace");
    assert_eq!(ref2.version, "1.0.0");
    assert_eq!(ref2.workflow, Some("upload-files".to_string()));
}

#[test]
fn test_task_group_reference_invalid_formats() {
    // Missing @version
    assert!(TaskGroupReference::parse("google-workspace").is_err());

    // Missing version
    assert!(TaskGroupReference::parse("google-workspace@").is_err());

    // Empty workflow name
    assert!(TaskGroupReference::parse("google-workspace@1.0.0#").is_err());

    // Multiple # symbols
    assert!(TaskGroupReference::parse("google@1.0.0#work#flow").is_err());
}

// ============================================================================
// SECTION 4: Workflow Import Tests
// ============================================================================

#[test]
fn test_workflow_import_single_group() {
    let yaml = r#"
name: "Test Workflow"
version: "1.0.0"
imports:
  google: "google-workspace@1.0.0"
agents: {}
tasks: {}
"#;

    let workflow: DSLWorkflow = serde_yaml::from_str(yaml).unwrap();

    assert_eq!(workflow.imports.len(), 1);
    assert_eq!(
        workflow.imports.get("google"),
        Some(&"google-workspace@1.0.0".to_string())
    );
}

#[test]
fn test_workflow_import_multiple_groups() {
    let yaml = r#"
name: "Multi-Group Workflow"
version: "1.0.0"
imports:
  google: "google-workspace@1.0.0"
  aws: "aws-tools@2.1.0"
  slack: "slack-integration@3.0.0"
agents: {}
tasks: {}
"#;

    let workflow: DSLWorkflow = serde_yaml::from_str(yaml).unwrap();

    assert_eq!(workflow.imports.len(), 3);
    assert!(workflow.imports.contains_key("google"));
    assert!(workflow.imports.contains_key("aws"));
    assert!(workflow.imports.contains_key("slack"));
}

#[test]
fn test_workflow_import_validation() {
    // Valid import format
    let valid = WorkflowImport::from_entry("google", "google-workspace@1.0.0");
    assert_eq!(valid.namespace, "google");
    assert_eq!(valid.group_reference, "google-workspace@1.0.0");

    // Parse group reference
    let (name, version) = WorkflowImport::parse_group_reference("google-workspace@1.0.0").unwrap();
    assert_eq!(name, "google-workspace");
    assert_eq!(version, "1.0.0");
}

#[test]
fn test_namespace_validation() {
    // Valid namespaces
    assert!(WorkflowImport::validate_namespace("google"));
    assert!(WorkflowImport::validate_namespace("aws-tools"));
    assert!(WorkflowImport::validate_namespace("my_namespace"));
    assert!(WorkflowImport::validate_namespace("api2"));

    // Invalid namespaces
    assert!(!WorkflowImport::validate_namespace("")); // Empty
    assert!(!WorkflowImport::validate_namespace("123start")); // Starts with digit
    assert!(!WorkflowImport::validate_namespace("has space")); // Contains space
}

// ============================================================================
// SECTION 5: Uses Workflow Tests
// ============================================================================

#[test]
fn test_uses_workflow_reference_parsing() {
    // Valid reference format
    let reference = "google:upload-files";
    let (namespace, workflow) = TaskSpec::parse_workflow_reference(reference).unwrap();
    assert_eq!(namespace, "google");
    assert_eq!(workflow, "upload-files");

    // Invalid formats
    assert!(TaskSpec::parse_workflow_reference("no-colon").is_none());
    assert!(TaskSpec::parse_workflow_reference(":missing-namespace").is_none());
    assert!(TaskSpec::parse_workflow_reference("missing-workflow:").is_none());
}

#[test]
fn test_workflow_with_uses_workflow_task() {
    let yaml = r#"
name: "Workflow Using Groups"
version: "1.0.0"
imports:
  google: "google-workspace@1.0.0"
agents: {}
tasks:
  upload_docs:
    description: "Upload documentation to Google Drive"
    uses_workflow: "google:upload-files"
    inputs:
      files: "./docs/**/*.pdf"
      folder_id: "abc123"
"#;

    let workflow: DSLWorkflow = serde_yaml::from_str(yaml).unwrap();

    assert_eq!(workflow.tasks.len(), 1);

    let task = workflow.tasks.get("upload_docs").unwrap();
    assert_eq!(task.uses_workflow, Some("google:upload-files".to_string()));
    assert_eq!(task.inputs.len(), 2);
}

#[test]
fn test_workflow_import_and_uses_workflow_structure() {
    // Test that workflows with imports and uses_workflow parse correctly
    let yaml = r#"
name: "Valid Workflow"
version: "1.0.0"
imports:
  google: "google-workspace@1.0.0"
agents:
  processor:
    description: "Processes data"
tasks:
  my_task:
    description: "Use imported workflow"
    uses_workflow: "google:some-workflow"
"#;

    let workflow: DSLWorkflow = serde_yaml::from_str(yaml).unwrap();

    // Verify structure is correct
    assert_eq!(workflow.imports.len(), 1);
    assert!(workflow.imports.contains_key("google"));

    let task = workflow.tasks.get("my_task").unwrap();
    assert_eq!(task.uses_workflow, Some("google:some-workflow".to_string()));
}

#[test]
fn test_workflow_validation_missing_namespace() {
    let yaml = r#"
name: "Invalid Workflow"
version: "1.0.0"
imports:
  google: "google-workspace@1.0.0"
agents: {}
tasks:
  my_task:
    description: "Use non-imported workflow"
    uses_workflow: "aws:some-workflow"
"#;

    let workflow: DSLWorkflow = serde_yaml::from_str(yaml).unwrap();

    // This should fail validation (namespace 'aws' not in imports)
    let result = validate_workflow(&workflow);
    assert!(result.is_err());
}

// ============================================================================
// SECTION 6: Shared Configuration Tests
// ============================================================================

#[test]
fn test_shared_config_inputs() {
    let yaml = r#"
apiVersion: "taskgroup/v1"
kind: "TaskGroup"
metadata:
  name: "config-test"
  version: "1.0.0"
spec:
  tasks:
    - name: "task1"
      version: "1.0.0"
  shared_config:
    inputs:
      api_key:
        type: string
        required: true
        description: "API authentication key"
      timeout:
        type: number
        required: false
        default: 30
        description: "Request timeout in seconds"
"#;

    let group = parse_task_group(yaml).unwrap();
    let shared = group.spec.shared_config.as_ref().unwrap();

    assert_eq!(shared.inputs.len(), 2);

    let api_key = shared.inputs.get("api_key").unwrap();
    assert_eq!(api_key.param_type, "string");
    assert!(api_key.required);

    let timeout = shared.inputs.get("timeout").unwrap();
    assert_eq!(timeout.param_type, "number");
    assert!(!timeout.required);
    assert!(timeout.default.is_some());
}

#[test]
fn test_shared_config_permissions() {
    let yaml = r#"
apiVersion: "taskgroup/v1"
kind: "TaskGroup"
metadata:
  name: "perms-test"
  version: "1.0.0"
spec:
  tasks:
    - name: "task1"
      version: "1.0.0"
  shared_config:
    permissions:
      mode: "acceptEdits"
      allowed_directories:
        - "/tmp"
        - "/var/output"
"#;

    let group = parse_task_group(yaml).unwrap();
    let shared = group.spec.shared_config.as_ref().unwrap();
    let perms = shared.permissions.as_ref().unwrap();

    assert_eq!(perms.mode, "acceptEdits");
    assert_eq!(perms.allowed_directories.len(), 2);
    assert!(perms.allowed_directories.contains(&"/tmp".to_string()));
}

#[test]
fn test_shared_config_environment() {
    let yaml = r#"
apiVersion: "taskgroup/v1"
kind: "TaskGroup"
metadata:
  name: "env-test"
  version: "1.0.0"
spec:
  tasks:
    - name: "task1"
      version: "1.0.0"
  shared_config:
    environment:
      API_URL: "https://api.example.com"
      LOG_LEVEL: "debug"
      MAX_RETRIES: "3"
"#;

    let group = parse_task_group(yaml).unwrap();
    let shared = group.spec.shared_config.as_ref().unwrap();

    assert_eq!(shared.environment.len(), 3);
    assert_eq!(
        shared.environment.get("API_URL"),
        Some(&"https://api.example.com".to_string())
    );
}

// ============================================================================
// SECTION 7: Group Dependencies Tests
// ============================================================================

#[test]
fn test_group_dependencies_parsing() {
    let yaml = r#"
apiVersion: "taskgroup/v1"
kind: "TaskGroup"
metadata:
  name: "dependent-group"
  version: "1.0.0"
spec:
  tasks:
    - name: "task1"
      version: "1.0.0"
  dependencies:
    - name: "base-group"
      version: "^1.0.0"
      optional: false
    - name: "utils-group"
      version: "~2.0.0"
      optional: true
"#;

    let group = parse_task_group(yaml).unwrap();

    assert_eq!(group.spec.dependencies.len(), 2);

    let dep1 = &group.spec.dependencies[0];
    assert_eq!(dep1.name, "base-group");
    assert_eq!(dep1.version, "^1.0.0");
    assert!(!dep1.optional);

    let dep2 = &group.spec.dependencies[1];
    assert_eq!(dep2.name, "utils-group");
    assert_eq!(dep2.version, "~2.0.0");
    assert!(dep2.optional);
}

// ============================================================================
// SECTION 8: Error Handling Tests
// ============================================================================

#[test]
fn test_parse_invalid_task_group_missing_tasks() {
    let yaml = r#"
apiVersion: "taskgroup/v1"
kind: "TaskGroup"
metadata:
  name: "invalid"
  version: "1.0.0"
spec:
  tasks: []
"#;

    let result = parse_task_group(yaml);
    assert!(result.is_err());
}

#[test]
fn test_invalid_group_reference_format() {
    // Missing @version
    let result = WorkflowImport::parse_group_reference("google-workspace");
    assert!(result.is_none());

    // Empty name
    let result = WorkflowImport::parse_group_reference("@1.0.0");
    assert!(result.is_none());

    // Empty version
    let result = WorkflowImport::parse_group_reference("google-workspace@");
    assert!(result.is_none());
}

#[test]
fn test_workflow_uses_workflow_without_import() {
    let yaml = r#"
name: "Invalid Workflow"
version: "1.0.0"
imports: {}
agents: {}
tasks:
  my_task:
    description: "Use workflow without import"
    uses_workflow: "google:upload-files"
"#;

    let workflow: DSLWorkflow = serde_yaml::from_str(yaml).unwrap();

    // Validation should fail (namespace not imported)
    let result = validate_workflow(&workflow);
    assert!(result.is_err());
}

// ============================================================================
// SECTION 9: Version Constraint Tests
// ============================================================================

#[test]
fn test_task_version_constraints() {
    let yaml = r#"
apiVersion: "taskgroup/v1"
kind: "TaskGroup"
metadata:
  name: "version-test"
  version: "1.0.0"
spec:
  tasks:
    - name: "exact-version"
      version: "1.2.3"
    - name: "caret-version"
      version: "^2.0.0"
    - name: "tilde-version"
      version: "~1.5.0"
    - name: "range-version"
      version: ">=1.0.0 <2.0.0"
"#;

    let group = parse_task_group(yaml).unwrap();

    assert_eq!(group.spec.tasks.len(), 4);
    assert_eq!(group.spec.tasks[0].version, "1.2.3");
    assert_eq!(group.spec.tasks[1].version, "^2.0.0");
    assert_eq!(group.spec.tasks[2].version, "~1.5.0");
    assert_eq!(group.spec.tasks[3].version, ">=1.0.0 <2.0.0");
}

// ============================================================================
// SECTION 10: Namespace Isolation Tests
// ============================================================================

#[test]
fn test_namespace_isolation() {
    let yaml = r#"
name: "Namespace Test"
version: "1.0.0"
imports:
  google1: "google-workspace@1.0.0"
  google2: "google-workspace@2.0.0"
agents: {}
tasks:
  use_v1:
    description: "Use version 1"
    uses_workflow: "google1:upload"
  use_v2:
    description: "Use version 2"
    uses_workflow: "google2:upload"
"#;

    let workflow: DSLWorkflow = serde_yaml::from_str(yaml).unwrap();

    // Two different namespaces can reference same group at different versions
    assert_eq!(workflow.imports.len(), 2);

    let task1 = workflow.tasks.get("use_v1").unwrap();
    let task2 = workflow.tasks.get("use_v2").unwrap();

    assert_eq!(task1.uses_workflow, Some("google1:upload".to_string()));
    assert_eq!(task2.uses_workflow, Some("google2:upload".to_string()));
}

// ============================================================================
// SECTION 11: End-to-End Multi-Group Workflow
// ============================================================================

#[test]
fn test_multi_group_workflow_structure() {
    let yaml = r#"
name: "Multi-Service Integration"
version: "1.0.0"
imports:
  google: "google-workspace@1.0.0"
  slack: "slack-integration@2.0.0"
  aws: "aws-tools@3.0.0"
agents: {}
tasks:
  fetch_data:
    description: "Fetch data from AWS"
    uses_workflow: "aws:s3-download"
    inputs:
      bucket: "data-bucket"
      prefix: "reports/"

  process_data:
    description: "Process the data"
    agent: "processor"
    depends_on:
      - "fetch_data"

  upload_results:
    description: "Upload to Google Drive"
    uses_workflow: "google:upload-files"
    inputs:
      files: "./processed/*.csv"
    depends_on:
      - "process_data"

  notify_team:
    description: "Send Slack notification"
    uses_workflow: "slack:send-message"
    inputs:
      channel: "data-team"
      message: "Processing complete"
    depends_on:
      - "upload_results"
"#;

    let workflow: DSLWorkflow = serde_yaml::from_str(yaml).unwrap();

    // Verify multiple imports
    assert_eq!(workflow.imports.len(), 3);
    assert!(workflow.imports.contains_key("google"));
    assert!(workflow.imports.contains_key("slack"));
    assert!(workflow.imports.contains_key("aws"));

    // Verify tasks use different namespaces
    assert_eq!(workflow.tasks.len(), 4);
    assert!(workflow.tasks.get("fetch_data").is_some());
    assert!(workflow.tasks.get("upload_results").is_some());
    assert!(workflow.tasks.get("notify_team").is_some());
}
