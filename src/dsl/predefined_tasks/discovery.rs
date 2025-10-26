//! Multi-Source Task Discovery
//!
//! Discovers and resolves predefined tasks from multiple sources with priority-based resolution.

use std::path::Path;

use crate::dsl::predefined_tasks::PredefinedTask;
use crate::error::Result;

use super::cache::TaskCache;
use super::sources::{
    GitTaskSource, HealthStatus, LocalTaskSource, SourceConfig, SourceInfo, TaskMetadata,
    TaskSource, TaskSourcesConfig, UpdateResult,
};

/// Multi-source task discovery system
pub struct TaskDiscovery {
    sources: Vec<Box<dyn TaskSource>>,
    cache: TaskCache,
}

impl TaskDiscovery {
    /// Create task discovery from configuration file
    pub async fn from_config(config_path: &Path) -> Result<Self> {
        let config = TaskSourcesConfig::from_yaml(config_path)?;
        Self::from_sources_config(config).await
    }

    /// Create task discovery from default configuration location (~/.claude/task-sources.yaml)
    pub async fn from_default_config() -> Result<Self> {
        let config = TaskSourcesConfig::from_default_path()?;
        Self::from_sources_config(config).await
    }

    /// Create task discovery from sources configuration
    pub async fn from_sources_config(config: TaskSourcesConfig) -> Result<Self> {
        let mut sources: Vec<Box<dyn TaskSource>> = Vec::new();

        for source_config in config.enabled_sources() {
            let source: Box<dyn TaskSource> = match source_config {
                SourceConfig::Local {
                    name,
                    path,
                    priority,
                    ..
                } => Box::new(LocalTaskSource::new(name, path, priority)?),

                SourceConfig::Git {
                    name,
                    url,
                    branch,
                    tag,
                    cache_dir,
                    update_policy,
                    priority,
                    trusted,
                    ..
                } => Box::new(GitTaskSource::new(
                    name,
                    url,
                    branch,
                    tag,
                    cache_dir,
                    update_policy,
                    priority,
                    trusted,
                )?),
            };

            sources.push(source);
        }

        // Sort by priority (descending - higher priority searched first)
        sources.sort_by_key(|b| std::cmp::Reverse(b.priority()));

        Ok(Self {
            sources,
            cache: TaskCache::with_default_ttl(),
        })
    }

    /// Create task discovery with default sources (project and user local directories)
    pub async fn with_default_sources() -> Result<Self> {
        let mut sources: Vec<Box<dyn TaskSource>> = Vec::new();

        // Add project local source (./.claude/tasks)
        if let Ok(project_source) =
            LocalTaskSource::new("project-tasks".to_string(), "./.claude/tasks", 10)
        {
            sources.push(Box::new(project_source));
        }

        // Add user global source (~/.claude/tasks)
        if let Some(home_dir) = dirs::home_dir() {
            let user_tasks_path = home_dir.join(".claude/tasks");
            if let Ok(user_source) =
                LocalTaskSource::new("user-tasks".to_string(), user_tasks_path, 8)
            {
                sources.push(Box::new(user_source));
            }
        }

        sources.sort_by_key(|b| std::cmp::Reverse(b.priority()));

        Ok(Self {
            sources,
            cache: TaskCache::with_default_ttl(),
        })
    }

    /// Discover all tasks from all sources
    pub async fn discover_all(&mut self) -> Result<Vec<TaskMetadata>> {
        let mut all_metadata = Vec::new();

        for source in &mut self.sources {
            match source.discover_tasks().await {
                Ok(metadata) => all_metadata.extend(metadata),
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to discover tasks from {}: {}",
                        source.name(),
                        e
                    );
                }
            }
        }

        Ok(all_metadata)
    }

    /// Find a task by name and optional version, searching all sources by priority
    pub async fn find_task(&mut self, name: &str, version: Option<&str>) -> Result<PredefinedTask> {
        // Check cache first
        let cache_key = TaskCache::cache_key(name, version);
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        // Search sources by priority
        for source in &mut self.sources {
            match source.load_task(name, version).await {
                Ok(task) => {
                    // Cache the result
                    self.cache
                        .insert(cache_key, task.clone(), source.name().to_string());
                    return Ok(task);
                }
                Err(_) => continue, // Try next source
            }
        }

        Err(crate::error::Error::TaskNotFoundInAnySources {
            name: name.to_string(),
            version: version.map(String::from),
            searched_sources: self.sources.iter().map(|s| s.name().to_string()).collect(),
        })
    }

    /// Search for tasks across all sources
    pub async fn search(&mut self, query: &str) -> Result<Vec<TaskMetadata>> {
        let all_tasks = self.discover_all().await?;

        let query_lower = query.to_lowercase();
        let results: Vec<TaskMetadata> = all_tasks
            .into_iter()
            .filter(|task| {
                task.name.to_lowercase().contains(&query_lower)
                    || task
                        .description
                        .as_ref()
                        .is_some_and(|d| d.to_lowercase().contains(&query_lower))
                    || task
                        .tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query_lower))
            })
            .collect();

        Ok(results)
    }

    /// List tasks by tag across all sources
    pub async fn list_by_tag(&mut self, tag: &str) -> Result<Vec<TaskMetadata>> {
        let all_tasks = self.discover_all().await?;

        let tag_lower = tag.to_lowercase();
        let results: Vec<TaskMetadata> = all_tasks
            .into_iter()
            .filter(|task| task.tags.iter().any(|t| t.to_lowercase() == tag_lower))
            .collect();

        Ok(results)
    }

    /// List all configured sources
    pub fn list_sources(&self) -> Vec<SourceInfo> {
        self.sources
            .iter()
            .map(|s| SourceInfo {
                name: s.name().to_string(),
                source_type: s.source_type(),
                priority: s.priority(),
                trusted: s.is_trusted(),
                enabled: true, // Only enabled sources are loaded
            })
            .collect()
    }

    /// Update all sources (git pull, etc.)
    pub async fn update_all(&mut self) -> Result<Vec<UpdateResult>> {
        let mut results = Vec::new();

        for source in &mut self.sources {
            match source.update().await {
                Ok(result) => results.push(result),
                Err(e) => {
                    eprintln!("Warning: Failed to update {}: {}", source.name(), e);
                }
            }
        }

        // Clear cache after updates
        self.cache.clear();

        Ok(results)
    }

    /// Update a specific source by name
    pub async fn update_source(&mut self, source_name: &str) -> Result<UpdateResult> {
        for source in &mut self.sources {
            if source.name() == source_name {
                let result = source.update().await?;
                // Invalidate cache for this source
                self.cache.clear(); // For simplicity, clear all cache
                return Ok(result);
            }
        }

        Err(crate::error::Error::SourceNotFound(source_name.to_string()))
    }

    /// Health check for all sources
    pub async fn health_check_all(&self) -> Result<Vec<(String, HealthStatus)>> {
        let mut results = Vec::new();

        for source in &self.sources {
            match source.health_check().await {
                Ok(status) => results.push((source.name().to_string(), status)),
                Err(e) => {
                    eprintln!("Warning: Health check failed for {}: {}", source.name(), e);
                }
            }
        }

        Ok(results)
    }

    /// Get number of configured sources
    pub fn source_count(&self) -> usize {
        self.sources.len()
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            cached_tasks: self.cache.len(),
        }
    }

    /// Clear the task cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Evict expired cache entries
    pub fn evict_expired_cache(&mut self) {
        self.cache.evict_expired();
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub cached_tasks: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    async fn create_test_task_file(dir: &Path, name: &str, version: &str) {
        let task_yaml = format!(
            r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "{}"
  version: "{}"
  description: "Test task"
  tags: ["test", "example"]
spec:
  agent_template:
    description: "Test task"
  inputs: {{}}
"#,
            name, version
        );

        let filename = format!("{}.task.yaml", name);
        let mut file = std::fs::File::create(dir.join(filename)).unwrap();
        file.write_all(task_yaml.as_bytes()).unwrap();
    }

    #[tokio::test]
    async fn test_default_sources_discovery() {
        let discovery = TaskDiscovery::with_default_sources().await.unwrap();
        let sources = discovery.list_sources();

        // Should have at least project-tasks source
        assert!(!sources.is_empty());
    }

    #[tokio::test]
    async fn test_discover_all_tasks() {
        let temp_dir = TempDir::new().unwrap();
        let tasks_dir = temp_dir.path().join("tasks");
        std::fs::create_dir(&tasks_dir).unwrap();

        create_test_task_file(&tasks_dir, "task1", "1.0.0").await;
        create_test_task_file(&tasks_dir, "task2", "2.0.0").await;

        let source = LocalTaskSource::new("test".to_string(), &tasks_dir, 10).unwrap();
        let mut discovery = TaskDiscovery {
            sources: vec![Box::new(source)],
            cache: TaskCache::with_default_ttl(),
        };

        let tasks = discovery.discover_all().await.unwrap();
        assert_eq!(tasks.len(), 2);
    }

    #[tokio::test]
    async fn test_find_task() {
        let temp_dir = TempDir::new().unwrap();
        let tasks_dir = temp_dir.path().join("tasks");
        std::fs::create_dir(&tasks_dir).unwrap();

        create_test_task_file(&tasks_dir, "my-task", "1.5.0").await;

        let source = LocalTaskSource::new("test".to_string(), &tasks_dir, 10).unwrap();
        let mut discovery = TaskDiscovery {
            sources: vec![Box::new(source)],
            cache: TaskCache::with_default_ttl(),
        };

        let task = discovery.find_task("my-task", Some("1.5.0")).await.unwrap();
        assert_eq!(task.metadata.name, "my-task");
        assert_eq!(task.metadata.version, "1.5.0");

        // Should use cache on second call
        let task2 = discovery.find_task("my-task", Some("1.5.0")).await.unwrap();
        assert_eq!(task2.metadata.name, "my-task");
    }

    #[tokio::test]
    async fn test_priority_resolution() {
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();

        let tasks_dir1 = temp_dir1.path().join("tasks");
        let tasks_dir2 = temp_dir2.path().join("tasks");
        std::fs::create_dir(&tasks_dir1).unwrap();
        std::fs::create_dir(&tasks_dir2).unwrap();

        // Same task in both sources, different versions
        create_test_task_file(&tasks_dir1, "shared-task", "1.0.0").await;
        create_test_task_file(&tasks_dir2, "shared-task", "2.0.0").await;

        let source1 = LocalTaskSource::new("low-priority".to_string(), &tasks_dir1, 5).unwrap();
        let source2 = LocalTaskSource::new("high-priority".to_string(), &tasks_dir2, 10).unwrap();

        let mut discovery = TaskDiscovery {
            sources: vec![Box::new(source1), Box::new(source2)],
            cache: TaskCache::with_default_ttl(),
        };

        // Sort by priority
        discovery
            .sources
            .sort_by(|a, b| b.priority().cmp(&a.priority()));

        // Should get version from high-priority source
        let task = discovery.find_task("shared-task", None).await.unwrap();
        assert_eq!(task.metadata.version, "2.0.0");
    }

    #[tokio::test]
    async fn test_search_tasks() {
        let temp_dir = TempDir::new().unwrap();
        let tasks_dir = temp_dir.path().join("tasks");
        std::fs::create_dir(&tasks_dir).unwrap();

        create_test_task_file(&tasks_dir, "google-drive-upload", "1.0.0").await;
        create_test_task_file(&tasks_dir, "google-docs-create", "1.0.0").await;
        create_test_task_file(&tasks_dir, "slack-notify", "1.0.0").await;

        let source = LocalTaskSource::new("test".to_string(), &tasks_dir, 10).unwrap();
        let mut discovery = TaskDiscovery {
            sources: vec![Box::new(source)],
            cache: TaskCache::with_default_ttl(),
        };

        let results = discovery.search("google").await.unwrap();
        assert_eq!(results.len(), 2);

        let results = discovery.search("slack").await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_list_by_tag() {
        let temp_dir = TempDir::new().unwrap();
        let tasks_dir = temp_dir.path().join("tasks");
        std::fs::create_dir(&tasks_dir).unwrap();

        create_test_task_file(&tasks_dir, "task1", "1.0.0").await;
        create_test_task_file(&tasks_dir, "task2", "1.0.0").await;

        let source = LocalTaskSource::new("test".to_string(), &tasks_dir, 10).unwrap();
        let mut discovery = TaskDiscovery {
            sources: vec![Box::new(source)],
            cache: TaskCache::with_default_ttl(),
        };

        let results = discovery.list_by_tag("test").await.unwrap();
        assert_eq!(results.len(), 2);
    }
}
