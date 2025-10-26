//! Comprehensive tests for Phase 1: Local Predefined Tasks
//!
//! Tests cover:
//! - Task schema parsing
//! - Local discovery
//! - Task reference resolution
//! - Input/output validation

use periplon_sdk::dsl::predefined_tasks::schema::{InputValidation, TaskReference};
use periplon_sdk::dsl::predefined_tasks::{
    parse_predefined_task, AgentTemplate, PredefinedTask, PredefinedTaskInputSpec,
    PredefinedTaskMetadata, PredefinedTaskOutputSpec, PredefinedTaskSpec, TaskApiVersion, TaskKind,
    TaskLoader, TaskResolver,
};
use periplon_sdk::dsl::schema::{InputSpec, OutputDataSource, OutputSpec, PermissionsSpec};
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

// ============================================================================
// SECTION 1: Task Schema Parsing Tests
// ============================================================================

#[test]
fn test_parse_minimal_task() {
    let yaml = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "minimal-task"
  version: "1.0.0"
spec:
  agent_template:
    description: "Minimal agent"
"#;

    let task = parse_predefined_task(yaml).unwrap();
    assert_eq!(task.metadata.name, "minimal-task");
    assert_eq!(task.metadata.version, "1.0.0");
    assert_eq!(task.spec.agent_template.description, "Minimal agent");
}

#[test]
fn test_parse_complete_task() {
    let yaml = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "complete-task"
  version: "2.1.0"
  author: "Test Author"
  description: "A complete task definition"
  license: "MIT"
  repository: "https://github.com/example/tasks"
  tags: ["integration", "google-drive", "upload"]
spec:
  agent_template:
    description: "Upload file ${input.file_path} to Google Drive"
    model: "claude-sonnet-4-5"
    system_prompt: "You are a Google Drive integration specialist."
    tools: ["Read", "Write", "WebFetch"]
    permissions:
      mode: "acceptEdits"
      allowed_directories: ["/tmp"]
    max_turns: 20
  inputs:
    file_path:
      type: string
      required: true
      description: "Path to file to upload"
      validation:
        pattern: "^[a-zA-Z0-9/_.-]+$"
        min_length: 1
        max_length: 255
    drive_folder:
      type: string
      required: false
      default: "root"
      description: "Target folder in Google Drive"
    overwrite:
      type: boolean
      required: false
      default: false
  outputs:
    file_id:
      type: string
      description: "Google Drive file ID"
      source:
        type: state
        key: "uploaded_file_id"
    share_link:
      type: string
      description: "Shareable link"
      source:
        type: state
        key: "share_link"
  examples:
    - name: "basic_upload"
      description: "Upload a PDF file"
      inputs:
        file_path: "report.pdf"
        drive_folder: "Documents"
"#;

    let task = parse_predefined_task(yaml).unwrap();

    // Validate metadata
    assert_eq!(task.metadata.name, "complete-task");
    assert_eq!(task.metadata.version, "2.1.0");
    assert_eq!(task.metadata.author, Some("Test Author".to_string()));
    assert_eq!(
        task.metadata.description,
        Some("A complete task definition".to_string())
    );
    assert_eq!(task.metadata.license, Some("MIT".to_string()));
    assert_eq!(task.metadata.tags.len(), 3);

    // Validate agent template
    assert!(task
        .spec
        .agent_template
        .description
        .contains("${input.file_path}"));
    assert_eq!(
        task.spec.agent_template.model,
        Some("claude-sonnet-4-5".to_string())
    );
    assert_eq!(task.spec.agent_template.tools.len(), 3);
    assert_eq!(task.spec.agent_template.permissions.mode, "acceptEdits");
    assert_eq!(task.spec.agent_template.max_turns, Some(20));

    // Validate inputs
    assert_eq!(task.spec.inputs.len(), 3);
    assert!(task.spec.inputs["file_path"].base.required);
    assert!(!task.spec.inputs["drive_folder"].base.required);

    let validation = task.spec.inputs["file_path"].validation.as_ref().unwrap();
    assert!(validation.pattern.is_some());
    assert_eq!(validation.min_length, Some(1));
    assert_eq!(validation.max_length, Some(255));

    // Validate outputs
    assert_eq!(task.spec.outputs.len(), 2);
    assert!(task.spec.outputs.contains_key("file_id"));

    // Validate examples
    assert_eq!(task.spec.examples.len(), 1);
    assert_eq!(task.spec.examples[0].name, "basic_upload");
}

#[test]
fn test_parse_task_with_all_input_types() {
    let yaml = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "input-types-task"
  version: "1.0.0"
spec:
  agent_template:
    description: "Test all input types"
  inputs:
    str_param:
      type: string
      required: true
    num_param:
      type: number
      required: true
    bool_param:
      type: boolean
      required: false
      default: true
    obj_param:
      type: object
      required: false
    arr_param:
      type: array
      required: false
    secret_param:
      type: secret
      required: false
"#;

    let task = parse_predefined_task(yaml).unwrap();
    assert_eq!(task.spec.inputs.len(), 6);
    assert_eq!(task.spec.inputs["str_param"].base.param_type, "string");
    assert_eq!(task.spec.inputs["num_param"].base.param_type, "number");
    assert_eq!(task.spec.inputs["bool_param"].base.param_type, "boolean");
    assert_eq!(task.spec.inputs["obj_param"].base.param_type, "object");
    assert_eq!(task.spec.inputs["arr_param"].base.param_type, "array");
    assert_eq!(task.spec.inputs["secret_param"].base.param_type, "secret");
}

#[test]
fn test_parse_task_with_validation_rules() {
    let yaml = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "validation-task"
  version: "1.0.0"
spec:
  agent_template:
    description: "Test validation"
  inputs:
    email:
      type: string
      required: true
      validation:
        pattern: "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"
    age:
      type: number
      required: true
      validation:
        min: 0
        max: 120
    username:
      type: string
      required: true
      validation:
        min_length: 3
        max_length: 20
    status:
      type: string
      required: true
      validation:
        allowed_values: ["active", "inactive", "pending"]
"#;

    let task = parse_predefined_task(yaml).unwrap();

    // Email pattern validation
    let email_validation = task.spec.inputs["email"].validation.as_ref().unwrap();
    assert!(email_validation.pattern.is_some());

    // Age min/max validation
    let age_validation = task.spec.inputs["age"].validation.as_ref().unwrap();
    assert_eq!(age_validation.min, Some(0.0));
    assert_eq!(age_validation.max, Some(120.0));

    // Username length validation
    let username_validation = task.spec.inputs["username"].validation.as_ref().unwrap();
    assert_eq!(username_validation.min_length, Some(3));
    assert_eq!(username_validation.max_length, Some(20));

    // Status enum validation
    let status_validation = task.spec.inputs["status"].validation.as_ref().unwrap();
    assert_eq!(status_validation.allowed_values.len(), 3);
}

#[test]
fn test_parse_task_missing_required_fields() {
    // Missing metadata.name
    let yaml_no_name = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  version: "1.0.0"
spec:
  agent_template:
    description: "Test"
"#;
    assert!(parse_predefined_task(yaml_no_name).is_err());

    // Missing metadata.version
    let yaml_no_version = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "test-task"
spec:
  agent_template:
    description: "Test"
"#;
    assert!(parse_predefined_task(yaml_no_version).is_err());

    // Missing agent_template.description
    let yaml_no_description = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "test-task"
  version: "1.0.0"
spec:
  agent_template:
    tools: ["Read"]
"#;
    assert!(parse_predefined_task(yaml_no_description).is_err());
}

#[test]
fn test_parse_task_invalid_task_name() {
    // Uppercase not allowed
    let yaml_uppercase = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "InvalidName"
  version: "1.0.0"
spec:
  agent_template:
    description: "Test"
"#;
    let result = parse_predefined_task(yaml_uppercase);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("lowercase"));

    // Underscores not allowed
    let yaml_underscore = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "my_task"
  version: "1.0.0"
spec:
  agent_template:
    description: "Test"
"#;
    let result = parse_predefined_task(yaml_underscore);
    assert!(result.is_err());

    // Starting with hyphen not allowed
    let yaml_start_hyphen = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "-task"
  version: "1.0.0"
spec:
  agent_template:
    description: "Test"
"#;
    let result = parse_predefined_task(yaml_start_hyphen);
    assert!(result.is_err());
}

#[test]
fn test_parse_task_invalid_input_type() {
    let yaml = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "test-task"
  version: "1.0.0"
spec:
  agent_template:
    description: "Test"
  inputs:
    param:
      type: invalid_type
      required: true
"#;

    let result = parse_predefined_task(yaml);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Type must be one of"));
}

#[test]
fn test_parse_task_invalid_validation_rules() {
    // Pattern validation on number (should fail)
    let yaml_pattern_on_number = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "test-task"
  version: "1.0.0"
spec:
  agent_template:
    description: "Test"
  inputs:
    count:
      type: number
      required: true
      validation:
        pattern: "^[0-9]+$"
"#;
    let result = parse_predefined_task(yaml_pattern_on_number);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Pattern validation only applies to string"));

    // Min/max validation on string (should fail)
    let yaml_minmax_on_string = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "test-task"
  version: "1.0.0"
spec:
  agent_template:
    description: "Test"
  inputs:
    name:
      type: string
      required: true
      validation:
        min: 0
        max: 100
"#;
    let result = parse_predefined_task(yaml_minmax_on_string);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Min/max validation only applies to number"));
}

// ============================================================================
// SECTION 2: Task Reference Parsing Tests
// ============================================================================

#[test]
fn test_task_reference_parse_valid() {
    let ref1 = TaskReference::parse("google-drive-upload@1.2.0").unwrap();
    assert_eq!(ref1.name, "google-drive-upload");
    assert_eq!(ref1.version, "1.2.0");

    let ref2 = TaskReference::parse("simple-task@0.1.0").unwrap();
    assert_eq!(ref2.name, "simple-task");
    assert_eq!(ref2.version, "0.1.0");
}

#[test]
fn test_task_reference_parse_with_whitespace() {
    let ref1 = TaskReference::parse("  my-task  @  1.0.0  ").unwrap();
    assert_eq!(ref1.name, "my-task");
    assert_eq!(ref1.version, "1.0.0");
}

#[test]
fn test_task_reference_parse_invalid() {
    // No @ symbol
    assert!(TaskReference::parse("my-task-1.0.0").is_err());

    // Multiple @ symbols
    assert!(TaskReference::parse("my-task@1.0@0").is_err());

    // Empty name
    assert!(TaskReference::parse("@1.0.0").is_err());

    // Empty version
    assert!(TaskReference::parse("my-task@").is_err());

    // Empty string
    assert!(TaskReference::parse("").is_err());
}

#[test]
fn test_task_reference_to_string() {
    let task_ref = TaskReference {
        name: "google-drive-upload".to_string(),
        version: "2.1.3".to_string(),
    };
    assert_eq!(task_ref.to_string(), "google-drive-upload@2.1.3");
}

// ============================================================================
// SECTION 3: Local Discovery Tests
// ============================================================================

fn create_task_file(dir: &std::path::Path, name: &str, version: &str) -> std::path::PathBuf {
    let content = format!(
        r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "{}"
  version: "{}"
  description: "Test task {}"
spec:
  agent_template:
    description: "Test agent for {}"
    tools: ["Read", "Write"]
"#,
        name, version, name, name
    );

    let file_path = dir.join(format!("{}.task.yaml", name));
    fs::write(&file_path, content).unwrap();
    file_path
}

#[test]
fn test_loader_discover_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let loader = TaskLoader::with_paths(vec![temp_dir.path().to_path_buf()]);

    let discovered = loader.discover_all().unwrap();
    assert_eq!(discovered.len(), 0);
}

#[test]
fn test_loader_discover_single_task() {
    let temp_dir = TempDir::new().unwrap();
    create_task_file(temp_dir.path(), "my-task", "1.0.0");

    let loader = TaskLoader::with_paths(vec![temp_dir.path().to_path_buf()]);
    let discovered = loader.discover_all().unwrap();

    assert_eq!(discovered.len(), 1);
    assert!(discovered.contains_key("my-task@1.0.0"));
}

#[test]
fn test_loader_discover_multiple_tasks() {
    let temp_dir = TempDir::new().unwrap();
    create_task_file(temp_dir.path(), "task-a", "1.0.0");
    create_task_file(temp_dir.path(), "task-b", "2.1.0");
    create_task_file(temp_dir.path(), "task-c", "0.5.0");

    let loader = TaskLoader::with_paths(vec![temp_dir.path().to_path_buf()]);
    let discovered = loader.discover_all().unwrap();

    assert_eq!(discovered.len(), 3);
    assert!(discovered.contains_key("task-a@1.0.0"));
    assert!(discovered.contains_key("task-b@2.1.0"));
    assert!(discovered.contains_key("task-c@0.5.0"));
}

#[test]
fn test_loader_discover_with_non_task_files() {
    let temp_dir = TempDir::new().unwrap();
    create_task_file(temp_dir.path(), "my-task", "1.0.0");

    // Create non-task files (should be ignored)
    fs::write(temp_dir.path().join("README.md"), "# Tasks").unwrap();
    fs::write(temp_dir.path().join("config.yaml"), "key: value").unwrap();
    fs::write(temp_dir.path().join("script.sh"), "#!/bin/bash").unwrap();

    let loader = TaskLoader::with_paths(vec![temp_dir.path().to_path_buf()]);
    let discovered = loader.discover_all().unwrap();

    // Only .task.yaml files should be discovered
    assert_eq!(discovered.len(), 1);
    assert!(discovered.contains_key("my-task@1.0.0"));
}

#[test]
fn test_loader_priority_multiple_directories() {
    let temp_dir1 = TempDir::new().unwrap();
    let temp_dir2 = TempDir::new().unwrap();

    // Create same task with different versions
    create_task_file(temp_dir1.path(), "my-task", "1.0.0");
    create_task_file(temp_dir2.path(), "my-task", "2.0.0");

    // dir2 has higher priority (added last)
    let loader = TaskLoader::with_paths(vec![
        temp_dir1.path().to_path_buf(),
        temp_dir2.path().to_path_buf(),
    ]);

    let discovered = loader.discover_all().unwrap();

    // Should find both versions as they have different versions
    // Later path takes priority during discovery
    assert!(discovered.contains_key("my-task@1.0.0") || discovered.contains_key("my-task@2.0.0"));
}

#[test]
fn test_loader_load_specific_task() {
    let temp_dir = TempDir::new().unwrap();
    create_task_file(temp_dir.path(), "my-task", "1.0.0");

    let mut loader = TaskLoader::with_paths(vec![temp_dir.path().to_path_buf()]);
    let task_ref = TaskReference::parse("my-task@1.0.0").unwrap();

    let task = loader.load(&task_ref).unwrap();
    assert_eq!(task.metadata.name, "my-task");
    assert_eq!(task.metadata.version, "1.0.0");
}

#[test]
fn test_loader_load_task_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let mut loader = TaskLoader::with_paths(vec![temp_dir.path().to_path_buf()]);
    let task_ref = TaskReference::parse("nonexistent@1.0.0").unwrap();

    let result = loader.load(&task_ref);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Task not found"));
}

#[test]
fn test_loader_caching() {
    let temp_dir = TempDir::new().unwrap();
    create_task_file(temp_dir.path(), "my-task", "1.0.0");

    let mut loader = TaskLoader::with_paths(vec![temp_dir.path().to_path_buf()]);
    let task_ref = TaskReference::parse("my-task@1.0.0").unwrap();

    // Load once
    loader.load(&task_ref).unwrap();
    assert_eq!(loader.cached_tasks().len(), 1);
    assert!(loader.cached_tasks().contains(&"my-task@1.0.0".to_string()));

    // Load again (should use cache)
    loader.load(&task_ref).unwrap();
    assert_eq!(loader.cached_tasks().len(), 1);

    // Clear cache
    loader.clear_cache();
    assert_eq!(loader.cached_tasks().len(), 0);
}

#[test]
fn test_loader_nonexistent_directory() {
    let loader = TaskLoader::with_paths(vec![std::path::PathBuf::from("/nonexistent/path")]);

    // Should not error, just return empty results
    let discovered = loader.discover_all().unwrap();
    assert_eq!(discovered.len(), 0);
}

// ============================================================================
// SECTION 4: Task Reference Resolution Tests
// ============================================================================

#[allow(dead_code)]
fn create_test_predefined_task() -> PredefinedTask {
    PredefinedTask {
        api_version: TaskApiVersion::V1,
        kind: TaskKind::PredefinedTask,
        metadata: PredefinedTaskMetadata {
            name: "test-task".to_string(),
            version: "1.0.0".to_string(),
            author: Some("Test Author".to_string()),
            description: Some("Test task".to_string()),
            license: None,
            repository: None,
            tags: vec![],
        },
        spec: PredefinedTaskSpec {
            agent_template: AgentTemplate {
                description: "Process file ${input.file_path} with mode ${input.mode}".to_string(),
                model: Some("claude-sonnet-4-5".to_string()),
                system_prompt: Some("You are a file processor.".to_string()),
                tools: vec!["Read".to_string(), "Write".to_string()],
                permissions: PermissionsSpec {
                    mode: "acceptEdits".to_string(),
                    allowed_directories: vec![],
                },
                max_turns: Some(10),
            },
            inputs: {
                let mut inputs = HashMap::new();
                inputs.insert(
                    "file_path".to_string(),
                    PredefinedTaskInputSpec {
                        base: InputSpec {
                            param_type: "string".to_string(),
                            required: true,
                            default: None,
                            description: Some("Path to file".to_string()),
                        },
                        validation: Some(InputValidation {
                            pattern: Some("^[a-zA-Z0-9/_.-]+$".to_string()),
                            min: None,
                            max: None,
                            min_length: Some(1),
                            max_length: Some(255),
                            allowed_values: vec![],
                        }),
                        source: None,
                    },
                );
                inputs.insert(
                    "mode".to_string(),
                    PredefinedTaskInputSpec {
                        base: InputSpec {
                            param_type: "string".to_string(),
                            required: false,
                            default: Some(serde_json::json!("normal")),
                            description: Some("Processing mode".to_string()),
                        },
                        validation: Some(InputValidation {
                            pattern: None,
                            min: None,
                            max: None,
                            min_length: None,
                            max_length: None,
                            allowed_values: vec![
                                serde_json::json!("normal"),
                                serde_json::json!("fast"),
                                serde_json::json!("thorough"),
                            ],
                        }),
                        source: None,
                    },
                );
                inputs
            },
            outputs: {
                let mut outputs = HashMap::new();
                outputs.insert(
                    "result".to_string(),
                    PredefinedTaskOutputSpec {
                        base: OutputSpec {
                            description: Some("Processing result".to_string()),
                            source: OutputDataSource::State {
                                key: "result_value".to_string(),
                            },
                        },
                        output_type: Some("string".to_string()),
                    },
                );
                outputs
            },
            dependencies: vec![],
            examples: vec![],
        },
    }
}

#[test]
fn test_resolver_basic_resolution() {
    let temp_dir = TempDir::new().unwrap();
    create_task_file(temp_dir.path(), "test-task", "1.0.0");

    let loader = TaskLoader::with_paths(vec![temp_dir.path().to_path_buf()]);
    let mut resolver = TaskResolver::with_loader(loader);

    // No inputs needed for the basic task
    let inputs = HashMap::new();

    let result = resolver.resolve("test-task@1.0.0", &inputs, &HashMap::new());
    assert!(result.is_ok());

    let (agent_id, agent_spec, _task_spec) = result.unwrap();
    assert_eq!(agent_id, "test-task_1_0_0");
    // The description should match what's in the task file
    assert!(agent_spec.description.contains("Test agent for test-task"));
}

#[test]
fn test_resolver_template_variable_substitution() {
    let temp_dir = TempDir::new().unwrap();

    // Create a task with template variables
    let task_yaml = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "template-task"
  version: "1.0.0"
spec:
  agent_template:
    description: "Upload ${input.filename} to ${input.destination}"
    tools: ["Write"]
  inputs:
    filename:
      type: string
      required: true
    destination:
      type: string
      required: true
"#;
    fs::write(temp_dir.path().join("template-task.task.yaml"), task_yaml).unwrap();

    let loader = TaskLoader::with_paths(vec![temp_dir.path().to_path_buf()]);
    let mut resolver = TaskResolver::with_loader(loader);

    let mut inputs = HashMap::new();
    inputs.insert("filename".to_string(), serde_json::json!("report.pdf"));
    inputs.insert("destination".to_string(), serde_json::json!("Google Drive"));

    let result = resolver.resolve("template-task@1.0.0", &inputs, &HashMap::new());
    assert!(result.is_ok());

    let (_agent_id, agent_spec, _task_spec) = result.unwrap();
    assert_eq!(agent_spec.description, "Upload report.pdf to Google Drive");
}

#[test]
fn test_resolver_default_values() {
    let temp_dir = TempDir::new().unwrap();

    let task_yaml = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "default-task"
  version: "1.0.0"
spec:
  agent_template:
    description: "Process with mode ${input.mode}"
    tools: ["Read"]
  inputs:
    mode:
      type: string
      required: false
      default: "standard"
"#;
    fs::write(temp_dir.path().join("default-task.task.yaml"), task_yaml).unwrap();

    let loader = TaskLoader::with_paths(vec![temp_dir.path().to_path_buf()]);
    let mut resolver = TaskResolver::with_loader(loader);

    // Don't provide the optional input - should use default
    let inputs = HashMap::new();

    let result = resolver.resolve("default-task@1.0.0", &inputs, &HashMap::new());
    assert!(result.is_ok());

    let (_agent_id, agent_spec, _task_spec) = result.unwrap();
    assert_eq!(agent_spec.description, "Process with mode standard");
}

// ============================================================================
// SECTION 5: Input/Output Validation Tests
// ============================================================================

#[test]
fn test_validation_missing_required_input() {
    let temp_dir = TempDir::new().unwrap();

    let task_yaml = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "required-task"
  version: "1.0.0"
spec:
  agent_template:
    description: "Test"
    tools: ["Read"]
  inputs:
    required_param:
      type: string
      required: true
"#;
    fs::write(temp_dir.path().join("required-task.task.yaml"), task_yaml).unwrap();

    let loader = TaskLoader::with_paths(vec![temp_dir.path().to_path_buf()]);
    let mut resolver = TaskResolver::with_loader(loader);

    // Missing required input
    let inputs = HashMap::new();

    let result = resolver.resolve("required-task@1.0.0", &inputs, &HashMap::new());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Missing required input"));
}

#[test]
fn test_validation_type_mismatch() {
    let temp_dir = TempDir::new().unwrap();

    let task_yaml = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "type-task"
  version: "1.0.0"
spec:
  agent_template:
    description: "Test"
    tools: ["Read"]
  inputs:
    count:
      type: number
      required: true
"#;
    fs::write(temp_dir.path().join("type-task.task.yaml"), task_yaml).unwrap();

    let loader = TaskLoader::with_paths(vec![temp_dir.path().to_path_buf()]);
    let mut resolver = TaskResolver::with_loader(loader);

    // Provide string instead of number
    let mut inputs = HashMap::new();
    inputs.insert("count".to_string(), serde_json::json!("not a number"));

    let result = resolver.resolve("type-task@1.0.0", &inputs, &HashMap::new());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid type"));
}

#[test]
fn test_validation_string_pattern() {
    let temp_dir = TempDir::new().unwrap();

    let task_yaml = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "pattern-task"
  version: "1.0.0"
spec:
  agent_template:
    description: "Test"
    tools: ["Read"]
  inputs:
    email:
      type: string
      required: true
      validation:
        pattern: "^[a-z]+@[a-z]+\\.[a-z]+$"
"#;
    fs::write(temp_dir.path().join("pattern-task.task.yaml"), task_yaml).unwrap();

    let loader = TaskLoader::with_paths(vec![temp_dir.path().to_path_buf()]);
    let mut resolver = TaskResolver::with_loader(loader);

    // Valid email
    let mut inputs_valid = HashMap::new();
    inputs_valid.insert("email".to_string(), serde_json::json!("test@example.com"));
    assert!(resolver
        .resolve("pattern-task@1.0.0", &inputs_valid, &HashMap::new())
        .is_ok());

    // Invalid email
    let mut inputs_invalid = HashMap::new();
    inputs_invalid.insert("email".to_string(), serde_json::json!("not-an-email"));
    let result = resolver.resolve("pattern-task@1.0.0", &inputs_invalid, &HashMap::new());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("does not match pattern"));
}

#[test]
fn test_validation_number_range() {
    let temp_dir = TempDir::new().unwrap();

    let task_yaml = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "range-task"
  version: "1.0.0"
spec:
  agent_template:
    description: "Test"
    tools: ["Read"]
  inputs:
    age:
      type: number
      required: true
      validation:
        min: 0
        max: 120
"#;
    fs::write(temp_dir.path().join("range-task.task.yaml"), task_yaml).unwrap();

    let loader = TaskLoader::with_paths(vec![temp_dir.path().to_path_buf()]);
    let mut resolver = TaskResolver::with_loader(loader);

    // Valid age
    let mut inputs_valid = HashMap::new();
    inputs_valid.insert("age".to_string(), serde_json::json!(25));
    assert!(resolver
        .resolve("range-task@1.0.0", &inputs_valid, &HashMap::new())
        .is_ok());

    // Too low
    let mut inputs_low = HashMap::new();
    inputs_low.insert("age".to_string(), serde_json::json!(-5));
    let result = resolver.resolve("range-task@1.0.0", &inputs_low, &HashMap::new());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("less than minimum"));

    // Too high
    let mut inputs_high = HashMap::new();
    inputs_high.insert("age".to_string(), serde_json::json!(150));
    let result = resolver.resolve("range-task@1.0.0", &inputs_high, &HashMap::new());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("exceeds maximum"));
}

#[test]
fn test_validation_string_length() {
    let temp_dir = TempDir::new().unwrap();

    let task_yaml = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "length-task"
  version: "1.0.0"
spec:
  agent_template:
    description: "Test"
    tools: ["Read"]
  inputs:
    username:
      type: string
      required: true
      validation:
        min_length: 3
        max_length: 20
"#;
    fs::write(temp_dir.path().join("length-task.task.yaml"), task_yaml).unwrap();

    let loader = TaskLoader::with_paths(vec![temp_dir.path().to_path_buf()]);
    let mut resolver = TaskResolver::with_loader(loader);

    // Valid length
    let mut inputs_valid = HashMap::new();
    inputs_valid.insert("username".to_string(), serde_json::json!("john"));
    assert!(resolver
        .resolve("length-task@1.0.0", &inputs_valid, &HashMap::new())
        .is_ok());

    // Too short
    let mut inputs_short = HashMap::new();
    inputs_short.insert("username".to_string(), serde_json::json!("ab"));
    let result = resolver.resolve("length-task@1.0.0", &inputs_short, &HashMap::new());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("less than minimum"));

    // Too long
    let mut inputs_long = HashMap::new();
    inputs_long.insert(
        "username".to_string(),
        serde_json::json!("verylongusernamethatexceedslimit"),
    );
    let result = resolver.resolve("length-task@1.0.0", &inputs_long, &HashMap::new());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("exceeds maximum"));
}

#[test]
fn test_validation_allowed_values() {
    let temp_dir = TempDir::new().unwrap();

    let task_yaml = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "enum-task"
  version: "1.0.0"
spec:
  agent_template:
    description: "Test"
    tools: ["Read"]
  inputs:
    status:
      type: string
      required: true
      validation:
        allowed_values: ["active", "inactive", "pending"]
"#;
    fs::write(temp_dir.path().join("enum-task.task.yaml"), task_yaml).unwrap();

    let loader = TaskLoader::with_paths(vec![temp_dir.path().to_path_buf()]);
    let mut resolver = TaskResolver::with_loader(loader);

    // Valid value
    let mut inputs_valid = HashMap::new();
    inputs_valid.insert("status".to_string(), serde_json::json!("active"));
    assert!(resolver
        .resolve("enum-task@1.0.0", &inputs_valid, &HashMap::new())
        .is_ok());

    // Invalid value
    let mut inputs_invalid = HashMap::new();
    inputs_invalid.insert("status".to_string(), serde_json::json!("unknown"));
    let result = resolver.resolve("enum-task@1.0.0", &inputs_invalid, &HashMap::new());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("not in allowed values"));
}

#[test]
fn test_validation_all_input_types() {
    let temp_dir = TempDir::new().unwrap();

    let task_yaml = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "types-task"
  version: "1.0.0"
spec:
  agent_template:
    description: "Test all types"
    tools: ["Read"]
  inputs:
    str_val:
      type: string
      required: true
    num_val:
      type: number
      required: true
    bool_val:
      type: boolean
      required: true
    obj_val:
      type: object
      required: true
    arr_val:
      type: array
      required: true
"#;
    fs::write(temp_dir.path().join("types-task.task.yaml"), task_yaml).unwrap();

    let loader = TaskLoader::with_paths(vec![temp_dir.path().to_path_buf()]);
    let mut resolver = TaskResolver::with_loader(loader);

    // All correct types
    let mut inputs = HashMap::new();
    inputs.insert("str_val".to_string(), serde_json::json!("hello"));
    inputs.insert("num_val".to_string(), serde_json::json!(42));
    inputs.insert("bool_val".to_string(), serde_json::json!(true));
    inputs.insert("obj_val".to_string(), serde_json::json!({"key": "value"}));
    inputs.insert("arr_val".to_string(), serde_json::json!([1, 2, 3]));

    let result = resolver.resolve("types-task@1.0.0", &inputs, &HashMap::new());
    assert!(result.is_ok());
}

#[test]
fn test_resolver_invalid_task_reference() {
    let mut resolver = TaskResolver::new();

    let result = resolver.resolve("invalid-reference", &HashMap::new(), &HashMap::new());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid task reference"));
}
