//! Predefined Task Loader
//!
//! This module handles discovering and loading predefined tasks from the filesystem.

use super::parser::{parse_predefined_task, ParseError};
use super::schema::{PredefinedTask, TaskReference};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur during task loading
#[derive(Debug, Error)]
pub enum LoadError {
    /// IO error reading task file
    #[error("Failed to read task file '{path}': {source}")]
    IoError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// Parse error
    #[error("Failed to parse task file '{path}': {source}")]
    ParseError {
        path: String,
        #[source]
        source: ParseError,
    },

    /// Task not found
    #[error("Task not found: {0}")]
    TaskNotFound(String),

    /// Directory not found
    #[error("Task directory not found: {0}")]
    DirectoryNotFound(String),

    /// Multiple tasks with same name/version
    #[error("Duplicate task found: {name}@{version} (paths: {path1}, {path2})")]
    DuplicateTask {
        name: String,
        version: String,
        path1: String,
        path2: String,
    },
}

/// Task loader for discovering and loading predefined tasks
pub struct TaskLoader {
    /// Search paths in priority order (higher index = higher priority)
    search_paths: Vec<PathBuf>,

    /// Cached loaded tasks (key: "name@version")
    cache: HashMap<String, (PredefinedTask, PathBuf)>,
}

impl TaskLoader {
    /// Create a new task loader with default search paths
    ///
    /// Default search paths (in priority order):
    /// 1. `./.claude/tasks/` (project local - highest priority)
    /// 2. `~/.claude/tasks/` (user global)
    pub fn new() -> Self {
        let mut search_paths = Vec::new();

        // Add user global path
        if let Some(home) = dirs::home_dir() {
            search_paths.push(home.join(".claude").join("tasks"));
        }

        // Add project local path (higher priority)
        search_paths.push(PathBuf::from(".").join(".claude").join("tasks"));

        TaskLoader {
            search_paths,
            cache: HashMap::new(),
        }
    }

    /// Create a task loader with custom search paths
    ///
    /// Paths are searched in the order provided (first = lowest priority, last = highest priority)
    pub fn with_paths(paths: Vec<PathBuf>) -> Self {
        TaskLoader {
            search_paths: paths,
            cache: HashMap::new(),
        }
    }

    /// Add a search path (will be searched with highest priority)
    pub fn add_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    /// Discover all available tasks from all search paths
    ///
    /// Returns a map of task references to their file paths
    pub fn discover_all(&self) -> Result<HashMap<String, PathBuf>, LoadError> {
        let mut discovered = HashMap::new();

        // Search paths in order (later paths override earlier ones due to priority)
        for search_path in &self.search_paths {
            if !search_path.exists() {
                continue;
            }

            let tasks = discover_tasks_in_directory(search_path)?;
            for (task_ref, path) in tasks {
                discovered.insert(task_ref, path);
            }
        }

        Ok(discovered)
    }

    /// Load a specific task by reference
    ///
    /// Searches all configured paths in priority order
    pub fn load(&mut self, task_ref: &TaskReference) -> Result<PredefinedTask, LoadError> {
        let cache_key = task_ref.to_string();

        // Check cache first
        if let Some((task, _path)) = self.cache.get(&cache_key) {
            return Ok(task.clone());
        }

        // Search for task in all paths (reverse order for priority)
        for search_path in self.search_paths.iter().rev() {
            if let Ok(task) = load_task_from_directory(search_path, task_ref) {
                self.cache
                    .insert(cache_key.clone(), (task.clone(), search_path.clone()));
                return Ok(task);
            }
        }

        Err(LoadError::TaskNotFound(task_ref.to_string()))
    }

    /// Load a task from a specific file path
    pub fn load_from_file(&mut self, path: &Path) -> Result<PredefinedTask, LoadError> {
        load_predefined_task(path)
    }

    /// Clear the task cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get the cached tasks
    pub fn cached_tasks(&self) -> Vec<String> {
        self.cache.keys().cloned().collect()
    }
}

impl Default for TaskLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Load a predefined task from a file
pub fn load_predefined_task(path: &Path) -> Result<PredefinedTask, LoadError> {
    let yaml_content = fs::read_to_string(path).map_err(|source| LoadError::IoError {
        path: path.display().to_string(),
        source,
    })?;

    parse_predefined_task(&yaml_content).map_err(|source| LoadError::ParseError {
        path: path.display().to_string(),
        source,
    })
}

/// Discover all tasks in a directory
fn discover_tasks_in_directory(dir: &Path) -> Result<HashMap<String, PathBuf>, LoadError> {
    let mut tasks = HashMap::new();

    if !dir.exists() {
        return Ok(tasks);
    }

    if !dir.is_dir() {
        return Err(LoadError::DirectoryNotFound(dir.display().to_string()));
    }

    // Walk through directory (non-recursive for Phase 1)
    let entries = fs::read_dir(dir).map_err(|source| LoadError::IoError {
        path: dir.display().to_string(),
        source,
    })?;

    for entry in entries {
        let entry = entry.map_err(|source| LoadError::IoError {
            path: dir.display().to_string(),
            source,
        })?;

        let path = entry.path();

        // Only process .task.yaml files
        if !is_task_file(&path) {
            continue;
        }

        // Try to load the task to get its metadata
        match load_predefined_task(&path) {
            Ok(task) => {
                let task_ref = format!("{}@{}", task.metadata.name, task.metadata.version);

                // Check for duplicates
                if let Some(existing_path) = tasks.get(&task_ref) {
                    return Err(LoadError::DuplicateTask {
                        name: task.metadata.name,
                        version: task.metadata.version,
                        path1: existing_path.display().to_string(),
                        path2: path.display().to_string(),
                    });
                }

                tasks.insert(task_ref, path);
            }
            Err(e) => {
                // Log warning but continue discovery
                eprintln!(
                    "Warning: Failed to load task from {}: {}",
                    path.display(),
                    e
                );
            }
        }
    }

    Ok(tasks)
}

/// Load a specific task from a directory by reference
fn load_task_from_directory(
    dir: &Path,
    task_ref: &TaskReference,
) -> Result<PredefinedTask, LoadError> {
    if !dir.exists() || !dir.is_dir() {
        return Err(LoadError::DirectoryNotFound(dir.display().to_string()));
    }

    // Try standard naming pattern: {name}.task.yaml
    let standard_path = dir.join(format!("{}.task.yaml", task_ref.name));
    if standard_path.exists() {
        let task = load_predefined_task(&standard_path)?;

        // Verify version matches
        if task.metadata.version == task_ref.version {
            return Ok(task);
        }
    }

    // If standard path didn't work, scan directory for matching task
    let discovered = discover_tasks_in_directory(dir)?;
    if let Some(path) = discovered.get(&task_ref.to_string()) {
        return load_predefined_task(path);
    }

    Err(LoadError::TaskNotFound(task_ref.to_string()))
}

/// Check if a path is a task file (ends with .task.yaml)
fn is_task_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.ends_with(".task.yaml"))
        .unwrap_or(false)
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

    #[test]
    fn test_is_task_file() {
        let temp_dir = TempDir::new().unwrap();
        let task_file = temp_dir.path().join("test.task.yaml");
        let other_file = temp_dir.path().join("test.yaml");

        fs::write(&task_file, "").unwrap();
        fs::write(&other_file, "").unwrap();

        assert!(is_task_file(&task_file));
        assert!(!is_task_file(&other_file));
        assert!(!is_task_file(temp_dir.path()));
    }

    #[test]
    fn test_discover_tasks() {
        let temp_dir = TempDir::new().unwrap();
        create_test_task_file(temp_dir.path(), "task1", "1.0.0");
        create_test_task_file(temp_dir.path(), "task2", "2.0.0");

        let discovered = discover_tasks_in_directory(temp_dir.path()).unwrap();

        assert_eq!(discovered.len(), 2);
        assert!(discovered.contains_key("task1@1.0.0"));
        assert!(discovered.contains_key("task2@2.0.0"));
    }

    #[test]
    fn test_load_task() {
        let temp_dir = TempDir::new().unwrap();
        create_test_task_file(temp_dir.path(), "my-task", "1.0.0");

        let mut loader = TaskLoader::with_paths(vec![temp_dir.path().to_path_buf()]);
        let task_ref = TaskReference::parse("my-task@1.0.0").unwrap();

        let task = loader.load(&task_ref).unwrap();
        assert_eq!(task.metadata.name, "my-task");
        assert_eq!(task.metadata.version, "1.0.0");
    }

    #[test]
    fn test_load_task_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let mut loader = TaskLoader::with_paths(vec![temp_dir.path().to_path_buf()]);
        let task_ref = TaskReference::parse("nonexistent@1.0.0").unwrap();

        let result = loader.load(&task_ref);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LoadError::TaskNotFound(_)));
    }

    #[test]
    fn test_task_priority() {
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();

        // Same task in both directories, different versions
        create_test_task_file(temp_dir1.path(), "my-task", "1.0.0");
        create_test_task_file(temp_dir2.path(), "my-task", "2.0.0");

        // Load with dir2 having higher priority (added last)
        let mut loader = TaskLoader::with_paths(vec![
            temp_dir1.path().to_path_buf(),
            temp_dir2.path().to_path_buf(),
        ]);

        let task_ref = TaskReference::parse("my-task@2.0.0").unwrap();
        let task = loader.load(&task_ref).unwrap();
        assert_eq!(task.metadata.version, "2.0.0");
    }

    #[test]
    fn test_cache() {
        let temp_dir = TempDir::new().unwrap();
        create_test_task_file(temp_dir.path(), "my-task", "1.0.0");

        let mut loader = TaskLoader::with_paths(vec![temp_dir.path().to_path_buf()]);
        let task_ref = TaskReference::parse("my-task@1.0.0").unwrap();

        // Load once
        loader.load(&task_ref).unwrap();
        assert_eq!(loader.cached_tasks().len(), 1);

        // Load again (should use cache)
        loader.load(&task_ref).unwrap();
        assert_eq!(loader.cached_tasks().len(), 1);

        // Clear cache
        loader.clear_cache();
        assert_eq!(loader.cached_tasks().len(), 0);
    }
}
