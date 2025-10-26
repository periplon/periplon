//! Mock backends and test doubles
//!
//! Provides mock implementations for testing TUI components in isolation.

use periplon_sdk::dsl::schema::DSLWorkflow;
use periplon_sdk::tui::state::WorkflowEntry;
use std::collections::HashMap;
use std::path::PathBuf;

/// Mock workflow entry builder
pub struct WorkflowEntryBuilder {
    name: String,
    path: PathBuf,
    description: Option<String>,
    version: Option<String>,
    valid: bool,
    errors: Vec<String>,
}

impl WorkflowEntryBuilder {
    /// Create a new workflow entry builder with default values
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            path: PathBuf::from(format!("/workflows/{}.yaml", name)),
            name,
            description: None,
            version: Some("1.0.0".to_string()),
            valid: true,
            errors: Vec::new(),
        }
    }

    /// Set the workflow path
    pub fn path(mut self, path: PathBuf) -> Self {
        self.path = path;
        self
    }

    /// Set the workflow description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set the workflow version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Mark the workflow as invalid
    pub fn invalid(mut self) -> Self {
        self.valid = false;
        self
    }

    /// Add a validation error
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.errors.push(error.into());
        self.valid = false;
        self
    }

    /// Build the workflow entry
    pub fn build(self) -> WorkflowEntry {
        WorkflowEntry {
            name: self.name,
            path: self.path,
            description: self.description,
            version: self.version,
            valid: self.valid,
            errors: self.errors,
        }
    }
}

/// Mock DSL workflow builder
pub struct DSLWorkflowBuilder {
    name: String,
    version: String,
    dsl_version: String,
    cwd: Option<PathBuf>,
    create_cwd: Option<bool>,
    secrets: HashMap<String, String>,
    agents: HashMap<String, periplon_sdk::dsl::schema::Agent>,
    tasks: HashMap<String, periplon_sdk::dsl::schema::Task>,
    workflows: HashMap<String, String>,
    inputs: HashMap<String, periplon_sdk::dsl::schema::Input>,
    outputs: HashMap<String, periplon_sdk::dsl::schema::Output>,
    tools: Option<Vec<String>>,
    communication: Option<periplon_sdk::dsl::schema::CommunicationChannel>,
    mcp_servers: HashMap<String, periplon_sdk::dsl::schema::McpServer>,
    subflows: HashMap<String, periplon_sdk::dsl::schema::Subflow>,
    imports: HashMap<String, String>,
    notifications: Option<periplon_sdk::dsl::schema::NotificationConfig>,
    limits: Option<periplon_sdk::dsl::schema::WorkflowLimits>,
}

impl DSLWorkflowBuilder {
    /// Create a new DSL workflow builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: "1.0.0".to_string(),
            dsl_version: "1.0.0".to_string(),
            cwd: None,
            create_cwd: None,
            secrets: HashMap::new(),
            agents: HashMap::new(),
            tasks: HashMap::new(),
            workflows: HashMap::new(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            tools: None,
            communication: None,
            mcp_servers: HashMap::new(),
            subflows: HashMap::new(),
            imports: HashMap::new(),
            notifications: None,
            limits: None,
        }
    }

    /// Set workflow version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Set DSL version
    pub fn dsl_version(mut self, version: impl Into<String>) -> Self {
        self.dsl_version = version.into();
        self
    }

    /// Set working directory
    pub fn cwd(mut self, cwd: PathBuf) -> Self {
        self.cwd = Some(cwd);
        self
    }

    /// Build the DSL workflow
    pub fn build(self) -> DSLWorkflow {
        DSLWorkflow {
            name: self.name,
            version: self.version,
            dsl_version: self.dsl_version,
            cwd: self.cwd,
            create_cwd: self.create_cwd,
            secrets: self.secrets,
            agents: self.agents,
            tasks: self.tasks,
            workflows: self.workflows,
            inputs: self.inputs,
            outputs: self.outputs,
            tools: self.tools,
            communication: self.communication,
            mcp_servers: self.mcp_servers,
            subflows: self.subflows,
            imports: self.imports,
            notifications: self.notifications,
            limits: self.limits,
        }
    }
}

/// Quick helper to create a simple workflow entry
pub fn mock_workflow_entry(name: &str) -> WorkflowEntry {
    WorkflowEntryBuilder::new(name).build()
}

/// Quick helper to create multiple workflow entries
pub fn mock_workflow_entries(names: &[&str]) -> Vec<WorkflowEntry> {
    names.iter().map(|name| mock_workflow_entry(name)).collect()
}

/// Quick helper to create a simple DSL workflow
pub fn mock_dsl_workflow(name: &str) -> DSLWorkflow {
    DSLWorkflowBuilder::new(name).build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_entry_builder() {
        let entry = WorkflowEntryBuilder::new("test-workflow")
            .description("Test description")
            .version("2.0.0")
            .build();

        assert_eq!(entry.name, "test-workflow");
        assert_eq!(entry.description, Some("Test description".to_string()));
        assert_eq!(entry.version, Some("2.0.0".to_string()));
        assert!(entry.valid);
        assert!(entry.errors.is_empty());
    }

    #[test]
    fn test_workflow_entry_builder_invalid() {
        let entry = WorkflowEntryBuilder::new("invalid-workflow")
            .with_error("Missing agent")
            .with_error("Invalid YAML")
            .build();

        assert!(!entry.valid);
        assert_eq!(entry.errors.len(), 2);
    }

    #[test]
    fn test_dsl_workflow_builder() {
        let workflow = DSLWorkflowBuilder::new("test-workflow")
            .version("1.5.0")
            .dsl_version("2.0.0")
            .build();

        assert_eq!(workflow.name, "test-workflow");
        assert_eq!(workflow.version, "1.5.0");
        assert_eq!(workflow.dsl_version, "2.0.0");
    }

    #[test]
    fn test_mock_workflow_entry() {
        let entry = mock_workflow_entry("quick-test");
        assert_eq!(entry.name, "quick-test");
        assert!(entry.valid);
    }

    #[test]
    fn test_mock_workflow_entries() {
        let entries = mock_workflow_entries(&["one", "two", "three"]);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].name, "one");
        assert_eq!(entries[1].name, "two");
        assert_eq!(entries[2].name, "three");
    }

    #[test]
    fn test_mock_dsl_workflow() {
        let workflow = mock_dsl_workflow("simple");
        assert_eq!(workflow.name, "simple");
        assert_eq!(workflow.version, "1.0.0");
    }
}
