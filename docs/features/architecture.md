# Task Groups Architecture

This document describes the internal architecture and implementation details of the task groups feature.

## Table of Contents

1. [Overview](#overview)
2. [Core Components](#core-components)
3. [Data Structures](#data-structures)
4. [Execution Model](#execution-model)
5. [Dependency Resolution](#dependency-resolution)
6. [Variable Resolution](#variable-resolution)
7. [State Management](#state-management)
8. [Error Handling](#error-handling)
9. [Performance Considerations](#performance-considerations)
10. [Extension Points](#extension-points)

## Overview

The task groups feature is implemented across several modules in the DSL system:

```
src/dsl/
├── schema.rs           # TaskGroup struct and configuration
├── parser.rs           # YAML deserialization
├── validator.rs        # Validation logic
├── executor.rs         # Execution orchestration
├── task_graph.rs       # Dependency graph and scheduling
├── variables.rs        # Variable resolution
└── state.rs            # State management
```

### Design Principles

1. **Separation of Concerns**: Each component has a single, well-defined responsibility
2. **Immutability**: Configuration is immutable after parsing
3. **Type Safety**: Strong typing throughout with minimal runtime checks
4. **Composability**: Groups compose cleanly into hierarchies
5. **Testability**: Each component can be tested in isolation

## Core Components

### Schema (`schema.rs`)

Defines the `TaskGroup` struct and related types.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskGroup {
    pub description: String,
    pub execution_mode: ExecutionMode,
    pub tasks: Vec<String>,
    pub groups: Option<Vec<String>>,
    pub depends_on: Option<Vec<String>>,
    pub parent: Option<String>,
    pub condition: Option<String>,
    pub on_error: Option<ErrorStrategy>,
    pub timeout: Option<u64>,
    pub max_concurrency: Option<usize>,
    pub inputs: Option<HashMap<String, VariableDefinition>>,
    pub outputs: Option<HashMap<String, VariableOutput>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    Sequential,
    Parallel,
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorStrategy {
    Stop,
    Continue,
    Rollback,
}
```

**Responsibilities:**
- Define data structures
- Implement serialization/deserialization
- Provide validation constraints
- Define enums for configuration options

### Parser (`parser.rs`)

Deserializes YAML into `TaskGroup` instances.

```rust
pub fn parse_workflow(yaml: &str) -> Result<Workflow, ParseError> {
    let workflow: Workflow = serde_yaml::from_str(yaml)
        .map_err(ParseError::YamlError)?;

    Ok(workflow)
}
```

**Responsibilities:**
- Parse YAML syntax
- Create TaskGroup instances
- Handle parsing errors
- Validate YAML structure

**Error Handling:**
- Syntax errors → `ParseError::YamlError`
- Type mismatches → `ParseError::TypeError`
- Unknown fields → Warning (serde `deny_unknown_fields` optional)

### Validator (`validator.rs`)

Performs semantic validation of task groups.

```rust
pub struct GroupValidator {
    workflow: Workflow,
    errors: Vec<ValidationError>,
}

impl GroupValidator {
    pub fn validate(&mut self) -> Result<(), Vec<ValidationError>> {
        self.validate_group_references()?;
        self.validate_hierarchy()?;
        self.validate_dependencies()?;
        self.validate_variables()?;
        self.validate_tasks()?;

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }
}
```

**Validation Rules:**

1. **Group References**: All referenced groups exist
2. **Hierarchy**: Parent-child relationships are valid
3. **Dependencies**: No circular dependencies
4. **Variables**: All variable references are valid
5. **Tasks**: All tasks assigned to groups exist

**Algorithms:**

**Cycle Detection** (Tarjan's Algorithm):
```rust
fn detect_cycles(&self, groups: &HashMap<String, TaskGroup>) -> Vec<ValidationError> {
    let mut visited = HashSet::new();
    let mut stack = HashSet::new();
    let mut errors = Vec::new();

    for group_id in groups.keys() {
        if !visited.contains(group_id) {
            self.visit_group(group_id, groups, &mut visited, &mut stack, &mut errors);
        }
    }

    errors
}

fn visit_group(
    &self,
    group_id: &str,
    groups: &HashMap<String, TaskGroup>,
    visited: &mut HashSet<String>,
    stack: &mut HashSet<String>,
    errors: &mut Vec<ValidationError>,
) {
    visited.insert(group_id.to_string());
    stack.insert(group_id.to_string());

    if let Some(group) = groups.get(group_id) {
        if let Some(deps) = &group.depends_on {
            for dep in deps {
                if stack.contains(dep) {
                    errors.push(ValidationError::CircularDependency {
                        cycle: format!("{} → {}", group_id, dep),
                    });
                } else if !visited.contains(dep) {
                    self.visit_group(dep, groups, visited, stack, errors);
                }
            }
        }
    }

    stack.remove(group_id);
}
```

**Hierarchy Validation**:
```rust
fn validate_hierarchy(&self) -> Result<(), ValidationError> {
    for (group_id, group) in &self.workflow.task_groups {
        // Check parent exists
        if let Some(parent_id) = &group.parent {
            if !self.workflow.task_groups.contains_key(parent_id) {
                return Err(ValidationError::UnknownParent {
                    group: group_id.clone(),
                    parent: parent_id.clone(),
                });
            }

            // Check parent includes this child
            let parent = &self.workflow.task_groups[parent_id];
            if let Some(children) = &parent.groups {
                if !children.contains(group_id) {
                    return Err(ValidationError::MissingChildReference {
                        parent: parent_id.clone(),
                        child: group_id.clone(),
                    });
                }
            }
        }

        // Check child groups exist
        if let Some(children) = &group.groups {
            for child_id in children {
                if !self.workflow.task_groups.contains_key(child_id) {
                    return Err(ValidationError::UnknownChildGroup {
                        parent: group_id.clone(),
                        child: child_id.clone(),
                    });
                }
            }
        }
    }

    Ok(())
}
```

### Executor (`executor.rs`)

Orchestrates task group execution.

```rust
pub struct GroupExecutor {
    workflow: Workflow,
    task_graph: TaskGraph,
    state: WorkflowState,
    variables: VariableContext,
}

impl GroupExecutor {
    pub async fn execute_group(&mut self, group_id: &str) -> Result<GroupResult, ExecutionError> {
        let group = self.workflow.task_groups.get(group_id)
            .ok_or_else(|| ExecutionError::UnknownGroup(group_id.to_string()))?;

        // Resolve input variables
        self.resolve_group_inputs(group_id)?;

        // Check condition
        if !self.evaluate_condition(group)? {
            return Ok(GroupResult::skipped());
        }

        // Execute based on mode
        let result = match group.execution_mode {
            ExecutionMode::Sequential => self.execute_sequential(group).await?,
            ExecutionMode::Parallel => self.execute_parallel(group).await?,
            ExecutionMode::Auto => self.execute_auto(group).await?,
        };

        // Capture output variables
        self.capture_group_outputs(group_id)?;

        Ok(result)
    }

    async fn execute_sequential(&mut self, group: &TaskGroup) -> Result<GroupResult, ExecutionError> {
        for task_id in &group.tasks {
            let result = self.execute_task(task_id).await?;

            if result.failed() {
                match group.on_error.as_ref().unwrap_or(&ErrorStrategy::Stop) {
                    ErrorStrategy::Stop => return Ok(GroupResult::failed()),
                    ErrorStrategy::Continue => continue,
                    ErrorStrategy::Rollback => return self.execute_rollback(group).await,
                }
            }
        }

        Ok(GroupResult::success())
    }

    async fn execute_parallel(&mut self, group: &TaskGroup) -> Result<GroupResult, ExecutionError> {
        let max_concurrency = group.max_concurrency.unwrap_or(usize::MAX);
        let semaphore = Arc::new(Semaphore::new(max_concurrency));

        let mut handles = vec![];

        for task_id in &group.tasks {
            let permit = semaphore.clone().acquire_owned().await?;
            let task_id = task_id.clone();
            let executor = self.clone();

            let handle = tokio::spawn(async move {
                let result = executor.execute_task(&task_id).await;
                drop(permit);
                result
            });

            handles.push(handle);
        }

        let results = futures::future::join_all(handles).await;

        let mut failures = 0;
        for result in results {
            if result.is_err() || result.unwrap().failed() {
                failures += 1;

                if let ErrorStrategy::Stop = group.on_error.as_ref().unwrap_or(&ErrorStrategy::Stop) {
                    return Ok(GroupResult::failed());
                }
            }
        }

        if failures > 0 {
            Ok(GroupResult::partial_failure(failures))
        } else {
            Ok(GroupResult::success())
        }
    }
}
```

**Execution Flow:**

1. **Resolve Dependencies**: Topologically sort groups
2. **Resolve Inputs**: Interpolate input variables
3. **Evaluate Condition**: Check if group should execute
4. **Execute Tasks/Groups**: Based on execution mode
5. **Handle Errors**: Apply error strategy
6. **Capture Outputs**: Save output variables
7. **Update State**: Persist execution state

### Task Graph (`task_graph.rs`)

Manages dependency graph and execution scheduling.

```rust
pub struct TaskGraph {
    nodes: HashMap<String, GraphNode>,
    edges: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: String,
    pub node_type: NodeType,
    pub dependencies: Vec<String>,
    pub status: NodeStatus,
}

#[derive(Debug, Clone)]
pub enum NodeType {
    Task(String),
    Group(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeStatus {
    Pending,
    Ready,
    Running,
    Completed,
    Failed,
    Skipped,
}

impl TaskGraph {
    pub fn new(workflow: &Workflow) -> Self {
        let mut graph = TaskGraph {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        };

        graph.build_from_workflow(workflow);
        graph
    }

    pub fn topological_sort(&self) -> Result<Vec<String>, GraphError> {
        let mut sorted = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_marks = HashSet::new();

        for node_id in self.nodes.keys() {
            if !visited.contains(node_id) {
                self.visit_node(
                    node_id,
                    &mut visited,
                    &mut temp_marks,
                    &mut sorted,
                )?;
            }
        }

        sorted.reverse();
        Ok(sorted)
    }

    fn visit_node(
        &self,
        node_id: &str,
        visited: &mut HashSet<String>,
        temp_marks: &mut HashSet<String>,
        sorted: &mut Vec<String>,
    ) -> Result<(), GraphError> {
        if temp_marks.contains(node_id) {
            return Err(GraphError::CyclicDependency);
        }

        if visited.contains(node_id) {
            return Ok(());
        }

        temp_marks.insert(node_id.to_string());

        if let Some(deps) = self.edges.get(node_id) {
            for dep in deps {
                self.visit_node(dep, visited, temp_marks, sorted)?;
            }
        }

        temp_marks.remove(node_id);
        visited.insert(node_id.to_string());
        sorted.push(node_id.to_string());

        Ok(())
    }

    pub fn get_ready_nodes(&self) -> Vec<String> {
        self.nodes
            .iter()
            .filter(|(_, node)| {
                node.status == NodeStatus::Pending
                    && node.dependencies.iter().all(|dep| {
                        self.nodes
                            .get(dep)
                            .map(|n| n.status == NodeStatus::Completed)
                            .unwrap_or(false)
                    })
            })
            .map(|(id, _)| id.clone())
            .collect()
    }
}
```

**Algorithms:**

**Topological Sort** (Depth-First Search):
- Builds execution order respecting dependencies
- Detects cycles during traversal
- Returns sorted list of groups/tasks

**Ready Node Detection**:
- Finds nodes with all dependencies satisfied
- Enables parallel execution scheduling
- Updates as nodes complete

## Data Structures

### TaskGroup

```rust
pub struct TaskGroup {
    pub description: String,           // Human-readable description
    pub execution_mode: ExecutionMode, // Sequential/Parallel/Auto
    pub tasks: Vec<String>,            // Task IDs in this group
    pub groups: Option<Vec<String>>,   // Child group IDs
    pub depends_on: Option<Vec<String>>, // Dependency group IDs
    pub parent: Option<String>,        // Parent group ID
    pub condition: Option<String>,     // Conditional expression
    pub on_error: Option<ErrorStrategy>, // Error handling
    pub timeout: Option<u64>,          // Timeout in seconds
    pub max_concurrency: Option<usize>, // Concurrency limit
    pub inputs: Option<HashMap<String, VariableDefinition>>,
    pub outputs: Option<HashMap<String, VariableOutput>>,
}
```

**Memory Layout:**
- Small footprint (~200 bytes)
- Cloneable (for parallel execution)
- Immutable after parsing

### VariableContext

```rust
pub struct VariableContext {
    workflow_vars: HashMap<String, Variable>,
    group_vars: HashMap<String, HashMap<String, Variable>>,
    task_vars: HashMap<String, HashMap<String, Variable>>,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub value: VariableValue,
    pub var_type: VariableType,
}

#[derive(Debug, Clone)]
pub enum VariableValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Object(serde_json::Value),
    Array(Vec<VariableValue>),
}
```

**Scoping Rules:**
1. Task scope (highest priority)
2. Group scope
3. Workflow scope (lowest priority)

### GroupResult

```rust
pub struct GroupResult {
    pub status: GroupStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub task_results: Vec<TaskResult>,
    pub outputs: HashMap<String, Variable>,
    pub error: Option<String>,
}

pub enum GroupStatus {
    Success,
    Failed,
    PartialFailure { success: usize, failed: usize },
    Skipped,
    Timeout,
}
```

## Execution Model

### Sequential Execution

```
Task 1 → Task 2 → Task 3 → Task 4
  |        |        |        |
  v        v        v        v
 Done     Done     Done     Done
```

**Implementation:**
```rust
async fn execute_sequential(&mut self, group: &TaskGroup) -> Result<GroupResult> {
    for task_id in &group.tasks {
        let result = self.execute_task(task_id).await?;

        if result.failed() && group.on_error == ErrorStrategy::Stop {
            return Ok(GroupResult::failed());
        }
    }

    Ok(GroupResult::success())
}
```

**Characteristics:**
- Linear execution
- Predictable order
- Lower memory usage
- Simpler error handling

### Parallel Execution

```
Task 1 ──┐
Task 2 ──┼─→ All complete
Task 3 ──┤
Task 4 ──┘
```

**Implementation:**
```rust
async fn execute_parallel(&mut self, group: &TaskGroup) -> Result<GroupResult> {
    let semaphore = Arc::new(Semaphore::new(group.max_concurrency.unwrap_or(usize::MAX)));

    let handles: Vec<_> = group.tasks.iter().map(|task_id| {
        let permit = semaphore.clone();
        let task = task_id.clone();

        tokio::spawn(async move {
            let _guard = permit.acquire().await;
            self.execute_task(&task).await
        })
    }).collect();

    let results = futures::join_all(handles).await;

    aggregate_results(results)
}
```

**Characteristics:**
- Concurrent execution
- Higher throughput
- Resource management via semaphore
- Complex error handling

### Auto Execution

Analyzes dependencies and chooses optimal strategy:

```rust
fn determine_execution_strategy(&self, group: &TaskGroup) -> ExecutionMode {
    // Build task dependency graph
    let has_dependencies = group.tasks.iter().any(|task_id| {
        self.workflow.tasks.get(task_id)
            .and_then(|t| t.depends_on.as_ref())
            .map(|deps| !deps.is_empty())
            .unwrap_or(false)
    });

    if has_dependencies {
        // Hybrid: topological sort + parallel levels
        ExecutionMode::Parallel
    } else {
        // No dependencies: fully parallel
        ExecutionMode::Parallel
    }
}
```

## Dependency Resolution

### Algorithm

**Input:** Set of groups with dependencies

**Output:** Execution order

**Steps:**
1. Build directed acyclic graph (DAG)
2. Detect cycles (Tarjan's algorithm)
3. Topological sort (DFS)
4. Return ordered list

**Pseudocode:**
```
function resolve_dependencies(groups):
    graph = build_graph(groups)

    if has_cycle(graph):
        return error("Circular dependency")

    return topological_sort(graph)

function topological_sort(graph):
    visited = set()
    stack = []

    for node in graph:
        if node not in visited:
            dfs(node, visited, stack)

    return reversed(stack)

function dfs(node, visited, stack):
    visited.add(node)

    for dependency in node.dependencies:
        if dependency not in visited:
            dfs(dependency, visited, stack)

    stack.append(node)
```

### Example

**Input:**
```yaml
task_groups:
  A: {}
  B: { depends_on: [A] }
  C: { depends_on: [A] }
  D: { depends_on: [B, C] }
```

**Graph:**
```
    A
   ↙ ↘
  B   C
   ↘ ↙
    D
```

**Execution Order:** A → (B, C in parallel) → D

## Variable Resolution

### Resolution Algorithm

```rust
pub fn resolve_variable(&self, expr: &str) -> Result<Variable, VariableError> {
    // Parse expression: ${scope.variable}
    let (scope, var_name) = self.parse_expression(expr)?;

    match scope {
        Some("workflow") => self.resolve_workflow_var(var_name),
        Some("group") => {
            let (group_id, var) = self.split_group_var(var_name)?;
            self.resolve_group_var(group_id, var)
        },
        Some("task") => {
            let (task_id, var) = self.split_task_var(var_name)?;
            self.resolve_task_var(task_id, var)
        },
        None => {
            // Implicit scope: search upward
            self.resolve_implicit(var_name)
        },
        _ => Err(VariableError::UnknownScope(scope.unwrap().to_string())),
    }
}

fn resolve_implicit(&self, var_name: &str) -> Result<Variable, VariableError> {
    // Search order: task → group → workflow
    if let Ok(var) = self.resolve_current_task_var(var_name) {
        return Ok(var);
    }

    if let Ok(var) = self.resolve_current_group_var(var_name) {
        return Ok(var);
    }

    self.resolve_workflow_var(var_name)
}
```

### Interpolation

Variable interpolation in strings:

```rust
pub fn interpolate(&self, template: &str) -> Result<String, VariableError> {
    let re = Regex::new(r"\$\{([^}]+)\}").unwrap();

    let mut result = template.to_string();

    for cap in re.captures_iter(template) {
        let expr = &cap[1];
        let value = self.resolve_variable(expr)?;
        let replacement = value.to_string();

        result = result.replace(&format!("${{{}}}", expr), &replacement);
    }

    Ok(result)
}
```

**Example:**
```
Input:  "Deploy ${workflow.service} to ${workflow.env}"
Output: "Deploy api-server to staging"
```

## State Management

### State Persistence

```rust
pub struct WorkflowState {
    groups: HashMap<String, GroupState>,
    tasks: HashMap<String, TaskState>,
    variables: VariableContext,
    checkpoints: Vec<Checkpoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupState {
    pub status: GroupStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub outputs: HashMap<String, Variable>,
}

impl WorkflowState {
    pub fn save(&self, path: &Path) -> Result<(), StateError> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, StateError> {
        let json = std::fs::read_to_string(path)?;
        let state = serde_json::from_str(&json)?;
        Ok(state)
    }

    pub fn checkpoint(&mut self, name: &str) -> Result<(), StateError> {
        let checkpoint = Checkpoint {
            name: name.to_string(),
            timestamp: Utc::now(),
            state: self.clone(),
        };

        self.checkpoints.push(checkpoint);
        Ok(())
    }
}
```

### Resume Capability

```rust
pub fn resume_workflow(state_path: &Path, workflow: &Workflow) -> Result<(), ExecutionError> {
    let state = WorkflowState::load(state_path)?;

    // Find incomplete groups
    let incomplete_groups: Vec<_> = workflow
        .task_groups
        .keys()
        .filter(|id| {
            state.groups.get(*id)
                .map(|s| s.status != GroupStatus::Completed)
                .unwrap_or(true)
        })
        .collect();

    // Re-execute from first incomplete group
    let mut executor = GroupExecutor::with_state(workflow, state);

    for group_id in incomplete_groups {
        executor.execute_group(group_id).await?;
    }

    Ok(())
}
```

## Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("Group not found: {0}")]
    UnknownGroup(String),

    #[error("Task execution failed: {0}")]
    TaskFailed(String),

    #[error("Timeout after {0}s")]
    Timeout(u64),

    #[error("Variable error: {0}")]
    VariableError(#[from] VariableError),

    #[error("Condition evaluation failed: {0}")]
    ConditionError(String),

    #[error("Rollback failed: {0}")]
    RollbackFailed(String),
}
```

### Error Recovery

```rust
async fn execute_with_recovery(&mut self, group: &TaskGroup) -> Result<GroupResult> {
    let start = Instant::now();

    let result = tokio::select! {
        res = self.execute_group_inner(group) => res,
        _ = tokio::time::sleep(Duration::from_secs(group.timeout.unwrap_or(u64::MAX))) => {
            Err(ExecutionError::Timeout(group.timeout.unwrap()))
        }
    };

    match result {
        Ok(r) => Ok(r),
        Err(e) => {
            match group.on_error.as_ref().unwrap_or(&ErrorStrategy::Stop) {
                ErrorStrategy::Stop => Err(e),
                ErrorStrategy::Continue => {
                    self.log_error(&e);
                    Ok(GroupResult::partial_failure(1))
                },
                ErrorStrategy::Rollback => {
                    self.execute_rollback(group).await
                }
            }
        }
    }
}
```

## Performance Considerations

### Memory Usage

**Per Group:** ~200 bytes
**Per Task:** ~150 bytes
**Variables:** ~50 bytes/variable

**Example Workflow:**
- 10 groups
- 50 tasks
- 100 variables

**Total:** ~15 KB (negligible)

### Concurrency

**Parallel Execution:**
- Uses Tokio for async execution
- Semaphore for concurrency control
- Configurable limits prevent resource exhaustion

**Overhead:**
- Task spawn: ~10μs
- Variable resolution: ~1μs
- Condition evaluation: ~5μs

### Optimization Strategies

1. **Lazy Evaluation**: Variables resolved on-demand
2. **Caching**: Dependency graph cached after validation
3. **Parallel Validation**: Independent checks run in parallel
4. **State Streaming**: Large state files streamed, not loaded entirely

## Extension Points

### Custom Execution Modes

```rust
pub trait ExecutionStrategy: Send + Sync {
    async fn execute(
        &self,
        tasks: &[String],
        executor: &GroupExecutor,
    ) -> Result<GroupResult, ExecutionError>;
}

// Register custom strategy
executor.register_strategy("custom", Box::new(MyStrategy));
```

### Custom Error Handlers

```rust
pub trait ErrorHandler: Send + Sync {
    async fn handle_error(
        &self,
        error: &ExecutionError,
        group: &TaskGroup,
        context: &ExecutionContext,
    ) -> Result<RecoveryAction, ExecutionError>;
}

pub enum RecoveryAction {
    Retry { max_attempts: usize },
    Rollback,
    Skip,
    Fail,
}
```

### Custom Variable Sources

```rust
pub trait VariableSource: Send + Sync {
    async fn resolve(
        &self,
        source: &VariableOutputSource,
    ) -> Result<Variable, VariableError>;
}

// Register custom source
variables.register_source("database", Box::new(DatabaseSource::new()));
```

### Hooks and Callbacks

```rust
pub trait GroupHooks: Send + Sync {
    async fn on_group_start(&self, group_id: &str, context: &ExecutionContext);
    async fn on_group_complete(&self, group_id: &str, result: &GroupResult);
    async fn on_group_error(&self, group_id: &str, error: &ExecutionError);
}

executor.register_hooks(Box::new(MyHooks));
```

## See Also

- [Main Documentation](./README.md)
- [API Reference](./api-reference.md)
- [Tutorial](./tutorial.md)
- [Examples](../../examples/task-groups/)
