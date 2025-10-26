//! Comprehensive tests for Phase 2: Git Repository Integration
//!
//! Tests cover:
//! - Git source configuration
//! - Git cloning and pulling
//! - Package manifest parsing
//! - Multi-source discovery
//! - Task caching with TTL

use periplon_sdk::dsl::predefined_tasks::cache::TaskCache;
use periplon_sdk::dsl::predefined_tasks::manifest::TaskPackage;
use periplon_sdk::dsl::predefined_tasks::sources::config::{
    SourceConfig, TaskSourcesConfig, UpdatePolicy,
};
use std::fs;
use std::time::Duration;
use tempfile::TempDir;

// ============================================================================
// SECTION 1: Source Configuration Tests
// ============================================================================

#[test]
fn test_parse_local_source_config() {
    let yaml = r#"
sources:
  - type: local
    name: "project-tasks"
    path: "./.claude/tasks"
    priority: 10
    enabled: true
"#;

    let config: TaskSourcesConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.sources.len(), 1);

    match &config.sources[0] {
        SourceConfig::Local {
            name,
            path,
            priority,
            enabled,
        } => {
            assert_eq!(name, "project-tasks");
            assert_eq!(path, "./.claude/tasks");
            assert_eq!(*priority, 10);
            assert!(enabled);
        }
        _ => panic!("Expected Local source"),
    }
}

#[test]
fn test_parse_git_source_config() {
    let yaml = r#"
sources:
  - type: git
    name: "community-tasks"
    url: "https://github.com/example/tasks"
    branch: "main"
    update_policy: daily
    priority: 5
    trusted: true
    enabled: true
"#;

    let config: TaskSourcesConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.sources.len(), 1);

    match &config.sources[0] {
        SourceConfig::Git {
            name,
            url,
            branch,
            update_policy,
            priority,
            trusted,
            enabled,
            ..
        } => {
            assert_eq!(name, "community-tasks");
            assert_eq!(url, "https://github.com/example/tasks");
            assert_eq!(branch.as_deref(), Some("main"));
            assert_eq!(*update_policy, UpdatePolicy::Daily);
            assert_eq!(*priority, 5);
            assert!(trusted);
            assert!(enabled);
        }
        _ => panic!("Expected Git source"),
    }
}

#[test]
fn test_parse_git_source_with_tag() {
    let yaml = r#"
sources:
  - type: git
    name: "stable-tasks"
    url: "https://github.com/example/tasks"
    tag: "v1.2.0"
    update_policy: manual
    priority: 8
    trusted: true
    enabled: true
"#;

    let config: TaskSourcesConfig = serde_yaml::from_str(yaml).unwrap();

    match &config.sources[0] {
        SourceConfig::Git { tag, .. } => {
            assert_eq!(tag.as_deref(), Some("v1.2.0"));
        }
        _ => panic!("Expected Git source"),
    }
}

#[test]
fn test_parse_multiple_sources() {
    let yaml = r#"
sources:
  - type: local
    name: "project"
    path: "./tasks"
    priority: 10
    enabled: true
  - type: git
    name: "community"
    url: "https://github.com/example/tasks"
    branch: "main"
    priority: 5
    trusted: true
    enabled: true
  - type: local
    name: "global"
    path: "~/.claude/tasks"
    priority: 3
    enabled: true
"#;

    let config: TaskSourcesConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.sources.len(), 3);
    assert_eq!(config.sources[0].name(), "project");
    assert_eq!(config.sources[1].name(), "community");
    assert_eq!(config.sources[2].name(), "global");
}

#[test]
fn test_parse_update_policies() {
    let yaml = r#"
sources:
  - type: git
    name: "daily"
    url: "https://example.com/repo"
    update_policy: daily
    priority: 5
    trusted: true
  - type: git
    name: "weekly"
    url: "https://example.com/repo"
    update_policy: weekly
    priority: 5
    trusted: true
  - type: git
    name: "manual"
    url: "https://example.com/repo"
    update_policy: manual
    priority: 5
    trusted: true
  - type: git
    name: "always"
    url: "https://example.com/repo"
    update_policy: always
    priority: 5
    trusted: true
"#;

    let config: TaskSourcesConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.sources.len(), 4);

    let policies: Vec<UpdatePolicy> = config
        .sources
        .iter()
        .filter_map(|s| match s {
            SourceConfig::Git { update_policy, .. } => Some(*update_policy),
            _ => None,
        })
        .collect();

    assert_eq!(policies[0], UpdatePolicy::Daily);
    assert_eq!(policies[1], UpdatePolicy::Weekly);
    assert_eq!(policies[2], UpdatePolicy::Manual);
    assert_eq!(policies[3], UpdatePolicy::Always);
}

#[test]
fn test_enabled_sources_filter() {
    let yaml = r#"
sources:
  - type: local
    name: "enabled-1"
    path: "./tasks1"
    priority: 10
    enabled: true
  - type: local
    name: "disabled"
    path: "./tasks2"
    priority: 8
    enabled: false
  - type: local
    name: "enabled-2"
    path: "./tasks3"
    priority: 6
    enabled: true
"#;

    let config: TaskSourcesConfig = serde_yaml::from_str(yaml).unwrap();
    let enabled = config.enabled_sources();

    assert_eq!(enabled.len(), 2);
    assert_eq!(enabled[0].name(), "enabled-1");
    assert_eq!(enabled[1].name(), "enabled-2");
}

#[test]
fn test_source_priority() {
    let yaml = r#"
sources:
  - type: local
    name: "low"
    path: "./low"
    priority: 1
    enabled: true
  - type: local
    name: "high"
    path: "./high"
    priority: 10
    enabled: true
  - type: local
    name: "medium"
    path: "./medium"
    priority: 5
    enabled: true
"#;

    let config: TaskSourcesConfig = serde_yaml::from_str(yaml).unwrap();

    assert_eq!(config.sources[0].priority(), 1);
    assert_eq!(config.sources[1].priority(), 10);
    assert_eq!(config.sources[2].priority(), 5);
}

#[test]
fn test_source_trust_settings() {
    let yaml = r#"
sources:
  - type: git
    name: "trusted"
    url: "https://example.com/repo"
    priority: 5
    trusted: true
    enabled: true
  - type: git
    name: "untrusted"
    url: "https://example.com/repo"
    priority: 5
    trusted: false
    enabled: true
"#;

    let config: TaskSourcesConfig = serde_yaml::from_str(yaml).unwrap();

    // Local sources are always trusted
    let local_yaml = r#"
sources:
  - type: local
    name: "local"
    path: "./tasks"
    priority: 10
    enabled: true
"#;
    let local_config: TaskSourcesConfig = serde_yaml::from_str(local_yaml).unwrap();
    assert!(local_config.sources[0].is_trusted());

    assert!(config.sources[0].is_trusted());
    assert!(!config.sources[1].is_trusted());
}

#[test]
fn test_default_values() {
    let yaml = r#"
sources:
  - type: git
    name: "minimal"
    url: "https://example.com/repo"
    priority: 5
"#;

    let config: TaskSourcesConfig = serde_yaml::from_str(yaml).unwrap();

    match &config.sources[0] {
        SourceConfig::Git {
            enabled,
            trusted,
            update_policy,
            ..
        } => {
            assert!(enabled); // Default is true
            assert!(trusted); // Default is true
            assert_eq!(*update_policy, UpdatePolicy::Daily); // Default is daily
        }
        _ => panic!("Expected Git source"),
    }
}

// ============================================================================
// SECTION 2: Package Manifest Tests
// ============================================================================

#[test]
fn test_parse_minimal_package_manifest() {
    let yaml = r#"
apiVersion: "package/v1"
kind: "TaskPackage"
metadata:
  name: "my-tasks"
  version: "1.0.0"
tasks:
  - path: "tasks/upload.task.yaml"
    name: "upload"
    version: "1.0.0"
"#;

    let package: TaskPackage = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(package.metadata.name, "my-tasks");
    assert_eq!(package.metadata.version, "1.0.0");
    assert_eq!(package.tasks.len(), 1);
    assert_eq!(package.tasks[0].name, "upload");
}

#[test]
fn test_parse_complete_package_manifest() {
    let yaml = r#"
apiVersion: "package/v1"
kind: "TaskPackage"
metadata:
  name: "google-workspace"
  version: "2.1.0"
  description: "Google Workspace integration tasks"
  author: "Example Team"
  homepage: "https://example.com/tasks"
  license: "MIT"
  tags: ["google", "workspace", "productivity"]
tasks:
  - path: "tasks/drive-upload.task.yaml"
    name: "drive-upload"
    version: "1.0.0"
  - path: "tasks/sheets-update.task.yaml"
    name: "sheets-update"
    version: "1.2.0"
  - path: "tasks/calendar-create.task.yaml"
    name: "calendar-create"
    version: "0.9.0"
dependencies:
  - name: "http-utils"
    version: "^1.0.0"
    repository: "https://github.com/example/http-utils"
  - name: "auth-helpers"
    version: "~2.1.0"
requires:
  sdk_version: ">=0.1.0"
  min_model: "claude-sonnet-4-5"
"#;

    let package: TaskPackage = serde_yaml::from_str(yaml).unwrap();

    // Validate metadata
    assert_eq!(package.metadata.name, "google-workspace");
    assert_eq!(package.metadata.version, "2.1.0");
    assert_eq!(
        package.metadata.description,
        Some("Google Workspace integration tasks".to_string())
    );
    assert_eq!(package.metadata.author, Some("Example Team".to_string()));
    assert_eq!(package.metadata.license, Some("MIT".to_string()));
    assert_eq!(package.metadata.tags.len(), 3);

    // Validate tasks
    assert_eq!(package.tasks.len(), 3);
    assert_eq!(package.tasks[0].name, "drive-upload");
    assert_eq!(package.tasks[1].name, "sheets-update");
    assert_eq!(package.tasks[2].name, "calendar-create");

    // Validate dependencies
    assert_eq!(package.dependencies.len(), 2);
    assert_eq!(package.dependencies[0].name, "http-utils");
    assert_eq!(package.dependencies[0].version, "^1.0.0");

    // Validate requirements
    assert!(package.requires.is_some());
    let reqs = package.requires.unwrap();
    assert_eq!(reqs.sdk_version, Some(">=0.1.0".to_string()));
    assert_eq!(reqs.min_model, Some("claude-sonnet-4-5".to_string()));
}

#[test]
fn test_parse_package_with_multiple_tasks() {
    let yaml = r#"
apiVersion: "package/v1"
kind: "TaskPackage"
metadata:
  name: "integration-suite"
  version: "1.0.0"
tasks:
  - path: "slack/notify.task.yaml"
    name: "slack-notify"
    version: "1.0.0"
  - path: "github/create-issue.task.yaml"
    name: "github-create-issue"
    version: "1.1.0"
  - path: "jira/update-ticket.task.yaml"
    name: "jira-update-ticket"
    version: "2.0.0"
  - path: "notion/create-page.task.yaml"
    name: "notion-create-page"
    version: "0.8.0"
"#;

    let package: TaskPackage = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(package.tasks.len(), 4);

    // Verify all task references
    assert_eq!(package.tasks[0].path, "slack/notify.task.yaml");
    assert_eq!(package.tasks[1].path, "github/create-issue.task.yaml");
    assert_eq!(package.tasks[2].path, "jira/update-ticket.task.yaml");
    assert_eq!(package.tasks[3].path, "notion/create-page.task.yaml");
}

#[test]
fn test_package_invalid_kind() {
    let yaml = r#"
apiVersion: "package/v1"
kind: "InvalidKind"
metadata:
  name: "test"
  version: "1.0.0"
tasks: []
"#;

    let package: Result<TaskPackage, _> = serde_yaml::from_str(yaml);
    assert!(package.is_ok()); // Parsing succeeds

    // But validation should fail when loading
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("package.yaml");
    fs::write(&manifest_path, yaml).unwrap();

    let result = TaskPackage::from_yaml(&manifest_path);
    assert!(result.is_err());
}

#[test]
fn test_package_with_nested_paths() {
    let yaml = r#"
apiVersion: "package/v1"
kind: "TaskPackage"
metadata:
  name: "nested-package"
  version: "1.0.0"
tasks:
  - path: "category1/subcategory/task1.task.yaml"
    name: "task1"
    version: "1.0.0"
  - path: "category2/task2.task.yaml"
    name: "task2"
    version: "1.0.0"
"#;

    let package: TaskPackage = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(
        package.tasks[0].path,
        "category1/subcategory/task1.task.yaml"
    );
    assert_eq!(package.tasks[1].path, "category2/task2.task.yaml");
}

// ============================================================================
// SECTION 3: Task Cache Tests
// ============================================================================

#[test]
fn test_cache_basic_operations() {
    use periplon_sdk::dsl::predefined_tasks::schema::{
        PredefinedTask, PredefinedTaskMetadata, PredefinedTaskSpec, TaskApiVersion, TaskKind,
    };
    use periplon_sdk::dsl::predefined_tasks::AgentTemplate;
    use std::collections::HashMap;

    let mut cache = TaskCache::new(Duration::from_secs(60));

    // Create a test task
    let task = PredefinedTask {
        api_version: TaskApiVersion::V1,
        kind: TaskKind::PredefinedTask,
        metadata: PredefinedTaskMetadata {
            name: "test-task".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            description: None,
            license: None,
            repository: None,
            tags: vec![],
        },
        spec: PredefinedTaskSpec {
            agent_template: AgentTemplate {
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

    // Cache should be empty initially
    assert!(cache.is_empty());
    assert_eq!(cache.len(), 0);

    // Insert task
    let key = TaskCache::cache_key("test-task", Some("1.0.0"));
    cache.insert(key.clone(), task.clone(), "local".to_string());

    // Should be able to retrieve it
    assert!(!cache.is_empty());
    assert_eq!(cache.len(), 1);
    assert!(cache.get(&key).is_some());
    assert_eq!(cache.get_source(&key), Some("local"));

    // Clear cache
    cache.clear();
    assert!(cache.is_empty());
}

#[test]
fn test_cache_expiration() {
    use periplon_sdk::dsl::predefined_tasks::schema::{
        PredefinedTask, PredefinedTaskMetadata, PredefinedTaskSpec, TaskApiVersion, TaskKind,
    };
    use periplon_sdk::dsl::predefined_tasks::AgentTemplate;
    use std::collections::HashMap;
    use std::thread;

    // Create cache with very short TTL (100ms)
    let mut cache = TaskCache::new(Duration::from_millis(100));

    let task = PredefinedTask {
        api_version: TaskApiVersion::V1,
        kind: TaskKind::PredefinedTask,
        metadata: PredefinedTaskMetadata {
            name: "test-task".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            description: None,
            license: None,
            repository: None,
            tags: vec![],
        },
        spec: PredefinedTaskSpec {
            agent_template: AgentTemplate {
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

    let key = TaskCache::cache_key("test-task", Some("1.0.0"));
    cache.insert(key.clone(), task, "local".to_string());

    // Should be available immediately
    assert!(cache.get(&key).is_some());

    // Wait for expiration
    thread::sleep(Duration::from_millis(150));

    // Should be expired now
    assert!(cache.get(&key).is_none());
}

#[test]
fn test_cache_invalidation() {
    use periplon_sdk::dsl::predefined_tasks::schema::{
        PredefinedTask, PredefinedTaskMetadata, PredefinedTaskSpec, TaskApiVersion, TaskKind,
    };
    use periplon_sdk::dsl::predefined_tasks::AgentTemplate;
    use std::collections::HashMap;

    let mut cache = TaskCache::new(Duration::from_secs(60));

    let task = PredefinedTask {
        api_version: TaskApiVersion::V1,
        kind: TaskKind::PredefinedTask,
        metadata: PredefinedTaskMetadata {
            name: "test-task".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            description: None,
            license: None,
            repository: None,
            tags: vec![],
        },
        spec: PredefinedTaskSpec {
            agent_template: AgentTemplate {
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

    let key = TaskCache::cache_key("test-task", Some("1.0.0"));
    cache.insert(key.clone(), task, "local".to_string());

    assert!(cache.get(&key).is_some());

    // Invalidate specific entry
    cache.invalidate(&key);

    assert!(cache.get(&key).is_none());
    assert!(cache.is_empty());
}

#[test]
fn test_cache_key_generation() {
    // With version
    let key1 = TaskCache::cache_key("my-task", Some("1.0.0"));
    assert_eq!(key1, "my-task@1.0.0");

    // Without version
    let key2 = TaskCache::cache_key("my-task", None);
    assert_eq!(key2, "my-task");
}

#[test]
fn test_cache_multiple_tasks() {
    use periplon_sdk::dsl::predefined_tasks::schema::{
        PredefinedTask, PredefinedTaskMetadata, PredefinedTaskSpec, TaskApiVersion, TaskKind,
    };
    use periplon_sdk::dsl::predefined_tasks::AgentTemplate;
    use std::collections::HashMap;

    let mut cache = TaskCache::new(Duration::from_secs(60));

    for i in 1..=5 {
        let task = PredefinedTask {
            api_version: TaskApiVersion::V1,
            kind: TaskKind::PredefinedTask,
            metadata: PredefinedTaskMetadata {
                name: format!("task-{}", i),
                version: "1.0.0".to_string(),
                author: None,
                description: None,
                license: None,
                repository: None,
                tags: vec![],
            },
            spec: PredefinedTaskSpec {
                agent_template: AgentTemplate {
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

        let key = TaskCache::cache_key(&format!("task-{}", i), Some("1.0.0"));
        cache.insert(key, task, "local".to_string());
    }

    assert_eq!(cache.len(), 5);

    // All tasks should be retrievable
    for i in 1..=5 {
        let key = TaskCache::cache_key(&format!("task-{}", i), Some("1.0.0"));
        assert!(cache.get(&key).is_some());
    }
}

#[test]
fn test_cache_evict_expired() {
    use periplon_sdk::dsl::predefined_tasks::schema::{
        PredefinedTask, PredefinedTaskMetadata, PredefinedTaskSpec, TaskApiVersion, TaskKind,
    };
    use periplon_sdk::dsl::predefined_tasks::AgentTemplate;
    use std::collections::HashMap;
    use std::thread;

    let mut cache = TaskCache::new(Duration::from_millis(100));

    let task = PredefinedTask {
        api_version: TaskApiVersion::V1,
        kind: TaskKind::PredefinedTask,
        metadata: PredefinedTaskMetadata {
            name: "test-task".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            description: None,
            license: None,
            repository: None,
            tags: vec![],
        },
        spec: PredefinedTaskSpec {
            agent_template: AgentTemplate {
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

    let key1 = TaskCache::cache_key("task-1", Some("1.0.0"));
    cache.insert(key1.clone(), task.clone(), "local".to_string());

    // Wait a bit
    thread::sleep(Duration::from_millis(50));

    // Add another task
    let key2 = TaskCache::cache_key("task-2", Some("1.0.0"));
    cache.insert(key2.clone(), task, "local".to_string());

    assert_eq!(cache.len(), 2);

    // Wait for first task to expire
    thread::sleep(Duration::from_millis(60));

    // Evict expired entries
    cache.evict_expired();

    // First task should be evicted, second should remain
    assert_eq!(cache.len(), 1);
    assert!(cache.get(&key2).is_some());
}

// ============================================================================
// SECTION 4: Integration Tests (require actual git operations)
// ============================================================================

// Note: These tests are marked with #[ignore] by default as they require
// network access and git operations. Run with: cargo test -- --ignored

#[test]
#[ignore]
fn test_git_clone_public_repository() {
    // This would test actual git cloning
    // Implementation would depend on having a test repository
}

#[test]
#[ignore]
fn test_git_pull_updates() {
    // This would test git pull operations
}

#[test]
#[ignore]
fn test_git_checkout_branch() {
    // This would test branch checkout
}

#[test]
#[ignore]
fn test_git_checkout_tag() {
    // This would test tag checkout
}

// ============================================================================
// SECTION 5: Multi-Source Priority Tests
// ============================================================================

#[test]
fn test_source_priority_ordering() {
    let yaml = r#"
sources:
  - type: local
    name: "low-priority"
    path: "./low"
    priority: 1
    enabled: true
  - type: local
    name: "high-priority"
    path: "./high"
    priority: 10
    enabled: true
  - type: local
    name: "medium-priority"
    path: "./medium"
    priority: 5
    enabled: true
"#;

    let config: TaskSourcesConfig = serde_yaml::from_str(yaml).unwrap();

    // Priority values should be as configured
    let priorities: Vec<u8> = config.sources.iter().map(|s| s.priority()).collect();
    assert_eq!(priorities, vec![1, 10, 5]);

    // When creating discovery, sources should be sorted by priority (descending)
    // This would be tested in the actual TaskDiscovery implementation
}

#[test]
fn test_mixed_source_types_priority() {
    let yaml = r#"
sources:
  - type: local
    name: "local-high"
    path: "./tasks"
    priority: 10
    enabled: true
  - type: git
    name: "git-medium"
    url: "https://example.com/repo"
    priority: 5
    trusted: true
    enabled: true
  - type: local
    name: "local-low"
    path: "~/.claude/tasks"
    priority: 1
    enabled: true
"#;

    let config: TaskSourcesConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.sources.len(), 3);

    // Verify priority values
    assert_eq!(config.sources[0].priority(), 10);
    assert_eq!(config.sources[1].priority(), 5);
    assert_eq!(config.sources[2].priority(), 1);
}
