//! Lock File Support for Predefined Tasks
//!
//! This module provides lock file functionality to ensure reproducible dependency resolution
//! across different environments and time periods. Lock files capture the exact resolved
//! versions, checksums, and sources of all dependencies.
//!
//! # Overview
//!
//! Lock files serve multiple purposes:
//! - **Reproducibility**: Ensure the same dependency versions are used across different machines
//! - **Integrity**: Verify task content hasn't changed via checksums
//! - **Transparency**: Track where each task came from (local, git, registry)
//! - **Performance**: Skip resolution when lock file is valid
//!
//! # Lock File Format
//!
//! ```yaml
//! # .claude/tasks.lock.yaml
//! version: "1.0.0"
//! generated_at: "2025-01-15T10:30:00Z"
//! generated_by: "periplon@0.1.0"
//!
//! tasks:
//!   google-drive-upload:
//!     version: "1.2.0"
//!     checksum: "sha256:abc123..."
//!     source:
//!       type: "local"
//!       path: "./.claude/tasks/google-drive-upload.task.yaml"
//!     resolved_at: "2025-01-15T10:30:00Z"
//!     dependencies:
//!       auth-helper: "2.1.0"
//!
//!   auth-helper:
//!     version: "2.1.0"
//!     checksum: "sha256:def456..."
//!     source:
//!       type: "git"
//!       url: "https://github.com/org/tasks.git"
//!       ref: "v2.1.0"
//!       subpath: "auth-helper"
//!     resolved_at: "2025-01-15T10:30:00Z"
//!     dependencies: {}
//! ```

use super::deps::ResolvedTask;
use super::schema::PredefinedTask;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Lock file format version
pub const LOCK_FILE_VERSION: &str = "1.0.0";

/// Default lock file name
pub const LOCK_FILE_NAME: &str = "tasks.lock.yaml";

/// Errors that can occur during lock file operations
#[derive(Debug, Error)]
pub enum LockFileError {
    /// Failed to read lock file
    #[error("Failed to read lock file: {0}")]
    ReadError(#[from] std::io::Error),

    /// Failed to parse lock file YAML
    #[error("Failed to parse lock file: {0}")]
    ParseError(#[from] serde_yaml::Error),

    /// Lock file version incompatible
    #[error("Lock file version '{found}' is incompatible with current version '{expected}'")]
    IncompatibleVersion { found: String, expected: String },

    /// Checksum verification failed
    #[error("Checksum verification failed for task '{task}': expected {expected}, got {actual}")]
    ChecksumMismatch {
        task: String,
        expected: String,
        actual: String,
    },

    /// Task missing from lock file
    #[error("Task '{0}' not found in lock file")]
    TaskNotFound(String),

    /// Lock file is stale (dependencies changed)
    #[error("Lock file is stale: {0}")]
    Stale(String),

    /// Invalid lock file format
    #[error("Invalid lock file format: {0}")]
    InvalidFormat(String),

    /// Failed to generate checksum
    #[error("Failed to generate checksum: {0}")]
    ChecksumError(String),

    /// Source tracking error
    #[error("Source tracking error: {0}")]
    SourceError(String),
}

/// Complete lock file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockFile {
    /// Lock file format version
    pub version: String,

    /// When the lock file was generated
    pub generated_at: DateTime<Utc>,

    /// Tool and version that generated the lock file
    pub generated_by: String,

    /// Locked tasks with their resolved information
    pub tasks: HashMap<String, LockedTask>,

    /// Metadata about the lock file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<LockFileMetadata>,
}

/// Metadata about the lock file
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LockFileMetadata {
    /// Original workflow or package that created this lock file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_workflow: Option<String>,

    /// Number of tasks in the lock file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_count: Option<usize>,

    /// Custom metadata fields
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

/// A locked task entry with resolved version and verification data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedTask {
    /// Exact resolved version
    pub version: String,

    /// Content checksum for integrity verification
    pub checksum: String,

    /// Source information for this task
    pub source: TaskSource,

    /// When this task was resolved
    pub resolved_at: DateTime<Utc>,

    /// Resolved dependencies (name -> version)
    pub dependencies: HashMap<String, String>,

    /// Optional task metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<LockedTaskMetadata>,
}

/// Metadata about a locked task
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LockedTaskMetadata {
    /// Task description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Task author
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Task license
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

/// Source information for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TaskSource {
    /// Local filesystem source
    Local {
        /// Absolute or relative path to the task file
        path: String,
    },

    /// Git repository source
    Git {
        /// Git repository URL
        url: String,
        /// Git reference (branch, tag, or commit)
        #[serde(rename = "ref")]
        git_ref: String,
        /// Optional subpath within the repository
        #[serde(default, skip_serializing_if = "Option::is_none")]
        subpath: Option<String>,
    },

    /// Registry source (for future use)
    Registry {
        /// Registry URL
        url: String,
        /// Package identifier
        package: String,
    },
}

impl LockFile {
    /// Create a new empty lock file
    pub fn new() -> Self {
        Self {
            version: LOCK_FILE_VERSION.to_string(),
            generated_at: Utc::now(),
            generated_by: format!("periplon@{}", env!("CARGO_PKG_VERSION")),
            tasks: HashMap::new(),
            metadata: None,
        }
    }

    /// Load a lock file from disk
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, LockFileError> {
        let content = fs::read_to_string(path)?;
        let lock_file: LockFile = serde_yaml::from_str(&content)?;

        // Validate version compatibility
        if !is_compatible_version(&lock_file.version, LOCK_FILE_VERSION) {
            return Err(LockFileError::IncompatibleVersion {
                found: lock_file.version.clone(),
                expected: LOCK_FILE_VERSION.to_string(),
            });
        }

        Ok(lock_file)
    }

    /// Save the lock file to disk
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), LockFileError> {
        let content = serde_yaml::to_string(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Add a locked task entry
    pub fn add_task(&mut self, name: String, locked_task: LockedTask) {
        self.tasks.insert(name, locked_task);

        // Update metadata
        if let Some(ref mut metadata) = self.metadata {
            metadata.task_count = Some(self.tasks.len());
        }
    }

    /// Get a locked task by name
    pub fn get_task(&self, name: &str) -> Option<&LockedTask> {
        self.tasks.get(name)
    }

    /// Verify a task's checksum against its current content
    pub fn verify_task(&self, name: &str, task: &PredefinedTask) -> Result<(), LockFileError> {
        let locked = self
            .get_task(name)
            .ok_or_else(|| LockFileError::TaskNotFound(name.to_string()))?;

        let current_checksum = compute_task_checksum(task)?;

        if locked.checksum != current_checksum {
            return Err(LockFileError::ChecksumMismatch {
                task: name.to_string(),
                expected: locked.checksum.clone(),
                actual: current_checksum,
            });
        }

        Ok(())
    }

    /// Verify all tasks in the lock file
    pub fn verify_all(&self, tasks: &HashMap<String, PredefinedTask>) -> Result<(), LockFileError> {
        for (name, task) in tasks {
            self.verify_task(name, task)?;
        }
        Ok(())
    }

    /// Check if the lock file is stale compared to current dependencies
    pub fn is_stale(&self, resolved_tasks: &[ResolvedTask]) -> bool {
        // Check if all resolved tasks are in the lock file with matching versions
        for resolved in resolved_tasks {
            match self.get_task(&resolved.name) {
                None => return true, // Task not in lock file
                Some(locked) => {
                    if locked.version != resolved.version {
                        return true; // Version mismatch
                    }
                    // Check dependencies match
                    if locked.dependencies != resolved.dependencies {
                        return true; // Dependency mismatch
                    }
                }
            }
        }

        // Check for extra tasks in lock file (might be okay, but flag it)
        if self.tasks.len() != resolved_tasks.len() {
            return true;
        }

        false
    }

    /// Update the generation timestamp
    pub fn update_timestamp(&mut self) {
        self.generated_at = Utc::now();
    }

    /// Get all task names in the lock file
    pub fn task_names(&self) -> Vec<String> {
        self.tasks.keys().cloned().collect()
    }

    /// Get the number of tasks
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }
}

impl Default for LockFile {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a lock file from resolved tasks
pub fn generate_lock_file(
    resolved_tasks: &[ResolvedTask],
    source_resolver: &impl SourceResolver,
) -> Result<LockFile, LockFileError> {
    let mut lock_file = LockFile::new();

    for resolved in resolved_tasks {
        let locked_task = create_locked_task(resolved, source_resolver)?;
        lock_file.add_task(resolved.name.clone(), locked_task);
    }

    // Add metadata
    lock_file.metadata = Some(LockFileMetadata {
        source_workflow: None,
        task_count: Some(lock_file.tasks.len()),
        custom: HashMap::new(),
    });

    Ok(lock_file)
}

/// Create a locked task entry from a resolved task
fn create_locked_task(
    resolved: &ResolvedTask,
    source_resolver: &impl SourceResolver,
) -> Result<LockedTask, LockFileError> {
    let checksum = compute_task_checksum(&resolved.task)?;
    let source = source_resolver.resolve_source(&resolved.name, &resolved.version)?;

    let metadata = Some(LockedTaskMetadata {
        description: resolved.task.metadata.description.clone(),
        author: resolved.task.metadata.author.clone(),
        license: resolved.task.metadata.license.clone(),
    });

    Ok(LockedTask {
        version: resolved.version.clone(),
        checksum,
        source,
        resolved_at: Utc::now(),
        dependencies: resolved.dependencies.clone(),
        metadata,
    })
}

/// Compute a checksum for a task
pub fn compute_task_checksum(task: &PredefinedTask) -> Result<String, LockFileError> {
    // Serialize task to canonical YAML for consistent hashing
    let yaml = serde_yaml::to_string(task)
        .map_err(|e| LockFileError::ChecksumError(format!("Failed to serialize task: {}", e)))?;

    let mut hasher = Sha256::new();
    hasher.update(yaml.as_bytes());
    let result = hasher.finalize();

    Ok(format!("sha256:{}", hex::encode(result)))
}

/// Trait for resolving task sources
pub trait SourceResolver {
    /// Resolve the source information for a task
    fn resolve_source(&self, name: &str, version: &str) -> Result<TaskSource, LockFileError>;
}

/// Default source resolver that assumes local sources
pub struct LocalSourceResolver {
    base_path: PathBuf,
}

impl LocalSourceResolver {
    /// Create a new local source resolver
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }
}

impl SourceResolver for LocalSourceResolver {
    fn resolve_source(&self, name: &str, _version: &str) -> Result<TaskSource, LockFileError> {
        let task_path = self.base_path.join(format!("{}.task.yaml", name));

        let path_str = task_path
            .to_str()
            .ok_or_else(|| LockFileError::SourceError(format!("Invalid path for task '{}'", name)))?
            .to_string();

        Ok(TaskSource::Local { path: path_str })
    }
}

/// Check version compatibility (major version must match)
fn is_compatible_version(found: &str, expected: &str) -> bool {
    let found_parts: Vec<&str> = found.split('.').collect();
    let expected_parts: Vec<&str> = expected.split('.').collect();

    if found_parts.is_empty() || expected_parts.is_empty() {
        return false;
    }

    // Major version must match
    found_parts[0] == expected_parts[0]
}

/// Validate a lock file against current resolved tasks
pub fn validate_lock_file(
    lock_file: &LockFile,
    resolved_tasks: &[ResolvedTask],
) -> Result<ValidationResult, LockFileError> {
    let mut issues = Vec::new();

    // Check for missing tasks
    for resolved in resolved_tasks {
        match lock_file.get_task(&resolved.name) {
            None => {
                issues.push(ValidationIssue::MissingTask {
                    name: resolved.name.clone(),
                });
            }
            Some(locked) => {
                // Check version match
                if locked.version != resolved.version {
                    issues.push(ValidationIssue::VersionMismatch {
                        name: resolved.name.clone(),
                        locked: locked.version.clone(),
                        resolved: resolved.version.clone(),
                    });
                }

                // Check dependencies
                if locked.dependencies != resolved.dependencies {
                    issues.push(ValidationIssue::DependencyMismatch {
                        name: resolved.name.clone(),
                    });
                }

                // Verify checksum
                if let Err(e) = lock_file.verify_task(&resolved.name, &resolved.task) {
                    issues.push(ValidationIssue::ChecksumFailed {
                        name: resolved.name.clone(),
                        error: e.to_string(),
                    });
                }
            }
        }
    }

    // Check for extra tasks in lock file
    for name in lock_file.tasks.keys() {
        if !resolved_tasks.iter().any(|r| &r.name == name) {
            issues.push(ValidationIssue::ExtraTask { name: name.clone() });
        }
    }

    Ok(ValidationResult { issues })
}

/// Result of lock file validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// List of validation issues found
    pub issues: Vec<ValidationIssue>,
}

impl ValidationResult {
    /// Check if validation passed (no issues)
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }

    /// Get a summary of validation issues
    pub fn summary(&self) -> String {
        if self.is_valid() {
            return "Lock file is valid".to_string();
        }

        let mut summary = format!(
            "Lock file validation failed with {} issue(s):\n",
            self.issues.len()
        );
        for issue in &self.issues {
            summary.push_str(&format!("  - {}\n", issue));
        }
        summary
    }
}

/// Validation issue types
#[derive(Debug, Clone)]
pub enum ValidationIssue {
    /// Task missing from lock file
    MissingTask { name: String },

    /// Task in lock file but not in resolved tasks
    ExtraTask { name: String },

    /// Version mismatch
    VersionMismatch {
        name: String,
        locked: String,
        resolved: String,
    },

    /// Dependency mismatch
    DependencyMismatch { name: String },

    /// Checksum verification failed
    ChecksumFailed { name: String, error: String },
}

impl std::fmt::Display for ValidationIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationIssue::MissingTask { name } => {
                write!(f, "Task '{}' missing from lock file", name)
            }
            ValidationIssue::ExtraTask { name } => {
                write!(f, "Task '{}' in lock file but not in dependencies", name)
            }
            ValidationIssue::VersionMismatch {
                name,
                locked,
                resolved,
            } => {
                write!(
                    f,
                    "Version mismatch for '{}': locked={}, resolved={}",
                    name, locked, resolved
                )
            }
            ValidationIssue::DependencyMismatch { name } => {
                write!(f, "Dependency mismatch for '{}'", name)
            }
            ValidationIssue::ChecksumFailed { name, error } => {
                write!(f, "Checksum failed for '{}': {}", name, error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::predefined_tasks::{
        AgentTemplate, PredefinedTaskMetadata, PredefinedTaskSpec, TaskApiVersion, TaskKind,
    };
    use crate::dsl::schema::PermissionsSpec;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_task(name: &str, version: &str) -> PredefinedTask {
        PredefinedTask {
            api_version: TaskApiVersion::V1,
            kind: TaskKind::PredefinedTask,
            metadata: PredefinedTaskMetadata {
                name: name.to_string(),
                version: version.to_string(),
                description: Some("Test task".to_string()),
                author: Some("Test Author".to_string()),
                license: Some("MIT".to_string()),
                repository: None,
                tags: vec![],
            },
            spec: PredefinedTaskSpec {
                agent_template: AgentTemplate {
                    description: "Test task".to_string(),
                    model: None,
                    system_prompt: None,
                    tools: vec![],
                    permissions: PermissionsSpec::default(),
                    max_turns: None,
                },
                inputs: HashMap::new(),
                outputs: HashMap::new(),
                dependencies: vec![],
                examples: vec![],
            },
        }
    }

    fn create_test_resolved_task(name: &str, version: &str) -> ResolvedTask {
        ResolvedTask {
            name: name.to_string(),
            version: version.to_string(),
            task: create_test_task(name, version),
            dependencies: HashMap::new(),
        }
    }

    #[test]
    fn test_lock_file_new() {
        let lock_file = LockFile::new();
        assert_eq!(lock_file.version, LOCK_FILE_VERSION);
        assert_eq!(lock_file.tasks.len(), 0);
    }

    #[test]
    fn test_lock_file_add_task() {
        let mut lock_file = LockFile::new();

        let locked_task = LockedTask {
            version: "1.0.0".to_string(),
            checksum: "sha256:abc123".to_string(),
            source: TaskSource::Local {
                path: "./task.yaml".to_string(),
            },
            resolved_at: Utc::now(),
            dependencies: HashMap::new(),
            metadata: None,
        };

        lock_file.add_task("test-task".to_string(), locked_task);
        assert_eq!(lock_file.tasks.len(), 1);
        assert!(lock_file.get_task("test-task").is_some());
    }

    #[test]
    fn test_lock_file_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let lock_path = temp_dir.path().join("tasks.lock.yaml");

        let mut lock_file = LockFile::new();
        lock_file.add_task(
            "test-task".to_string(),
            LockedTask {
                version: "1.0.0".to_string(),
                checksum: "sha256:abc123".to_string(),
                source: TaskSource::Local {
                    path: "./task.yaml".to_string(),
                },
                resolved_at: Utc::now(),
                dependencies: HashMap::new(),
                metadata: None,
            },
        );

        // Save
        lock_file.save(&lock_path).unwrap();
        assert!(lock_path.exists());

        // Load
        let loaded = LockFile::load(&lock_path).unwrap();
        assert_eq!(loaded.version, LOCK_FILE_VERSION);
        assert_eq!(loaded.tasks.len(), 1);
        assert!(loaded.get_task("test-task").is_some());
    }

    #[test]
    fn test_compute_task_checksum() {
        let task = create_test_task("test", "1.0.0");
        let checksum = compute_task_checksum(&task).unwrap();

        assert!(checksum.starts_with("sha256:"));
        assert!(checksum.len() > 7); // "sha256:" + hex digest

        // Same task should produce same checksum
        let checksum2 = compute_task_checksum(&task).unwrap();
        assert_eq!(checksum, checksum2);
    }

    #[test]
    fn test_verify_task_success() {
        let task = create_test_task("test", "1.0.0");
        let checksum = compute_task_checksum(&task).unwrap();

        let mut lock_file = LockFile::new();
        lock_file.add_task(
            "test".to_string(),
            LockedTask {
                version: "1.0.0".to_string(),
                checksum,
                source: TaskSource::Local {
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
    fn test_verify_task_checksum_mismatch() {
        let task = create_test_task("test", "1.0.0");

        let mut lock_file = LockFile::new();
        lock_file.add_task(
            "test".to_string(),
            LockedTask {
                version: "1.0.0".to_string(),
                checksum: "sha256:wrongchecksum".to_string(),
                source: TaskSource::Local {
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
    fn test_is_stale_no_changes() {
        let resolved = vec![create_test_resolved_task("test", "1.0.0")];
        let checksum = compute_task_checksum(&resolved[0].task).unwrap();

        let mut lock_file = LockFile::new();
        lock_file.add_task(
            "test".to_string(),
            LockedTask {
                version: "1.0.0".to_string(),
                checksum,
                source: TaskSource::Local {
                    path: "./test.yaml".to_string(),
                },
                resolved_at: Utc::now(),
                dependencies: HashMap::new(),
                metadata: None,
            },
        );

        assert!(!lock_file.is_stale(&resolved));
    }

    #[test]
    fn test_is_stale_version_changed() {
        let resolved = vec![create_test_resolved_task("test", "2.0.0")];
        let checksum = compute_task_checksum(&resolved[0].task).unwrap();

        let mut lock_file = LockFile::new();
        lock_file.add_task(
            "test".to_string(),
            LockedTask {
                version: "1.0.0".to_string(), // Old version
                checksum,
                source: TaskSource::Local {
                    path: "./test.yaml".to_string(),
                },
                resolved_at: Utc::now(),
                dependencies: HashMap::new(),
                metadata: None,
            },
        );

        assert!(lock_file.is_stale(&resolved));
    }

    #[test]
    fn test_is_stale_task_added() {
        let resolved = vec![
            create_test_resolved_task("test1", "1.0.0"),
            create_test_resolved_task("test2", "1.0.0"),
        ];

        let mut lock_file = LockFile::new();
        let checksum = compute_task_checksum(&resolved[0].task).unwrap();
        lock_file.add_task(
            "test1".to_string(),
            LockedTask {
                version: "1.0.0".to_string(),
                checksum,
                source: TaskSource::Local {
                    path: "./test1.yaml".to_string(),
                },
                resolved_at: Utc::now(),
                dependencies: HashMap::new(),
                metadata: None,
            },
        );

        assert!(lock_file.is_stale(&resolved));
    }

    #[test]
    fn test_generate_lock_file() {
        struct TestResolver;
        impl SourceResolver for TestResolver {
            fn resolve_source(
                &self,
                name: &str,
                _version: &str,
            ) -> Result<TaskSource, LockFileError> {
                Ok(TaskSource::Local {
                    path: format!("./{}.yaml", name),
                })
            }
        }

        let resolved = vec![create_test_resolved_task("test", "1.0.0")];
        let lock_file = generate_lock_file(&resolved, &TestResolver).unwrap();

        assert_eq!(lock_file.tasks.len(), 1);
        assert!(lock_file.get_task("test").is_some());

        let locked = lock_file.get_task("test").unwrap();
        assert_eq!(locked.version, "1.0.0");
        assert!(locked.checksum.starts_with("sha256:"));
    }

    #[test]
    fn test_validate_lock_file_valid() {
        struct TestResolver;
        impl SourceResolver for TestResolver {
            fn resolve_source(
                &self,
                name: &str,
                _version: &str,
            ) -> Result<TaskSource, LockFileError> {
                Ok(TaskSource::Local {
                    path: format!("./{}.yaml", name),
                })
            }
        }

        let resolved = vec![create_test_resolved_task("test", "1.0.0")];
        let lock_file = generate_lock_file(&resolved, &TestResolver).unwrap();

        let result = validate_lock_file(&lock_file, &resolved).unwrap();
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_lock_file_version_mismatch() {
        struct TestResolver;
        impl SourceResolver for TestResolver {
            fn resolve_source(
                &self,
                name: &str,
                _version: &str,
            ) -> Result<TaskSource, LockFileError> {
                Ok(TaskSource::Local {
                    path: format!("./{}.yaml", name),
                })
            }
        }

        let resolved_old = vec![create_test_resolved_task("test", "1.0.0")];
        let lock_file = generate_lock_file(&resolved_old, &TestResolver).unwrap();

        let resolved_new = vec![create_test_resolved_task("test", "2.0.0")];
        let result = validate_lock_file(&lock_file, &resolved_new).unwrap();

        assert!(!result.is_valid());
        // Should have 2 issues: version mismatch and checksum failure (due to different version in metadata)
        assert_eq!(result.issues.len(), 2);

        // Check that we have both expected issue types
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
    fn test_is_compatible_version() {
        assert!(is_compatible_version("1.0.0", "1.0.0"));
        assert!(is_compatible_version("1.0.0", "1.1.0"));
        assert!(is_compatible_version("1.1.0", "1.0.0"));
        assert!(!is_compatible_version("1.0.0", "2.0.0"));
        assert!(!is_compatible_version("2.0.0", "1.0.0"));
    }

    #[test]
    fn test_task_source_serialization() {
        let local = TaskSource::Local {
            path: "./task.yaml".to_string(),
        };
        let yaml = serde_yaml::to_string(&local).unwrap();
        assert!(yaml.contains("type: local"));
        assert!(yaml.contains("path: ./task.yaml"));

        let git = TaskSource::Git {
            url: "https://github.com/org/repo.git".to_string(),
            git_ref: "v1.0.0".to_string(),
            subpath: Some("tasks".to_string()),
        };
        let yaml = serde_yaml::to_string(&git).unwrap();
        assert!(yaml.contains("type: git"));
        assert!(yaml.contains("url: https://github.com/org/repo.git"));
    }

    #[test]
    fn test_local_source_resolver() {
        let temp_dir = TempDir::new().unwrap();
        let resolver = LocalSourceResolver::new(temp_dir.path().to_path_buf());

        let source = resolver.resolve_source("test-task", "1.0.0").unwrap();
        match source {
            TaskSource::Local { path } => {
                assert!(path.contains("test-task.task.yaml"));
            }
            _ => panic!("Expected local source"),
        }
    }
}
