# Phase 4: Task Group Integration Analysis

**Date:** 2025-10-19
**Phase:** 4 - Task Groups and Workflow Imports

## Executive Summary

This document provides a detailed analysis of integration points for adding task group support to the DSL workflow system. Task groups enable bundling multiple related tasks with shared configuration, namespace isolation, and prebuilt workflows that can be imported and reused across workflows.

## Current Architecture Overview

### 1. DSLWorkflow Structure (src/dsl/schema.rs:10-58)

**Current fields:**
- `name`, `version`, `dsl_version` - Basic metadata
- `cwd`, `create_cwd` - Working directory config
- `secrets` - Secure credential management
- `inputs`, `outputs` - Variable definitions
- `agents` - Agent specifications
- `tasks` - Task definitions
- `workflows` - Workflow orchestration
- `tools`, `communication`, `mcp_servers` - Tool configuration
- `subflows` - Inline or external subflow definitions

**Missing:** No import mechanism for external task groups

### 2. TaskSpec Structure (src/dsl/schema.rs:215-287)

**Current execution types (mutually exclusive):**
- `agent` - Execute with specified agent
- `subflow` - Execute a subflow
- `uses` - Reference predefined task (e.g., "task@1.2.0")
- `embed` - Embed predefined task definition
- `script` - Execute script
- `command` - Execute command
- `http` - HTTP request
- `mcp_tool` - MCP tool invocation

**Missing:** No way to reference prebuilt workflows from task groups

### 3. Execution Flow (src/dsl/executor.rs:363-472)

**Current workflow:**
1. `execute()` - Entry point, runs pre/post hooks
2. `execute_tasks()` - Gets topological sort order from task_graph
3. Iterates through ordered tasks
4. Checks for parallel execution opportunities
5. Executes tasks sequentially or in parallel
6. Updates state and handles errors

**Integration point:** Before task execution begins, need to load and resolve imported task groups

### 4. Task Resolution (src/dsl/task_graph.rs:38-100)

**Current behavior:**
- `add_task()` - Adds task to graph with dependencies (line 53)
- Builds adjacency list for dependency resolution (line 64)
- `get_ready_tasks()` - Returns tasks with completed dependencies (line 76)
- `topological_sort()` - Determines execution order

**Integration point:** Task graph needs to handle tasks from imported groups with namespace prefixes

### 5. Validator (src/dsl/validator.rs:86-116)

**Current validations:**
- Agent references
- Task dependencies
- Circular dependencies
- Tool references
- Permission modes
- Workflow stages
- Loop specifications
- Subflow references
- Variable references

**Missing:** Validation for imports and namespace resolution

### 6. Existing Task Group Infrastructure

**Schema (src/dsl/predefined_tasks/groups/schema.rs):**
- `TaskGroup` - Complete group definition with metadata and spec
- `TaskGroupSpec` - Contains tasks, shared_config, workflows, dependencies
- `PrebuiltWorkflow` - Pre-configured workflow templates (line 151-170)
- `TaskGroupReference` - Name/version reference
- `SharedConfig` - Inputs, permissions, environment, max_turns

**Loader (src/dsl/predefined_tasks/groups/loader.rs):**
- `TaskGroupLoader` - Discovers and loads groups from filesystem
- `ResolvedTaskGroup` - Fully loaded group with resolved tasks
- `load()` - Loads group by reference (line 196)
- `resolve_group_tasks()` - Resolves all tasks in group
- `discover_groups_in_directory()` - Scans for .taskgroup.yaml files

**Parser (src/dsl/predefined_tasks/groups/parser.rs):**
- `parse_task_group()` - Parses YAML to TaskGroup
- Validates API version, kind, metadata
- Validates at least one task present
- Validates version constraints

## Required Changes

### 1. Add `imports` Field to DSLWorkflow

**Location:** `src/dsl/schema.rs:10-58`

**Implementation:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DSLWorkflow {
    // ... existing fields ...

    /// Imported task groups (namespace -> group reference)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub imports: HashMap<String, String>,

    // ... rest of fields ...
}
```

**YAML syntax:**
```yaml
imports:
  google: "google-workspace@1.0.0"
  slack: "slack-integrations@2.1.0"
  github: "github-actions@3.0.0"
```

**Behavior:**
- Keys are namespace identifiers (used in tasks)
- Values are group references in format `"name@version"`
- Namespaces must be unique within a workflow
- Imports are resolved before workflow execution

**Validation requirements:**
- Namespace must be valid identifier (alphanumeric, dash, underscore)
- Group reference must parse as valid TaskGroupReference
- No duplicate namespaces
- Referenced groups must exist and be loadable

### 2. Add `uses_workflow` Field to TaskSpec

**Location:** `src/dsl/schema.rs:215-287`

**Implementation:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskSpec {
    // ... existing fields ...

    /// Reference to a prebuilt workflow from a task group
    /// Format: "namespace:workflow_name" (e.g., "google:upload-files")
    /// Mutually exclusive with agent, subflow, uses, embed, script, command, http, mcp_tool
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses_workflow: Option<String>,

    // ... rest of fields ...
}
```

**YAML syntax:**
```yaml
tasks:
  upload_to_drive:
    description: "Upload files to Google Drive"
    uses_workflow: "google:upload-files"
    inputs:
      folder_id: "abc123"
      files: ["doc1.pdf", "doc2.pdf"]
```

**Behavior:**
- Format is `"namespace:workflow_name"`
- Namespace must match an entry in workflow's `imports`
- Workflow name must exist in the referenced task group's `workflows`
- Task inputs are passed to the prebuilt workflow
- Prebuilt workflow's tasks are expanded inline with namespace prefix

**Validation requirements:**
- Must be mutually exclusive with other execution types
- Namespace must exist in imports
- Workflow must exist in the imported group
- Input parameters must match workflow's input spec
- No circular workflow references

### 3. Create Namespace Resolver

**Location:** `src/dsl/predefined_tasks/groups/namespace.rs` (new file)

**Purpose:** Resolve namespace-prefixed references to actual task group components

**Implementation structure:**

```rust
/// Namespace resolver for task group imports
pub struct NamespaceResolver {
    /// Loaded groups (namespace -> ResolvedTaskGroup)
    groups: HashMap<String, ResolvedTaskGroup>,

    /// Namespace mappings (namespace -> group@version)
    mappings: HashMap<String, String>,
}

impl NamespaceResolver {
    /// Create resolver from workflow imports
    pub fn from_imports(
        imports: &HashMap<String, String>,
        loader: &mut TaskGroupLoader,
    ) -> Result<Self, ResolverError>;

    /// Resolve a workflow reference (e.g., "google:upload-files")
    pub fn resolve_workflow(
        &self,
        reference: &str,
    ) -> Result<&PrebuiltWorkflow, ResolverError>;

    /// Resolve a task reference (e.g., "google:create-folder")
    pub fn resolve_task(
        &self,
        reference: &str,
    ) -> Result<&PredefinedTask, ResolverError>;

    /// Get all tasks from a namespace
    pub fn get_namespace_tasks(
        &self,
        namespace: &str,
    ) -> Result<&HashMap<String, PredefinedTask>, ResolverError>;

    /// Check if namespace exists
    pub fn has_namespace(&self, namespace: &str) -> bool;

    /// Apply shared config to task
    fn apply_shared_config(
        &self,
        task: &mut PredefinedTask,
        namespace: &str,
    );
}

#[derive(Debug, Error)]
pub enum ResolverError {
    #[error("Namespace not found: {0}")]
    NamespaceNotFound(String),

    #[error("Workflow not found: {workflow} in namespace {namespace}")]
    WorkflowNotFound { namespace: String, workflow: String },

    #[error("Task not found: {task} in namespace {namespace}")]
    TaskNotFound { namespace: String, task: String },

    #[error("Invalid reference format: {0}")]
    InvalidReference(String),

    #[error("Failed to load group: {0}")]
    LoadError(#[from] GroupLoadError),
}
```

**Usage pattern:**
```rust
// In executor.rs
let resolver = NamespaceResolver::from_imports(
    &self.workflow.imports,
    &mut task_group_loader,
)?;

// Later when processing task with uses_workflow
if let Some(ref workflow_ref) = task.uses_workflow {
    let workflow = resolver.resolve_workflow(workflow_ref)?;
    // Expand workflow tasks into execution plan
}
```

### 4. Integrate with Validator

**Location:** `src/dsl/validator.rs`

**New validation functions:**

```rust
/// Validate import declarations
fn validate_imports(workflow: &DSLWorkflow, errors: &mut ValidationErrors) {
    for (namespace, group_ref) in &workflow.imports {
        // Validate namespace is valid identifier
        if !is_valid_namespace(namespace) {
            errors.add(format!("Invalid namespace: {}", namespace));
        }

        // Validate group reference format
        if let Err(e) = TaskGroupReference::parse(group_ref) {
            errors.add(format!(
                "Invalid group reference '{}' in namespace '{}': {}",
                group_ref, namespace, e
            ));
        }

        // Check for duplicate namespaces (already unique in HashMap)
    }
}

/// Validate uses_workflow references
fn validate_uses_workflow_references(
    workflow: &DSLWorkflow,
    errors: &mut ValidationErrors,
) {
    for (task_id, task) in &workflow.tasks {
        if let Some(ref workflow_ref) = task.uses_workflow {
            // Check mutually exclusive with other execution types
            if task.agent.is_some()
                || task.subflow.is_some()
                || task.uses.is_some()
                || task.embed.is_some()
                || task.script.is_some()
                || task.command.is_some()
                || task.http.is_some()
                || task.mcp_tool.is_some()
            {
                errors.add(format!(
                    "Task '{}': uses_workflow is mutually exclusive with other execution types",
                    task_id
                ));
            }

            // Validate format
            if let Some((namespace, workflow_name)) = parse_workflow_ref(workflow_ref) {
                // Check namespace exists in imports
                if !workflow.imports.contains_key(namespace) {
                    errors.add(format!(
                        "Task '{}': namespace '{}' not found in imports",
                        task_id, namespace
                    ));
                }
            } else {
                errors.add(format!(
                    "Task '{}': invalid workflow reference format '{}'",
                    task_id, workflow_ref
                ));
            }
        }
    }
}

/// Validate namespace is a valid identifier
fn is_valid_namespace(namespace: &str) -> bool {
    !namespace.is_empty()
        && namespace.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        && !namespace.starts_with(|c: char| c.is_numeric())
}

/// Parse workflow reference into (namespace, workflow_name)
fn parse_workflow_ref(reference: &str) -> Option<(&str, &str)> {
    let parts: Vec<&str> = reference.split(':').collect();
    if parts.len() == 2 {
        Some((parts[0], parts[1]))
    } else {
        None
    }
}
```

**Add to validate_workflow():**
```rust
pub fn validate_workflow(workflow: &DSLWorkflow) -> Result<()> {
    let mut errors = ValidationErrors::new();

    // ... existing validations ...

    // NEW: Validate imports
    validate_imports(workflow, &mut errors);

    // NEW: Validate uses_workflow references
    validate_uses_workflow_references(workflow, &mut errors);

    errors.into_result()
}
```

### 5. Integrate with Executor

**Location:** `src/dsl/executor.rs`

**Add to WorkflowExecutor struct:**

```rust
pub struct WorkflowExecutor {
    // ... existing fields ...

    /// Namespace resolver for task group imports
    namespace_resolver: Option<NamespaceResolver>,
}
```

**Modify initialization (in `new()` or before `execute()`):**

```rust
impl WorkflowExecutor {
    pub fn new(/* ... */) -> Self {
        // ... existing initialization ...

        // Load and resolve imports
        let namespace_resolver = if !workflow.imports.is_empty() {
            let mut loader = TaskGroupLoader::new();
            // Add search paths from config or defaults
            Some(NamespaceResolver::from_imports(
                &workflow.imports,
                &mut loader,
            )?)
        } else {
            None
        };

        Self {
            // ... existing fields ...
            namespace_resolver,
        }
    }

    // ... rest of implementation ...
}
```

**Modify task execution to handle uses_workflow:**

```rust
// In execute_task_parallel() or similar
async fn execute_task_internal(
    &mut self,
    task_id: &str,
    task: &TaskSpec,
) -> Result<()> {
    // ... existing execution type checks ...

    // NEW: Handle uses_workflow
    if let Some(ref workflow_ref) = task.uses_workflow {
        return self.execute_prebuilt_workflow(task_id, workflow_ref, task).await;
    }

    // ... rest of existing logic ...
}

async fn execute_prebuilt_workflow(
    &mut self,
    task_id: &str,
    workflow_ref: &str,
    parent_task: &TaskSpec,
) -> Result<()> {
    let resolver = self.namespace_resolver.as_ref()
        .ok_or_else(|| Error::ExecutionError(
            "No namespace resolver available for uses_workflow".to_string()
        ))?;

    // Resolve workflow
    let prebuilt = resolver.resolve_workflow(workflow_ref)?;

    // Parse workflow tasks from YAML
    let workflow_tasks: HashMap<String, TaskSpec> =
        serde_yaml::from_value(prebuilt.tasks.clone())?;

    // Prefix task IDs with namespace to avoid collisions
    let (namespace, workflow_name) = parse_workflow_ref(workflow_ref)
        .ok_or_else(|| Error::ExecutionError(
            format!("Invalid workflow reference: {}", workflow_ref)
        ))?;

    // Create sub-task graph for workflow tasks
    let mut sub_graph = TaskGraph::new();
    for (sub_task_id, sub_task) in workflow_tasks {
        let prefixed_id = format!("{}:{}:{}", namespace, workflow_name, sub_task_id);

        // Merge parent task inputs with workflow inputs
        let mut merged_task = sub_task.clone();
        for (key, value) in &parent_task.inputs {
            merged_task.inputs.insert(key.clone(), value.clone());
        }

        sub_graph.add_task(prefixed_id, merged_task);
    }

    // Execute sub-tasks in dependency order
    let sub_order = sub_graph.topological_sort()?;
    for sub_task_id in sub_order {
        let sub_task = sub_graph.get_task(&sub_task_id)?;
        self.execute_task_internal(&sub_task_id, sub_task).await?;
    }

    Ok(())
}
```

### 6. Integrate with Task Graph

**Location:** `src/dsl/task_graph.rs`

**Minimal changes required:**
- Task IDs with colons (namespace prefixes) should work as-is
- No special handling needed for namespaced tasks
- Dependency resolution already handles arbitrary task IDs

**Potential enhancement:**
```rust
impl TaskGraph {
    /// Check if task ID is namespaced
    pub fn is_namespaced_task(task_id: &str) -> bool {
        task_id.contains(':')
    }

    /// Extract namespace from task ID
    pub fn extract_namespace(task_id: &str) -> Option<&str> {
        task_id.split(':').next()
    }
}
```

## Implementation Sequence

### Step 1: Schema Changes (Low Risk)
1. Add `imports` field to `DSLWorkflow`
2. Add `uses_workflow` field to `TaskSpec`
3. Update serialization tests
4. Verify no breaking changes to existing workflows

**Files:** `src/dsl/schema.rs`
**Tests:** Unit tests for serialization/deserialization

### Step 2: Namespace Resolver (Medium Risk)
1. Create `src/dsl/predefined_tasks/groups/namespace.rs`
2. Implement `NamespaceResolver` struct
3. Implement resolution methods
4. Add comprehensive unit tests
5. Handle error cases gracefully

**Files:**
- `src/dsl/predefined_tasks/groups/namespace.rs` (new)
- `src/dsl/predefined_tasks/groups/mod.rs` (add module)

**Tests:** Unit tests for all resolution scenarios

### Step 3: Validator Integration (Low Risk)
1. Add `validate_imports()` function
2. Add `validate_uses_workflow_references()` function
3. Add helper functions for parsing
4. Integrate with `validate_workflow()`
5. Add validation tests

**Files:** `src/dsl/validator.rs`
**Tests:** Validation tests for imports and uses_workflow

### Step 4: Executor Integration (High Risk)
1. Add `namespace_resolver` field to `WorkflowExecutor`
2. Initialize resolver in constructor/before execution
3. Add `execute_prebuilt_workflow()` method
4. Modify task execution to check for `uses_workflow`
5. Handle input merging and task prefixing
6. Add integration tests

**Files:** `src/dsl/executor.rs`
**Tests:** Integration tests with real task groups

### Step 5: Task Graph Enhancement (Low Risk)
1. Add helper methods for namespace handling (optional)
2. Verify topological sort works with namespaced tasks
3. Add tests for namespaced task dependencies

**Files:** `src/dsl/task_graph.rs`
**Tests:** Unit tests with namespaced task IDs

## Testing Strategy

### Unit Tests
1. **Schema tests:** Serialization with imports and uses_workflow
2. **Namespace resolver tests:** All resolution scenarios
3. **Validator tests:** Import and workflow reference validation
4. **Parser tests:** Workflow reference parsing

### Integration Tests (tests/phase4_taskgroups_tests.rs)
1. Load task group with prebuilt workflow
2. Import group into workflow
3. Execute task with uses_workflow
4. Verify namespace isolation
5. Test input parameter passing
6. Test shared configuration application
7. Test dependency resolution across namespaces
8. Test error cases:
   - Missing namespace
   - Missing workflow
   - Invalid reference format
   - Circular workflow dependencies

### Test Fixtures
```
tests/fixtures/taskgroups/
  ├── google-workspace.taskgroup.yaml
  ├── slack-integrations.taskgroup.yaml
  └── workflows/
      ├── import-single-group.yaml
      ├── import-multiple-groups.yaml
      └── uses-workflow.yaml
```

## Error Handling

### Graceful Degradation
- If import fails to load, provide clear error with group name/version
- If workflow not found, list available workflows in namespace
- If namespace not found, list available namespaces

### Error Messages
- "Namespace 'google' not found in imports"
- "Workflow 'upload-files' not found in namespace 'google'. Available: [create-folder, share-file]"
- "Failed to load task group 'google-workspace@1.0.0': file not found"

## Backwards Compatibility

All changes are additive:
- `imports` is optional (default: empty HashMap)
- `uses_workflow` is optional (default: None)
- Existing workflows work without modification
- No breaking changes to existing APIs

## Performance Considerations

### Import Resolution
- Load and resolve imports once during executor initialization
- Cache resolved groups in NamespaceResolver
- Avoid reloading groups multiple times

### Task Expansion
- Expand prebuilt workflow tasks on-demand
- Prefix task IDs to avoid collisions
- Reuse existing task graph infrastructure

### Memory Usage
- Reasonable: groups typically contain 5-20 tasks
- Shared config applied once during resolution
- No significant overhead vs inline tasks

## Open Questions

1. **Should namespace resolution be case-sensitive?**
   - Recommendation: Yes, for consistency with task/agent names

2. **How to handle version conflicts in transitive dependencies?**
   - Recommendation: Use existing dependency resolution in groups/deps.rs

3. **Should we support aliasing imported namespaces?**
   - Example: `imports: { gw: "google-workspace@1.0.0" }`
   - Recommendation: Not in Phase 4, consider for future

4. **How to handle output from prebuilt workflows?**
   - Recommendation: Support output mapping in parent task

## Success Criteria

Phase 4 is complete when:
1. ✅ Workflows can declare imports
2. ✅ Tasks can use prebuilt workflows via uses_workflow
3. ✅ Namespace resolution works correctly
4. ✅ Shared configuration is applied
5. ✅ All validations pass
6. ✅ Integration tests pass (15+ tests)
7. ✅ Example workflows demonstrate features
8. ✅ No regressions in existing tests

## Estimated Effort

- Schema changes: 1-2 hours
- Namespace resolver: 4-6 hours
- Validator integration: 2-3 hours
- Executor integration: 6-8 hours
- Task graph enhancements: 1-2 hours
- Testing: 6-8 hours
- Documentation: 2-3 hours

**Total: 22-32 hours** (3-4 days)

## Next Steps

After Phase 4 completion:
1. Create example task groups for common integrations
2. Add CLI support for managing task groups
3. Implement task group registry/marketplace
4. Add versioning and upgrade tools
5. Create task group templates/scaffolding

---

**End of Analysis**
