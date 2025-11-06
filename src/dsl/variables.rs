//! Variable Resolution and Interpolation
//!
//! This module provides variable management for the DSL, including:
//! - Variable context management across workflow/agent/task scopes
//! - Variable interpolation in strings using ${scope.var} or ${var} syntax
//! - Type-safe variable resolution
//! - Scope hierarchy and variable shadowing

use crate::error::{Error, Result};
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Regular expression for matching variable references in strings
/// Matches: ${scope.variable_name} or ${variable_name}
static VAR_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_var_regex() -> &'static Regex {
    VAR_REGEX.get_or_init(|| {
        Regex::new(r"\$\{([a-zA-Z_][a-zA-Z0-9_]*(?:\.[a-zA-Z_][a-zA-Z0-9_]*)?)\}")
            .expect("Invalid variable regex")
    })
}

/// Variable scope identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Scope {
    /// Workflow-level variables
    Workflow,
    /// Agent-level variables
    Agent(String),
    /// Task-level variables
    Task(String),
    /// Subflow-level variables
    Subflow(String),
    /// Loop-level variables (iteration, item, index, etc.)
    Loop(String),
    /// Secret values (read-only)
    Secret,
}

impl Scope {
    /// Get the scope prefix for variable references
    pub fn prefix(&self) -> &str {
        match self {
            Scope::Workflow => "workflow",
            Scope::Agent(_) => "agent",
            Scope::Task(_) => "task",
            Scope::Subflow(_) => "subflow",
            Scope::Loop(_) => "loop",
            Scope::Secret => "secret",
        }
    }

    /// Get the scope identifier (name)
    pub fn identifier(&self) -> Option<&str> {
        match self {
            Scope::Workflow | Scope::Secret => None,
            Scope::Agent(name) | Scope::Task(name) | Scope::Subflow(name) | Scope::Loop(name) => {
                Some(name)
            }
        }
    }
}

/// Variable context that tracks all variables in the current execution scope
#[derive(Debug, Clone)]
pub struct VariableContext {
    /// Variables by scope
    variables: HashMap<String, Value>,
    /// Current active scope for unqualified variable resolution
    current_scope: Option<Scope>,
}

impl VariableContext {
    /// Create a new empty variable context
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            current_scope: None,
        }
    }

    /// Set the current active scope for unqualified variable resolution
    pub fn set_current_scope(&mut self, scope: Scope) {
        self.current_scope = Some(scope);
    }

    /// Get the current active scope
    pub fn current_scope(&self) -> Option<&Scope> {
        self.current_scope.as_ref()
    }

    /// Insert a variable into the context
    ///
    /// # Arguments
    ///
    /// * `scope` - The scope this variable belongs to
    /// * `name` - The variable name
    /// * `value` - The variable value
    pub fn insert(&mut self, scope: &Scope, name: &str, value: Value) {
        let key = Self::make_key(scope, name);
        self.variables.insert(key, value);
    }

    /// Insert multiple variables into the context
    pub fn insert_all(&mut self, scope: &Scope, vars: HashMap<String, Value>) {
        for (name, value) in vars {
            self.insert(scope, &name, value);
        }
    }

    /// Get a variable from the context
    ///
    /// # Arguments
    ///
    /// * `scope` - The scope to look in (None = use current scope)
    /// * `name` - The variable name
    ///
    /// # Returns
    ///
    /// The variable value if found
    pub fn get(&self, scope: Option<&Scope>, name: &str) -> Option<&Value> {
        let scope = scope.or(self.current_scope.as_ref())?;
        let key = Self::make_key(scope, name);
        self.variables.get(&key)
    }

    /// Resolve a variable reference (e.g., "workflow.var", "secret.name", or "var")
    ///
    /// # Arguments
    ///
    /// * `reference` - The variable reference (with or without scope prefix)
    ///
    /// # Returns
    ///
    /// The variable value if found
    pub fn resolve(&self, reference: &str) -> Result<&Value> {
        if let Some((scope_prefix, var_name)) = reference.split_once('.') {
            // Qualified reference: scope.variable
            let scope = match scope_prefix {
                "workflow" => Scope::Workflow,
                "secret" => Scope::Secret,
                "agent" => {
                    if let Some(Scope::Agent(name)) = &self.current_scope {
                        Scope::Agent(name.clone())
                    } else {
                        return Err(Error::InvalidInput(format!(
                            "Cannot resolve agent variable '{}' outside agent scope",
                            reference
                        )));
                    }
                }
                "task" => {
                    if let Some(Scope::Task(name)) = &self.current_scope {
                        Scope::Task(name.clone())
                    } else {
                        return Err(Error::InvalidInput(format!(
                            "Cannot resolve task variable '{}' outside task scope",
                            reference
                        )));
                    }
                }
                "subflow" => {
                    if let Some(Scope::Subflow(name)) = &self.current_scope {
                        Scope::Subflow(name.clone())
                    } else {
                        return Err(Error::InvalidInput(format!(
                            "Cannot resolve subflow variable '{}' outside subflow scope",
                            reference
                        )));
                    }
                }
                "loop" => {
                    if let Some(Scope::Loop(name)) = &self.current_scope {
                        Scope::Loop(name.clone())
                    } else {
                        return Err(Error::InvalidInput(format!(
                            "Cannot resolve loop variable '{}' outside loop scope",
                            reference
                        )));
                    }
                }
                _ => {
                    return Err(Error::InvalidInput(format!(
                        "Unknown scope prefix '{}' in variable reference '{}'",
                        scope_prefix, reference
                    )))
                }
            };

            self.get(Some(&scope), var_name).ok_or_else(|| {
                Error::InvalidInput(format!("Variable '{}' not found in context", reference))
            })
        } else {
            // Unqualified reference: try current scope, then workflow scope, but not secrets
            if let Some(current) = &self.current_scope {
                if let Some(value) = self.get(Some(current), reference) {
                    return Ok(value);
                }
            }

            // Fall back to workflow scope (secrets must be explicitly qualified)
            self.get(Some(&Scope::Workflow), reference).ok_or_else(|| {
                Error::InvalidInput(format!("Variable '{}' not found in context", reference))
            })
        }
    }

    /// Interpolate variables in a string
    ///
    /// Replaces all ${scope.variable} and ${variable} references with their values
    ///
    /// # Arguments
    ///
    /// * `template` - The string template containing variable references
    ///
    /// # Returns
    ///
    /// The interpolated string with all variables resolved
    pub fn interpolate(&self, template: &str) -> Result<String> {
        let regex = get_var_regex();
        let mut result = String::with_capacity(template.len());
        let mut last_match = 0;

        for captures in regex.captures_iter(template) {
            let full_match = captures.get(0).unwrap();
            let var_ref = captures.get(1).unwrap().as_str();

            // Append text before this match
            result.push_str(&template[last_match..full_match.start()]);

            // Resolve and append variable value
            let value = self.resolve(var_ref)?;
            result.push_str(&value_to_string(value));

            last_match = full_match.end();
        }

        // Append remaining text
        result.push_str(&template[last_match..]);

        Ok(result)
    }

    /// Check if a variable exists in the context
    pub fn contains(&self, scope: &Scope, name: &str) -> bool {
        let key = Self::make_key(scope, name);
        self.variables.contains_key(&key)
    }

    /// Get all variables in a specific scope
    pub fn get_scope_variables(&self, scope: &Scope) -> HashMap<String, Value> {
        let prefix = format!("{}:", scope.prefix());
        if let Some(id) = scope.identifier() {
            let full_prefix = format!("{}{}:", prefix, id);
            self.variables
                .iter()
                .filter(|(k, _)| k.starts_with(&full_prefix))
                .map(|(k, v)| {
                    let name = k.strip_prefix(&full_prefix).unwrap();
                    (name.to_string(), v.clone())
                })
                .collect()
        } else {
            // Workflow scope
            self.variables
                .iter()
                .filter(|(k, _)| k.starts_with(&prefix) && !k[prefix.len()..].contains(':'))
                .map(|(k, v)| {
                    let name = k.strip_prefix(&prefix).unwrap();
                    (name.to_string(), v.clone())
                })
                .collect()
        }
    }

    /// Create a storage key for a variable
    fn make_key(scope: &Scope, name: &str) -> String {
        match scope {
            Scope::Workflow => format!("workflow:{}", name),
            Scope::Secret => format!("secret:{}", name),
            Scope::Agent(agent_name) => format!("agent:{}:{}", agent_name, name),
            Scope::Task(task_name) => format!("task:{}:{}", task_name, name),
            Scope::Subflow(subflow_name) => format!("subflow:{}:{}", subflow_name, name),
            Scope::Loop(loop_id) => format!("loop:{}:{}", loop_id, name),
        }
    }

    /// Create a child context with a new scope
    pub fn with_scope(&self, scope: Scope) -> Self {
        let mut child = self.clone();
        child.set_current_scope(scope);
        child
    }

    /// Merge another context into this one (child context merges into parent)
    pub fn merge(&mut self, other: &VariableContext) {
        for (key, value) in &other.variables {
            self.variables.insert(key.clone(), value.clone());
        }
    }
}

impl Default for VariableContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a JSON value to a string for interpolation
fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        Value::Array(_) | Value::Object(_) => value.to_string(),
    }
}

/// Extract all variable references from a string template
///
/// Returns a list of variable references (without ${} wrappers)
pub fn extract_variable_references(template: &str) -> Vec<String> {
    let regex = get_var_regex();
    regex
        .captures_iter(template)
        .map(|cap| cap.get(1).unwrap().as_str().to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_variable_context_insert_and_get() {
        let mut ctx = VariableContext::new();
        let scope = Scope::Workflow;

        ctx.insert(&scope, "test_var", json!("test_value"));

        assert_eq!(
            ctx.get(Some(&scope), "test_var"),
            Some(&json!("test_value"))
        );
        assert_eq!(ctx.get(Some(&scope), "nonexistent"), None);
    }

    #[test]
    fn test_variable_context_scopes() {
        let mut ctx = VariableContext::new();

        ctx.insert(&Scope::Workflow, "global_var", json!("global"));
        ctx.insert(&Scope::Agent("agent1".into()), "agent_var", json!("agent"));
        ctx.insert(&Scope::Task("task1".into()), "task_var", json!("task"));

        assert_eq!(
            ctx.get(Some(&Scope::Workflow), "global_var"),
            Some(&json!("global"))
        );
        assert_eq!(
            ctx.get(Some(&Scope::Agent("agent1".into())), "agent_var"),
            Some(&json!("agent"))
        );
        assert_eq!(
            ctx.get(Some(&Scope::Task("task1".into())), "task_var"),
            Some(&json!("task"))
        );
    }

    #[test]
    fn test_variable_resolution_qualified() {
        let mut ctx = VariableContext::new();
        ctx.insert(&Scope::Workflow, "my_var", json!("workflow_value"));

        let result = ctx.resolve("workflow.my_var");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), &json!("workflow_value"));
    }

    #[test]
    fn test_variable_resolution_unqualified() {
        let mut ctx = VariableContext::new();
        ctx.set_current_scope(Scope::Task("task1".into()));
        ctx.insert(&Scope::Task("task1".into()), "local_var", json!("local"));

        let result = ctx.resolve("local_var");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), &json!("local"));
    }

    #[test]
    fn test_variable_resolution_fallback_to_workflow() {
        let mut ctx = VariableContext::new();
        ctx.set_current_scope(Scope::Task("task1".into()));
        ctx.insert(&Scope::Workflow, "global_var", json!("global"));

        // Should fall back to workflow scope when not found in task scope
        let result = ctx.resolve("global_var");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), &json!("global"));
    }

    #[test]
    fn test_variable_interpolation_simple() {
        let mut ctx = VariableContext::new();
        ctx.insert(&Scope::Workflow, "name", json!("Alice"));

        let template = "Hello, ${workflow.name}!";
        let result = ctx.interpolate(template);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, Alice!");
    }

    #[test]
    fn test_variable_interpolation_multiple() {
        let mut ctx = VariableContext::new();
        ctx.insert(&Scope::Workflow, "first", json!("Alice"));
        ctx.insert(&Scope::Workflow, "last", json!("Smith"));

        let template = "${workflow.first} ${workflow.last}";
        let result = ctx.interpolate(template);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Alice Smith");
    }

    #[test]
    fn test_variable_interpolation_unqualified() {
        let mut ctx = VariableContext::new();
        ctx.set_current_scope(Scope::Workflow);
        ctx.insert(&Scope::Workflow, "name", json!("Bob"));

        let template = "Hello, ${name}!";
        let result = ctx.interpolate(template);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, Bob!");
    }

    #[test]
    fn test_variable_interpolation_numbers() {
        let mut ctx = VariableContext::new();
        ctx.insert(&Scope::Workflow, "count", json!(42));

        let template = "Count: ${workflow.count}";
        let result = ctx.interpolate(template);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Count: 42");
    }

    #[test]
    fn test_variable_interpolation_booleans() {
        let mut ctx = VariableContext::new();
        ctx.insert(&Scope::Workflow, "enabled", json!(true));

        let template = "Enabled: ${workflow.enabled}";
        let result = ctx.interpolate(template);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Enabled: true");
    }

    #[test]
    fn test_variable_interpolation_undefined_error() {
        let ctx = VariableContext::new();
        let template = "Hello, ${workflow.undefined}!";

        let result = ctx.interpolate(template);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not found in context"));
    }

    #[test]
    fn test_extract_variable_references() {
        let template = "Hello ${workflow.name}, you have ${count} messages from ${sender}";
        let refs = extract_variable_references(template);

        assert_eq!(refs.len(), 3);
        assert!(refs.contains(&"workflow.name".to_string()));
        assert!(refs.contains(&"count".to_string()));
        assert!(refs.contains(&"sender".to_string()));
    }

    #[test]
    fn test_variable_context_with_scope() {
        let mut ctx = VariableContext::new();
        ctx.insert(&Scope::Workflow, "global", json!("value"));

        let child = ctx.with_scope(Scope::Task("task1".into()));
        assert_eq!(child.current_scope(), Some(&Scope::Task("task1".into())));
        assert_eq!(
            child.get(Some(&Scope::Workflow), "global"),
            Some(&json!("value"))
        );
    }

    #[test]
    fn test_variable_context_merge() {
        let mut parent = VariableContext::new();
        parent.insert(&Scope::Workflow, "var1", json!("value1"));

        let mut child = VariableContext::new();
        child.insert(&Scope::Task("task1".into()), "var2", json!("value2"));

        parent.merge(&child);

        assert_eq!(
            parent.get(Some(&Scope::Workflow), "var1"),
            Some(&json!("value1"))
        );
        assert_eq!(
            parent.get(Some(&Scope::Task("task1".into())), "var2"),
            Some(&json!("value2"))
        );
    }

    #[test]
    fn test_get_scope_variables() {
        let mut ctx = VariableContext::new();
        ctx.insert(&Scope::Workflow, "var1", json!("value1"));
        ctx.insert(&Scope::Workflow, "var2", json!("value2"));
        ctx.insert(&Scope::Agent("agent1".into()), "var3", json!("value3"));

        let workflow_vars = ctx.get_scope_variables(&Scope::Workflow);
        assert_eq!(workflow_vars.len(), 2);
        assert_eq!(workflow_vars.get("var1"), Some(&json!("value1")));
        assert_eq!(workflow_vars.get("var2"), Some(&json!("value2")));

        let agent_vars = ctx.get_scope_variables(&Scope::Agent("agent1".into()));
        assert_eq!(agent_vars.len(), 1);
        assert_eq!(agent_vars.get("var3"), Some(&json!("value3")));
    }
}
