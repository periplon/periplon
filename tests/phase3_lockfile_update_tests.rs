//! Comprehensive Integration Tests for Phase 3: Lock Files and Update Checking
//!
//! Tests cover:
//! - Lock file generation from resolved tasks
//! - Lock file loading and version validation
//! - Lock file validation (checksums, versions, dependencies)
//! - Update checking across multiple sources
//! - Version resolution and comparison
//! - Breaking change detection
//! - Automatic update policies
//! - Multi-source update scenarios

use async_trait::async_trait;
use chrono::Utc;
use periplon_sdk::dsl::predefined_tasks::lockfile::{
    compute_task_checksum, generate_lock_file, validate_lock_file, LocalSourceResolver, LockFile,
    LockFileError, LockedTask, TaskSource as LockFileTaskSource, ValidationIssue,
    LOCK_FILE_VERSION,
};
use periplon_sdk::dsl::predefined_tasks::update::{
    UpdateChecker, UpdateInfo, UpdateRecommendation, VersionUpdatePolicy,
};
use periplon_sdk::dsl::predefined_tasks::{
    deps::ResolvedTask, AgentTemplate, PredefinedTask, PredefinedTaskMetadata, PredefinedTaskSpec,
    SourceType, TaskApiVersion, TaskKind, TaskMetadata,
};
use periplon_sdk::dsl::schema::PermissionsSpec;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// ============================================================================
// SECTION 1: Lock File Generation Tests
// ============================================================================

/// Helper to create a test PredefinedTask
fn create_test_task(name: &str, version: &str, description: &str) -> PredefinedTask {
    PredefinedTask {
        api_version: TaskApiVersion::V1,
        kind: TaskKind::PredefinedTask,
        metadata: PredefinedTaskMetadata {
            name: name.to_string(),
            version: version.to_string(),
            description: Some(description.to_string()),
            author: Some("Test Author".to_string()),
            license: Some("MIT".to_string()),
            repository: None,
            tags: vec!["test".to_string()],
        },
        spec: PredefinedTaskSpec {
            agent_template: AgentTemplate {
                description: description.to_string(),
                model: Some("claude-sonnet-4-5".to_string()),
                system_prompt: None,
                tools: vec!["Read".to_string(), "Write".to_string()],
                permissions: PermissionsSpec::default(),
                max_turns: Some(10),
            },
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            dependencies: vec![],
            examples: vec![],
        },
    }
}

/// Helper to create a ResolvedTask
fn create_resolved_task(
    name: &str,
    version: &str,
    dependencies: HashMap<String, String>,
) -> ResolvedTask {
    ResolvedTask {
        name: name.to_string(),
        version: version.to_string(),
        task: create_test_task(name, version, &format!("Test task {}", name)),
        dependencies,
    }
}

#[test]
fn test_generate_lock_file_single_task() {
    let resolved = vec![create_resolved_task("test-task", "1.0.0", HashMap::new())];

    let resolver = LocalSourceResolver::new(PathBuf::from("/tmp/tasks"));
    let lock_file = generate_lock_file(&resolved, &resolver).unwrap();

    assert_eq!(lock_file.version, LOCK_FILE_VERSION);
    assert_eq!(lock_file.tasks.len(), 1);
    assert!(lock_file.get_task("test-task").is_some());

    let locked = lock_file.get_task("test-task").unwrap();
    assert_eq!(locked.version, "1.0.0");
    assert!(locked.checksum.starts_with("sha256:"));
    assert_eq!(locked.dependencies.len(), 0);
}

#[test]
fn test_generate_lock_file_with_dependencies() {
    let mut deps = HashMap::new();
    deps.insert("dependency-a".to_string(), "2.0.0".to_string());
    deps.insert("dependency-b".to_string(), "1.5.0".to_string());

    let resolved = vec![
        create_resolved_task("main-task", "1.0.0", deps),
        create_resolved_task("dependency-a", "2.0.0", HashMap::new()),
        create_resolved_task("dependency-b", "1.5.0", HashMap::new()),
    ];

    let resolver = LocalSourceResolver::new(PathBuf::from("/tmp/tasks"));
    let lock_file = generate_lock_file(&resolved, &resolver).unwrap();

    assert_eq!(lock_file.tasks.len(), 3);

    let main_locked = lock_file.get_task("main-task").unwrap();
    assert_eq!(main_locked.dependencies.len(), 2);
    assert_eq!(
        main_locked.dependencies.get("dependency-a"),
        Some(&"2.0.0".to_string())
    );
    assert_eq!(
        main_locked.dependencies.get("dependency-b"),
        Some(&"1.5.0".to_string())
    );
}

#[test]
fn test_generate_lock_file_metadata() {
    let resolved = vec![create_resolved_task("test", "1.0.0", HashMap::new())];

    let resolver = LocalSourceResolver::new(PathBuf::from("/tmp/tasks"));
    let lock_file = generate_lock_file(&resolved, &resolver).unwrap();

    assert!(lock_file.metadata.is_some());
    let metadata = lock_file.metadata.unwrap();
    assert_eq!(metadata.task_count, Some(1));
}

// ============================================================================
// SECTION 2: Lock File Loading and Version Validation Tests
// ============================================================================

#[test]
fn test_save_and_load_lock_file() {
    let temp_dir = TempDir::new().unwrap();
    let lock_path = temp_dir.path().join("tasks.lock.yaml");

    // Create and save lock file
    let mut lock_file = LockFile::new();
    lock_file.add_task(
        "test-task".to_string(),
        LockedTask {
            version: "1.0.0".to_string(),
            checksum: "sha256:abc123".to_string(),
            source: LockFileTaskSource::Local {
                path: "./test.yaml".to_string(),
            },
            resolved_at: Utc::now(),
            dependencies: HashMap::new(),
            metadata: None,
        },
    );

    lock_file.save(&lock_path).unwrap();
    assert!(lock_path.exists());

    // Load and verify
    let loaded = LockFile::load(&lock_path).unwrap();
    assert_eq!(loaded.version, LOCK_FILE_VERSION);
    assert_eq!(loaded.tasks.len(), 1);
    assert!(loaded.get_task("test-task").is_some());
}

#[test]
fn test_load_incompatible_version() {
    let temp_dir = TempDir::new().unwrap();
    let lock_path = temp_dir.path().join("tasks.lock.yaml");

    // Create lock file with incompatible version
    let incompatible_yaml = r#"
version: "2.0.0"
generated_at: "2025-01-15T10:00:00Z"
generated_by: "test"
tasks: {}
"#;
    fs::write(&lock_path, incompatible_yaml).unwrap();

    // Should fail to load
    let result = LockFile::load(&lock_path);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        LockFileError::IncompatibleVersion { .. }
    ));
}

#[test]
fn test_lock_file_version_compatibility() {
    let temp_dir = TempDir::new().unwrap();
    let lock_path = temp_dir.path().join("tasks.lock.yaml");

    // Same minor version should be compatible
    let compatible_yaml = r#"
version: "1.1.0"
generated_at: "2025-01-15T10:00:00Z"
generated_by: "test"
tasks: {}
"#;
    fs::write(&lock_path, compatible_yaml).unwrap();

    let result = LockFile::load(&lock_path);
    assert!(result.is_ok());
}

#[test]
fn test_load_malformed_lock_file() {
    let temp_dir = TempDir::new().unwrap();
    let lock_path = temp_dir.path().join("tasks.lock.yaml");

    let malformed_yaml = r#"
version: "1.0.0"
generated_at: "invalid-date"
tasks: {this is not valid yaml
"#;
    fs::write(&lock_path, malformed_yaml).unwrap();

    let result = LockFile::load(&lock_path);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LockFileError::ParseError(_)));
}

// ============================================================================
// SECTION 3: Lock File Validation Tests
// ============================================================================

#[test]
fn test_validate_lock_file_all_valid() {
    let resolved = vec![create_resolved_task("test", "1.0.0", HashMap::new())];

    let resolver = LocalSourceResolver::new(PathBuf::from("/tmp/tasks"));
    let lock_file = generate_lock_file(&resolved, &resolver).unwrap();

    let result = validate_lock_file(&lock_file, &resolved).unwrap();
    assert!(result.is_valid());
    assert_eq!(result.issues.len(), 0);
}

#[test]
fn test_validate_lock_file_missing_task() {
    let resolved = vec![create_resolved_task("test", "1.0.0", HashMap::new())];

    let resolver = LocalSourceResolver::new(PathBuf::from("/tmp/tasks"));
    let lock_file = generate_lock_file(&resolved, &resolver).unwrap();

    // Add a new task to resolved that's not in lock file
    let new_resolved = vec![
        create_resolved_task("test", "1.0.0", HashMap::new()),
        create_resolved_task("new-task", "1.0.0", HashMap::new()),
    ];

    let result = validate_lock_file(&lock_file, &new_resolved).unwrap();
    assert!(!result.is_valid());
    assert_eq!(result.issues.len(), 1);

    match &result.issues[0] {
        ValidationIssue::MissingTask { name } => {
            assert_eq!(name, "new-task");
        }
        _ => panic!("Expected MissingTask issue"),
    }
}

#[test]
fn test_validate_lock_file_extra_task() {
    let resolved = vec![
        create_resolved_task("test1", "1.0.0", HashMap::new()),
        create_resolved_task("test2", "1.0.0", HashMap::new()),
    ];

    let resolver = LocalSourceResolver::new(PathBuf::from("/tmp/tasks"));
    let lock_file = generate_lock_file(&resolved, &resolver).unwrap();

    // Remove one task from resolved
    let new_resolved = vec![create_resolved_task("test1", "1.0.0", HashMap::new())];

    let result = validate_lock_file(&lock_file, &new_resolved).unwrap();
    assert!(!result.is_valid());

    // Should have ExtraTask issue
    let has_extra = result
        .issues
        .iter()
        .any(|i| matches!(i, ValidationIssue::ExtraTask { .. }));
    assert!(has_extra);
}

#[test]
fn test_validate_lock_file_version_mismatch() {
    let resolved_old = vec![create_resolved_task("test", "1.0.0", HashMap::new())];

    let resolver = LocalSourceResolver::new(PathBuf::from("/tmp/tasks"));
    let lock_file = generate_lock_file(&resolved_old, &resolver).unwrap();

    // Create resolved with different version
    let resolved_new = vec![create_resolved_task("test", "2.0.0", HashMap::new())];

    let result = validate_lock_file(&lock_file, &resolved_new).unwrap();
    assert!(!result.is_valid());

    // Should have VersionMismatch and ChecksumFailed issues
    let has_version_mismatch = result
        .issues
        .iter()
        .any(|i| matches!(i, ValidationIssue::VersionMismatch { .. }));
    let has_checksum_failed = result
        .issues
        .iter()
        .any(|i| matches!(i, ValidationIssue::ChecksumFailed { .. }));

    assert!(has_version_mismatch);
    assert!(has_checksum_failed);
}

#[test]
fn test_validate_lock_file_dependency_mismatch() {
    let mut deps1 = HashMap::new();
    deps1.insert("dep-a".to_string(), "1.0.0".to_string());

    let resolved = vec![
        create_resolved_task("main", "1.0.0", deps1.clone()),
        create_resolved_task("dep-a", "1.0.0", HashMap::new()),
    ];

    let resolver = LocalSourceResolver::new(PathBuf::from("/tmp/tasks"));
    let lock_file = generate_lock_file(&resolved, &resolver).unwrap();

    // Change dependency version
    let mut deps2 = HashMap::new();
    deps2.insert("dep-a".to_string(), "2.0.0".to_string());

    let new_resolved = vec![
        create_resolved_task("main", "1.0.0", deps2),
        create_resolved_task("dep-a", "2.0.0", HashMap::new()),
    ];

    let result = validate_lock_file(&lock_file, &new_resolved).unwrap();
    assert!(!result.is_valid());

    let has_dep_mismatch = result
        .issues
        .iter()
        .any(|i| matches!(i, ValidationIssue::DependencyMismatch { .. }));
    assert!(has_dep_mismatch);
}

#[test]
fn test_checksum_verification_success() {
    let task = create_test_task("test", "1.0.0", "Test task");
    let checksum = compute_task_checksum(&task).unwrap();

    let mut lock_file = LockFile::new();
    lock_file.add_task(
        "test".to_string(),
        LockedTask {
            version: "1.0.0".to_string(),
            checksum,
            source: LockFileTaskSource::Local {
                path: "./test.yaml".to_string(),
            },
            resolved_at: Utc::now(),
            dependencies: HashMap::new(),
            metadata: None,
        },
    );

    let result = lock_file.verify_task("test", &task);
    assert!(result.is_ok());
}

#[test]
fn test_checksum_verification_failure() {
    let task = create_test_task("test", "1.0.0", "Test task");

    let mut lock_file = LockFile::new();
    lock_file.add_task(
        "test".to_string(),
        LockedTask {
            version: "1.0.0".to_string(),
            checksum: "sha256:incorrect_checksum".to_string(),
            source: LockFileTaskSource::Local {
                path: "./test.yaml".to_string(),
            },
            resolved_at: Utc::now(),
            dependencies: HashMap::new(),
            metadata: None,
        },
    );

    let result = lock_file.verify_task("test", &task);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        LockFileError::ChecksumMismatch { .. }
    ));
}

#[test]
fn test_checksum_consistency() {
    let task = create_test_task("test", "1.0.0", "Test task");

    let checksum1 = compute_task_checksum(&task).unwrap();
    let checksum2 = compute_task_checksum(&task).unwrap();

    // Same task should always produce the same checksum
    assert_eq!(checksum1, checksum2);
    assert!(checksum1.starts_with("sha256:"));
}

#[test]
fn test_lock_file_is_stale_no_changes() {
    let resolved = vec![create_resolved_task("test", "1.0.0", HashMap::new())];

    let resolver = LocalSourceResolver::new(PathBuf::from("/tmp/tasks"));
    let lock_file = generate_lock_file(&resolved, &resolver).unwrap();

    assert!(!lock_file.is_stale(&resolved));
}

#[test]
fn test_lock_file_is_stale_version_changed() {
    let resolved_old = vec![create_resolved_task("test", "1.0.0", HashMap::new())];

    let resolver = LocalSourceResolver::new(PathBuf::from("/tmp/tasks"));
    let lock_file = generate_lock_file(&resolved_old, &resolver).unwrap();

    let resolved_new = vec![create_resolved_task("test", "2.0.0", HashMap::new())];

    assert!(lock_file.is_stale(&resolved_new));
}

#[test]
fn test_lock_file_is_stale_task_added() {
    let resolved_old = vec![create_resolved_task("test1", "1.0.0", HashMap::new())];

    let resolver = LocalSourceResolver::new(PathBuf::from("/tmp/tasks"));
    let lock_file = generate_lock_file(&resolved_old, &resolver).unwrap();

    let resolved_new = vec![
        create_resolved_task("test1", "1.0.0", HashMap::new()),
        create_resolved_task("test2", "1.0.0", HashMap::new()),
    ];

    assert!(lock_file.is_stale(&resolved_new));
}

#[test]
fn test_lock_file_is_stale_dependencies_changed() {
    let mut deps_old = HashMap::new();
    deps_old.insert("dep-a".to_string(), "1.0.0".to_string());

    let resolved_old = vec![
        create_resolved_task("main", "1.0.0", deps_old),
        create_resolved_task("dep-a", "1.0.0", HashMap::new()),
    ];

    let resolver = LocalSourceResolver::new(PathBuf::from("/tmp/tasks"));
    let lock_file = generate_lock_file(&resolved_old, &resolver).unwrap();

    let mut deps_new = HashMap::new();
    deps_new.insert("dep-a".to_string(), "2.0.0".to_string());

    let resolved_new = vec![
        create_resolved_task("main", "1.0.0", deps_new),
        create_resolved_task("dep-a", "2.0.0", HashMap::new()),
    ];

    assert!(lock_file.is_stale(&resolved_new));
}

// ============================================================================
// SECTION 4: Update Checking Tests
// ============================================================================

// Mock task source for update testing
struct MockTaskSource {
    name: String,
    tasks: Vec<TaskMetadata>,
}

#[async_trait]
impl periplon_sdk::dsl::predefined_tasks::TaskSource for MockTaskSource {
    fn name(&self) -> &str {
        &self.name
    }

    fn source_type(&self) -> SourceType {
        SourceType::Local
    }

    fn priority(&self) -> u8 {
        100
    }

    fn is_trusted(&self) -> bool {
        true
    }

    async fn discover_tasks(&mut self) -> periplon_sdk::error::Result<Vec<TaskMetadata>> {
        Ok(self.tasks.clone())
    }

    async fn load_task(
        &mut self,
        _name: &str,
        _version: Option<&str>,
    ) -> periplon_sdk::error::Result<PredefinedTask> {
        unimplemented!()
    }

    async fn update(
        &mut self,
    ) -> periplon_sdk::error::Result<periplon_sdk::dsl::predefined_tasks::UpdateResult>
    {
        unimplemented!()
    }

    async fn health_check(
        &self,
    ) -> periplon_sdk::error::Result<periplon_sdk::dsl::predefined_tasks::HealthStatus>
    {
        Ok(periplon_sdk::dsl::predefined_tasks::HealthStatus {
            available: true,
            message: None,
            last_check: Utc::now(),
        })
    }
}

fn create_test_metadata(name: &str, versions: Vec<&str>) -> Vec<TaskMetadata> {
    versions
        .iter()
        .map(|v| TaskMetadata {
            name: name.to_string(),
            version: v.to_string(),
            description: Some(format!("Test task {} v{}", name, v)),
            author: Some("test".to_string()),
            tags: vec!["test".to_string()],
            source_name: "test-source".to_string(),
            source_type: SourceType::Local,
        })
        .collect()
}

#[tokio::test]
async fn test_check_update_patch() {
    let tasks = create_test_metadata("my-task", vec!["1.0.0", "1.0.1", "1.0.2"]);
    let source = Box::new(MockTaskSource {
        name: "test-source".to_string(),
        tasks,
    });

    let mut checker = UpdateChecker::new(vec![source]);
    let info = checker.check_update("my-task", "1.0.0").await.unwrap();

    assert_eq!(info.latest_version, "1.0.2");
    assert!(info.has_updates());
    assert!(info.is_patch);
    assert!(!info.is_minor);
    assert!(!info.is_breaking);
    assert_eq!(info.recommendation, UpdateRecommendation::Recommended);
    assert_eq!(info.update_type(), "patch");
}

#[tokio::test]
async fn test_check_update_minor() {
    let tasks = create_test_metadata("my-task", vec!["1.0.0", "1.1.0", "1.2.0"]);
    let source = Box::new(MockTaskSource {
        name: "test-source".to_string(),
        tasks,
    });

    let mut checker = UpdateChecker::new(vec![source]);
    let info = checker.check_update("my-task", "1.0.0").await.unwrap();

    assert_eq!(info.latest_version, "1.2.0");
    assert!(info.has_updates());
    assert!(!info.is_patch);
    assert!(info.is_minor);
    assert!(!info.is_breaking);
    assert_eq!(info.recommendation, UpdateRecommendation::Recommended);
    assert_eq!(info.update_type(), "minor");
}

#[tokio::test]
async fn test_check_update_major_breaking() {
    let tasks = create_test_metadata("my-task", vec!["1.0.0", "2.0.0", "3.0.0"]);
    let source = Box::new(MockTaskSource {
        name: "test-source".to_string(),
        tasks,
    });

    let mut checker = UpdateChecker::new(vec![source]);
    let info = checker.check_update("my-task", "1.0.0").await.unwrap();

    assert_eq!(info.latest_version, "3.0.0");
    assert!(info.has_updates());
    assert!(!info.is_patch);
    assert!(!info.is_minor);
    assert!(info.is_breaking);
    assert_eq!(info.recommendation, UpdateRecommendation::ReviewRequired);
    assert_eq!(info.update_type(), "major");
}

#[tokio::test]
async fn test_check_update_already_latest() {
    let tasks = create_test_metadata("my-task", vec!["1.0.0", "1.0.1", "1.0.2"]);
    let source = Box::new(MockTaskSource {
        name: "test-source".to_string(),
        tasks,
    });

    let mut checker = UpdateChecker::new(vec![source]);
    let info = checker.check_update("my-task", "1.0.2").await.unwrap();

    assert_eq!(info.latest_version, "1.0.2");
    assert!(!info.has_updates());
    assert_eq!(info.recommendation, UpdateRecommendation::UpToDate);
}

#[tokio::test]
async fn test_check_update_task_not_found() {
    let tasks = create_test_metadata("other-task", vec!["1.0.0"]);
    let source = Box::new(MockTaskSource {
        name: "test-source".to_string(),
        tasks,
    });

    let mut checker = UpdateChecker::new(vec![source]);
    let result = checker.check_update("missing-task", "1.0.0").await;

    assert!(result.is_err());
}

// ============================================================================
// SECTION 5: Version Resolution Tests
// ============================================================================

#[tokio::test]
async fn test_version_resolution_multiple_sources() {
    let source1_tasks = create_test_metadata("task-a", vec!["1.0.0", "1.1.0"]);
    let source2_tasks = create_test_metadata("task-a", vec!["1.2.0", "1.3.0"]);

    let source1 = Box::new(MockTaskSource {
        name: "source1".to_string(),
        tasks: source1_tasks,
    });
    let source2 = Box::new(MockTaskSource {
        name: "source2".to_string(),
        tasks: source2_tasks,
    });

    let mut checker = UpdateChecker::new(vec![source1, source2]);
    let info = checker.check_update("task-a", "1.0.0").await.unwrap();

    // Should find the latest version across all sources
    assert_eq!(info.latest_version, "1.3.0");
    assert_eq!(info.update_source, "source2");
    assert_eq!(info.available_versions.len(), 4);

    // Verify sorted order (newest first)
    assert_eq!(info.available_versions[0], "1.3.0");
    assert_eq!(info.available_versions[3], "1.0.0");
}

#[tokio::test]
async fn test_version_resolution_prerelease_exclusion() {
    let tasks = vec![
        TaskMetadata {
            name: "my-task".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Stable".to_string()),
            author: None,
            tags: vec![],
            source_name: "test".to_string(),
            source_type: SourceType::Local,
        },
        TaskMetadata {
            name: "my-task".to_string(),
            version: "1.1.0-beta.1".to_string(),
            description: Some("Beta".to_string()),
            author: None,
            tags: vec![],
            source_name: "test".to_string(),
            source_type: SourceType::Local,
        },
        TaskMetadata {
            name: "my-task".to_string(),
            version: "1.0.1".to_string(),
            description: Some("Patch".to_string()),
            author: None,
            tags: vec![],
            source_name: "test".to_string(),
            source_type: SourceType::Local,
        },
    ];

    let source = Box::new(MockTaskSource {
        name: "test".to_string(),
        tasks,
    });

    let mut checker = UpdateChecker::new(vec![source]);
    // By default, prerelease versions should be excluded
    checker.set_include_prerelease(false);

    let info = checker.check_update("my-task", "1.0.0").await.unwrap();

    // Should use 1.0.1, not 1.1.0-beta.1
    assert_eq!(info.latest_version, "1.0.1");
}

#[tokio::test]
async fn test_version_resolution_prerelease_inclusion() {
    let tasks = vec![
        TaskMetadata {
            name: "my-task".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Stable".to_string()),
            author: None,
            tags: vec![],
            source_name: "test".to_string(),
            source_type: SourceType::Local,
        },
        TaskMetadata {
            name: "my-task".to_string(),
            version: "1.1.0-beta.1".to_string(),
            description: Some("Beta".to_string()),
            author: None,
            tags: vec![],
            source_name: "test".to_string(),
            source_type: SourceType::Local,
        },
    ];

    let source = Box::new(MockTaskSource {
        name: "test".to_string(),
        tasks,
    });

    let mut checker = UpdateChecker::new(vec![source]);
    checker.set_include_prerelease(true);

    let info = checker.check_update("my-task", "1.0.0").await.unwrap();

    // Should include prerelease
    assert_eq!(info.latest_version, "1.1.0-beta.1");
}

// ============================================================================
// SECTION 6: Breaking Change Detection Tests
// ============================================================================

#[test]
fn test_detect_breaking_changes_major_bump() {
    let checker = UpdateChecker::new(vec![]);
    let info = checker.detect_breaking_changes("1.5.2", "2.0.0").unwrap();

    assert!(info.has_breaking_changes);
    assert_eq!(info.major_version_bumps, 1);
    assert!(!info.recommendations.is_empty());
    assert_eq!(info.from_version, "1.5.2");
    assert_eq!(info.to_version, "2.0.0");
}

#[test]
fn test_detect_breaking_changes_multiple_major_bumps() {
    let checker = UpdateChecker::new(vec![]);
    let info = checker.detect_breaking_changes("1.0.0", "4.0.0").unwrap();

    assert!(info.has_breaking_changes);
    assert_eq!(info.major_version_bumps, 3);
}

#[test]
fn test_detect_breaking_changes_minor_update() {
    let checker = UpdateChecker::new(vec![]);
    let info = checker.detect_breaking_changes("1.2.0", "1.3.0").unwrap();

    assert!(!info.has_breaking_changes);
    assert_eq!(info.major_version_bumps, 0);
    assert!(info.recommendations.is_empty());
}

#[test]
fn test_detect_breaking_changes_patch_update() {
    let checker = UpdateChecker::new(vec![]);
    let info = checker.detect_breaking_changes("1.2.3", "1.2.4").unwrap();

    assert!(!info.has_breaking_changes);
    assert_eq!(info.major_version_bumps, 0);
}

// ============================================================================
// SECTION 7: Automatic Update Policy Tests
// ============================================================================

#[test]
fn test_update_policy_patch_only() {
    let info = UpdateInfo {
        task_name: "test".to_string(),
        current_version: "1.0.0".to_string(),
        latest_version: "1.0.1".to_string(),
        available_versions: vec![],
        update_source: "test".to_string(),
        is_breaking: false,
        is_minor: false,
        is_patch: true,
        changelog: None,
        recommendation: UpdateRecommendation::Recommended,
    };

    assert!(info.is_allowed(VersionUpdatePolicy::PatchOnly));
    assert!(info.is_allowed(VersionUpdatePolicy::MinorAndPatch));
    assert!(info.is_allowed(VersionUpdatePolicy::All));
    assert!(!info.is_allowed(VersionUpdatePolicy::Manual));
}

#[test]
fn test_update_policy_minor_update() {
    let info = UpdateInfo {
        task_name: "test".to_string(),
        current_version: "1.0.0".to_string(),
        latest_version: "1.1.0".to_string(),
        available_versions: vec![],
        update_source: "test".to_string(),
        is_breaking: false,
        is_minor: true,
        is_patch: false,
        changelog: None,
        recommendation: UpdateRecommendation::Recommended,
    };

    assert!(!info.is_allowed(VersionUpdatePolicy::PatchOnly));
    assert!(info.is_allowed(VersionUpdatePolicy::MinorAndPatch));
    assert!(info.is_allowed(VersionUpdatePolicy::All));
    assert!(!info.is_allowed(VersionUpdatePolicy::Manual));
}

#[test]
fn test_update_policy_major_breaking() {
    let info = UpdateInfo {
        task_name: "test".to_string(),
        current_version: "1.0.0".to_string(),
        latest_version: "2.0.0".to_string(),
        available_versions: vec![],
        update_source: "test".to_string(),
        is_breaking: true,
        is_minor: false,
        is_patch: false,
        changelog: None,
        recommendation: UpdateRecommendation::ReviewRequired,
    };

    assert!(!info.is_allowed(VersionUpdatePolicy::PatchOnly));
    assert!(!info.is_allowed(VersionUpdatePolicy::MinorAndPatch));
    assert!(info.is_allowed(VersionUpdatePolicy::All));
    assert!(!info.is_allowed(VersionUpdatePolicy::Manual));
}

#[tokio::test]
async fn test_auto_update_with_policy() {
    let tasks = create_test_metadata("my-task", vec!["1.0.0", "1.0.1", "1.0.2"]);
    let source = Box::new(MockTaskSource {
        name: "test".to_string(),
        tasks,
    });

    let mut checker = UpdateChecker::new(vec![source]);

    // Patch update should succeed with PatchOnly policy
    let result = checker
        .auto_update("my-task", "1.0.0", VersionUpdatePolicy::PatchOnly)
        .await
        .unwrap();

    assert!(result.success);
    assert_eq!(result.from_version, "1.0.0");
    assert_eq!(result.to_version, "1.0.2");
}

#[tokio::test]
async fn test_auto_update_policy_violation() {
    let tasks = create_test_metadata("my-task", vec!["1.0.0", "1.1.0"]);
    let source = Box::new(MockTaskSource {
        name: "test".to_string(),
        tasks,
    });

    let mut checker = UpdateChecker::new(vec![source]);

    // Minor update should fail with PatchOnly policy
    let result = checker
        .auto_update("my-task", "1.0.0", VersionUpdatePolicy::PatchOnly)
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_auto_update_already_latest() {
    let tasks = create_test_metadata("my-task", vec!["1.0.0"]);
    let source = Box::new(MockTaskSource {
        name: "test".to_string(),
        tasks,
    });

    let mut checker = UpdateChecker::new(vec![source]);

    let result = checker
        .auto_update("my-task", "1.0.0", VersionUpdatePolicy::All)
        .await
        .unwrap();

    assert!(result.success);
    assert_eq!(result.from_version, "1.0.0");
    assert_eq!(result.to_version, "1.0.0");
    assert!(result.error.is_some());
}

#[tokio::test]
async fn test_auto_update_all_tasks() {
    let tasks = create_test_metadata("task-a", vec!["1.0.0", "1.0.1"])
        .into_iter()
        .chain(create_test_metadata("task-b", vec!["2.0.0", "2.1.0"]))
        .collect();

    let source = Box::new(MockTaskSource {
        name: "test".to_string(),
        tasks,
    });

    let mut checker = UpdateChecker::new(vec![source]);

    let task_list = vec![
        ("task-a".to_string(), "1.0.0".to_string()),
        ("task-b".to_string(), "2.0.0".to_string()),
    ];

    let results = checker
        .auto_update_all(&task_list, VersionUpdatePolicy::MinorAndPatch)
        .await
        .unwrap();

    assert_eq!(results.len(), 2);

    // Both should succeed
    assert!(results[0].success);
    assert!(results[1].success);

    // task-a: patch update
    assert_eq!(results[0].to_version, "1.0.1");

    // task-b: minor update
    assert_eq!(results[1].to_version, "2.1.0");
}

// ============================================================================
// SECTION 8: Multi-Source Update Scenarios
// ============================================================================

#[tokio::test]
async fn test_check_updates_batch() {
    let tasks = create_test_metadata("task-a", vec!["1.0.0", "1.1.0"])
        .into_iter()
        .chain(create_test_metadata("task-b", vec!["2.0.0", "2.0.1"]))
        .collect();

    let source = Box::new(MockTaskSource {
        name: "test".to_string(),
        tasks,
    });

    let mut checker = UpdateChecker::new(vec![source]);

    let task_list = vec![
        ("task-a".to_string(), "1.0.0".to_string()),
        ("task-b".to_string(), "2.0.0".to_string()),
    ];

    let results = checker.check_updates(&task_list).await.unwrap();

    assert_eq!(results.len(), 2);

    // task-a has minor update
    assert!(results[0].is_minor);
    assert_eq!(results[0].latest_version, "1.1.0");

    // task-b has patch update
    assert!(results[1].is_patch);
    assert_eq!(results[1].latest_version, "2.0.1");
}

#[tokio::test]
async fn test_cache_refresh() {
    let tasks = create_test_metadata("my-task", vec!["1.0.0"]);
    let source = Box::new(MockTaskSource {
        name: "test".to_string(),
        tasks,
    });

    let mut checker = UpdateChecker::new(vec![source]);

    // Initially cache should be empty
    let stats = checker.get_update_stats();
    assert_eq!(stats.total_tasks, 0);

    // Refresh cache
    checker.refresh_cache().await.unwrap();

    let stats = checker.get_update_stats();
    assert_eq!(stats.total_tasks, 1);
    assert_eq!(stats.sources_checked, 1);
}

// ============================================================================
// SECTION 9: Integration Tests (Lock Files + Updates)
// ============================================================================

#[tokio::test]
async fn test_integration_lock_file_update_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let lock_path = temp_dir.path().join("tasks.lock.yaml");

    // Step 1: Generate initial lock file
    let resolved = vec![
        create_resolved_task("task-a", "1.0.0", HashMap::new()),
        create_resolved_task("task-b", "2.0.0", HashMap::new()),
    ];

    let resolver = LocalSourceResolver::new(temp_dir.path().to_path_buf());
    let lock_file = generate_lock_file(&resolved, &resolver).unwrap();
    lock_file.save(&lock_path).unwrap();

    // Step 2: Check for updates
    let source_tasks = create_test_metadata("task-a", vec!["1.0.0", "1.1.0"])
        .into_iter()
        .chain(create_test_metadata("task-b", vec!["2.0.0", "2.1.0"]))
        .collect();

    let source = Box::new(MockTaskSource {
        name: "test".to_string(),
        tasks: source_tasks,
    });

    let mut checker = UpdateChecker::new(vec![source]);

    let task_list = vec![
        ("task-a".to_string(), "1.0.0".to_string()),
        ("task-b".to_string(), "2.0.0".to_string()),
    ];

    let update_infos = checker.check_updates(&task_list).await.unwrap();

    // Step 3: Verify updates are available
    assert_eq!(update_infos.len(), 2);
    assert!(update_infos[0].has_updates());
    assert!(update_infos[1].has_updates());

    // Step 4: Load and verify lock file is now stale
    let loaded_lock = LockFile::load(&lock_path).unwrap();

    let new_resolved = vec![
        create_resolved_task("task-a", "1.1.0", HashMap::new()),
        create_resolved_task("task-b", "2.1.0", HashMap::new()),
    ];

    assert!(loaded_lock.is_stale(&new_resolved));
}

#[test]
fn test_integration_complete_lifecycle() {
    let temp_dir = TempDir::new().unwrap();
    let lock_path = temp_dir.path().join("tasks.lock.yaml");

    // Create initial tasks
    let mut deps = HashMap::new();
    deps.insert("dep-task".to_string(), "1.0.0".to_string());

    let resolved = vec![
        create_resolved_task("main-task", "1.0.0", deps),
        create_resolved_task("dep-task", "1.0.0", HashMap::new()),
    ];

    // Generate lock file
    let resolver = LocalSourceResolver::new(temp_dir.path().to_path_buf());
    let lock_file = generate_lock_file(&resolved, &resolver).unwrap();

    // Save lock file
    lock_file.save(&lock_path).unwrap();
    assert!(lock_path.exists());

    // Load lock file
    let loaded = LockFile::load(&lock_path).unwrap();
    assert_eq!(loaded.tasks.len(), 2);

    // Validate lock file
    let validation = validate_lock_file(&loaded, &resolved).unwrap();
    assert!(validation.is_valid());

    // Verify checksums
    for resolved_task in &resolved {
        let result = loaded.verify_task(&resolved_task.name, &resolved_task.task);
        assert!(result.is_ok());
    }

    // Check staleness (should not be stale)
    assert!(!loaded.is_stale(&resolved));
}
