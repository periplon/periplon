//! Git Repository Task Source
//!
//! Discovers and loads predefined tasks from git repositories with caching and update policies.

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use git2::{build::CheckoutBuilder, FetchOptions, Repository};
use std::path::{Path, PathBuf};

use crate::dsl::predefined_tasks::{load_predefined_task, PredefinedTask};
use crate::error::Result;

use super::config::{expand_path, UpdatePolicy};
use super::{HealthStatus, SourceType, TaskMetadata, TaskSource, UpdateResult};

/// Git repository task source
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
    last_update: Option<DateTime<Utc>>,
}

impl GitTaskSource {
    /// Create a new git task source
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        url: String,
        branch: Option<String>,
        tag: Option<String>,
        cache_dir: Option<String>,
        update_policy: UpdatePolicy,
        priority: u8,
        trusted: bool,
    ) -> Result<Self> {
        let cache_dir = if let Some(dir) = cache_dir {
            expand_path(&dir)?
        } else {
            // Default cache location: ~/.claude/cache/{source-name}
            let home = dirs::home_dir().ok_or_else(|| {
                crate::error::Error::SourceConfigError(
                    "Cannot determine home directory".to_string(),
                )
            })?;
            home.join(".claude/cache").join(&name)
        };

        Ok(Self {
            name,
            url,
            branch,
            tag,
            cache_dir,
            update_policy,
            priority,
            trusted,
            last_update: None,
        })
    }

    /// Initialize or open the git repository
    async fn ensure_repository(&mut self) -> Result<Repository> {
        if self.cache_dir.exists() {
            // Open existing repository
            let repo = Repository::open(&self.cache_dir)?;
            Ok(repo)
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

            self.last_update = Some(Utc::now());
            Ok(repo)
        }
    }

    /// Pull latest changes from remote
    async fn pull_changes(&mut self) -> Result<bool> {
        let repo = self.ensure_repository().await?;

        // If tag is specified, don't pull (tags are immutable)
        if self.tag.is_some() {
            return Ok(false);
        }

        // Fetch from remote
        let mut remote = repo.find_remote("origin")?;
        let mut fetch_options = FetchOptions::new();

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
        // If tag is specified, never update (immutable)
        if self.tag.is_some() {
            return false;
        }

        match self.update_policy {
            UpdatePolicy::Always => true,
            UpdatePolicy::Manual => false,
            UpdatePolicy::Daily => self
                .last_update
                .is_none_or(|last| Utc::now() - last > Duration::days(1)),
            UpdatePolicy::Weekly => self
                .last_update
                .is_none_or(|last| Utc::now() - last > Duration::weeks(1)),
        }
    }

    /// Discover all .task.yaml files in repository
    async fn scan_tasks(&self) -> Result<Vec<PathBuf>> {
        let mut task_files = Vec::new();
        self.visit_dirs(&self.cache_dir, &mut task_files)?;
        Ok(task_files)
    }

    /// Recursively visit directories to find task files
    fn visit_dirs(&self, dir: &Path, task_files: &mut Vec<PathBuf>) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            // Skip .git directory
            if path.file_name().and_then(|s| s.to_str()) == Some(".git") {
                continue;
            }

            if path.is_dir() {
                self.visit_dirs(&path, task_files)?;
            } else if self.is_task_file(&path) {
                task_files.push(path);
            }
        }

        Ok(())
    }

    /// Check if a file is a task definition file
    fn is_task_file(&self, path: &Path) -> bool {
        path.extension().and_then(|s| s.to_str()) == Some("yaml")
            && path
                .file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s.ends_with(".task.yaml"))
    }

    /// Checkout a specific branch
    fn checkout_branch(&self, repo: &Repository, branch: &str) -> Result<()> {
        let refname = format!("refs/remotes/origin/{}", branch);
        let obj = repo.revparse_single(&refname)?;

        let mut checkout_builder = CheckoutBuilder::new();
        checkout_builder.force();
        repo.checkout_tree(&obj, Some(&mut checkout_builder))?;
        repo.set_head(&refname)?;

        Ok(())
    }

    /// Checkout a specific tag
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

    async fn discover_tasks(&mut self) -> Result<Vec<TaskMetadata>> {
        // Check if update needed
        if self.should_update() {
            let _ = self.update().await;
        } else {
            // Ensure repo is cloned
            let _ = self.ensure_repository().await?;
        }

        let task_files = self.scan_tasks().await?;
        let mut metadata = Vec::new();

        for task_file in task_files {
            match load_predefined_task(&task_file) {
                Ok(task) => {
                    metadata.push(TaskMetadata::from((
                        &task.metadata,
                        self.name.as_str(),
                        SourceType::Git,
                    )));
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to parse task file {}: {}",
                        task_file.display(),
                        e
                    );
                }
            }
        }

        Ok(metadata)
    }

    async fn load_task(&mut self, name: &str, version: Option<&str>) -> Result<PredefinedTask> {
        // Check if update needed
        if self.should_update() {
            let _ = self.update().await;
        } else {
            // Ensure repo is cloned
            let _ = self.ensure_repository().await?;
        }

        let task_files = self.scan_tasks().await?;

        // Find matching task
        for task_file in task_files {
            if let Ok(task) = load_predefined_task(&task_file) {
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

        Err(crate::error::Error::TaskNotFound {
            name: name.to_string(),
            version: version.map(String::from),
            source_name: self.name.clone(),
        })
    }

    async fn update(&mut self) -> Result<UpdateResult> {
        let updated = self.pull_changes().await?;

        Ok(UpdateResult {
            updated,
            message: if updated {
                format!("Updated git source: {}", self.name)
            } else {
                format!("Git source already up to date: {}", self.name)
            },
            new_tasks: 0,     // TODO: Track this
            updated_tasks: 0, // TODO: Track this
        })
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let available = self.cache_dir.exists();

        Ok(HealthStatus {
            available,
            message: if available {
                None
            } else {
                Some(format!(
                    "Repository not cloned. Run update to clone: {}",
                    self.url
                ))
            },
            last_check: Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_update_policy() {
        let source = GitTaskSource::new(
            "test".to_string(),
            "https://github.com/test/repo".to_string(),
            None,
            None,
            None,
            UpdatePolicy::Always,
            5,
            true,
        )
        .unwrap();

        assert!(source.should_update());
    }

    #[test]
    fn test_should_not_update_with_tag() {
        let source = GitTaskSource::new(
            "test".to_string(),
            "https://github.com/test/repo".to_string(),
            None,
            Some("v1.0.0".to_string()),
            None,
            UpdatePolicy::Always,
            5,
            true,
        )
        .unwrap();

        // Tags are immutable, should never update
        assert!(!source.should_update());
    }

    #[test]
    fn test_should_update_daily_policy() {
        let mut source = GitTaskSource::new(
            "test".to_string(),
            "https://github.com/test/repo".to_string(),
            None,
            None,
            None,
            UpdatePolicy::Daily,
            5,
            true,
        )
        .unwrap();

        // No last update, should update
        assert!(source.should_update());

        // Set last update to now
        source.last_update = Some(Utc::now());
        assert!(!source.should_update());

        // Set last update to 2 days ago
        source.last_update = Some(Utc::now() - Duration::days(2));
        assert!(source.should_update());
    }
}
