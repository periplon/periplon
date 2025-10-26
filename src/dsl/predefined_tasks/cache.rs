//! Task Cache
//!
//! In-memory cache for loaded predefined tasks with TTL-based expiration.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::dsl::predefined_tasks::PredefinedTask;

/// In-memory cache for loaded tasks
pub struct TaskCache {
    tasks: HashMap<String, CachedTask>,
    ttl: Duration,
}

struct CachedTask {
    task: PredefinedTask,
    loaded_at: Instant,
    source: String,
}

impl TaskCache {
    /// Create a new task cache with specified TTL
    pub fn new(ttl: Duration) -> Self {
        Self {
            tasks: HashMap::new(),
            ttl,
        }
    }

    /// Create a cache with default 5-minute TTL
    pub fn with_default_ttl() -> Self {
        Self::new(Duration::from_secs(300))
    }

    /// Get a cached task if it exists and hasn't expired
    pub fn get(&self, key: &str) -> Option<&PredefinedTask> {
        self.tasks.get(key).and_then(|cached| {
            if cached.loaded_at.elapsed() < self.ttl {
                Some(&cached.task)
            } else {
                None
            }
        })
    }

    /// Insert a task into the cache
    pub fn insert(&mut self, key: String, task: PredefinedTask, source: String) {
        self.tasks.insert(
            key,
            CachedTask {
                task,
                loaded_at: Instant::now(),
                source,
            },
        );
    }

    /// Invalidate a specific cached task
    pub fn invalidate(&mut self, key: &str) {
        self.tasks.remove(key);
    }

    /// Clear all cached tasks
    pub fn clear(&mut self) {
        self.tasks.clear();
    }

    /// Remove expired entries from the cache
    pub fn evict_expired(&mut self) {
        self.tasks
            .retain(|_, cached| cached.loaded_at.elapsed() < self.ttl);
    }

    /// Get the number of cached tasks
    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// Generate cache key from task name and optional version
    pub fn cache_key(name: &str, version: Option<&str>) -> String {
        match version {
            Some(v) => format!("{}@{}", name, v),
            None => name.to_string(),
        }
    }

    /// Get source name for a cached task
    pub fn get_source(&self, key: &str) -> Option<&str> {
        self.tasks.get(key).map(|cached| cached.source.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::predefined_tasks::{
        AgentTemplate, PredefinedTask, PredefinedTaskMetadata, PredefinedTaskSpec, TaskApiVersion,
        TaskKind,
    };
    use crate::dsl::schema::PermissionsSpec;

    fn create_test_task(name: &str, version: &str) -> PredefinedTask {
        PredefinedTask {
            api_version: TaskApiVersion::V1,
            kind: TaskKind::PredefinedTask,
            metadata: PredefinedTaskMetadata {
                name: name.to_string(),
                version: version.to_string(),
                description: None,
                author: None,
                license: None,
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
                inputs: Default::default(),
                outputs: Default::default(),
                dependencies: vec![],
                examples: vec![],
            },
        }
    }

    #[test]
    fn test_cache_insert_and_get() {
        let mut cache = TaskCache::with_default_ttl();
        let task = create_test_task("test-task", "1.0.0");
        let key = TaskCache::cache_key("test-task", Some("1.0.0"));

        cache.insert(key.clone(), task, "test-source".to_string());

        let cached = cache.get(&key);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().metadata.name, "test-task");
        assert_eq!(cached.unwrap().metadata.version, "1.0.0");
    }

    #[test]
    fn test_cache_key_generation() {
        let key_with_version = TaskCache::cache_key("my-task", Some("2.0.0"));
        assert_eq!(key_with_version, "my-task@2.0.0");

        let key_without_version = TaskCache::cache_key("my-task", None);
        assert_eq!(key_without_version, "my-task");
    }

    #[test]
    fn test_cache_invalidation() {
        let mut cache = TaskCache::with_default_ttl();
        let task = create_test_task("test-task", "1.0.0");
        let key = TaskCache::cache_key("test-task", Some("1.0.0"));

        cache.insert(key.clone(), task, "test-source".to_string());
        assert!(cache.get(&key).is_some());

        cache.invalidate(&key);
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = TaskCache::with_default_ttl();

        cache.insert(
            "task1".to_string(),
            create_test_task("task1", "1.0.0"),
            "source1".to_string(),
        );
        cache.insert(
            "task2".to_string(),
            create_test_task("task2", "2.0.0"),
            "source2".to_string(),
        );

        assert_eq!(cache.len(), 2);

        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_expiration() {
        let mut cache = TaskCache::new(Duration::from_millis(10));
        let task = create_test_task("test-task", "1.0.0");
        let key = TaskCache::cache_key("test-task", Some("1.0.0"));

        cache.insert(key.clone(), task, "test-source".to_string());
        assert!(cache.get(&key).is_some());

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(20));

        // Task should be expired
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_evict_expired() {
        let mut cache = TaskCache::new(Duration::from_millis(10));

        cache.insert(
            "task1".to_string(),
            create_test_task("task1", "1.0.0"),
            "source1".to_string(),
        );

        std::thread::sleep(Duration::from_millis(20));

        cache.insert(
            "task2".to_string(),
            create_test_task("task2", "2.0.0"),
            "source2".to_string(),
        );

        assert_eq!(cache.len(), 2);

        cache.evict_expired();

        // Only task2 should remain
        assert_eq!(cache.len(), 1);
        assert!(cache.get("task1").is_none());
        assert!(cache.get("task2").is_some());
    }

    #[test]
    fn test_get_source() {
        let mut cache = TaskCache::with_default_ttl();
        let task = create_test_task("test-task", "1.0.0");
        let key = "test-task@1.0.0";

        cache.insert(key.to_string(), task, "my-source".to_string());

        assert_eq!(cache.get_source(key), Some("my-source"));
        assert_eq!(cache.get_source("nonexistent"), None);
    }
}
