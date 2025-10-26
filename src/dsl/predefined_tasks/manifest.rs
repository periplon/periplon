//! Package Manifest
//!
//! Parsing and validation for task package manifests (package.yaml).

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::dsl::predefined_tasks::{load_predefined_task, PredefinedTask};
use crate::error::Result;

/// Package manifest for a collection of tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPackage {
    #[serde(rename = "apiVersion")]
    pub api_version: String,

    pub kind: String, // "TaskPackage"

    pub metadata: PackageMetadata,

    pub tasks: Vec<TaskReference>,

    #[serde(default)]
    pub dependencies: Vec<PackageDependency>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires: Option<PackageRequirements>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageRequirements {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sdk_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_model: Option<String>,
}

impl TaskPackage {
    /// Load package manifest from YAML file
    pub fn from_yaml(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let package: TaskPackage = serde_yaml::from_str(&content)?;

        // Validate kind
        if package.kind != "TaskPackage" {
            return Err(crate::error::Error::InvalidPackageKind(
                package.kind.clone(),
            ));
        }

        Ok(package)
    }

    /// Load all tasks referenced in the package manifest
    pub async fn load_tasks(&self, base_path: &Path) -> Result<Vec<PredefinedTask>> {
        let mut tasks = Vec::new();

        for task_ref in &self.tasks {
            let task_path = base_path.join(&task_ref.path);

            if !task_path.exists() {
                return Err(crate::error::Error::TaskFileNotFound {
                    path: task_path.display().to_string(),
                    package: self.metadata.name.clone(),
                });
            }

            let task = load_predefined_task(&task_path)?;

            // Validate task matches reference
            if task.metadata.name != task_ref.name {
                return Err(crate::error::Error::TaskReferenceMismatch {
                    expected: task_ref.name.clone(),
                    found: task.metadata.name.clone(),
                });
            }

            if task.metadata.version != task_ref.version {
                return Err(crate::error::Error::TaskVersionMismatch {
                    task: task_ref.name.clone(),
                    expected: task_ref.version.clone(),
                    found: task.metadata.version.clone(),
                });
            }

            tasks.push(task);
        }

        Ok(tasks)
    }

    /// Validate package manifest
    pub fn validate(&self) -> Result<()> {
        // Check kind
        if self.kind != "TaskPackage" {
            return Err(crate::error::Error::InvalidPackageKind(self.kind.clone()));
        }

        // Check version format (basic semver check)
        if !is_valid_semver(&self.metadata.version) {
            return Err(crate::error::Error::InvalidVersion {
                version: self.metadata.version.clone(),
                context: "package metadata".to_string(),
            });
        }

        // Check task references
        for task_ref in &self.tasks {
            if !is_valid_semver(&task_ref.version) {
                return Err(crate::error::Error::InvalidVersion {
                    version: task_ref.version.clone(),
                    context: format!("task reference '{}'", task_ref.name),
                });
            }
        }

        Ok(())
    }
}

/// Basic semantic version validation
fn is_valid_semver(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return false;
    }

    parts.iter().all(|part| part.parse::<u32>().is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_parse_package_manifest() {
        let yaml = r#"
apiVersion: "package/v1"
kind: "TaskPackage"
metadata:
  name: "test-package"
  version: "1.0.0"
  description: "Test package"
  author: "Test Author"
  license: "MIT"
  tags: ["test", "example"]

tasks:
  - path: "tasks/task1.task.yaml"
    name: "task1"
    version: "1.0.0"
  - path: "tasks/task2.task.yaml"
    name: "task2"
    version: "2.0.0"

dependencies:
  - name: "other-package"
    version: "1.5.0"
    repository: "https://github.com/test/other"

requires:
  sdk_version: ">=0.2.0"
  min_model: "claude-sonnet-4"
"#;

        let package: TaskPackage = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(package.metadata.name, "test-package");
        assert_eq!(package.metadata.version, "1.0.0");
        assert_eq!(package.tasks.len(), 2);
        assert_eq!(package.dependencies.len(), 1);
        assert!(package.requires.is_some());

        assert!(package.validate().is_ok());
    }

    #[test]
    fn test_invalid_package_kind() {
        let yaml = r#"
apiVersion: "package/v1"
kind: "WrongKind"
metadata:
  name: "test"
  version: "1.0.0"
tasks: []
"#;

        let package: TaskPackage = serde_yaml::from_str(yaml).unwrap();
        assert!(package.validate().is_err());
    }

    #[test]
    fn test_invalid_semver() {
        assert!(is_valid_semver("1.0.0"));
        assert!(is_valid_semver("10.20.30"));
        assert!(!is_valid_semver("1.0"));
        assert!(!is_valid_semver("1.0.0.0"));
        assert!(!is_valid_semver("1.0.a"));
        assert!(!is_valid_semver("invalid"));
    }

    #[tokio::test]
    async fn test_load_tasks_from_package() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create tasks directory
        let tasks_dir = base_path.join("tasks");
        std::fs::create_dir(&tasks_dir).unwrap();

        // Create task files
        let task1_yaml = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "task1"
  version: "1.0.0"
spec:
  agent_template:
    description: "Task 1"
  inputs: {}
"#;
        let mut task1_file = std::fs::File::create(tasks_dir.join("task1.task.yaml")).unwrap();
        task1_file.write_all(task1_yaml.as_bytes()).unwrap();

        let task2_yaml = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "task2"
  version: "2.0.0"
spec:
  agent_template:
    description: "Task 2"
  inputs: {}
"#;
        let mut task2_file = std::fs::File::create(tasks_dir.join("task2.task.yaml")).unwrap();
        task2_file.write_all(task2_yaml.as_bytes()).unwrap();

        // Create package manifest
        let package_yaml = r#"
apiVersion: "package/v1"
kind: "TaskPackage"
metadata:
  name: "test-package"
  version: "1.0.0"
tasks:
  - path: "tasks/task1.task.yaml"
    name: "task1"
    version: "1.0.0"
  - path: "tasks/task2.task.yaml"
    name: "task2"
    version: "2.0.0"
"#;

        let package: TaskPackage = serde_yaml::from_str(package_yaml).unwrap();
        let tasks = package.load_tasks(base_path).await.unwrap();

        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].metadata.name, "task1");
        assert_eq!(tasks[1].metadata.name, "task2");
    }

    #[tokio::test]
    async fn test_load_tasks_version_mismatch() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        let tasks_dir = base_path.join("tasks");
        std::fs::create_dir(&tasks_dir).unwrap();

        // Task file with wrong version
        let task_yaml = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "task1"
  version: "2.0.0"
spec:
  agent_template:
    description: "Task 1"
  inputs: {}
"#;
        let mut task_file = std::fs::File::create(tasks_dir.join("task1.task.yaml")).unwrap();
        task_file.write_all(task_yaml.as_bytes()).unwrap();

        // Package expects version 1.0.0
        let package_yaml = r#"
apiVersion: "package/v1"
kind: "TaskPackage"
metadata:
  name: "test-package"
  version: "1.0.0"
tasks:
  - path: "tasks/task1.task.yaml"
    name: "task1"
    version: "1.0.0"
"#;

        let package: TaskPackage = serde_yaml::from_str(package_yaml).unwrap();
        let result = package.load_tasks(base_path).await;

        assert!(result.is_err());
    }
}
