//! Task Group Loader
//!
//! This module handles discovering and loading task groups from the filesystem,
//! resolving all tasks within a group, and applying shared configuration.

use super::parser::{parse_task_group, ParseError};
use super::schema::{SharedConfig, TaskGroup, TaskGroupReference};
use crate::dsl::predefined_tasks::loader::TaskLoader;
use crate::dsl::predefined_tasks::schema::{PredefinedTask, TaskReference};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur during task group loading
#[derive(Debug, Error)]
pub enum GroupLoadError {
    /// IO error reading group file
    #[error("Failed to read task group file '{path}': {source}")]
    IoError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// Parse error
    #[error("Failed to parse task group file '{path}': {source}")]
    ParseError {
        path: String,
        #[source]
        source: ParseError,
    },

    /// Task group not found
    #[error("Task group not found: {0}")]
    GroupNotFound(String),

    /// Directory not found
    #[error("Task group directory not found: {0}")]
    DirectoryNotFound(String),

    /// Multiple groups with same name/version
    #[error("Duplicate task group found: {name}@{version} (paths: {path1}, {path2})")]
    DuplicateGroup {
        name: String,
        version: String,
        path1: String,
        path2: String,
    },

    /// Task not found in any source
    #[error("Task '{task}' (version {version}) required by group '{group}' not found")]
    TaskNotFound {
        group: String,
        task: String,
        version: String,
    },

    /// Version constraint violation
    #[error("Task '{task}' version mismatch: required {required}, found {found}")]
    VersionMismatch {
        task: String,
        required: String,
        found: String,
    },

    /// Required task missing
    #[error("Required task '{task}' missing from group '{group}'")]
    RequiredTaskMissing { group: String, task: String },
}

/// Resolved task group with all tasks loaded and configuration applied
#[derive(Debug, Clone)]
pub struct ResolvedTaskGroup {
    /// Original task group definition
    pub group: TaskGroup,

    /// Resolved tasks (task name -> PredefinedTask)
    pub tasks: HashMap<String, PredefinedTask>,

    /// File path where the group was loaded from
    pub source_path: PathBuf,
}

impl ResolvedTaskGroup {
    /// Get a task by name from the resolved group
    pub fn get_task(&self, name: &str) -> Option<&PredefinedTask> {
        self.tasks.get(name)
    }

    /// Get all task names in the group
    pub fn task_names(&self) -> Vec<String> {
        self.tasks.keys().cloned().collect()
    }

    /// Check if a task is in the group
    pub fn contains_task(&self, name: &str) -> bool {
        self.tasks.contains_key(name)
    }
}

/// Task group loader for discovering and loading task groups
pub struct TaskGroupLoader {
    /// Search paths in priority order (higher index = higher priority)
    pub search_paths: Vec<PathBuf>,

    /// Task loader for resolving individual tasks
    task_loader: TaskLoader,

    /// Cached loaded groups (key: "name@version")
    cache: HashMap<String, ResolvedTaskGroup>,
}

impl TaskGroupLoader {
    /// Create a new task group loader with default search paths
    ///
    /// Default search paths (in priority order):
    /// 1. `./.claude/task-groups/` (project local - highest priority)
    /// 2. `~/.claude/task-groups/` (user global)
    pub fn new() -> Self {
        let mut search_paths = Vec::new();

        // Add user global path
        if let Some(home) = dirs::home_dir() {
            search_paths.push(home.join(".claude").join("task-groups"));
        }

        // Add project local path (higher priority)
        search_paths.push(PathBuf::from(".").join(".claude").join("task-groups"));

        TaskGroupLoader {
            search_paths,
            task_loader: TaskLoader::new(),
            cache: HashMap::new(),
        }
    }

    /// Create a task group loader with custom search paths
    ///
    /// Paths are searched in the order provided (first = lowest priority, last = highest priority)
    pub fn with_paths(paths: Vec<PathBuf>) -> Self {
        TaskGroupLoader {
            search_paths: paths,
            task_loader: TaskLoader::new(),
            cache: HashMap::new(),
        }
    }

    /// Create a task group loader with custom task loader
    pub fn with_task_loader(task_loader: TaskLoader) -> Self {
        let mut search_paths = Vec::new();

        if let Some(home) = dirs::home_dir() {
            search_paths.push(home.join(".claude").join("task-groups"));
        }

        search_paths.push(PathBuf::from(".").join(".claude").join("task-groups"));

        TaskGroupLoader {
            search_paths,
            task_loader,
            cache: HashMap::new(),
        }
    }

    /// Add a search path (will be searched with highest priority)
    pub fn add_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    /// Discover all available task groups from all search paths
    ///
    /// Returns a map of group references to their file paths
    pub fn discover_all(&self) -> Result<HashMap<String, PathBuf>, GroupLoadError> {
        let mut discovered = HashMap::new();

        // Search paths in order (later paths override earlier ones due to priority)
        for search_path in &self.search_paths {
            if !search_path.exists() {
                continue;
            }

            let groups = discover_groups_in_directory(search_path)?;
            for (group_ref, path) in groups {
                discovered.insert(group_ref, path);
            }
        }

        Ok(discovered)
    }

    /// Load a specific task group by reference
    ///
    /// Searches all configured paths in priority order, resolves all tasks,
    /// and applies shared configuration.
    pub fn load(
        &mut self,
        group_ref: &TaskGroupReference,
    ) -> Result<ResolvedTaskGroup, GroupLoadError> {
        let cache_key = group_ref.to_string();

        // Check cache first
        if let Some(resolved) = self.cache.get(&cache_key) {
            return Ok(resolved.clone());
        }

        // Search for group in all paths (reverse order for priority)
        // Collect paths to avoid borrow checker issues
        let search_paths: Vec<_> = self.search_paths.iter().rev().cloned().collect();
        for search_path in search_paths {
            if let Ok(resolved) = self.load_group_from_directory(&search_path, group_ref) {
                self.cache.insert(cache_key.clone(), resolved.clone());
                return Ok(resolved);
            }
        }

        Err(GroupLoadError::GroupNotFound(group_ref.to_string()))
    }

    /// Load a task group from a specific file path
    pub fn load_from_file(&mut self, path: &Path) -> Result<ResolvedTaskGroup, GroupLoadError> {
        let group = load_task_group(path)?;
        let resolved = self.resolve_group_tasks(&group, path)?;
        Ok(resolved)
    }

    /// Load and resolve a task group from a directory
    fn load_group_from_directory(
        &mut self,
        dir: &Path,
        group_ref: &TaskGroupReference,
    ) -> Result<ResolvedTaskGroup, GroupLoadError> {
        if !dir.exists() || !dir.is_dir() {
            return Err(GroupLoadError::DirectoryNotFound(dir.display().to_string()));
        }

        // Try standard naming pattern: {name}.taskgroup.yaml
        let standard_path = dir.join(format!("{}.taskgroup.yaml", group_ref.name));
        if standard_path.exists() {
            let group = load_task_group(&standard_path)?;

            // Verify version matches
            if group.metadata.version == group_ref.version {
                return self.resolve_group_tasks(&group, &standard_path);
            }
        }

        // If standard path didn't work, scan directory for matching group
        let discovered = discover_groups_in_directory(dir)?;
        if let Some(path) = discovered.get(&group_ref.to_string()) {
            let group = load_task_group(path)?;
            return self.resolve_group_tasks(&group, path);
        }

        Err(GroupLoadError::GroupNotFound(group_ref.to_string()))
    }

    /// Resolve all tasks in a group and apply shared configuration
    fn resolve_group_tasks(
        &mut self,
        group: &TaskGroup,
        source_path: &Path,
    ) -> Result<ResolvedTaskGroup, GroupLoadError> {
        let mut resolved_tasks = HashMap::new();

        // Resolve each task in the group
        for task_ref in &group.spec.tasks {
            let task_reference = TaskReference {
                name: task_ref.name.clone(),
                version: task_ref.version.clone(),
            };

            // Try to load the task
            let mut task = self.task_loader.load(&task_reference).map_err(|_| {
                GroupLoadError::TaskNotFound {
                    group: group.metadata.name.clone(),
                    task: task_ref.name.clone(),
                    version: task_ref.version.clone(),
                }
            })?;

            // Verify the task meets requirements
            if task.metadata.version != task_ref.version {
                return Err(GroupLoadError::VersionMismatch {
                    task: task_ref.name.clone(),
                    required: task_ref.version.clone(),
                    found: task.metadata.version.clone(),
                });
            }

            // Apply shared configuration if present
            if let Some(ref shared_config) = group.spec.shared_config {
                apply_shared_config(&mut task, shared_config);
            }

            resolved_tasks.insert(task_ref.name.clone(), task);
        }

        // Verify all required tasks are present
        for task_ref in &group.spec.tasks {
            if task_ref.required && !resolved_tasks.contains_key(&task_ref.name) {
                return Err(GroupLoadError::RequiredTaskMissing {
                    group: group.metadata.name.clone(),
                    task: task_ref.name.clone(),
                });
            }
        }

        Ok(ResolvedTaskGroup {
            group: group.clone(),
            tasks: resolved_tasks,
            source_path: source_path.to_path_buf(),
        })
    }

    /// Clear the group cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get the cached groups
    pub fn cached_groups(&self) -> Vec<String> {
        self.cache.keys().cloned().collect()
    }

    /// Get mutable reference to the task loader
    pub fn task_loader_mut(&mut self) -> &mut TaskLoader {
        &mut self.task_loader
    }
}

impl Default for TaskGroupLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Load a task group from a file
pub fn load_task_group(path: &Path) -> Result<TaskGroup, GroupLoadError> {
    let yaml_content = fs::read_to_string(path).map_err(|source| GroupLoadError::IoError {
        path: path.display().to_string(),
        source,
    })?;

    parse_task_group(&yaml_content).map_err(|source| GroupLoadError::ParseError {
        path: path.display().to_string(),
        source,
    })
}

/// Discover all task groups in a directory
fn discover_groups_in_directory(dir: &Path) -> Result<HashMap<String, PathBuf>, GroupLoadError> {
    let mut groups = HashMap::new();

    if !dir.exists() {
        return Ok(groups);
    }

    if !dir.is_dir() {
        return Err(GroupLoadError::DirectoryNotFound(dir.display().to_string()));
    }

    // Walk through directory (non-recursive for Phase 1)
    let entries = fs::read_dir(dir).map_err(|source| GroupLoadError::IoError {
        path: dir.display().to_string(),
        source,
    })?;

    for entry in entries {
        let entry = entry.map_err(|source| GroupLoadError::IoError {
            path: dir.display().to_string(),
            source,
        })?;

        let path = entry.path();

        // Only process .taskgroup.yaml files
        if !is_task_group_file(&path) {
            continue;
        }

        // Try to load the group to get its metadata
        match load_task_group(&path) {
            Ok(group) => {
                let group_ref = format!("{}@{}", group.metadata.name, group.metadata.version);

                // Check for duplicates
                if let Some(existing_path) = groups.get(&group_ref) {
                    return Err(GroupLoadError::DuplicateGroup {
                        name: group.metadata.name,
                        version: group.metadata.version,
                        path1: existing_path.display().to_string(),
                        path2: path.display().to_string(),
                    });
                }

                groups.insert(group_ref, path);
            }
            Err(e) => {
                // Log warning but continue discovery
                eprintln!(
                    "Warning: Failed to load task group from {}: {}",
                    path.display(),
                    e
                );
            }
        }
    }

    Ok(groups)
}

/// Check if a path is a task group file (ends with .taskgroup.yaml)
fn is_task_group_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.ends_with(".taskgroup.yaml"))
        .unwrap_or(false)
}

/// Apply shared configuration to a task
fn apply_shared_config(task: &mut PredefinedTask, shared_config: &SharedConfig) {
    // Merge shared inputs (task-specific inputs take precedence)
    for (key, value) in &shared_config.inputs {
        task.spec.inputs.entry(key.clone()).or_insert_with(|| {
            crate::dsl::predefined_tasks::schema::PredefinedTaskInputSpec {
                base: value.clone(),
                validation: None,
                source: None,
            }
        });
    }

    // Apply shared permissions if task doesn't have specific permissions set
    if let Some(ref shared_perms) = shared_config.permissions {
        // Only apply if the task has default/minimal permissions
        if task.spec.agent_template.permissions.mode == "default" {
            task.spec.agent_template.permissions = shared_perms.clone();
        }
    }

    // Apply shared max_turns if not set
    if task.spec.agent_template.max_turns.is_none() {
        task.spec.agent_template.max_turns = shared_config.max_turns;
    }

    // Note: Shared environment variables would be applied at runtime
    // during workflow execution, not during task loading
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_task_file(dir: &Path, name: &str, version: &str) -> PathBuf {
        let content = format!(
            r#"
apiVersion: "task/v1"
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

    fn create_test_group_file(
        dir: &Path,
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
            r#"
apiVersion: "taskgroup/v1"
kind: "TaskGroup"
metadata:
  name: "{}"
  version: "{}"
  description: "Test group"
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

    #[test]
    fn test_is_task_group_file() {
        let temp_dir = TempDir::new().unwrap();
        let group_file = temp_dir.path().join("test.taskgroup.yaml");
        let other_file = temp_dir.path().join("test.yaml");

        fs::write(&group_file, "").unwrap();
        fs::write(&other_file, "").unwrap();

        assert!(is_task_group_file(&group_file));
        assert!(!is_task_group_file(&other_file));
        assert!(!is_task_group_file(temp_dir.path()));
    }

    #[test]
    fn test_discover_task_groups() {
        let temp_dir = TempDir::new().unwrap();
        create_test_group_file(temp_dir.path(), "group1", "1.0.0", vec![("task1", "1.0.0")]);
        create_test_group_file(temp_dir.path(), "group2", "2.0.0", vec![("task2", "2.0.0")]);

        let discovered = discover_groups_in_directory(temp_dir.path()).unwrap();

        assert_eq!(discovered.len(), 2);
        assert!(discovered.contains_key("group1@1.0.0"));
        assert!(discovered.contains_key("group2@2.0.0"));
    }

    #[test]
    fn test_load_task_group_with_tasks() {
        let temp_dir = TempDir::new().unwrap();
        let task_dir = TempDir::new().unwrap();

        // Create tasks
        create_test_task_file(task_dir.path(), "task1", "1.0.0");
        create_test_task_file(task_dir.path(), "task2", "2.0.0");

        // Create group that references these tasks
        create_test_group_file(
            temp_dir.path(),
            "my-group",
            "1.0.0",
            vec![("task1", "1.0.0"), ("task2", "2.0.0")],
        );

        // Create loader with custom task loader
        let task_loader = TaskLoader::with_paths(vec![task_dir.path().to_path_buf()]);
        let mut loader = TaskGroupLoader::with_task_loader(task_loader);
        loader.add_path(temp_dir.path().to_path_buf());

        // Load the group
        let group_ref = TaskGroupReference::parse("my-group@1.0.0").unwrap();
        let resolved = loader.load(&group_ref).unwrap();

        assert_eq!(resolved.group.metadata.name, "my-group");
        assert_eq!(resolved.group.metadata.version, "1.0.0");
        assert_eq!(resolved.tasks.len(), 2);
        assert!(resolved.contains_task("task1"));
        assert!(resolved.contains_task("task2"));
    }

    #[test]
    fn test_load_task_group_missing_task() {
        let temp_dir = TempDir::new().unwrap();
        let task_dir = TempDir::new().unwrap();

        // Only create one task
        create_test_task_file(task_dir.path(), "task1", "1.0.0");

        // Create group that references two tasks (one missing)
        create_test_group_file(
            temp_dir.path(),
            "my-group",
            "1.0.0",
            vec![("task1", "1.0.0"), ("task2", "2.0.0")],
        );

        let task_loader = TaskLoader::with_paths(vec![task_dir.path().to_path_buf()]);
        let mut loader = TaskGroupLoader::with_task_loader(task_loader);
        loader.add_path(temp_dir.path().to_path_buf());

        let group_ref = TaskGroupReference::parse("my-group@1.0.0").unwrap();
        let result = loader.load(&group_ref);

        assert!(result.is_err());
        let err = result.unwrap_err();
        // May be GroupNotFound if discovery fails, or TaskNotFound if group loads but task missing
        assert!(matches!(
            err,
            GroupLoadError::TaskNotFound { .. } | GroupLoadError::GroupNotFound(_)
        ));
    }

    #[test]
    fn test_cache() {
        let temp_dir = TempDir::new().unwrap();
        let task_dir = TempDir::new().unwrap();

        create_test_task_file(task_dir.path(), "task1", "1.0.0");
        create_test_group_file(
            temp_dir.path(),
            "my-group",
            "1.0.0",
            vec![("task1", "1.0.0")],
        );

        let task_loader = TaskLoader::with_paths(vec![task_dir.path().to_path_buf()]);
        let mut loader = TaskGroupLoader::with_task_loader(task_loader);
        loader.add_path(temp_dir.path().to_path_buf());

        let group_ref = TaskGroupReference::parse("my-group@1.0.0").unwrap();

        // Load once
        loader.load(&group_ref).unwrap();
        assert_eq!(loader.cached_groups().len(), 1);

        // Load again (should use cache)
        loader.load(&group_ref).unwrap();
        assert_eq!(loader.cached_groups().len(), 1);

        // Clear cache
        loader.clear_cache();
        assert_eq!(loader.cached_groups().len(), 0);
    }

    #[test]
    fn test_shared_config_application() {
        use crate::dsl::schema::InputSpec;

        let mut task = PredefinedTask {
            api_version: crate::dsl::predefined_tasks::schema::TaskApiVersion::V1,
            kind: crate::dsl::predefined_tasks::schema::TaskKind::PredefinedTask,
            metadata: crate::dsl::predefined_tasks::schema::PredefinedTaskMetadata {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                author: None,
                description: None,
                license: None,
                repository: None,
                tags: vec![],
            },
            spec: crate::dsl::predefined_tasks::schema::PredefinedTaskSpec {
                agent_template: crate::dsl::predefined_tasks::schema::AgentTemplate {
                    description: "Test".to_string(),
                    model: None,
                    system_prompt: None,
                    tools: vec![],
                    permissions: Default::default(),
                    max_turns: None,
                },
                inputs: HashMap::new(),
                outputs: HashMap::new(),
                dependencies: vec![],
                examples: vec![],
            },
        };

        let mut shared_inputs = HashMap::new();
        shared_inputs.insert(
            "api_key".to_string(),
            InputSpec {
                param_type: "string".to_string(),
                required: true,
                default: None,
                description: Some("API Key".to_string()),
            },
        );

        let shared_config = SharedConfig {
            inputs: shared_inputs,
            permissions: None,
            environment: HashMap::new(),
            max_turns: Some(10),
        };

        apply_shared_config(&mut task, &shared_config);

        // Check shared input was added
        assert!(task.spec.inputs.contains_key("api_key"));

        // Check max_turns was applied
        assert_eq!(task.spec.agent_template.max_turns, Some(10));
    }
}
