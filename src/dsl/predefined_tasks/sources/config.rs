//! Source Configuration
//!
//! Configuration types for task sources.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::error::Result;

/// Configuration for all task sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSourcesConfig {
    pub sources: Vec<SourceConfig>,
}

impl TaskSourcesConfig {
    /// Load configuration from YAML file
    pub fn from_yaml(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: TaskSourcesConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// Load from default location (~/.claude/task-sources.yaml)
    pub fn from_default_path() -> Result<Self> {
        let home = dirs::home_dir().ok_or_else(|| {
            crate::error::Error::SourceConfigError("Cannot determine home directory".to_string())
        })?;
        let config_path = home.join(".claude/task-sources.yaml");
        Self::from_yaml(&config_path)
    }

    /// Get enabled sources only
    pub fn enabled_sources(&self) -> Vec<SourceConfig> {
        self.sources
            .iter()
            .filter(|s| s.is_enabled())
            .cloned()
            .collect()
    }
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
        #[serde(skip_serializing_if = "Option::is_none")]
        branch: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tag: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_dir: Option<String>,
        #[serde(default = "default_update_policy")]
        update_policy: UpdatePolicy,
        priority: u8,
        #[serde(default = "default_true")]
        trusted: bool,
        #[serde(default = "default_true")]
        enabled: bool,
    },
}

impl SourceConfig {
    pub fn name(&self) -> &str {
        match self {
            SourceConfig::Local { name, .. } => name,
            SourceConfig::Git { name, .. } => name,
        }
    }

    pub fn priority(&self) -> u8 {
        match self {
            SourceConfig::Local { priority, .. } => *priority,
            SourceConfig::Git { priority, .. } => *priority,
        }
    }

    pub fn is_enabled(&self) -> bool {
        match self {
            SourceConfig::Local { enabled, .. } => *enabled,
            SourceConfig::Git { enabled, .. } => *enabled,
        }
    }

    pub fn is_trusted(&self) -> bool {
        match self {
            SourceConfig::Local { .. } => true, // Local sources always trusted
            SourceConfig::Git { trusted, .. } => *trusted,
        }
    }
}

/// Update policy for git sources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum UpdatePolicy {
    #[default]
    Daily,
    Weekly,
    Manual,
    Always, // Update on every access (development mode)
}

fn default_true() -> bool {
    true
}

fn default_update_policy() -> UpdatePolicy {
    UpdatePolicy::Daily
}

/// Expand path with tilde and environment variables
pub fn expand_path(path: &str) -> Result<PathBuf> {
    let expanded = shellexpand::full(path).map_err(|e| {
        crate::error::Error::SourceConfigError(format!("Failed to expand path '{}': {}", path, e))
    })?;
    Ok(PathBuf::from(expanded.as_ref()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_local_source_config() {
        let yaml = r#"
sources:
  - type: local
    name: "test-local"
    path: "./.claude/tasks"
    priority: 10
    enabled: true
"#;
        let config: TaskSourcesConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.sources.len(), 1);
        assert_eq!(config.sources[0].name(), "test-local");
        assert_eq!(config.sources[0].priority(), 10);
    }

    #[test]
    fn test_parse_git_source_config() {
        let yaml = r#"
sources:
  - type: git
    name: "test-git"
    url: "https://github.com/test/repo"
    branch: "main"
    update_policy: daily
    priority: 5
    trusted: true
    enabled: true
"#;
        let config: TaskSourcesConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.sources.len(), 1);

        if let SourceConfig::Git {
            name,
            url,
            branch,
            update_policy,
            ..
        } = &config.sources[0]
        {
            assert_eq!(name, "test-git");
            assert_eq!(url, "https://github.com/test/repo");
            assert_eq!(branch.as_deref(), Some("main"));
            assert_eq!(*update_policy, UpdatePolicy::Daily);
        } else {
            panic!("Expected Git source config");
        }
    }

    #[test]
    fn test_enabled_sources_filter() {
        let yaml = r#"
sources:
  - type: local
    name: "enabled"
    path: "./tasks"
    priority: 10
    enabled: true
  - type: local
    name: "disabled"
    path: "./other"
    priority: 5
    enabled: false
"#;
        let config: TaskSourcesConfig = serde_yaml::from_str(yaml).unwrap();
        let enabled = config.enabled_sources();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].name(), "enabled");
    }
}
