# Phase 2 Design: Git Repository Support & Multi-Source Discovery

## Overview

Phase 2 extends the predefined tasks system with:
- Git repository integration for remote task sources
- Intelligent caching mechanism with update policies
- Package manifest parsing for repository-based task collections
- Multi-source task discovery with priority resolution
- Comprehensive task source management

## Architecture

### Module Structure

```
src/dsl/predefined_tasks/
├── mod.rs                    # Module root (existing)
├── schema.rs                 # Task definitions (existing)
├── parser.rs                 # YAML parsing (existing)
├── loader.rs                 # Local filesystem loading (existing)
├── resolver.rs               # Task reference resolution (existing)
├── sources/                  # NEW: Source abstraction layer
│   ├── mod.rs                # Source trait and types
│   ├── local.rs              # Local filesystem source
│   ├── git.rs                # Git repository source
│   └── config.rs             # Source configuration
├── cache.rs                  # NEW: Caching layer
├── discovery.rs              # NEW: Multi-source discovery
└── manifest.rs               # NEW: Package manifest parsing
```

### Core Abstractions

#### 1. TaskSource Trait

Unified abstraction for all task sources (local, git, registry):

```rust
use async_trait::async_trait;
use std::path::Path;
use crate::error::Result;

/// Represents a source of predefined tasks
#[async_trait]
pub trait TaskSource: Send + Sync {
    /// Unique name of this source
    fn name(&self) -> &str;

    /// Source type identifier
    fn source_type(&self) -> SourceType;

    /// Priority for resolution (higher = searched first)
    fn priority(&self) -> u8;

    /// Whether this source is trusted
    fn is_trusted(&self) -> bool;

    /// Discover all tasks from this source
    async fn discover_tasks(&self) -> Result<Vec<TaskMetadata>>;

    /// Load a specific task by name and optional version
    async fn load_task(&self, name: &str, version: Option<&str>) -> Result<PredefinedTask>;

    /// Update/refresh the source (git pull, registry sync, etc.)
    async fn update(&self) -> Result<UpdateResult>;

    /// Check if source is available/healthy
    async fn health_check(&self) -> Result<HealthStatus>;
}

/// Source type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceType {
    Local,
    Git,
    Registry,
}

/// Task metadata for discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub source_name: String,
    pub source_type: SourceType,
}

/// Update operation result
#[derive(Debug)]
pub struct UpdateResult {
    pub updated: bool,
    pub message: String,
    pub new_tasks: usize,
    pub updated_tasks: usize,
}

/// Health status for a source
#[derive(Debug)]
pub struct HealthStatus {
    pub available: bool,
    pub message: Option<String>,
    pub last_check: chrono::DateTime<chrono::Utc>,
}
```

#### 2. Source Configuration

Configuration for task sources defined in `~/.claude/task-sources.yaml`:

```rust
/// Configuration for all task sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSourcesConfig {
    pub sources: Vec<SourceConfig>,
}

/// Individual source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SourceConfig {
    Local {
        name: String,
        path: String,
        priority: u8,
        #[serde(default = "default_true")]
        enabled: bool,
    },
    Git {
        name: String,
        url: String,
        branch: Option<String>,
        tag: Option<String>,
        cache_dir: Option<String>,
        update_policy: UpdatePolicy,
        priority: u8,
        #[serde(default = "default_true")]
        trusted: bool,
        #[serde(default = "default_true")]
        enabled: bool,
    },
}

/// Update policy for git sources
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UpdatePolicy {
    Daily,
    Weekly,
    Manual,
    Always,  // Update on every access (development mode)
}

fn default_true() -> bool { true }
```

Example configuration file:

```yaml
# ~/.claude/task-sources.yaml
sources:
  # Local project tasks (highest priority)
  - type: local
    name: "project-tasks"
    path: "./.claude/tasks"
    priority: 10
    enabled: true

  # Local user tasks
  - type: local
    name: "user-tasks"
    path: "~/.claude/tasks"
    priority: 8
    enabled: true

  # Official git repository
  - type: git
    name: "official-tasks"
    url: "https://github.com/claude-tasks/official"
    branch: "main"
    cache_dir: "~/.claude/cache/official-tasks"
    update_policy: daily
    priority: 5
    trusted: true
    enabled: true

  # Google integrations
  - type: git
    name: "google-integrations"
    url: "https://github.com/claude-tasks/google-integrations"
    tag: "v2.1.0"  # Pin to specific version
    cache_dir: "~/.claude/cache/google-integrations"
    update_policy: manual
    priority: 5
    trusted: true
    enabled: true

  # Community tasks (lower priority)
  - type: git
    name: "community-tasks"
    url: "https://github.com/claude-tasks/community"
    branch: "main"
    update_policy: weekly
    priority: 3
    trusted: false
    enabled: true
```

### Component Design

#### 1. Git Source Implementation

```rust
// src/dsl/predefined_tasks/sources/git.rs

use git2::{Repository, FetchOptions, build::CheckoutBuilder};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc, Duration};

pub struct GitTaskSource {
    name: String,
    url: String,
    branch: Option<String>,
    tag: Option<String>,
    cache_dir: PathBuf,
    update_policy: UpdatePolicy,
    priority: u8,
    trusted: bool,

    // Runtime state
    repo: Option<Repository>,
    last_update: Option<DateTime<Utc>>,
}

impl GitTaskSource {
    pub fn new(config: GitSourceConfig) -> Result<Self> {
        let cache_dir = expand_path(&config.cache_dir.unwrap_or_else(|| {
            format!("~/.claude/cache/{}", config.name)
        }))?;

        Ok(Self {
            name: config.name,
            url: config.url,
            branch: config.branch,
            tag: config.tag,
            cache_dir,
            update_policy: config.update_policy,
            priority: config.priority,
            trusted: config.trusted,
            repo: None,
            last_update: None,
        })
    }

    /// Initialize or open the git repository
    async fn ensure_repository(&mut self) -> Result<&Repository> {
        if self.repo.is_some() {
            return Ok(self.repo.as_ref().unwrap());
        }

        if self.cache_dir.exists() {
            // Open existing repository
            let repo = Repository::open(&self.cache_dir)?;
            self.repo = Some(repo);
        } else {
            // Clone repository
            std::fs::create_dir_all(&self.cache_dir)?;
            let repo = Repository::clone(&self.url, &self.cache_dir)?;

            // Checkout specific branch or tag if specified
            if let Some(ref tag) = self.tag {
                self.checkout_tag(&repo, tag)?;
            } else if let Some(ref branch) = self.branch {
                self.checkout_branch(&repo, branch)?;
            }

            self.repo = Some(repo);
            self.last_update = Some(Utc::now());
        }

        Ok(self.repo.as_ref().unwrap())
    }

    /// Pull latest changes from remote
    async fn pull_changes(&mut self) -> Result<bool> {
        let repo = self.ensure_repository().await?;

        // Fetch from remote
        let mut remote = repo.find_remote("origin")?;
        let mut fetch_options = FetchOptions::new();
        // TODO: Add authentication support

        let refspec = if let Some(ref branch) = self.branch {
            format!("+refs/heads/{}:refs/remotes/origin/{}", branch, branch)
        } else {
            "+refs/heads/*:refs/remotes/origin/*".to_string()
        };

        remote.fetch(&[&refspec], Some(&mut fetch_options), None)?;

        // Check if there are new commits
        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
        let head = repo.head()?;
        let head_commit = repo.reference_to_annotated_commit(&head)?;

        let updated = fetch_commit.id() != head_commit.id();

        if updated {
            // Fast-forward merge
            let refname = format!("refs/heads/{}", self.branch.as_deref().unwrap_or("main"));
            repo.reference(&refname, fetch_commit.id(), true, "fast-forward")?;

            // Checkout
            let mut checkout_builder = CheckoutBuilder::new();
            checkout_builder.force();
            repo.checkout_head(Some(&mut checkout_builder))?;

            self.last_update = Some(Utc::now());
        }

        Ok(updated)
    }

    /// Check if update is needed based on policy
    fn should_update(&self) -> bool {
        match self.update_policy {
            UpdatePolicy::Always => true,
            UpdatePolicy::Manual => false,
            UpdatePolicy::Daily => {
                self.last_update.map_or(true, |last| {
                    Utc::now() - last > Duration::days(1)
                })
            }
            UpdatePolicy::Weekly => {
                self.last_update.map_or(true, |last| {
                    Utc::now() - last > Duration::weeks(1)
                })
            }
        }
    }

    /// Discover all .task.yaml files in repository
    async fn scan_tasks(&self) -> Result<Vec<PathBuf>> {
        let repo_path = &self.cache_dir;
        let mut task_files = Vec::new();

        fn visit_dirs(dir: &Path, task_files: &mut Vec<PathBuf>) -> Result<()> {
            if dir.is_dir() {
                for entry in std::fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();

                    if path.is_dir() {
                        visit_dirs(&path, task_files)?;
                    } else if path.extension().and_then(|s| s.to_str()) == Some("yaml")
                        && path.file_name()
                            .and_then(|s| s.to_str())
                            .map_or(false, |s| s.ends_with(".task.yaml"))
                    {
                        task_files.push(path);
                    }
                }
            }
            Ok(())
        }

        visit_dirs(repo_path, &mut task_files)?;
        Ok(task_files)
    }

    fn checkout_branch(&self, repo: &Repository, branch: &str) -> Result<()> {
        let refname = format!("refs/remotes/origin/{}", branch);
        let obj = repo.revparse_single(&refname)?;

        let mut checkout_builder = CheckoutBuilder::new();
        checkout_builder.force();
        repo.checkout_tree(&obj, Some(&mut checkout_builder))?;
        repo.set_head(&refname)?;

        Ok(())
    }

    fn checkout_tag(&self, repo: &Repository, tag: &str) -> Result<()> {
        let refname = format!("refs/tags/{}", tag);
        let obj = repo.revparse_single(&refname)?;

        let mut checkout_builder = CheckoutBuilder::new();
        checkout_builder.force();
        repo.checkout_tree(&obj, Some(&mut checkout_builder))?;
        repo.set_head_detached(obj.id())?;

        Ok(())
    }
}

#[async_trait]
impl TaskSource for GitTaskSource {
    fn name(&self) -> &str {
        &self.name
    }

    fn source_type(&self) -> SourceType {
        SourceType::Git
    }

    fn priority(&self) -> u8 {
        self.priority
    }

    fn is_trusted(&self) -> bool {
        self.trusted
    }

    async fn discover_tasks(&self) -> Result<Vec<TaskMetadata>> {
        // Check if update needed
        if self.should_update() {
            let _ = self.update().await;
        }

        let task_files = self.scan_tasks().await?;
        let mut metadata = Vec::new();

        for task_file in task_files {
            match parse_predefined_task(&task_file).await {
                Ok(task) => {
                    metadata.push(TaskMetadata {
                        name: task.metadata.name.clone(),
                        version: task.metadata.version.clone(),
                        description: task.metadata.description.clone(),
                        author: task.metadata.author.clone(),
                        tags: task.metadata.tags.clone(),
                        source_name: self.name.clone(),
                        source_type: SourceType::Git,
                    });
                }
                Err(e) => {
                    eprintln!("Warning: Failed to parse task file {}: {}",
                        task_file.display(), e);
                }
            }
        }

        Ok(metadata)
    }

    async fn load_task(&self, name: &str, version: Option<&str>) -> Result<PredefinedTask> {
        // Check if update needed
        if self.should_update() {
            let _ = self.update().await;
        }

        let task_files = self.scan_tasks().await?;

        // Find matching task
        for task_file in task_files {
            if let Ok(task) = parse_predefined_task(&task_file).await {
                if task.metadata.name == name {
                    if let Some(ver) = version {
                        if task.metadata.version == ver {
                            return Ok(task);
                        }
                    } else {
                        return Ok(task);
                    }
                }
            }
        }

        Err(Error::TaskNotFound {
            name: name.to_string(),
            version: version.map(String::from),
            source: self.name.clone(),
        })
    }

    async fn update(&self) -> Result<UpdateResult> {
        let updated = self.pull_changes().await?;

        Ok(UpdateResult {
            updated,
            message: if updated {
                format!("Updated git source: {}", self.name)
            } else {
                format!("Git source already up to date: {}", self.name)
            },
            new_tasks: 0,  // TODO: Track this
            updated_tasks: 0,  // TODO: Track this
        })
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let available = self.cache_dir.exists() ||
            can_reach_url(&self.url).await;

        Ok(HealthStatus {
            available,
            message: if available {
                None
            } else {
                Some(format!("Cannot reach git repository: {}", self.url))
            },
            last_check: Utc::now(),
        })
    }
}
```

#### 2. Local Source Implementation

```rust
// src/dsl/predefined_tasks/sources/local.rs

pub struct LocalTaskSource {
    name: String,
    path: PathBuf,
    priority: u8,
}

impl LocalTaskSource {
    pub fn new(name: String, path: impl AsRef<Path>, priority: u8) -> Result<Self> {
        let path = expand_path(path.as_ref())?;
        Ok(Self { name, path, priority })
    }

    async fn scan_tasks(&self) -> Result<Vec<PathBuf>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let mut task_files = Vec::new();

        fn visit_dirs(dir: &Path, task_files: &mut Vec<PathBuf>) -> Result<()> {
            if dir.is_dir() {
                for entry in std::fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();

                    if path.is_dir() {
                        visit_dirs(&path, task_files)?;
                    } else if path.extension().and_then(|s| s.to_str()) == Some("yaml")
                        && path.file_name()
                            .and_then(|s| s.to_str())
                            .map_or(false, |s| s.ends_with(".task.yaml"))
                    {
                        task_files.push(path);
                    }
                }
            }
            Ok(())
        }

        visit_dirs(&self.path, &mut task_files)?;
        Ok(task_files)
    }
}

#[async_trait]
impl TaskSource for LocalTaskSource {
    fn name(&self) -> &str {
        &self.name
    }

    fn source_type(&self) -> SourceType {
        SourceType::Local
    }

    fn priority(&self) -> u8 {
        self.priority
    }

    fn is_trusted(&self) -> bool {
        true  // Local sources are always trusted
    }

    async fn discover_tasks(&self) -> Result<Vec<TaskMetadata>> {
        let task_files = self.scan_tasks().await?;
        let mut metadata = Vec::new();

        for task_file in task_files {
            match parse_predefined_task(&task_file).await {
                Ok(task) => {
                    metadata.push(TaskMetadata {
                        name: task.metadata.name.clone(),
                        version: task.metadata.version.clone(),
                        description: task.metadata.description.clone(),
                        author: task.metadata.author.clone(),
                        tags: task.metadata.tags.clone(),
                        source_name: self.name.clone(),
                        source_type: SourceType::Local,
                    });
                }
                Err(e) => {
                    eprintln!("Warning: Failed to parse task file {}: {}",
                        task_file.display(), e);
                }
            }
        }

        Ok(metadata)
    }

    async fn load_task(&self, name: &str, version: Option<&str>) -> Result<PredefinedTask> {
        let task_files = self.scan_tasks().await?;

        for task_file in task_files {
            if let Ok(task) = parse_predefined_task(&task_file).await {
                if task.metadata.name == name {
                    if let Some(ver) = version {
                        if task.metadata.version == ver {
                            return Ok(task);
                        }
                    } else {
                        return Ok(task);
                    }
                }
            }
        }

        Err(Error::TaskNotFound {
            name: name.to_string(),
            version: version.map(String::from),
            source: self.name.clone(),
        })
    }

    async fn update(&self) -> Result<UpdateResult> {
        // Local sources don't need updating
        Ok(UpdateResult {
            updated: false,
            message: format!("Local source does not require updates: {}", self.name),
            new_tasks: 0,
            updated_tasks: 0,
        })
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let available = self.path.exists();

        Ok(HealthStatus {
            available,
            message: if available {
                None
            } else {
                Some(format!("Path does not exist: {}", self.path.display()))
            },
            last_check: Utc::now(),
        })
    }
}
```

#### 3. Package Manifest

```rust
// src/dsl/predefined_tasks/manifest.rs

/// Package manifest for a collection of tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPackage {
    #[serde(rename = "apiVersion")]
    pub api_version: String,

    pub kind: String,  // "TaskPackage"

    pub metadata: PackageMetadata,

    pub tasks: Vec<TaskReference>,

    #[serde(default)]
    pub dependencies: Vec<PackageDependency>,

    pub requires: Option<PackageRequirements>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub homepage: Option<String>,
    pub license: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskReference {
    pub path: String,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDependency {
    pub name: String,
    pub version: String,
    pub repository: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageRequirements {
    pub sdk_version: Option<String>,
    pub min_model: Option<String>,
}

impl TaskPackage {
    pub fn from_yaml(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let package: TaskPackage = serde_yaml::from_str(&content)?;

        // Validate
        if package.kind != "TaskPackage" {
            return Err(Error::InvalidPackageKind(package.kind.clone()));
        }

        Ok(package)
    }

    pub async fn load_tasks(&self, base_path: &Path) -> Result<Vec<PredefinedTask>> {
        let mut tasks = Vec::new();

        for task_ref in &self.tasks {
            let task_path = base_path.join(&task_ref.path);
            let task = parse_predefined_task(&task_path).await?;

            // Validate task matches reference
            if task.metadata.name != task_ref.name {
                return Err(Error::TaskReferenceMismatch {
                    expected: task_ref.name.clone(),
                    found: task.metadata.name.clone(),
                });
            }

            if task.metadata.version != task_ref.version {
                return Err(Error::TaskVersionMismatch {
                    task: task_ref.name.clone(),
                    expected: task_ref.version.clone(),
                    found: task.metadata.version.clone(),
                });
            }

            tasks.push(task);
        }

        Ok(tasks)
    }
}
```

#### 4. Caching Layer

```rust
// src/dsl/predefined_tasks/cache.rs

use std::collections::HashMap;
use std::time::{Duration, Instant};

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
    pub fn new(ttl: Duration) -> Self {
        Self {
            tasks: HashMap::new(),
            ttl,
        }
    }

    pub fn get(&self, key: &str) -> Option<&PredefinedTask> {
        self.tasks.get(key).and_then(|cached| {
            if cached.loaded_at.elapsed() < self.ttl {
                Some(&cached.task)
            } else {
                None
            }
        })
    }

    pub fn insert(&mut self, key: String, task: PredefinedTask, source: String) {
        self.tasks.insert(key, CachedTask {
            task,
            loaded_at: Instant::now(),
            source,
        });
    }

    pub fn invalidate(&mut self, key: &str) {
        self.tasks.remove(key);
    }

    pub fn clear(&mut self) {
        self.tasks.clear();
    }

    pub fn evict_expired(&mut self) {
        self.tasks.retain(|_, cached| cached.loaded_at.elapsed() < self.ttl);
    }

    fn cache_key(name: &str, version: Option<&str>) -> String {
        match version {
            Some(v) => format!("{}@{}", name, v),
            None => name.to_string(),
        }
    }
}
```

#### 5. Multi-Source Task Discovery

```rust
// src/dsl/predefined_tasks/discovery.rs

pub struct TaskDiscovery {
    sources: Vec<Box<dyn TaskSource>>,
    cache: TaskCache,
}

impl TaskDiscovery {
    pub async fn from_config(config_path: &Path) -> Result<Self> {
        let config = TaskSourcesConfig::from_yaml(config_path)?;
        let mut sources: Vec<Box<dyn TaskSource>> = Vec::new();

        for source_config in config.sources {
            if !source_config.is_enabled() {
                continue;
            }

            let source: Box<dyn TaskSource> = match source_config {
                SourceConfig::Local { name, path, priority, .. } => {
                    Box::new(LocalTaskSource::new(name, path, priority)?)
                }
                SourceConfig::Git { name, url, branch, tag, cache_dir,
                    update_policy, priority, trusted, .. } => {
                    Box::new(GitTaskSource::new(GitSourceConfig {
                        name,
                        url,
                        branch,
                        tag,
                        cache_dir,
                        update_policy,
                        priority,
                        trusted,
                    })?)
                }
            };

            sources.push(source);
        }

        // Sort by priority (descending)
        sources.sort_by(|a, b| b.priority().cmp(&a.priority()));

        Ok(Self {
            sources,
            cache: TaskCache::new(Duration::from_secs(300)), // 5 min TTL
        })
    }

    pub async fn with_default_sources() -> Result<Self> {
        let mut sources: Vec<Box<dyn TaskSource>> = Vec::new();

        // Add project local source
        if let Ok(project_source) = LocalTaskSource::new(
            "project-tasks".to_string(),
            "./.claude/tasks",
            10,
        ) {
            sources.push(Box::new(project_source));
        }

        // Add user global source
        if let Ok(user_dir) = dirs::home_dir() {
            let user_tasks_path = user_dir.join(".claude/tasks");
            if let Ok(user_source) = LocalTaskSource::new(
                "user-tasks".to_string(),
                user_tasks_path,
                8,
            ) {
                sources.push(Box::new(user_source));
            }
        }

        sources.sort_by(|a, b| b.priority().cmp(&a.priority()));

        Ok(Self {
            sources,
            cache: TaskCache::new(Duration::from_secs(300)),
        })
    }

    pub async fn discover_all(&self) -> Result<Vec<TaskMetadata>> {
        let mut all_metadata = Vec::new();

        for source in &self.sources {
            match source.discover_tasks().await {
                Ok(metadata) => all_metadata.extend(metadata),
                Err(e) => {
                    eprintln!("Warning: Failed to discover tasks from {}: {}",
                        source.name(), e);
                }
            }
        }

        Ok(all_metadata)
    }

    pub async fn find_task(&mut self, name: &str, version: Option<&str>)
        -> Result<PredefinedTask>
    {
        // Check cache first
        let cache_key = TaskCache::cache_key(name, version);
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        // Search sources by priority
        for source in &self.sources {
            match source.load_task(name, version).await {
                Ok(task) => {
                    // Cache the result
                    self.cache.insert(
                        cache_key,
                        task.clone(),
                        source.name().to_string(),
                    );
                    return Ok(task);
                }
                Err(_) => continue,  // Try next source
            }
        }

        Err(Error::TaskNotFoundInAnySources {
            name: name.to_string(),
            version: version.map(String::from),
            searched_sources: self.sources.iter()
                .map(|s| s.name().to_string())
                .collect(),
        })
    }

    pub async fn search(&self, query: &str) -> Result<Vec<TaskMetadata>> {
        let all_tasks = self.discover_all().await?;

        let query_lower = query.to_lowercase();
        let results: Vec<TaskMetadata> = all_tasks
            .into_iter()
            .filter(|task| {
                task.name.to_lowercase().contains(&query_lower)
                    || task.description.as_ref()
                        .map_or(false, |d| d.to_lowercase().contains(&query_lower))
                    || task.tags.iter()
                        .any(|t| t.to_lowercase().contains(&query_lower))
            })
            .collect();

        Ok(results)
    }

    pub async fn list_by_tag(&self, tag: &str) -> Result<Vec<TaskMetadata>> {
        let all_tasks = self.discover_all().await?;

        let tag_lower = tag.to_lowercase();
        let results: Vec<TaskMetadata> = all_tasks
            .into_iter()
            .filter(|task| {
                task.tags.iter()
                    .any(|t| t.to_lowercase() == tag_lower)
            })
            .collect();

        Ok(results)
    }

    pub fn list_sources(&self) -> Vec<SourceInfo> {
        self.sources.iter()
            .map(|s| SourceInfo {
                name: s.name().to_string(),
                source_type: s.source_type(),
                priority: s.priority(),
                trusted: s.is_trusted(),
            })
            .collect()
    }

    pub async fn update_all(&self) -> Result<Vec<UpdateResult>> {
        let mut results = Vec::new();

        for source in &self.sources {
            match source.update().await {
                Ok(result) => results.push(result),
                Err(e) => {
                    eprintln!("Warning: Failed to update {}: {}", source.name(), e);
                }
            }
        }

        Ok(results)
    }

    pub async fn health_check_all(&self) -> Result<Vec<(String, HealthStatus)>> {
        let mut results = Vec::new();

        for source in &self.sources {
            match source.health_check().await {
                Ok(status) => results.push((source.name().to_string(), status)),
                Err(e) => {
                    eprintln!("Warning: Health check failed for {}: {}",
                        source.name(), e);
                }
            }
        }

        Ok(results)
    }
}

#[derive(Debug, Clone)]
pub struct SourceInfo {
    pub name: String,
    pub source_type: SourceType,
    pub priority: u8,
    pub trusted: bool,
}
```

## Dependencies

Add to `Cargo.toml`:

```toml
[dependencies]
# Existing dependencies...

# Git integration
git2 = "0.18"

# Path expansion and system directories
dirs = "5.0"  # Already present
shellexpand = "3.0"  # For tilde expansion

# Date/time for caching
chrono = { version = "0.4", features = ["serde"] }  # Already present
```

## Error Handling

Extend error types in `src/error.rs`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    // Existing errors...

    #[error("Task '{name}' not found in source '{source}'")]
    TaskNotFound {
        name: String,
        version: Option<String>,
        source: String,
    },

    #[error("Task '{name}' not found in any configured sources. Searched: {searched_sources:?}")]
    TaskNotFoundInAnySources {
        name: String,
        version: Option<String>,
        searched_sources: Vec<String>,
    },

    #[error("Git operation failed: {0}")]
    GitError(#[from] git2::Error),

    #[error("Invalid package kind: expected 'TaskPackage', found '{0}'")]
    InvalidPackageKind(String),

    #[error("Task reference mismatch: expected '{expected}', found '{found}'")]
    TaskReferenceMismatch {
        expected: String,
        found: String,
    },

    #[error("Task version mismatch for '{task}': expected '{expected}', found '{found}'")]
    TaskVersionMismatch {
        task: String,
        expected: String,
        found: String,
    },

    #[error("Source configuration error: {0}")]
    SourceConfigError(String),
}
```

## CLI Integration

Extend `dsl-executor` CLI with new commands:

```rust
// src/bin/dsl_executor.rs

#[derive(Subcommand)]
enum Commands {
    // Existing commands...

    /// Manage task sources
    Sources(SourcesCommand),
}

#[derive(Args)]
struct SourcesCommand {
    #[command(subcommand)]
    command: SourcesSubcommand,
}

#[derive(Subcommand)]
enum SourcesSubcommand {
    /// List all configured sources
    List,

    /// Add a new source
    Add {
        #[arg(long)]
        name: String,

        #[arg(long)]
        r#type: String,  // "local" or "git"

        #[arg(long)]
        path_or_url: String,

        #[arg(long)]
        priority: Option<u8>,

        #[arg(long)]
        branch: Option<String>,

        #[arg(long)]
        tag: Option<String>,
    },

    /// Remove a source
    Remove {
        name: String,
    },

    /// Update all sources (git pull)
    Update {
        #[arg(long)]
        source: Option<String>,  // Update specific source
    },

    /// Check health of all sources
    Health,
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_local_source_discovery() {
        let temp_dir = TempDir::new().unwrap();
        let tasks_dir = temp_dir.path().join("tasks");
        std::fs::create_dir(&tasks_dir).unwrap();

        // Create test task
        let task_yaml = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "test-task"
  version: "1.0.0"
spec:
  agent_template:
    description: "Test task"
  inputs:
    input1: {type: string, required: true}
"#;
        std::fs::write(
            tasks_dir.join("test.task.yaml"),
            task_yaml,
        ).unwrap();

        let source = LocalTaskSource::new(
            "test".to_string(),
            &tasks_dir,
            10,
        ).unwrap();

        let tasks = source.discover_tasks().await.unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name, "test-task");
    }

    #[tokio::test]
    async fn test_priority_resolution() {
        // Create two sources with different priorities
        // Add same task to both
        // Verify higher priority wins
    }

    #[tokio::test]
    async fn test_git_source_cloning() {
        // Test cloning a public repo
        // Verify tasks are discovered
    }

    #[tokio::test]
    async fn test_package_manifest_parsing() {
        // Test parsing package.yaml
        // Verify task references are loaded correctly
    }
}
```

### Integration Tests

```rust
// tests/predefined_tasks_phase2_tests.rs

#[tokio::test]
async fn test_multi_source_discovery() {
    // Set up multiple sources
    // Discover all tasks
    // Verify correct priority order
}

#[tokio::test]
async fn test_git_update_policy() {
    // Test daily/weekly/manual update policies
    // Verify updates happen at correct intervals
}

#[tokio::test]
async fn test_task_caching() {
    // Load task multiple times
    // Verify cache is used
    // Test cache expiration
}
```

## Implementation Checklist

- [ ] Create `src/dsl/predefined_tasks/sources/mod.rs` with `TaskSource` trait
- [ ] Create `src/dsl/predefined_tasks/sources/config.rs` with configuration types
- [ ] Create `src/dsl/predefined_tasks/sources/local.rs` with `LocalTaskSource`
- [ ] Create `src/dsl/predefined_tasks/sources/git.rs` with `GitTaskSource`
- [ ] Create `src/dsl/predefined_tasks/cache.rs` with `TaskCache`
- [ ] Create `src/dsl/predefined_tasks/manifest.rs` with `TaskPackage`
- [ ] Create `src/dsl/predefined_tasks/discovery.rs` with `TaskDiscovery`
- [ ] Update `src/error.rs` with new error types
- [ ] Add git2 and related dependencies to `Cargo.toml`
- [ ] Extend CLI with `sources` subcommands
- [ ] Write unit tests for each component
- [ ] Write integration tests for multi-source scenarios
- [ ] Update documentation

## Timeline

**Total Duration**: 2 weeks

**Week 1**:
- Day 1-2: Core abstractions (TaskSource trait, config types)
- Day 3-4: Local and Git source implementations
- Day 5: Package manifest parsing

**Week 2**:
- Day 1-2: Caching layer and task discovery
- Day 3: CLI integration
- Day 4-5: Testing and documentation

## Success Criteria

1. ✅ Git repositories can be configured as task sources
2. ✅ Tasks are automatically discovered from multiple sources
3. ✅ Priority-based resolution works correctly
4. ✅ Update policies (daily/weekly/manual) function as expected
5. ✅ Package manifests are parsed correctly
6. ✅ Caching reduces redundant git operations
7. ✅ CLI provides source management commands
8. ✅ All tests pass with >85% coverage
9. ✅ Zero breaking changes to Phase 1 functionality
