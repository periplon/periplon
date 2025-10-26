//! Namespace Resolution for Task Group Imports
//!
//! This module provides namespace resolution for task groups imported into workflows.
//! It handles resolving namespace-prefixed references to tasks and workflows within
//! imported groups, managing namespace isolation and conflict resolution.

use super::loader::{GroupLoadError, ResolvedTaskGroup, TaskGroupLoader};
use super::schema::{PrebuiltWorkflow, TaskGroup, TaskGroupReference};
use crate::dsl::predefined_tasks::schema::PredefinedTask;
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during namespace resolution
#[derive(Debug, Error)]
pub enum ResolverError {
    /// Namespace not found in resolver
    #[error("Namespace not found: {0}")]
    NamespaceNotFound(String),

    /// Workflow not found in namespace
    #[error("Workflow '{workflow}' not found in namespace '{namespace}'. Available workflows: {available:?}")]
    WorkflowNotFound {
        namespace: String,
        workflow: String,
        available: Vec<String>,
    },

    /// Task not found in namespace
    #[error("Task '{task}' not found in namespace '{namespace}'. Available tasks: {available:?}")]
    TaskNotFound {
        namespace: String,
        task: String,
        available: Vec<String>,
    },

    /// Invalid reference format
    #[error("Invalid reference format: {0}. Expected 'namespace:name'")]
    InvalidReference(String),

    /// Failed to load group
    #[error("Failed to load task group: {0}")]
    LoadError(#[from] GroupLoadError),

    /// Duplicate namespace
    #[error("Duplicate namespace: {0}")]
    DuplicateNamespace(String),

    /// Invalid namespace format
    #[error("Invalid namespace format: {0}")]
    InvalidNamespace(String),
}

/// Namespace resolver for task group imports
///
/// Manages loaded task groups and provides resolution of namespace-prefixed
/// references to tasks and workflows within those groups.
pub struct NamespaceResolver {
    /// Loaded groups (namespace -> ResolvedTaskGroup)
    groups: HashMap<String, ResolvedTaskGroup>,

    /// Namespace mappings (namespace -> group@version)
    mappings: HashMap<String, String>,
}

impl NamespaceResolver {
    /// Create a new empty namespace resolver
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
            mappings: HashMap::new(),
        }
    }

    /// Create resolver from workflow imports
    ///
    /// # Arguments
    ///
    /// * `imports` - HashMap of namespace -> group reference string
    /// * `loader` - TaskGroupLoader to use for loading groups
    ///
    /// # Example
    ///
    /// ```no_run
    /// use periplon_sdk::dsl::predefined_tasks::groups::namespace::NamespaceResolver;
    /// use periplon_sdk::dsl::predefined_tasks::groups::loader::TaskGroupLoader;
    /// use std::collections::HashMap;
    ///
    /// let mut imports = HashMap::new();
    /// imports.insert("google".to_string(), "google-workspace@1.0.0".to_string());
    ///
    /// let mut loader = TaskGroupLoader::new();
    /// let resolver = NamespaceResolver::from_imports(&imports, &mut loader);
    /// ```
    pub fn from_imports(
        imports: &HashMap<String, String>,
        loader: &mut TaskGroupLoader,
    ) -> Result<Self, ResolverError> {
        let mut resolver = Self::new();

        for (namespace, group_ref) in imports {
            resolver.load_import(namespace, group_ref, loader)?;
        }

        Ok(resolver)
    }

    /// Load a single import into the resolver
    ///
    /// # Arguments
    ///
    /// * `namespace` - Namespace identifier
    /// * `group_ref` - Group reference string (e.g., "google-workspace@1.0.0")
    /// * `loader` - TaskGroupLoader to use for loading
    pub fn load_import(
        &mut self,
        namespace: &str,
        group_ref: &str,
        loader: &mut TaskGroupLoader,
    ) -> Result<(), ResolverError> {
        // Check for duplicate namespace
        if self.mappings.contains_key(namespace) {
            return Err(ResolverError::DuplicateNamespace(namespace.to_string()));
        }

        // Parse and load the group
        let group_reference = TaskGroupReference::parse(group_ref)
            .map_err(|_| ResolverError::InvalidReference(group_ref.to_string()))?;

        let resolved_group = loader.load(&group_reference)?;

        // Store mapping and group
        self.mappings
            .insert(namespace.to_string(), group_ref.to_string());
        self.groups.insert(namespace.to_string(), resolved_group);

        Ok(())
    }

    /// Resolve a namespace to its loaded task group
    ///
    /// # Arguments
    ///
    /// * `namespace` - Namespace identifier
    ///
    /// # Returns
    ///
    /// Reference to the ResolvedTaskGroup
    pub fn resolve_import(&self, namespace: &str) -> Result<&ResolvedTaskGroup, ResolverError> {
        self.groups
            .get(namespace)
            .ok_or_else(|| ResolverError::NamespaceNotFound(namespace.to_string()))
    }

    /// Resolve a workflow reference (format: "namespace:workflow_name")
    ///
    /// # Arguments
    ///
    /// * `reference` - Workflow reference string
    ///
    /// # Returns
    ///
    /// Reference to the PrebuiltWorkflow
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use periplon_sdk::dsl::predefined_tasks::groups::namespace::NamespaceResolver;
    /// # let resolver = NamespaceResolver::new();
    /// let workflow = resolver.resolve_workflow_reference("google:upload-files")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn resolve_workflow_reference(
        &self,
        reference: &str,
    ) -> Result<&PrebuiltWorkflow, ResolverError> {
        let (namespace, workflow_name) = Self::parse_reference(reference)?;

        let group = self.resolve_import(namespace)?;

        // Find workflow in group
        group
            .group
            .spec
            .workflows
            .iter()
            .find(|w| w.name == workflow_name)
            .ok_or_else(|| {
                let available: Vec<String> = group
                    .group
                    .spec
                    .workflows
                    .iter()
                    .map(|w| w.name.clone())
                    .collect();

                ResolverError::WorkflowNotFound {
                    namespace: namespace.to_string(),
                    workflow: workflow_name.to_string(),
                    available,
                }
            })
    }

    /// Resolve a task reference (format: "namespace:task_name")
    ///
    /// # Arguments
    ///
    /// * `reference` - Task reference string
    ///
    /// # Returns
    ///
    /// Reference to the PredefinedTask
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use periplon_sdk::dsl::predefined_tasks::groups::namespace::NamespaceResolver;
    /// # let resolver = NamespaceResolver::new();
    /// let task = resolver.resolve_task_reference("google:create-folder")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn resolve_task_reference(
        &self,
        reference: &str,
    ) -> Result<&PredefinedTask, ResolverError> {
        let (namespace, task_name) = Self::parse_reference(reference)?;

        let group = self.resolve_import(namespace)?;

        // Find task in resolved tasks
        group.tasks.get(task_name).ok_or_else(|| {
            let available: Vec<String> = group.tasks.keys().cloned().collect();

            ResolverError::TaskNotFound {
                namespace: namespace.to_string(),
                task: task_name.to_string(),
                available,
            }
        })
    }

    /// Get all tasks from a namespace
    ///
    /// # Arguments
    ///
    /// * `namespace` - Namespace identifier
    ///
    /// # Returns
    ///
    /// Reference to HashMap of task name -> PredefinedTask
    pub fn get_namespace_tasks(
        &self,
        namespace: &str,
    ) -> Result<&HashMap<String, PredefinedTask>, ResolverError> {
        let group = self.resolve_import(namespace)?;
        Ok(&group.tasks)
    }

    /// Get all workflows from a namespace
    ///
    /// # Arguments
    ///
    /// * `namespace` - Namespace identifier
    ///
    /// # Returns
    ///
    /// Reference to Vec of PrebuiltWorkflow
    pub fn get_namespace_workflows(
        &self,
        namespace: &str,
    ) -> Result<&Vec<PrebuiltWorkflow>, ResolverError> {
        let group = self.resolve_import(namespace)?;
        Ok(&group.group.spec.workflows)
    }

    /// Check if a namespace exists
    ///
    /// # Arguments
    ///
    /// * `namespace` - Namespace identifier
    pub fn has_namespace(&self, namespace: &str) -> bool {
        self.groups.contains_key(namespace)
    }

    /// Get all registered namespaces
    ///
    /// # Returns
    ///
    /// Vector of namespace identifiers
    pub fn namespaces(&self) -> Vec<String> {
        self.groups.keys().cloned().collect()
    }

    /// Get the task group for a namespace
    ///
    /// # Arguments
    ///
    /// * `namespace` - Namespace identifier
    ///
    /// # Returns
    ///
    /// Reference to the TaskGroup
    pub fn get_task_group(&self, namespace: &str) -> Result<&TaskGroup, ResolverError> {
        let resolved = self.resolve_import(namespace)?;
        Ok(&resolved.group)
    }

    /// Get the group reference string for a namespace
    ///
    /// # Arguments
    ///
    /// * `namespace` - Namespace identifier
    ///
    /// # Returns
    ///
    /// Group reference string (e.g., "google-workspace@1.0.0")
    pub fn get_group_reference(&self, namespace: &str) -> Option<&String> {
        self.mappings.get(namespace)
    }

    /// Parse a reference into (namespace, name) tuple
    ///
    /// # Arguments
    ///
    /// * `reference` - Reference string in format "namespace:name"
    ///
    /// # Returns
    ///
    /// Tuple of (namespace, name)
    fn parse_reference(reference: &str) -> Result<(&str, &str), ResolverError> {
        let parts: Vec<&str> = reference.split(':').collect();

        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(ResolverError::InvalidReference(reference.to_string()));
        }

        Ok((parts[0], parts[1]))
    }

    /// List all tasks across all namespaces with their full references
    ///
    /// # Returns
    ///
    /// HashMap of "namespace:task_name" -> PredefinedTask reference
    pub fn list_all_tasks(&self) -> HashMap<String, &PredefinedTask> {
        let mut all_tasks = HashMap::new();

        for (namespace, group) in &self.groups {
            for (task_name, task) in &group.tasks {
                let full_ref = format!("{}:{}", namespace, task_name);
                all_tasks.insert(full_ref, task);
            }
        }

        all_tasks
    }

    /// List all workflows across all namespaces with their full references
    ///
    /// # Returns
    ///
    /// HashMap of "namespace:workflow_name" -> PrebuiltWorkflow reference
    pub fn list_all_workflows(&self) -> HashMap<String, &PrebuiltWorkflow> {
        let mut all_workflows = HashMap::new();

        for (namespace, group) in &self.groups {
            for workflow in &group.group.spec.workflows {
                let full_ref = format!("{}:{}", namespace, workflow.name);
                all_workflows.insert(full_ref, workflow);
            }
        }

        all_workflows
    }
}

impl Default for NamespaceResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::predefined_tasks::groups::schema::{
        TaskGroup, TaskGroupApiVersion, TaskGroupKind, TaskGroupMetadata, TaskGroupSpec,
        TaskGroupTask,
    };
    use std::path::PathBuf;

    fn create_test_group(
        name: &str,
        version: &str,
        tasks: Vec<(&str, &str)>,
        workflows: Vec<(&str, &str)>,
    ) -> TaskGroup {
        let task_refs: Vec<TaskGroupTask> = tasks
            .into_iter()
            .map(|(task_name, task_version)| TaskGroupTask {
                name: task_name.to_string(),
                version: task_version.to_string(),
                required: true,
                description: Some(format!("Test task {}", task_name)),
            })
            .collect();

        let workflow_specs: Vec<PrebuiltWorkflow> = workflows
            .into_iter()
            .map(|(wf_name, wf_desc)| PrebuiltWorkflow {
                name: wf_name.to_string(),
                description: Some(wf_desc.to_string()),
                tasks: serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
                inputs: HashMap::new(),
                outputs: HashMap::new(),
            })
            .collect();

        TaskGroup {
            api_version: TaskGroupApiVersion::V1,
            kind: TaskGroupKind::TaskGroup,
            metadata: TaskGroupMetadata {
                name: name.to_string(),
                version: version.to_string(),
                author: None,
                description: Some(format!("Test group {}", name)),
                license: None,
                repository: None,
                tags: vec![],
            },
            spec: TaskGroupSpec {
                tasks: task_refs,
                shared_config: None,
                workflows: workflow_specs,
                dependencies: vec![],
                hooks: None,
            },
        }
    }

    fn create_resolved_group(group: TaskGroup) -> ResolvedTaskGroup {
        ResolvedTaskGroup {
            group,
            tasks: HashMap::new(), // Empty for simplicity in tests
            source_path: PathBuf::from("/test/path"),
        }
    }

    #[test]
    fn test_new_resolver() {
        let resolver = NamespaceResolver::new();
        assert_eq!(resolver.namespaces().len(), 0);
        assert!(!resolver.has_namespace("test"));
    }

    #[test]
    fn test_parse_reference_valid() {
        assert_eq!(
            NamespaceResolver::parse_reference("google:upload-files").unwrap(),
            ("google", "upload-files")
        );
        assert_eq!(
            NamespaceResolver::parse_reference("slack-api:send-message").unwrap(),
            ("slack-api", "send-message")
        );
    }

    #[test]
    fn test_parse_reference_invalid() {
        assert!(NamespaceResolver::parse_reference("no-colon").is_err());
        assert!(NamespaceResolver::parse_reference(":missing-namespace").is_err());
        assert!(NamespaceResolver::parse_reference("missing-name:").is_err());
        assert!(NamespaceResolver::parse_reference("too:many:colons").is_err());
    }

    #[test]
    fn test_resolve_import_not_found() {
        let resolver = NamespaceResolver::new();
        let result = resolver.resolve_import("nonexistent");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ResolverError::NamespaceNotFound(_)
        ));
    }

    #[test]
    fn test_has_namespace() {
        let mut resolver = NamespaceResolver::new();
        assert!(!resolver.has_namespace("google"));

        // Manually add a group for testing
        let group = create_test_group("test-group", "1.0.0", vec![], vec![]);
        let resolved = create_resolved_group(group);
        resolver.groups.insert("google".to_string(), resolved);
        resolver
            .mappings
            .insert("google".to_string(), "test-group@1.0.0".to_string());

        assert!(resolver.has_namespace("google"));
        assert!(!resolver.has_namespace("slack"));
    }

    #[test]
    fn test_namespaces() {
        let mut resolver = NamespaceResolver::new();

        let group1 = create_test_group("group1", "1.0.0", vec![], vec![]);
        let group2 = create_test_group("group2", "2.0.0", vec![], vec![]);

        resolver
            .groups
            .insert("google".to_string(), create_resolved_group(group1));
        resolver
            .groups
            .insert("slack".to_string(), create_resolved_group(group2));

        let mut namespaces = resolver.namespaces();
        namespaces.sort();

        assert_eq!(namespaces, vec!["google", "slack"]);
    }

    #[test]
    fn test_resolve_workflow_reference_not_found() {
        let mut resolver = NamespaceResolver::new();

        let group = create_test_group("test-group", "1.0.0", vec![], vec![("upload", "Upload")]);
        resolver
            .groups
            .insert("google".to_string(), create_resolved_group(group));
        resolver
            .mappings
            .insert("google".to_string(), "test-group@1.0.0".to_string());

        let result = resolver.resolve_workflow_reference("google:nonexistent");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ResolverError::WorkflowNotFound { .. }
        ));
    }

    #[test]
    fn test_resolve_workflow_reference_success() {
        let mut resolver = NamespaceResolver::new();

        let group = create_test_group(
            "test-group",
            "1.0.0",
            vec![],
            vec![("upload-files", "Upload files workflow")],
        );
        resolver
            .groups
            .insert("google".to_string(), create_resolved_group(group));
        resolver
            .mappings
            .insert("google".to_string(), "test-group@1.0.0".to_string());

        let result = resolver.resolve_workflow_reference("google:upload-files");
        assert!(result.is_ok());
        let workflow = result.unwrap();
        assert_eq!(workflow.name, "upload-files");
    }

    #[test]
    fn test_resolve_workflow_reference_invalid_format() {
        let resolver = NamespaceResolver::new();
        let result = resolver.resolve_workflow_reference("invalid-format");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ResolverError::InvalidReference(_)
        ));
    }

    #[test]
    fn test_resolve_workflow_reference_namespace_not_found() {
        let resolver = NamespaceResolver::new();
        let result = resolver.resolve_workflow_reference("nonexistent:workflow");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ResolverError::NamespaceNotFound(_)
        ));
    }

    #[test]
    fn test_get_namespace_workflows() {
        let mut resolver = NamespaceResolver::new();

        let group = create_test_group(
            "test-group",
            "1.0.0",
            vec![],
            vec![
                ("upload", "Upload workflow"),
                ("download", "Download workflow"),
            ],
        );
        resolver
            .groups
            .insert("google".to_string(), create_resolved_group(group));
        resolver
            .mappings
            .insert("google".to_string(), "test-group@1.0.0".to_string());

        let workflows = resolver.get_namespace_workflows("google").unwrap();
        assert_eq!(workflows.len(), 2);
        assert!(workflows.iter().any(|w| w.name == "upload"));
        assert!(workflows.iter().any(|w| w.name == "download"));
    }

    #[test]
    fn test_get_task_group() {
        let mut resolver = NamespaceResolver::new();

        let group = create_test_group("test-group", "1.0.0", vec![], vec![]);
        resolver
            .groups
            .insert("google".to_string(), create_resolved_group(group));
        resolver
            .mappings
            .insert("google".to_string(), "test-group@1.0.0".to_string());

        let task_group = resolver.get_task_group("google").unwrap();
        assert_eq!(task_group.metadata.name, "test-group");
        assert_eq!(task_group.metadata.version, "1.0.0");
    }

    #[test]
    fn test_get_group_reference() {
        let mut resolver = NamespaceResolver::new();

        resolver
            .mappings
            .insert("google".to_string(), "google-workspace@1.0.0".to_string());

        assert_eq!(
            resolver.get_group_reference("google"),
            Some(&"google-workspace@1.0.0".to_string())
        );
        assert_eq!(resolver.get_group_reference("nonexistent"), None);
    }

    #[test]
    fn test_list_all_workflows() {
        let mut resolver = NamespaceResolver::new();

        let group1 = create_test_group(
            "group1",
            "1.0.0",
            vec![],
            vec![("upload", "Upload"), ("download", "Download")],
        );
        let group2 = create_test_group("group2", "2.0.0", vec![], vec![("send", "Send")]);

        resolver
            .groups
            .insert("google".to_string(), create_resolved_group(group1));
        resolver
            .groups
            .insert("slack".to_string(), create_resolved_group(group2));

        let all_workflows = resolver.list_all_workflows();
        assert_eq!(all_workflows.len(), 3);
        assert!(all_workflows.contains_key("google:upload"));
        assert!(all_workflows.contains_key("google:download"));
        assert!(all_workflows.contains_key("slack:send"));
    }
}
