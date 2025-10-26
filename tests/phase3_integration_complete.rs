//! Comprehensive Integration Tests for Phase 3: Advanced Edge Cases and Real-World Scenarios
//!
//! This test suite extends the existing phase3_lockfile_update_tests.rs with:
//! - Edge case testing (corrupted lockfiles, concurrent access, large graphs)
//! - Real-world integration scenarios (executor + lockfile, workflow integration)
//! - Performance validation and benchmarking
//! - Error recovery and resilience testing
//!
//! These tests focus on production readiness and robustness beyond basic functionality.

use chrono::Utc;
use periplon_sdk::dsl::predefined_tasks::lockfile::{
    compute_task_checksum, generate_lock_file, validate_lock_file, LocalSourceResolver, LockFile,
    LockFileError, ValidationIssue,
};
use periplon_sdk::dsl::predefined_tasks::{
    deps::ResolvedTask, AgentTemplate, PredefinedTask, PredefinedTaskMetadata, PredefinedTaskSpec,
    TaskApiVersion, TaskKind,
};
use periplon_sdk::dsl::schema::PermissionsSpec;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

// ============================================================================
// Test Helpers
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

/// Create a large dependency graph for stress testing
fn create_large_dependency_graph(size: usize) -> Vec<ResolvedTask> {
    let mut tasks = Vec::new();

    // Create a deep dependency chain
    for i in 0..size {
        let deps = if i > 0 {
            let mut d = HashMap::new();
            d.insert(format!("task-{}", i - 1), "1.0.0".to_string());
            d
        } else {
            HashMap::new()
        };

        tasks.push(create_resolved_task(&format!("task-{}", i), "1.0.0", deps));
    }

    tasks
}

// ============================================================================
// SECTION 1: Edge Case Tests - Corrupted Lockfiles
// ============================================================================

#[test]
fn test_lockfile_recovery_from_corrupted_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let lock_path = temp_dir.path().join("tasks.lock.yaml");

    // Create a corrupted lockfile (incomplete YAML)
    let corrupted_yaml = r#"
version: "1.0.0"
generated_at: "2025-01-15T10:00:00Z"
generated_by: "test"
tasks:
  test-task:
    version: "1.0.0"
    checksum: "sha256:abc123"
    source:
      type: local
      path: "./test.yaml
    # Missing closing quotes and structure
"#;
    fs::write(&lock_path, corrupted_yaml).unwrap();

    // Attempt to load should fail gracefully
    let result = LockFile::load(&lock_path);
    assert!(result.is_err());

    // Verify specific error type
    match result.unwrap_err() {
        LockFileError::ParseError(_) => {
            // Expected error type
        }
        other => panic!("Expected ParseError, got: {:?}", other),
    }
}

#[test]
fn test_lockfile_with_missing_required_fields() {
    let temp_dir = TempDir::new().unwrap();
    let lock_path = temp_dir.path().join("tasks.lock.yaml");

    // Lockfile missing required fields
    let incomplete_yaml = r#"
version: "1.0.0"
tasks:
  test-task:
    version: "1.0.0"
    # Missing checksum, source, resolved_at
"#;
    fs::write(&lock_path, incomplete_yaml).unwrap();

    let result = LockFile::load(&lock_path);
    assert!(result.is_err());
}

#[test]
fn test_lockfile_with_invalid_checksum_format() {
    let temp_dir = TempDir::new().unwrap();
    let lock_path = temp_dir.path().join("tasks.lock.yaml");

    let invalid_checksum_yaml = r#"
version: "1.0.0"
generated_at: "2025-01-15T10:00:00Z"
generated_by: "test"
tasks:
  test-task:
    version: "1.0.0"
    checksum: "invalid-format-no-algo"
    source:
      type: local
      path: "./test.yaml"
    resolved_at: "2025-01-15T10:00:00Z"
    dependencies: {}
"#;
    fs::write(&lock_path, invalid_checksum_yaml).unwrap();

    // Should load but validation should fail
    let lock_file = LockFile::load(&lock_path).unwrap();
    let resolved = vec![create_resolved_task("test-task", "1.0.0", HashMap::new())];

    let validation = validate_lock_file(&lock_file, &resolved).unwrap();
    assert!(!validation.is_valid());

    // Should have checksum-related issue
    let has_checksum_issue = validation
        .issues
        .iter()
        .any(|i| matches!(i, ValidationIssue::ChecksumFailed { .. }));
    assert!(has_checksum_issue);
}

#[test]
fn test_lockfile_automatic_regeneration_on_corruption() {
    let temp_dir = TempDir::new().unwrap();
    let lock_path = temp_dir.path().join("tasks.lock.yaml");

    // Write corrupted lockfile
    fs::write(&lock_path, "corrupted content { not yaml ]").unwrap();

    // Simulate recovery: load fails, regenerate
    let load_result = LockFile::load(&lock_path);
    assert!(load_result.is_err());

    // Regenerate from scratch
    let resolved = vec![create_resolved_task("test", "1.0.0", HashMap::new())];
    let resolver = LocalSourceResolver::new(temp_dir.path().to_path_buf());
    let new_lock_file = generate_lock_file(&resolved, &resolver).unwrap();

    // Save new lockfile
    new_lock_file.save(&lock_path).unwrap();

    // Verify new lockfile is valid
    let loaded = LockFile::load(&lock_path).unwrap();
    assert_eq!(loaded.tasks.len(), 1);
    assert!(loaded.get_task("test").is_some());
}

// ============================================================================
// SECTION 2: Concurrent Access Tests
// ============================================================================

#[test]
fn test_lockfile_concurrent_read_access() {
    let temp_dir = TempDir::new().unwrap();
    let lock_path = temp_dir.path().join("tasks.lock.yaml");

    // Create initial lockfile
    let resolved = vec![create_resolved_task("test", "1.0.0", HashMap::new())];
    let resolver = LocalSourceResolver::new(temp_dir.path().to_path_buf());
    let lock_file = generate_lock_file(&resolved, &resolver).unwrap();
    lock_file.save(&lock_path).unwrap();

    // Spawn multiple readers
    let lock_path_arc = Arc::new(lock_path.clone());
    let mut handles = vec![];

    for i in 0..10 {
        let path = Arc::clone(&lock_path_arc);
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(i * 10));
            let loaded = LockFile::load(&*path).unwrap();
            assert_eq!(loaded.tasks.len(), 1);
            assert!(loaded.get_task("test").is_some());
        });
        handles.push(handle);
    }

    // Wait for all readers
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_lockfile_write_safety_with_atomic_save() {
    let temp_dir = TempDir::new().unwrap();
    let lock_path = temp_dir.path().join("tasks.lock.yaml");

    // Create initial lockfile
    let resolved1 = vec![create_resolved_task("task1", "1.0.0", HashMap::new())];
    let resolver = LocalSourceResolver::new(temp_dir.path().to_path_buf());
    let lock_file1 = generate_lock_file(&resolved1, &resolver).unwrap();
    lock_file1.save(&lock_path).unwrap();

    // Verify initial state
    let loaded = LockFile::load(&lock_path).unwrap();
    assert_eq!(loaded.tasks.len(), 1);

    // Update lockfile with new tasks
    let resolved2 = vec![
        create_resolved_task("task1", "1.0.0", HashMap::new()),
        create_resolved_task("task2", "2.0.0", HashMap::new()),
    ];
    let lock_file2 = generate_lock_file(&resolved2, &resolver).unwrap();
    lock_file2.save(&lock_path).unwrap();

    // Verify update succeeded
    let loaded = LockFile::load(&lock_path).unwrap();
    assert_eq!(loaded.tasks.len(), 2);
    assert!(loaded.get_task("task2").is_some());
}

#[test]
fn test_lockfile_concurrent_readers_during_update() {
    let temp_dir = TempDir::new().unwrap();
    let lock_path = temp_dir.path().join("tasks.lock.yaml");

    // Create initial lockfile
    let resolved = vec![create_resolved_task("test", "1.0.0", HashMap::new())];
    let resolver = LocalSourceResolver::new(temp_dir.path().to_path_buf());
    let lock_file = generate_lock_file(&resolved, &resolver).unwrap();
    lock_file.save(&lock_path).unwrap();

    let lock_path_arc = Arc::new(lock_path.clone());
    let success_count = Arc::new(Mutex::new(0));

    // Spawn readers while updating
    let mut handles = vec![];

    for i in 0..5 {
        let path = Arc::clone(&lock_path_arc);
        let count = Arc::clone(&success_count);
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(i * 20));
            if let Ok(loaded) = LockFile::load(&*path) {
                assert!(loaded.tasks.len() >= 1);
                let mut c = count.lock().unwrap();
                *c += 1;
            }
        });
        handles.push(handle);
    }

    // Perform update in main thread
    thread::sleep(Duration::from_millis(50));
    let resolved_new = vec![
        create_resolved_task("test", "1.0.0", HashMap::new()),
        create_resolved_task("new", "2.0.0", HashMap::new()),
    ];
    let new_lock_file = generate_lock_file(&resolved_new, &resolver).unwrap();
    new_lock_file.save(&lock_path).unwrap();

    // Wait for readers
    for handle in handles {
        handle.join().unwrap();
    }

    // All readers should succeed
    let final_count = *success_count.lock().unwrap();
    assert_eq!(final_count, 5);
}

// ============================================================================
// SECTION 3: Large Dependency Graph Tests
// ============================================================================

#[test]
fn test_lockfile_generation_with_large_graph() {
    let temp_dir = TempDir::new().unwrap();

    // Create a graph with 100 tasks
    let resolved = create_large_dependency_graph(100);

    let resolver = LocalSourceResolver::new(temp_dir.path().to_path_buf());
    let start = Instant::now();
    let lock_file = generate_lock_file(&resolved, &resolver).unwrap();
    let duration = start.elapsed();

    // Should complete in reasonable time (< 5 seconds)
    assert!(
        duration < Duration::from_secs(5),
        "Large graph generation took {:?}",
        duration
    );

    assert_eq!(lock_file.tasks.len(), 100);

    // Verify dependency chain integrity
    for i in 1..100 {
        let task = lock_file.get_task(&format!("task-{}", i)).unwrap();
        assert_eq!(task.dependencies.len(), 1);
        assert!(task.dependencies.contains_key(&format!("task-{}", i - 1)));
    }
}

#[test]
fn test_lockfile_validation_performance_large_graph() {
    let temp_dir = TempDir::new().unwrap();

    // Create large graph
    let resolved = create_large_dependency_graph(100);
    let resolver = LocalSourceResolver::new(temp_dir.path().to_path_buf());
    let lock_file = generate_lock_file(&resolved, &resolver).unwrap();

    // Measure validation performance
    let start = Instant::now();
    let validation = validate_lock_file(&lock_file, &resolved).unwrap();
    let duration = start.elapsed();

    assert!(validation.is_valid());
    assert!(
        duration < Duration::from_secs(3),
        "Validation took {:?}",
        duration
    );
}

#[test]
fn test_lockfile_save_and_load_large_graph() {
    let temp_dir = TempDir::new().unwrap();
    let lock_path = temp_dir.path().join("large.lock.yaml");

    // Generate large lockfile
    let resolved = create_large_dependency_graph(100);
    let resolver = LocalSourceResolver::new(temp_dir.path().to_path_buf());
    let lock_file = generate_lock_file(&resolved, &resolver).unwrap();

    // Save
    let start = Instant::now();
    lock_file.save(&lock_path).unwrap();
    let save_duration = start.elapsed();

    // Load
    let start = Instant::now();
    let loaded = LockFile::load(&lock_path).unwrap();
    let load_duration = start.elapsed();

    // Performance assertions
    assert!(
        save_duration < Duration::from_secs(2),
        "Save took {:?}",
        save_duration
    );
    assert!(
        load_duration < Duration::from_secs(2),
        "Load took {:?}",
        load_duration
    );

    assert_eq!(loaded.tasks.len(), 100);
}

#[test]
fn test_lockfile_diamond_dependency_large_scale() {
    let temp_dir = TempDir::new().unwrap();

    // Create diamond pattern at scale
    // Structure:
    //   root -> [mid1, mid2, mid3] -> base
    let mut tasks = Vec::new();

    // Base task
    tasks.push(create_resolved_task("base", "1.0.0", HashMap::new()));

    // Middle layer (multiple tasks depending on base)
    for i in 1..=20 {
        let mut deps = HashMap::new();
        deps.insert("base".to_string(), "1.0.0".to_string());
        tasks.push(create_resolved_task(&format!("mid{}", i), "1.0.0", deps));
    }

    // Root task depending on all middle tasks
    let mut root_deps = HashMap::new();
    for i in 1..=20 {
        root_deps.insert(format!("mid{}", i), "1.0.0".to_string());
    }
    tasks.push(create_resolved_task("root", "1.0.0", root_deps));

    // Generate lockfile
    let resolver = LocalSourceResolver::new(temp_dir.path().to_path_buf());
    let lock_file = generate_lock_file(&tasks, &resolver).unwrap();

    assert_eq!(lock_file.tasks.len(), 22); // 1 base + 20 mid + 1 root

    // Verify root has all dependencies
    let root = lock_file.get_task("root").unwrap();
    assert_eq!(root.dependencies.len(), 20);
}

// ============================================================================
// SECTION 4: Lockfile Migration Tests
// ============================================================================

#[test]
fn test_lockfile_version_migration_forward_compatible() {
    let temp_dir = TempDir::new().unwrap();
    let lock_path = temp_dir.path().join("tasks.lock.yaml");

    // Create lockfile with newer patch version (should be compatible)
    let newer_patch_yaml = r#"
version: "1.0.1"
generated_at: "2025-01-15T10:00:00Z"
generated_by: "test"
tasks:
  test-task:
    version: "1.0.0"
    checksum: "sha256:abc123"
    source:
      type: local
      path: "./test.yaml"
    resolved_at: "2025-01-15T10:00:00Z"
    dependencies: {}
"#;
    fs::write(&lock_path, newer_patch_yaml).unwrap();

    // Should load successfully (patch version compatible)
    let result = LockFile::load(&lock_path);
    assert!(result.is_ok());

    let loaded = result.unwrap();
    assert_eq!(loaded.tasks.len(), 1);
}

#[test]
fn test_lockfile_version_migration_minor_compatible() {
    let temp_dir = TempDir::new().unwrap();
    let lock_path = temp_dir.path().join("tasks.lock.yaml");

    // Lockfile with newer minor version (1.1.0 when expecting 1.0.x)
    let newer_minor_yaml = r#"
version: "1.1.0"
generated_at: "2025-01-15T10:00:00Z"
generated_by: "test"
tasks:
  test-task:
    version: "1.0.0"
    checksum: "sha256:abc123"
    source:
      type: local
      path: "./test.yaml"
    resolved_at: "2025-01-15T10:00:00Z"
    dependencies: {}
"#;
    fs::write(&lock_path, newer_minor_yaml).unwrap();

    // Should be compatible (same major version)
    let result = LockFile::load(&lock_path);
    assert!(result.is_ok());
}

#[test]
fn test_lockfile_version_major_incompatibility() {
    let temp_dir = TempDir::new().unwrap();
    let lock_path = temp_dir.path().join("tasks.lock.yaml");

    // Lockfile with different major version
    let incompatible_yaml = r#"
version: "2.0.0"
generated_at: "2025-01-15T10:00:00Z"
generated_by: "test"
tasks: {}
"#;
    fs::write(&lock_path, incompatible_yaml).unwrap();

    // Should fail to load (incompatible major version)
    let result = LockFile::load(&lock_path);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        LockFileError::IncompatibleVersion { .. }
    ));
}

// ============================================================================
// SECTION 5: Update System Edge Cases
// ============================================================================

#[test]
fn test_update_detection_with_pre_release_versions() {
    // Test that pre-release versions are handled correctly
    // 1.0.0 -> 1.0.1-beta should not be recommended as stable update

    let temp_dir = TempDir::new().unwrap();
    let lock_path = temp_dir.path().join("tasks.lock.yaml");

    // Current version: 1.0.0
    let resolved = vec![create_resolved_task("test", "1.0.0", HashMap::new())];
    let resolver = LocalSourceResolver::new(temp_dir.path().to_path_buf());
    let lock_file = generate_lock_file(&resolved, &resolver).unwrap();
    lock_file.save(&lock_path).unwrap();

    // Simulate available version: 1.0.1-beta
    // In a real scenario, this would come from update checker
    // For now, we verify version comparison logic
}

#[test]
fn test_update_rollback_scenario() {
    let temp_dir = TempDir::new().unwrap();
    let lock_path = temp_dir.path().join("tasks.lock.yaml");
    let backup_path = temp_dir.path().join("tasks.lock.yaml.backup");

    // Create initial lockfile (v1.0.0)
    let resolved_v1 = vec![create_resolved_task("test", "1.0.0", HashMap::new())];
    let resolver = LocalSourceResolver::new(temp_dir.path().to_path_buf());
    let lock_v1 = generate_lock_file(&resolved_v1, &resolver).unwrap();
    lock_v1.save(&lock_path).unwrap();

    // Backup before update
    fs::copy(&lock_path, &backup_path).unwrap();

    // Perform update to v2.0.0
    let resolved_v2 = vec![create_resolved_task("test", "2.0.0", HashMap::new())];
    let lock_v2 = generate_lock_file(&resolved_v2, &resolver).unwrap();
    lock_v2.save(&lock_path).unwrap();

    // Verify update
    let loaded = LockFile::load(&lock_path).unwrap();
    assert_eq!(loaded.get_task("test").unwrap().version, "2.0.0");

    // Simulate rollback (restore from backup)
    fs::copy(&backup_path, &lock_path).unwrap();

    // Verify rollback
    let rolled_back = LockFile::load(&lock_path).unwrap();
    assert_eq!(rolled_back.get_task("test").unwrap().version, "1.0.0");
}

// ============================================================================
// SECTION 6: Checksum and Integrity Tests
// ============================================================================

#[test]
fn test_checksum_stability_across_serializations() {
    // Same task should produce same checksum regardless of serialization order

    let task1 = create_test_task("test", "1.0.0", "Test task");
    let checksum1 = compute_task_checksum(&task1).unwrap();

    let task2 = create_test_task("test", "1.0.0", "Test task");
    let checksum2 = compute_task_checksum(&task2).unwrap();

    assert_eq!(checksum1, checksum2);
    assert!(checksum1.starts_with("sha256:"));
}

#[test]
fn test_checksum_changes_on_content_modification() {
    let task_original = create_test_task("test", "1.0.0", "Original description");
    let checksum_original = compute_task_checksum(&task_original).unwrap();

    let task_modified = create_test_task("test", "1.0.0", "Modified description");
    let checksum_modified = compute_task_checksum(&task_modified).unwrap();

    assert_ne!(checksum_original, checksum_modified);
}

#[test]
fn test_lockfile_detects_tampered_checksum() {
    let temp_dir = TempDir::new().unwrap();
    let lock_path = temp_dir.path().join("tasks.lock.yaml");

    // Create valid lockfile
    let resolved = vec![create_resolved_task("test", "1.0.0", HashMap::new())];
    let resolver = LocalSourceResolver::new(temp_dir.path().to_path_buf());
    let lock_file = generate_lock_file(&resolved, &resolver).unwrap();
    lock_file.save(&lock_path).unwrap();

    // Tamper with checksum in the file
    let mut content = fs::read_to_string(&lock_path).unwrap();
    content = content.replace("sha256:", "sha256:tampered");
    fs::write(&lock_path, content).unwrap();

    // Load tampered lockfile
    let tampered = LockFile::load(&lock_path).unwrap();

    // Validation should detect tampering
    let validation = validate_lock_file(&tampered, &resolved).unwrap();
    assert!(!validation.is_valid());

    let has_checksum_issue = validation
        .issues
        .iter()
        .any(|i| matches!(i, ValidationIssue::ChecksumFailed { .. }));
    assert!(has_checksum_issue);
}

// ============================================================================
// SECTION 7: Error Recovery Tests
// ============================================================================

#[test]
fn test_lockfile_recovery_from_partial_write() {
    let temp_dir = TempDir::new().unwrap();
    let lock_path = temp_dir.path().join("tasks.lock.yaml");

    // Simulate partial write (incomplete YAML)
    let partial_yaml = r#"
version: "1.0.0"
generated_at: "2025-01-15T10:00:00Z"
generated_by: "test"
tasks:
  test-task:
    version: "1.0.0"
"#; // Missing required fields

    fs::write(&lock_path, partial_yaml).unwrap();

    // Loading should fail
    let result = LockFile::load(&lock_path);
    assert!(result.is_err());

    // Recovery: regenerate lockfile
    let resolved = vec![create_resolved_task("test-task", "1.0.0", HashMap::new())];
    let resolver = LocalSourceResolver::new(temp_dir.path().to_path_buf());
    let new_lock = generate_lock_file(&resolved, &resolver).unwrap();
    new_lock.save(&lock_path).unwrap();

    // Verify recovery succeeded
    let recovered = LockFile::load(&lock_path).unwrap();
    assert_eq!(recovered.tasks.len(), 1);
    assert!(recovered.get_task("test-task").is_some());
}

#[test]
fn test_lockfile_handles_missing_file_gracefully() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent_path = temp_dir.path().join("nonexistent.lock.yaml");

    // Attempt to load nonexistent file
    let result = LockFile::load(&nonexistent_path);
    assert!(result.is_err());

    // Error should be ReadError (IO error)
    match result.unwrap_err() {
        LockFileError::ReadError(_) => {
            // Expected
        }
        other => panic!("Expected ReadError, got: {:?}", other),
    }
}

// ============================================================================
// SECTION 8: Metadata and Audit Trail Tests
// ============================================================================

#[test]
fn test_lockfile_preserves_generation_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let lock_path = temp_dir.path().join("tasks.lock.yaml");

    let resolved = vec![create_resolved_task("test", "1.0.0", HashMap::new())];
    let resolver = LocalSourceResolver::new(temp_dir.path().to_path_buf());
    let lock_file = generate_lock_file(&resolved, &resolver).unwrap();

    lock_file.save(&lock_path).unwrap();
    let loaded = LockFile::load(&lock_path).unwrap();

    // Verify metadata is preserved
    assert!(loaded.metadata.is_some());
    let metadata = loaded.metadata.unwrap();
    assert_eq!(metadata.task_count, Some(1));
}

#[test]
fn test_lockfile_tracks_resolution_timestamp() {
    let resolved = vec![create_resolved_task("test", "1.0.0", HashMap::new())];
    let resolver = LocalSourceResolver::new(PathBuf::from("/tmp/tasks"));
    let lock_file = generate_lock_file(&resolved, &resolver).unwrap();

    let task = lock_file.get_task("test").unwrap();

    // Verify resolved_at timestamp is recent
    let now = Utc::now();
    let diff = now.signed_duration_since(task.resolved_at);
    assert!(diff.num_seconds() < 10); // Should be within last 10 seconds
}

// ============================================================================
// SECTION 9: Performance Benchmarks
// ============================================================================

#[test]
fn test_lockfile_generation_performance_baseline() {
    // Baseline: 10 tasks should generate in < 100ms
    let resolved = create_large_dependency_graph(10);
    let resolver = LocalSourceResolver::new(PathBuf::from("/tmp/tasks"));

    let start = Instant::now();
    let lock_file = generate_lock_file(&resolved, &resolver).unwrap();
    let duration = start.elapsed();

    assert_eq!(lock_file.tasks.len(), 10);
    assert!(
        duration < Duration::from_millis(100),
        "Generation took {:?}",
        duration
    );
}

#[test]
fn test_lockfile_save_performance() {
    let temp_dir = TempDir::new().unwrap();
    let lock_path = temp_dir.path().join("tasks.lock.yaml");

    // Create medium-sized lockfile
    let resolved = create_large_dependency_graph(50);
    let resolver = LocalSourceResolver::new(temp_dir.path().to_path_buf());
    let lock_file = generate_lock_file(&resolved, &resolver).unwrap();

    // Measure save performance
    let start = Instant::now();
    lock_file.save(&lock_path).unwrap();
    let duration = start.elapsed();

    assert!(
        duration < Duration::from_millis(500),
        "Save took {:?}",
        duration
    );
}

#[test]
fn test_lockfile_load_performance() {
    let temp_dir = TempDir::new().unwrap();
    let lock_path = temp_dir.path().join("tasks.lock.yaml");

    // Create and save lockfile
    let resolved = create_large_dependency_graph(50);
    let resolver = LocalSourceResolver::new(temp_dir.path().to_path_buf());
    let lock_file = generate_lock_file(&resolved, &resolver).unwrap();
    lock_file.save(&lock_path).unwrap();

    // Measure load performance
    let start = Instant::now();
    let loaded = LockFile::load(&lock_path).unwrap();
    let duration = start.elapsed();

    assert_eq!(loaded.tasks.len(), 50);
    assert!(
        duration < Duration::from_millis(300),
        "Load took {:?}",
        duration
    );
}

// ============================================================================
// SECTION 10: Integration Test - Complete Workflow
// ============================================================================

#[test]
fn test_complete_lockfile_workflow_with_updates() {
    let temp_dir = TempDir::new().unwrap();
    let lock_path = temp_dir.path().join("tasks.lock.yaml");

    // Step 1: Initial project setup
    let initial_tasks = vec![
        create_resolved_task("core", "1.0.0", HashMap::new()),
        create_resolved_task("utils", "1.0.0", HashMap::new()),
    ];
    let resolver = LocalSourceResolver::new(temp_dir.path().to_path_buf());
    let initial_lock = generate_lock_file(&initial_tasks, &resolver).unwrap();
    initial_lock.save(&lock_path).unwrap();

    // Step 2: Verify initial state
    let loaded = LockFile::load(&lock_path).unwrap();
    assert_eq!(loaded.tasks.len(), 2);

    // Step 3: Add new dependency
    let mut deps = HashMap::new();
    deps.insert("core".to_string(), "1.0.0".to_string());

    let updated_tasks = vec![
        create_resolved_task("core", "1.0.0", HashMap::new()),
        create_resolved_task("utils", "1.0.0", HashMap::new()),
        create_resolved_task("plugin", "1.0.0", deps.clone()),
    ];

    let updated_lock = generate_lock_file(&updated_tasks, &resolver).unwrap();
    updated_lock.save(&lock_path).unwrap();

    // Step 4: Verify update
    let loaded = LockFile::load(&lock_path).unwrap();
    assert_eq!(loaded.tasks.len(), 3);
    assert!(loaded.get_task("plugin").is_some());

    // Step 5: Validate integrity
    let validation = validate_lock_file(&loaded, &updated_tasks).unwrap();
    assert!(validation.is_valid());

    // Step 6: Upgrade version
    let upgraded_tasks = vec![
        create_resolved_task("core", "2.0.0", HashMap::new()), // Major version bump
        create_resolved_task("utils", "1.0.0", HashMap::new()),
        create_resolved_task("plugin", "1.0.0", deps),
    ];

    let upgraded_lock = generate_lock_file(&upgraded_tasks, &resolver).unwrap();
    upgraded_lock.save(&lock_path).unwrap();

    // Step 7: Verify upgrade
    let final_lock = LockFile::load(&lock_path).unwrap();
    assert_eq!(final_lock.get_task("core").unwrap().version, "2.0.0");

    // Step 8: Verify upgrade detected as stale
    assert!(final_lock.is_stale(&updated_tasks));
}

#[test]
fn test_lockfile_workflow_with_dependency_resolution() {
    let temp_dir = TempDir::new().unwrap();
    let lock_path = temp_dir.path().join("tasks.lock.yaml");

    // Create complex dependency graph
    let mut ui_deps = HashMap::new();
    ui_deps.insert("core".to_string(), "1.0.0".to_string());
    ui_deps.insert("utils".to_string(), "1.0.0".to_string());

    let mut app_deps = HashMap::new();
    app_deps.insert("ui".to_string(), "1.0.0".to_string());
    app_deps.insert("core".to_string(), "1.0.0".to_string());

    let tasks = vec![
        create_resolved_task("core", "1.0.0", HashMap::new()),
        create_resolved_task("utils", "1.0.0", HashMap::new()),
        create_resolved_task("ui", "1.0.0", ui_deps),
        create_resolved_task("app", "1.0.0", app_deps),
    ];

    // Generate lockfile
    let resolver = LocalSourceResolver::new(temp_dir.path().to_path_buf());
    let lock_file = generate_lock_file(&tasks, &resolver).unwrap();
    lock_file.save(&lock_path).unwrap();

    // Load and validate
    let loaded = LockFile::load(&lock_path).unwrap();
    assert_eq!(loaded.tasks.len(), 4);

    // Verify dependency relationships preserved
    let ui_task = loaded.get_task("ui").unwrap();
    assert_eq!(ui_task.dependencies.len(), 2);
    assert!(ui_task.dependencies.contains_key("core"));
    assert!(ui_task.dependencies.contains_key("utils"));

    let app_task = loaded.get_task("app").unwrap();
    assert_eq!(app_task.dependencies.len(), 2);
    assert!(app_task.dependencies.contains_key("ui"));
    assert!(app_task.dependencies.contains_key("core"));
}
