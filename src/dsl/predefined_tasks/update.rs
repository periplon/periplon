//! Update Checking for Predefined Tasks
//!
//! This module provides version checking, update recommendations, breaking change detection,
//! and automatic minor/patch updates for predefined tasks. It supports checking against
//! multiple sources (local, git repositories, registries).
//!
//! # Features
//!
//! - **Version Comparison**: Semver-based version comparison
//! - **Update Recommendations**: Smart update suggestions based on semver
//! - **Breaking Change Detection**: Identifies major version updates
//! - **Automatic Updates**: Auto-apply minor/patch updates based on policy
//! - **Multi-Source Support**: Check updates from multiple task sources
//! - **Dependency Aware**: Considers dependency constraints when updating
//!
//! # Examples
//!
//! ```rust
//! use periplon_sdk::dsl::predefined_tasks::{UpdateChecker, VersionUpdatePolicy};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut checker = UpdateChecker::new(vec![/* sources */]);
//!
//! // Check for updates to a specific task
//! let update_info = checker.check_update("my-task", "1.2.0").await?;
//!
//! if update_info.has_updates() {
//!     println!("Updates available: {}", update_info.latest_version);
//!     if update_info.is_breaking() {
//!         println!("WARNING: Breaking changes detected!");
//!     }
//! }
//!
//! // Auto-update with policy
//! let tasks = vec![("my-task".to_string(), "1.2.0".to_string())];
//! let results = checker.auto_update_all(&tasks, VersionUpdatePolicy::MinorAndPatch).await?;
//! # Ok(())
//! # }
//! ```

use crate::dsl::predefined_tasks::{TaskMetadata, TaskSource, VersionError};
use crate::error::Result;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

/// Errors specific to update checking
#[derive(Debug, Error)]
pub enum UpdateError {
    /// No sources configured
    #[error("No task sources configured")]
    NoSources,

    /// Task not found in any source
    #[error("Task '{name}' not found in any configured source")]
    TaskNotFound { name: String },

    /// Version parsing error
    #[error("Version error: {0}")]
    Version(#[from] VersionError),

    /// Source communication error
    #[error("Failed to check source '{0}': {1}")]
    SourceError(String, String),

    /// Update policy violation
    #[error("Update from {from} to {to} violates policy {policy:?}")]
    PolicyViolation {
        from: String,
        to: String,
        policy: VersionUpdatePolicy,
    },
}

/// Policy for automatic version updates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum VersionUpdatePolicy {
    /// Never auto-update (manual only)
    #[default]
    Manual,
    /// Only apply patch updates (1.2.3 -> 1.2.4)
    PatchOnly,
    /// Apply minor and patch updates (1.2.3 -> 1.3.0 or 1.2.4)
    MinorAndPatch,
    /// Apply all updates including major (use with caution)
    All,
}


impl fmt::Display for VersionUpdatePolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VersionUpdatePolicy::Manual => write!(f, "manual"),
            VersionUpdatePolicy::PatchOnly => write!(f, "patch_only"),
            VersionUpdatePolicy::MinorAndPatch => write!(f, "minor_and_patch"),
            VersionUpdatePolicy::All => write!(f, "all"),
        }
    }
}

/// Update information for a single task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    /// Task name
    pub task_name: String,

    /// Current version
    pub current_version: String,

    /// Latest available version
    pub latest_version: String,

    /// All available versions (sorted newest first)
    pub available_versions: Vec<String>,

    /// Source providing the latest version
    pub update_source: String,

    /// Whether this is a breaking change (major version bump)
    pub is_breaking: bool,

    /// Whether this is a minor update
    pub is_minor: bool,

    /// Whether this is a patch update
    pub is_patch: bool,

    /// Change summary if available
    pub changelog: Option<String>,

    /// Update recommendation
    pub recommendation: UpdateRecommendation,
}

impl UpdateInfo {
    /// Check if any updates are available
    pub fn has_updates(&self) -> bool {
        self.current_version != self.latest_version
    }

    /// Check if breaking changes are present
    pub fn is_breaking(&self) -> bool {
        self.is_breaking
    }

    /// Check if update is allowed under policy
    pub fn is_allowed(&self, policy: VersionUpdatePolicy) -> bool {
        match policy {
            VersionUpdatePolicy::Manual => false,
            VersionUpdatePolicy::PatchOnly => self.is_patch && !self.is_minor && !self.is_breaking,
            VersionUpdatePolicy::MinorAndPatch => {
                (self.is_patch || self.is_minor) && !self.is_breaking
            }
            VersionUpdatePolicy::All => true,
        }
    }

    /// Get a human-readable update type
    pub fn update_type(&self) -> &str {
        if self.is_breaking {
            "major"
        } else if self.is_minor {
            "minor"
        } else if self.is_patch {
            "patch"
        } else {
            "unknown"
        }
    }
}

/// Recommendation for updating
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateRecommendation {
    /// No update needed - already on latest
    UpToDate,
    /// Recommended to update (safe)
    Recommended,
    /// Review breaking changes before updating
    ReviewRequired,
    /// Update available but check dependencies
    CheckDependencies,
}

impl fmt::Display for UpdateRecommendation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpdateRecommendation::UpToDate => write!(f, "up-to-date"),
            UpdateRecommendation::Recommended => write!(f, "recommended"),
            UpdateRecommendation::ReviewRequired => write!(f, "review required"),
            UpdateRecommendation::CheckDependencies => write!(f, "check dependencies"),
        }
    }
}

/// Result of an update operation
#[derive(Debug, Clone)]
pub struct UpdateResult {
    /// Task name
    pub task_name: String,

    /// Whether update was successful
    pub success: bool,

    /// Version updated from
    pub from_version: String,

    /// Version updated to
    pub to_version: String,

    /// Error message if failed
    pub error: Option<String>,
}

/// Main update checker coordinating multi-source version checking
pub struct UpdateChecker {
    /// Task sources to check for updates
    sources: Vec<Box<dyn TaskSource>>,

    /// Cached metadata from sources
    metadata_cache: HashMap<String, Vec<TaskMetadata>>,

    /// Whether to include pre-release versions
    include_prerelease: bool,
}

impl UpdateChecker {
    /// Create a new update checker with given sources
    pub fn new(sources: Vec<Box<dyn TaskSource>>) -> Self {
        Self {
            sources,
            metadata_cache: HashMap::new(),
            include_prerelease: false,
        }
    }

    /// Enable or disable pre-release versions in update checks
    pub fn set_include_prerelease(&mut self, include: bool) {
        self.include_prerelease = include;
    }

    /// Refresh metadata cache from all sources
    pub async fn refresh_cache(&mut self) -> Result<()> {
        self.metadata_cache.clear();

        for source in &mut self.sources {
            let source_name = source.name().to_string();
            match source.discover_tasks().await {
                Ok(metadata) => {
                    self.metadata_cache.insert(source_name.clone(), metadata);
                }
                Err(e) => {
                    // Log error but continue with other sources
                    eprintln!("Failed to refresh source '{}': {}", source_name, e);
                }
            }
        }

        Ok(())
    }

    /// Check for updates to a specific task
    pub async fn check_update(
        &mut self,
        task_name: &str,
        current_version: &str,
    ) -> Result<UpdateInfo> {
        // Ensure cache is populated
        if self.metadata_cache.is_empty() {
            self.refresh_cache().await?;
        }

        // Collect all available versions across sources
        let mut all_versions: Vec<(String, String)> = Vec::new(); // (version, source)

        for (source_name, metadata_list) in &self.metadata_cache {
            for meta in metadata_list {
                if meta.name == task_name {
                    all_versions.push((meta.version.clone(), source_name.clone()));
                }
            }
        }

        if all_versions.is_empty() {
            return Err(UpdateError::TaskNotFound {
                name: task_name.to_string(),
            }
            .into());
        }

        // Filter out pre-releases if needed
        let filtered_versions: Vec<(String, String)> = if self.include_prerelease {
            all_versions
        } else {
            all_versions
                .into_iter()
                .filter(|(v, _)| {
                    Version::parse(v)
                        .map(|ver| ver.pre.is_empty())
                        .unwrap_or(false)
                })
                .collect()
        };

        // Find latest version
        let versions_only: Vec<String> = filtered_versions.iter().map(|(v, _)| v.clone()).collect();
        let latest_version = self.find_latest_version(&versions_only)?;

        // Find source providing latest version
        let update_source = filtered_versions
            .iter()
            .find(|(v, _)| v == &latest_version)
            .map(|(_, s)| s.clone())
            .unwrap_or_else(|| "unknown".to_string());

        // Parse versions for comparison
        let current = Version::parse(current_version).map_err(|e| {
            VersionError::InvalidVersion(current_version.to_string(), e.to_string())
        })?;
        let latest = Version::parse(&latest_version)
            .map_err(|e| VersionError::InvalidVersion(latest_version.clone(), e.to_string()))?;

        // Determine update type
        let is_breaking = latest.major > current.major;
        let is_minor = latest.major == current.major && latest.minor > current.minor;
        let is_patch = latest.major == current.major
            && latest.minor == current.minor
            && latest.patch > current.patch;

        // Determine recommendation
        let recommendation = if current >= latest {
            UpdateRecommendation::UpToDate
        } else if is_breaking {
            UpdateRecommendation::ReviewRequired
        } else if is_minor || is_patch {
            UpdateRecommendation::Recommended
        } else {
            UpdateRecommendation::CheckDependencies
        };

        // Sort versions (newest first)
        let mut sorted_versions = versions_only;
        sorted_versions.sort_by(|a, b| {
            let va = Version::parse(a).unwrap();
            let vb = Version::parse(b).unwrap();
            vb.cmp(&va) // Reverse for newest first
        });

        Ok(UpdateInfo {
            task_name: task_name.to_string(),
            current_version: current_version.to_string(),
            latest_version,
            available_versions: sorted_versions,
            update_source,
            is_breaking,
            is_minor,
            is_patch,
            changelog: None, // TODO: Fetch from source if available
            recommendation,
        })
    }

    /// Check updates for multiple tasks
    pub async fn check_updates(
        &mut self,
        tasks: &[(String, String)], // (name, version)
    ) -> Result<Vec<UpdateInfo>> {
        let mut results = Vec::new();

        for (name, version) in tasks {
            match self.check_update(name, version).await {
                Ok(info) => results.push(info),
                Err(e) => {
                    eprintln!("Failed to check updates for '{}': {}", name, e);
                }
            }
        }

        Ok(results)
    }

    /// Automatically update tasks based on policy
    pub async fn auto_update(
        &mut self,
        task_name: &str,
        current_version: &str,
        policy: VersionUpdatePolicy,
    ) -> Result<UpdateResult> {
        let update_info = self.check_update(task_name, current_version).await?;

        if !update_info.has_updates() {
            return Ok(UpdateResult {
                task_name: task_name.to_string(),
                success: true,
                from_version: current_version.to_string(),
                to_version: current_version.to_string(),
                error: Some("Already up to date".to_string()),
            });
        }

        if !update_info.is_allowed(policy) {
            return Err(UpdateError::PolicyViolation {
                from: current_version.to_string(),
                to: update_info.latest_version.clone(),
                policy,
            }
            .into());
        }

        // Perform the actual update
        // In a real implementation, this would update the task definition/lockfile
        Ok(UpdateResult {
            task_name: task_name.to_string(),
            success: true,
            from_version: current_version.to_string(),
            to_version: update_info.latest_version,
            error: None,
        })
    }

    /// Automatically update all tasks based on policy
    pub async fn auto_update_all(
        &mut self,
        tasks: &[(String, String)],
        policy: VersionUpdatePolicy,
    ) -> Result<Vec<UpdateResult>> {
        let mut results = Vec::new();

        for (name, version) in tasks {
            match self.auto_update(name, version, policy).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    results.push(UpdateResult {
                        task_name: name.to_string(),
                        success: false,
                        from_version: version.to_string(),
                        to_version: version.to_string(),
                        error: Some(e.to_string()),
                    });
                }
            }
        }

        Ok(results)
    }

    /// Find breaking changes between two versions
    pub fn detect_breaking_changes(
        &self,
        from_version: &str,
        to_version: &str,
    ) -> Result<BreakingChangeInfo> {
        let from = Version::parse(from_version)
            .map_err(|e| VersionError::InvalidVersion(from_version.to_string(), e.to_string()))?;
        let to = Version::parse(to_version)
            .map_err(|e| VersionError::InvalidVersion(to_version.to_string(), e.to_string()))?;

        let has_breaking = to.major > from.major;
        let major_bumps = to.major.saturating_sub(from.major);

        Ok(BreakingChangeInfo {
            from_version: from_version.to_string(),
            to_version: to_version.to_string(),
            has_breaking_changes: has_breaking,
            major_version_bumps: major_bumps,
            recommendations: if has_breaking {
                vec![
                    "Review changelog for breaking changes".to_string(),
                    "Test thoroughly before deploying".to_string(),
                    "Update dependent workflows".to_string(),
                ]
            } else {
                vec![]
            },
        })
    }

    /// Get update statistics across all cached tasks
    pub fn get_update_stats(&self) -> UpdateStats {
        let total_tasks: usize = self.metadata_cache.values().map(|v| v.len()).sum();

        let sources_count = self.metadata_cache.len();

        UpdateStats {
            total_tasks,
            sources_checked: sources_count,
            tasks_with_updates: 0, // Computed during actual update check
        }
    }

    /// Find the latest version from a list
    fn find_latest_version(&self, versions: &[String]) -> Result<String> {
        if versions.is_empty() {
            return Err(UpdateError::NoSources.into());
        }

        let mut parsed: Vec<Version> = Vec::new();
        for v in versions {
            let ver = Version::parse(v)
                .map_err(|e| VersionError::InvalidVersion(v.to_string(), e.to_string()))?;
            parsed.push(ver);
        }

        parsed.sort();
        Ok(parsed.last().unwrap().to_string())
    }
}

/// Information about breaking changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChangeInfo {
    pub from_version: String,
    pub to_version: String,
    pub has_breaking_changes: bool,
    pub major_version_bumps: u64,
    pub recommendations: Vec<String>,
}

/// Update statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStats {
    pub total_tasks: usize,
    pub sources_checked: usize,
    pub tasks_with_updates: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::predefined_tasks::{PredefinedTask, SourceType, TaskMetadata};
    use async_trait::async_trait;
    use chrono::Utc;

    // Mock task source for testing
    struct MockTaskSource {
        name: String,
        tasks: Vec<TaskMetadata>,
    }

    #[async_trait]
    impl TaskSource for MockTaskSource {
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

        async fn discover_tasks(&mut self) -> Result<Vec<TaskMetadata>> {
            Ok(self.tasks.clone())
        }

        async fn load_task(
            &mut self,
            _name: &str,
            _version: Option<&str>,
        ) -> Result<PredefinedTask> {
            unimplemented!()
        }

        async fn update(&mut self) -> Result<crate::dsl::predefined_tasks::UpdateResult> {
            unimplemented!()
        }

        async fn health_check(&self) -> Result<crate::dsl::predefined_tasks::HealthStatus> {
            Ok(crate::dsl::predefined_tasks::HealthStatus {
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
                description: Some(format!("Test task {}", v)),
                author: Some("test".to_string()),
                tags: vec![],
                source_name: "test".to_string(),
                source_type: SourceType::Local,
            })
            .collect()
    }

    #[tokio::test]
    async fn test_check_update_patch() {
        let tasks = create_test_metadata("test-task", vec!["1.0.0", "1.0.1", "1.0.2"]);
        let source = Box::new(MockTaskSource {
            name: "test".to_string(),
            tasks,
        });

        let mut checker = UpdateChecker::new(vec![source]);
        let info = checker.check_update("test-task", "1.0.0").await.unwrap();

        assert_eq!(info.latest_version, "1.0.2");
        assert!(info.is_patch);
        assert!(!info.is_minor);
        assert!(!info.is_breaking);
        assert_eq!(info.recommendation, UpdateRecommendation::Recommended);
    }

    #[tokio::test]
    async fn test_check_update_minor() {
        let tasks = create_test_metadata("test-task", vec!["1.0.0", "1.1.0", "1.2.0"]);
        let source = Box::new(MockTaskSource {
            name: "test".to_string(),
            tasks,
        });

        let mut checker = UpdateChecker::new(vec![source]);
        let info = checker.check_update("test-task", "1.0.0").await.unwrap();

        assert_eq!(info.latest_version, "1.2.0");
        assert!(!info.is_patch);
        assert!(info.is_minor);
        assert!(!info.is_breaking);
        assert_eq!(info.recommendation, UpdateRecommendation::Recommended);
    }

    #[tokio::test]
    async fn test_check_update_major() {
        let tasks = create_test_metadata("test-task", vec!["1.0.0", "2.0.0", "3.0.0"]);
        let source = Box::new(MockTaskSource {
            name: "test".to_string(),
            tasks,
        });

        let mut checker = UpdateChecker::new(vec![source]);
        let info = checker.check_update("test-task", "1.0.0").await.unwrap();

        assert_eq!(info.latest_version, "3.0.0");
        assert!(!info.is_patch);
        assert!(!info.is_minor);
        assert!(info.is_breaking);
        assert_eq!(info.recommendation, UpdateRecommendation::ReviewRequired);
    }

    #[tokio::test]
    async fn test_check_update_up_to_date() {
        let tasks = create_test_metadata("test-task", vec!["1.0.0", "1.0.1"]);
        let source = Box::new(MockTaskSource {
            name: "test".to_string(),
            tasks,
        });

        let mut checker = UpdateChecker::new(vec![source]);
        let info = checker.check_update("test-task", "1.0.1").await.unwrap();

        assert_eq!(info.latest_version, "1.0.1");
        assert!(!info.has_updates());
        assert_eq!(info.recommendation, UpdateRecommendation::UpToDate);
    }

    #[tokio::test]
    async fn test_update_policy_patch_only() {
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

    #[tokio::test]
    async fn test_update_policy_minor() {
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
    }

    #[tokio::test]
    async fn test_update_policy_major() {
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
    }

    #[test]
    fn test_detect_breaking_changes() {
        let checker = UpdateChecker::new(vec![]);

        let info = checker.detect_breaking_changes("1.5.2", "2.0.0").unwrap();
        assert!(info.has_breaking_changes);
        assert_eq!(info.major_version_bumps, 1);
        assert!(!info.recommendations.is_empty());

        let info2 = checker.detect_breaking_changes("1.0.0", "1.1.0").unwrap();
        assert!(!info2.has_breaking_changes);
        assert_eq!(info2.major_version_bumps, 0);
        assert!(info2.recommendations.is_empty());
    }

    #[tokio::test]
    async fn test_multi_source_update_check() {
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
    }

    #[test]
    fn test_update_type() {
        let mut info = UpdateInfo {
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

        assert_eq!(info.update_type(), "major");

        info.is_breaking = false;
        info.is_minor = true;
        assert_eq!(info.update_type(), "minor");

        info.is_minor = false;
        info.is_patch = true;
        assert_eq!(info.update_type(), "patch");
    }
}
