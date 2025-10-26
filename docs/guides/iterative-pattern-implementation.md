# Iterative Pattern Implementation Plan

**Version:** 1.0
**Date:** 2025-10-19
**Status:** Planning

---

## Table of Contents

1. [Overview](#overview)
2. [Design Goals](#design-goals)
3. [Loop Pattern Types](#loop-pattern-types)
4. [Architecture Changes](#architecture-changes)
5. [Implementation Phases](#implementation-phases)
6. [Testing Strategy](#testing-strategy)
7. [Security & Safety](#security--safety)
8. [Success Criteria](#success-criteria)

---

## Overview

This document outlines the comprehensive plan for adding multistep iterative patterns (loops) to the DSL. The implementation will enable workflows to process collections, poll for conditions, retry with backoff, and execute tasks repeatedly - all while maintaining backward compatibility with existing workflows.

### Current Architecture Strengths

The existing DSL already provides strong foundations for loop implementation:

- **Robust Dependency Management** - Topological sorting with cycle detection
- **Flexible Error Handling** - Multiple retry strategies with backoff
- **Quality Assurance** - Definition of Done with automatic retries
- **State Persistence** - Checkpoint/resume capability
- **Conditional Execution** - Rich condition syntax with logical operators
- **Parallel Execution** - Concurrent task execution with coordination

### What This Adds

- **Collection Iteration** - Process arrays, files, ranges
- **Conditional Loops** - While/until patterns
- **Repeat Patterns** - Count-based iteration
- **Variable Substitution** - Use loop variables in task definitions
- **Loop Control Flow** - Break/continue semantics
- **Result Aggregation** - Collect outputs from iterations
- **Nested Loops** - Multi-level iteration support

---

## Design Goals

### Core Objectives

1. **Enable Collection Iteration** - ForEach patterns for processing arrays
2. **Support Conditional Loops** - While/until patterns for dynamic workflows
3. **Add Repeat Patterns** - Count-based iteration for batch operations
4. **Maintain Backward Compatibility** - All existing workflows continue to work
5. **Support Nested Loops** - Multi-level iteration contexts
6. **Enable Loop State Persistence** - Checkpoint and resume interrupted loops
7. **Provide Loop Control Flow** - Break/continue semantics

### Integration Principles

- **Leverage Existing Systems** - Build on retry and state mechanisms
- **Extend Task Graph** - Handle virtual loop instances
- **Use Condition Evaluation** - Reuse existing condition system
- **Maintain Performance** - Support parallel loop execution
- **Ensure Safety** - Resource limits and timeout protection

---

## Loop Pattern Types

### 1. ForEach Loop

Iterate over collections (arrays, files, ranges).

**YAML Syntax:**
```yaml
tasks:
  process_files:
    description: "Process each file"
    agent: "processor"
    loop:
      type: "for_each"
      collection:
        source: "state"  # or "file", "range", "inline"
        key: "file_list"  # state key containing array
      iterator: "file"  # variable name for current item
      parallel: false  # execute sequentially by default
      max_parallel: 3  # optional: limit concurrent iterations
    loop_control:
      break_on_error: false
      collect_results: true
      result_key: "processed_files"
```

**Use Cases:**
- Batch file processing
- Multi-entity operations
- Data transformation pipelines
- Parallel task execution

---

### 2. While Loop

Execute while condition is true.

**YAML Syntax:**
```yaml
tasks:
  retry_until_success:
    description: "Keep trying until success"
    agent: "worker"
    loop:
      type: "while"
      condition:
        type: "state_equals"
        key: "retry_needed"
        value: true
      max_iterations: 10  # safety limit
      iteration_variable: "attempt"  # track iteration count
      delay_between_secs: 2  # delay between iterations
    loop_control:
      break_condition:
        type: "state_equals"
        key: "success"
        value: true
```

**Use Cases:**
- Retry with custom logic
- Conditional processing
- State-driven workflows
- Adaptive execution

---

### 3. RepeatUntil Loop

Execute until condition becomes true (do-while pattern).

**YAML Syntax:**
```yaml
tasks:
  poll_api:
    description: "Poll until data ready"
    agent: "poller"
    loop:
      type: "repeat_until"
      condition:  # exit condition
        type: "state_equals"
        key: "data_ready"
        value: true
      min_iterations: 1
      max_iterations: 20
      iteration_variable: "poll_count"
      delay_between_secs: 5  # seconds between iterations
```

**Use Cases:**
- API polling
- Waiting for resources
- Event-driven workflows
- Convergence checking

---

### 4. Repeat Loop

Execute N times (count-based).

**YAML Syntax:**
```yaml
tasks:
  generate_reports:
    description: "Generate N reports"
    agent: "generator"
    loop:
      type: "repeat"
      count: 5
      iterator: "index"  # 0-based index variable
      parallel: true
      max_parallel: 3
```

**Use Cases:**
- Batch generation
- Load testing
- Redundancy creation
- Sampling operations

---

## Architecture Changes

### 1. Schema Extensions (`schema.rs`)

**New Types:**

```rust
// Loop specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LoopSpec {
    ForEach {
        collection: CollectionSource,
        iterator: String,  // variable name
        #[serde(default)]
        parallel: bool,
        max_parallel: Option<usize>,
        #[serde(default)]
        break_on_error: bool,
    },
    While {
        condition: ConditionSpec,
        max_iterations: usize,
        iteration_variable: Option<String>,
        delay_between_secs: Option<u64>,
    },
    RepeatUntil {
        condition: ConditionSpec,  // exit condition
        min_iterations: Option<usize>,
        max_iterations: usize,
        iteration_variable: Option<String>,
        delay_between_secs: Option<u64>,
    },
    Repeat {
        count: usize,
        iterator: Option<String>,  // variable for index
        #[serde(default)]
        parallel: bool,
        max_parallel: Option<usize>,
    },
}

// Collection sources
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum CollectionSource {
    State { key: String },
    File { path: String, format: FileFormat },
    Range { start: i64, end: i64, step: Option<i64> },
    Inline { items: Vec<serde_json::Value> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileFormat {
    Json,      // read JSON array
    JsonLines, // each line is an item
    Csv,       // each row is an item
    Lines,     // each line is a string
}

// Loop control flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopControl {
    pub break_condition: Option<ConditionSpec>,
    pub continue_condition: Option<ConditionSpec>,
    pub collect_results: bool,  // aggregate iteration outputs
    pub result_key: Option<String>,  // state key for results
}
```

**TaskSpec Extension:**

```rust
pub struct TaskSpec {
    // ... existing fields ...
    pub loop_spec: Option<LoopSpec>,
    pub loop_control: Option<LoopControl>,
}
```

**Estimated Lines:** +200 lines to `schema.rs`

---

### 2. Executor Modifications (`executor.rs`)

**New Core Functions:**

```rust
// Main loop dispatcher
async fn execute_task_with_loop(
    &self,
    task_id: &str,
    task: &TaskSpec,
    loop_spec: &LoopSpec,
) -> Result<TaskOutput>;

// Loop type implementations
async fn execute_foreach_loop(
    &self,
    task_id: &str,
    task: &TaskSpec,
    collection: &CollectionSource,
    iterator: &str,
    parallel: bool,
) -> Result<TaskOutput>;

async fn execute_while_loop(
    &self,
    task_id: &str,
    task: &TaskSpec,
    condition: &ConditionSpec,
    max_iterations: usize,
) -> Result<TaskOutput>;

async fn execute_repeat_until_loop(
    &self,
    task_id: &str,
    task: &TaskSpec,
    condition: &ConditionSpec,
    max_iterations: usize,
) -> Result<TaskOutput>;

async fn execute_repeat_loop(
    &self,
    task_id: &str,
    task: &TaskSpec,
    count: usize,
    parallel: bool,
) -> Result<TaskOutput>;

// Helper functions
async fn resolve_collection(
    &self,
    collection: &CollectionSource,
) -> Result<Vec<serde_json::Value>>;

async fn execute_loop_iteration(
    &self,
    task_id: &str,
    task: &TaskSpec,
    iteration: usize,
    iterator: &str,
    value: &serde_json::Value,
) -> Result<TaskOutput>;
```

**ForEach Loop Implementation Example:**

```rust
async fn execute_foreach_loop(
    &self,
    task_id: &str,
    task: &TaskSpec,
    collection: &CollectionSource,
    iterator: &str,
    parallel: bool,
) -> Result<TaskOutput> {
    // 1. Resolve collection to Vec<Value>
    let items = self.resolve_collection(collection).await?;

    // 2. Create loop context
    let mut results = Vec::new();
    let mut iteration = 0;

    if parallel {
        // 3a. Parallel execution
        let futures: Vec<_> = items.iter().enumerate().map(|(idx, item)| {
            let task_clone = task.clone();
            let item_clone = item.clone();
            async move {
                self.execute_loop_iteration(
                    task_id,
                    &task_clone,
                    idx,
                    iterator,
                    &item_clone,
                ).await
            }
        }).collect();

        results = futures::future::try_join_all(futures).await?;
    } else {
        // 3b. Sequential execution
        for (idx, item) in items.iter().enumerate() {
            // Check break condition
            if let Some(break_cond) = &task.loop_control.as_ref()
                .and_then(|lc| lc.break_condition.as_ref()) {
                if self.evaluate_condition(break_cond).await? {
                    break;
                }
            }

            // Execute iteration
            let result = self.execute_loop_iteration(
                task_id,
                task,
                idx,
                iterator,
                item,
            ).await?;

            results.push(result);
            iteration += 1;
        }
    }

    // 4. Aggregate results if requested
    if task.loop_control.as_ref().map_or(false, |lc| lc.collect_results) {
        self.store_loop_results(task_id, results).await?;
    }

    Ok(TaskOutput::LoopCompleted { iterations: iteration })
}
```

**Estimated Lines:** +300 lines to `executor.rs`

---

### 3. Loop Context & Variable Substitution

**New File: `loop_context.rs`**

```rust
use std::collections::HashMap;
use serde_json::Value;

/// Loop iteration context with variable substitution
pub struct LoopContext {
    pub iteration: usize,
    pub variables: HashMap<String, Value>,
    pub parent_context: Option<Box<LoopContext>>,  // for nested loops
}

impl LoopContext {
    pub fn new(iteration: usize) -> Self {
        Self {
            iteration,
            variables: HashMap::new(),
            parent_context: None,
        }
    }

    pub fn with_parent(iteration: usize, parent: LoopContext) -> Self {
        Self {
            iteration,
            variables: HashMap::new(),
            parent_context: Some(Box::new(parent)),
        }
    }

    pub fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
            .or_else(|| {
                self.parent_context.as_ref()
                    .and_then(|p| p.get_variable(name))
            })
    }

    /// Substitute {{variable}} placeholders in text
    pub fn substitute_variables(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Replace {{iterator}} with current value
        for (key, value) in &self.variables {
            let placeholder = format!("{{{{{}}}}}", key);
            let value_str = match value {
                Value::String(s) => s.clone(),
                _ => value.to_string(),
            };
            result = result.replace(&placeholder, &value_str);
        }

        // Replace {{iteration}} with index
        result = result.replace("{{iteration}}", &self.iteration.to_string());

        // Check parent context for nested loops
        if let Some(parent) = &self.parent_context {
            result = parent.substitute_variables(&result);
        }

        result
    }
}

/// Substitute variables in a task spec before execution
pub fn substitute_task_variables(
    task: &mut TaskSpec,
    context: &LoopContext,
) {
    // Substitute in description
    task.description = context.substitute_variables(&task.description);

    // Substitute in conditions
    if let Some(condition) = &mut task.condition {
        substitute_condition_variables(condition, context);
    }

    // Substitute in definition_of_done
    if let Some(dod) = &mut task.definition_of_done {
        for criterion in &mut dod.criteria {
            substitute_criterion_variables(criterion, context);
        }
    }
}

fn substitute_condition_variables(
    condition: &mut ConditionSpec,
    context: &LoopContext,
) {
    match condition {
        ConditionSpec::StateEquals { key, value, .. } => {
            *key = context.substitute_variables(key);
            *value = context.substitute_variables(value);
        }
        ConditionSpec::StateExists { key, .. } => {
            *key = context.substitute_variables(key);
        }
        ConditionSpec::And { conditions } | ConditionSpec::Or { conditions } => {
            for cond in conditions {
                substitute_condition_variables(cond, context);
            }
        }
        ConditionSpec::Not { condition } => {
            substitute_condition_variables(condition, context);
        }
        _ => {}
    }
}
```

**Estimated Lines:** +150 lines (new file)

---

### 4. State Management Extensions (`state.rs`)

**New State Tracking:**

```rust
pub struct WorkflowState {
    // ... existing fields ...
    pub loop_states: HashMap<String, LoopState>,
    pub loop_results: HashMap<String, Vec<Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopState {
    pub task_id: String,
    pub current_iteration: usize,
    pub total_iterations: Option<usize>,  // known total for repeat loops
    pub iterator_value: Option<Value>,
    pub loop_variables: HashMap<String, Value>,
    pub iteration_statuses: Vec<TaskStatus>,
    pub started_at: DateTime<Utc>,
    pub last_iteration_at: Option<DateTime<Utc>>,
}
```

**New Methods:**

```rust
impl WorkflowState {
    pub fn init_loop(&mut self, task_id: &str, total_iterations: Option<usize>) {
        self.loop_states.insert(
            task_id.to_string(),
            LoopState {
                task_id: task_id.to_string(),
                current_iteration: 0,
                total_iterations,
                iterator_value: None,
                loop_variables: HashMap::new(),
                iteration_statuses: Vec::new(),
                started_at: Utc::now(),
                last_iteration_at: None,
            },
        );
    }

    pub fn update_loop_iteration(
        &mut self,
        task_id: &str,
        iteration: usize,
        status: TaskStatus,
        iterator_value: Option<Value>,
    ) {
        if let Some(loop_state) = self.loop_states.get_mut(task_id) {
            loop_state.current_iteration = iteration;
            loop_state.iterator_value = iterator_value;
            loop_state.last_iteration_at = Some(Utc::now());

            if iteration >= loop_state.iteration_statuses.len() {
                loop_state.iteration_statuses.resize(iteration + 1, TaskStatus::Pending);
            }
            loop_state.iteration_statuses[iteration] = status;
        }
    }

    pub fn set_loop_variable(&mut self, task_id: &str, name: String, value: Value) {
        if let Some(loop_state) = self.loop_states.get_mut(task_id) {
            loop_state.loop_variables.insert(name, value);
        }
    }

    pub fn get_loop_progress(&self, task_id: &str) -> Option<f64> {
        self.loop_states.get(task_id).and_then(|ls| {
            ls.total_iterations.map(|total| {
                if total == 0 {
                    0.0
                } else {
                    (ls.current_iteration as f64 / total as f64) * 100.0
                }
            })
        })
    }

    pub fn store_loop_result(&mut self, task_id: &str, result: Value) {
        self.loop_results.entry(task_id.to_string())
            .or_insert_with(Vec::new)
            .push(result);
    }

    pub fn get_loop_results(&self, task_id: &str) -> Option<&Vec<Value>> {
        self.loop_results.get(task_id)
    }
}
```

**Estimated Lines:** +150 lines to `state.rs`

---

### 5. Task Graph Updates (`task_graph.rs`)

**Loop Task Handling:**

```rust
#[derive(Debug, Clone)]
pub struct TaskNode {
    // ... existing fields ...
    pub is_loop: bool,
    pub loop_iteration_count: Option<usize>,
}

impl TaskGraph {
    pub fn add_loop_task(&mut self, task_id: String, spec: TaskSpec) {
        let mut node = TaskNode::new(task_id.clone(), spec);
        node.is_loop = true;
        self.nodes.insert(task_id, node);
    }

    pub fn is_loop_complete(&self, task_id: &str, state: &WorkflowState) -> bool {
        if let Some(node) = self.nodes.get(task_id) {
            if node.is_loop {
                return state.get_task_status(task_id)
                    .map_or(false, |s| s == TaskStatus::Completed);
            }
        }
        self.is_task_complete(task_id)
    }
}
```

**Strategy:** Dynamic loop handling - keep loop task as single node, handle iterations in executor without graph modification.

**Estimated Lines:** +50 lines to `task_graph.rs`

---

### 6. Validation Extensions (`validator.rs`)

**Loop Validation Rules:**

```rust
fn validate_loop_spec(loop_spec: &LoopSpec) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    match loop_spec {
        LoopSpec::ForEach { collection, max_parallel, .. } => {
            // Validate collection source
            validate_collection_source(collection, &mut errors);

            // Validate parallel limits
            if let Some(max) = max_parallel {
                if *max > MAX_PARALLEL_ITERATIONS {
                    errors.push(ValidationError::new(
                        format!("max_parallel ({}) exceeds limit ({})", max, MAX_PARALLEL_ITERATIONS)
                    ));
                }
            }
        }
        LoopSpec::While { max_iterations, condition, .. } |
        LoopSpec::RepeatUntil { max_iterations, condition, .. } => {
            // Validate iteration limits
            if *max_iterations > MAX_LOOP_ITERATIONS {
                errors.push(ValidationError::new(
                    format!("max_iterations ({}) exceeds limit ({})", max_iterations, MAX_LOOP_ITERATIONS)
                ));
            }

            // Validate condition
            validate_condition(condition, &mut errors);
        }
        LoopSpec::Repeat { count, .. } => {
            if *count > MAX_LOOP_ITERATIONS {
                errors.push(ValidationError::new(
                    format!("count ({}) exceeds limit ({})", count, MAX_LOOP_ITERATIONS)
                ));
            }
        }
    }

    errors
}
```

**Estimated Lines:** +100 lines to `validator.rs`

---

## Implementation Phases

### Phase 1: Foundation (Week 1)

**Goal:** Core loop infrastructure

**Tasks:**
1. Add `LoopSpec` enum and variants to `schema.rs`
2. Add `CollectionSource` enum for data sources
3. Add `LoopControl` struct for break/continue
4. Add `loop_spec` field to `TaskSpec`
5. Update YAML parser to handle loop syntax
6. Add validation rules for loop specifications
7. Add `LoopState` to state management
8. Add loop state methods (init, update, progress)

**Deliverables:**
- ✅ Schema supports all loop types
- ✅ Parser validates loop syntax
- ✅ State tracks loop progress
- ✅ Unit tests for schema and validation

**Files Modified:**
- `src/dsl/schema.rs` (+200 lines)
- `src/dsl/state.rs` (+100 lines)
- `src/dsl/validator.rs` (+50 lines)
- `src/dsl/parser.rs` (+50 lines)

---

### Phase 2: Basic Loop Execution (Week 2)

**Goal:** Implement ForEach and Repeat loops

**Tasks:**
1. Create `src/dsl/loop_context.rs` for variable substitution
2. Implement `execute_task_with_loop()` dispatcher in executor
3. Implement `execute_foreach_loop()` for sequential execution
4. Implement `execute_repeat_loop()` for count-based loops
5. Add collection resolution (`resolve_collection()`)
6. Implement loop context and variable substitution
7. Add loop iteration tracking in state
8. Update task graph to mark loop tasks
9. Create basic loop examples

**Deliverables:**
- ✅ ForEach loops work (sequential only)
- ✅ Repeat loops work (sequential only)
- ✅ Variable substitution works
- ✅ Basic examples (`examples/foreach_demo.rs`, `examples/repeat_demo.rs`)
- ✅ Integration tests

**Files Created:**
- `src/dsl/loop_context.rs` (+150 lines)
- `examples/foreach_demo.rs` (+100 lines)
- `examples/repeat_demo.rs` (+80 lines)

**Files Modified:**
- `src/dsl/executor.rs` (+200 lines)
- `src/dsl/state.rs` (+50 lines)
- `src/dsl/task_graph.rs` (+50 lines)

---

### Phase 3: Conditional Loops (Week 3)

**Goal:** Implement While and RepeatUntil loops

**Tasks:**
1. Implement `execute_while_loop()` in executor
2. Implement `execute_repeat_until_loop()` in executor
3. Add iteration delay support (tokio::time::sleep)
4. Add max_iterations safety limits
5. Integrate with existing condition evaluation
6. Add iteration variable tracking in state
7. Create conditional loop examples
8. Add integration tests

**Deliverables:**
- ✅ While loops functional
- ✅ RepeatUntil loops functional
- ✅ Condition-based termination works
- ✅ Advanced examples (`examples/while_demo.rs`, `examples/polling_demo.rs`)
- ✅ Integration tests for conditional loops

**Files Created:**
- `examples/while_demo.rs` (+100 lines)
- `examples/polling_demo.rs` (+120 lines)

**Files Modified:**
- `src/dsl/executor.rs` (+150 lines)
- `src/dsl/state.rs` (+30 lines)

---

### Phase 4: Parallel Execution (Week 4)

**Goal:** Enable parallel loop iterations

**Tasks:**
1. Add parallel execution for ForEach loops (tokio::spawn)
2. Implement `max_parallel` limiting using semaphore
3. Handle error aggregation in parallel iterations
4. Add parallel execution for Repeat loops
5. Update state tracking for concurrent iterations
6. Performance testing and optimization
7. Add parallel execution examples

**Deliverables:**
- ✅ Parallel ForEach works
- ✅ Parallel Repeat works
- ✅ Concurrency limits enforced
- ✅ Performance benchmarks (target: 1000 iterations < 2s)
- ✅ Parallel examples

**Files Created:**
- `examples/parallel_processing.rs` (+150 lines)
- `benches/loop_performance.rs` (+200 lines)

**Files Modified:**
- `src/dsl/executor.rs` (+100 lines)
- `src/dsl/state.rs` (+50 lines)

---

### Phase 5: Advanced Features (Week 5)

**Goal:** Loop control flow and result aggregation

**Tasks:**
1. Implement break_condition evaluation in loops
2. Implement continue_condition evaluation in loops
3. Add result collection and aggregation
4. Store loop results in state (loop_results field)
5. Support nested loop contexts (parent_context in LoopContext)
6. Add loop progress reporting (get_loop_progress)
7. Update CLI to display loop progress
8. Create advanced loop examples

**Deliverables:**
- ✅ Break/continue conditions work
- ✅ Result aggregation works
- ✅ Nested loops supported
- ✅ CLI shows loop progress
- ✅ Advanced examples

**Files Created:**
- `examples/nested_loops.rs` (+150 lines)
- `examples/loop_control.rs` (+120 lines)

**Files Modified:**
- `src/dsl/executor.rs` (+80 lines)
- `src/dsl/state.rs` (+40 lines)
- `src/dsl/loop_context.rs` (+30 lines)
- CLI progress display (+50 lines)

---

### Phase 6: Persistence & Resume (Week 6)

**Goal:** Enable checkpoint/resume for loops

**Tasks:**
1. Extend state persistence for loop states (loop_states field in JSON)
2. Implement loop resume logic in executor
3. Skip completed iterations on resume
4. Test resume with partial loop completion
5. Handle loop state migration (version compatibility)
6. Document resume behavior
7. Create resume examples

**Deliverables:**
- ✅ Loops can be checkpointed
- ✅ Loops resume from interruption
- ✅ State migration tested
- ✅ Resume documentation
- ✅ Resume examples

**Files Created:**
- `examples/resumable_loop.rs` (+100 lines)
- `docs/loop-resume.md` (+80 lines)

**Files Modified:**
- `src/dsl/executor.rs` (+60 lines)
- `src/dsl/state.rs` (+40 lines)
- `tests/integration/resume_tests.rs` (+150 lines)

---

### Phase 7: Extended Data Sources - HTTP/HTTPS (Completed)

**Goal:** Support HTTP/HTTPS API endpoints as collection sources

**Tasks:**
1. ✅ Add HTTP collection source to schema
2. ✅ Add reqwest dependency for HTTP requests
3. ✅ Implement HTTP data fetching in executor
4. ✅ Add JSON path extraction support
5. ✅ Create HTTP collection examples
6. ✅ Add integration tests for HTTP sources
7. ✅ Document extended data sources

**Implementation Details:**

**HTTP Collection Source:**
- Supports GET, POST, PUT, DELETE, PATCH methods
- Configurable headers and request body
- Multiple response formats: JSON, JSON Lines, CSV, Lines
- JSON path extraction for nested data (e.g., "data.items")
- Proper error handling for HTTP failures

**YAML Syntax:**
```yaml
loop:
  type: for_each
  collection:
    source: http
    url: "https://api.example.com/data"
    method: "GET"  # default: GET
    headers:  # optional
      Authorization: "Bearer token"
    body: '{"query": "value"}'  # optional
    format: json  # default: json
    json_path: "data.items"  # optional path extraction
  iterator: "item"
```

**Deliverables:**
- ✅ HTTP collection source fully implemented
- ✅ JSON path extraction helper function
- ✅ Validation for HTTP URLs and methods
- ✅ Example workflow using JSONPlaceholder API
- ✅ 5 new integration tests (30 total tests passing)
- ✅ reqwest dependency added (v0.12)

**Files Created:**
- `examples/workflows/http_collection_demo.yaml` (+55 lines)
- `examples/http_collection_demo.rs` (+85 lines)

**Files Modified:**
- `src/dsl/schema.rs` (+30 lines) - Added Http variant to CollectionSource
- `src/dsl/executor.rs` (+170 lines) - HTTP fetching and JSON path extraction
- `src/dsl/validator.rs` (+30 lines) - HTTP source validation
- `tests/loop_tests.rs` (+250 lines) - 5 new HTTP collection tests
- `Cargo.toml` (+2 lines) - Added reqwest and http_collection_demo example

**Test Results:**
- All 30 loop tests passing (25 existing + 5 new HTTP tests)
- Parsing tests for HTTP collection with headers, JSON path
- Validation tests for invalid URLs and HTTP methods

---

### Phase 8: Polish & Documentation (Week 8)

**Goal:** Production-ready release

**Tasks:**
1. Comprehensive error messages for loops
2. Add loop validation warnings (infinite loop detection)
3. Write loop design guide (`docs/loop-patterns.md`)
4. Create loop pattern cookbook with real-world examples
5. Add loop tutorial examples
6. Performance optimization pass (profiling)
7. Security audit (loop bombs, resource limits)
8. Final integration tests
9. Update main documentation

**Deliverables:**
- ✅ Complete documentation
- ✅ Pattern cookbook (10+ real-world examples)
- ✅ Tutorial examples with explanations
- ✅ Production-ready code
- ✅ Security audit passed
- ✅ 90%+ test coverage

**Files Created:**
- `docs/loop-patterns.md` (+300 lines)
- `docs/loop-cookbook.md` (+400 lines)
- `docs/loop-tutorial.md` (+250 lines)
- `tests/integration/loop_security_tests.rs` (+150 lines)

---

## Testing Strategy

### Unit Tests

**Core Functionality:**

```rust
#[cfg(test)]
mod loop_tests {
    #[test]
    fn test_foreach_sequential() {
        // Test basic ForEach with 3 items
    }

    #[test]
    fn test_foreach_parallel() {
        // Test parallel execution with max_parallel
    }

    #[test]
    fn test_while_loop_termination() {
        // Test While loop exits on condition
    }

    #[test]
    fn test_repeat_until_max_iterations() {
        // Test safety limit enforcement
    }

    #[test]
    fn test_variable_substitution() {
        // Test {{iterator}} replacement
    }

    #[test]
    fn test_loop_state_persistence() {
        // Test checkpoint/resume
    }

    #[test]
    fn test_nested_loops() {
        // Test loop context inheritance
    }

    #[test]
    fn test_break_condition() {
        // Test early termination
    }

    #[test]
    fn test_collection_sources() {
        // Test all collection source types
    }

    #[test]
    fn test_loop_validation() {
        // Test validation rules
    }
}
```

### Integration Tests

**End-to-End Workflows:**

```rust
#[tokio::test]
async fn test_file_processing_workflow() {
    // End-to-end ForEach file processing
    // - Create test files
    // - Run ForEach loop
    // - Verify all files processed
    // - Check result aggregation
}

#[tokio::test]
async fn test_api_polling_workflow() {
    // End-to-end RepeatUntil with state updates
    // - Mock API endpoint
    // - Poll until ready
    // - Verify condition-based exit
}

#[tokio::test]
async fn test_resume_interrupted_loop() {
    // Interrupt loop mid-execution, resume
    // - Start ForEach loop
    // - Checkpoint after 5/10 iterations
    // - Simulate crash
    // - Resume and verify completion
}

#[tokio::test]
async fn test_loop_with_error_handling() {
    // Loop with retry and fallback
    // - ForEach with break_on_error: false
    // - Some iterations fail
    // - Verify error aggregation
    // - Check fallback behavior
}

#[tokio::test]
async fn test_nested_loop_execution() {
    // Nested ForEach loops
    // - Outer loop: 3 items
    // - Inner loop: 5 items per outer
    // - Verify variable scoping
    // - Check total iterations (15)
}

#[tokio::test]
async fn test_parallel_loop_performance() {
    // Performance validation
    // - 1000 iterations parallel
    // - max_parallel: 100
    // - Target: < 2 seconds
}
```

### Performance Tests

**Benchmarks (`benches/loop_performance.rs`):**

```rust
#[bench]
fn bench_foreach_sequential_1000(b: &mut Bencher) {
    // Target: < 10 seconds for 1000 iterations
}

#[bench]
fn bench_foreach_parallel_1000(b: &mut Bencher) {
    // Target: < 2 seconds for 1000 iterations (max_parallel: 100)
}

#[bench]
fn bench_nested_loops_10x10(b: &mut Bencher) {
    // Target: < 5 seconds for 100 total iterations
}

#[bench]
fn bench_state_checkpoint_large_loop(b: &mut Bencher) {
    // Target: < 100ms checkpoint with 1000 iterations
}

#[bench]
fn bench_variable_substitution(b: &mut Bencher) {
    // Target: < 1ms per substitution
}
```

### Test Coverage Goals

- **Unit Tests:** 90%+ coverage for loop-related code
- **Integration Tests:** All loop patterns covered
- **Performance Tests:** Meet all benchmarks
- **Security Tests:** Resource limits enforced

---

## Security & Safety

### Resource Limits

**Constants:**

```rust
// Maximum values to prevent resource exhaustion
const MAX_LOOP_ITERATIONS: usize = 10_000;
const MAX_COLLECTION_SIZE: usize = 100_000;
const MAX_NESTED_DEPTH: usize = 5;
const MAX_PARALLEL_ITERATIONS: usize = 100;
const MAX_ITERATION_TIMEOUT_SECS: u64 = 300;  // 5 minutes per iteration
```

### Validation Rules

**Safety Checks:**

1. **Iteration Limits**
   - Enforce `max_iterations` for all loop types
   - Default to `MAX_LOOP_ITERATIONS` if not specified
   - Reject loops exceeding limits at validation time

2. **Collection Size**
   - Validate collection size before expansion
   - Reject collections > `MAX_COLLECTION_SIZE`
   - Warn on large collections (> 1000 items)

3. **Infinite Loop Detection**
   - Warn on While loops without max_iterations
   - Detect static conditions (always true)
   - Require break_condition or timeout

4. **Nested Loop Depth**
   - Track nesting level in LoopContext
   - Reject nesting > `MAX_NESTED_DEPTH`
   - Prevent recursive loop definitions

5. **Parallel Execution**
   - Enforce `max_parallel` limits
   - Default to reasonable concurrency (10)
   - Prevent fork bombs

6. **Timeout Protection**
   - Per-iteration timeout
   - Total loop timeout (iterations × per-iteration)
   - Graceful timeout handling

### Error Handling

**Robust Error Management:**

1. **Iteration Errors**
   - Capture per-iteration errors
   - Aggregate errors in parallel loops
   - Provide iteration context in error messages

2. **Timeout Handling**
   - Graceful degradation on timeout
   - Mark timed-out iterations as Failed
   - Continue or abort based on `break_on_error`

3. **State Recovery**
   - Checkpoint after each iteration (configurable)
   - Resume from last checkpoint
   - Preserve error history

4. **Clear Error Messages**
   - Include iteration number
   - Show iterator value if available
   - Provide loop context (nested path)

### Security Audit Checklist

- [ ] Resource limits enforced at runtime
- [ ] Validation prevents malicious loop definitions
- [ ] File access for collections properly sandboxed
- [ ] No arbitrary code execution via variable substitution
- [ ] State files protected (permissions)
- [ ] Denial of service prevention (fork bombs, memory exhaustion)
- [ ] Input validation for all collection sources
- [ ] Proper error handling prevents information leakage

---

## Success Criteria

### Functional Requirements

- [ ] ForEach loops execute sequentially
- [ ] ForEach loops execute in parallel with limits
- [ ] While loops terminate on condition
- [ ] RepeatUntil loops respect min/max iterations
- [ ] Repeat loops execute N times
- [ ] Variable substitution works in all contexts (description, conditions, DoD)
- [ ] Break/continue conditions work
- [ ] Results can be aggregated
- [ ] Loops can be checkpointed and resumed
- [ ] All collection sources supported (state, file, range, inline)
- [ ] Nested loops work with proper variable scoping

### Non-Functional Requirements

- [ ] **Performance:** 1000 iterations < 10s (sequential)
- [ ] **Performance:** 1000 iterations < 2s (parallel, max_parallel=100)
- [ ] **Memory:** Constant overhead per iteration (< 1KB)
- [ ] **Safety:** Resource limits enforced
- [ ] **Safety:** No infinite loops possible
- [ ] **Compatibility:** All existing workflows still work
- [ ] **Compatibility:** Backward-compatible state format (with migration)

### Quality Requirements

- [ ] 90%+ test coverage for loop code
- [ ] Comprehensive documentation (design, patterns, cookbook, tutorial)
- [ ] 10+ example workflows covering all patterns
- [ ] Zero loop-related security vulnerabilities
- [ ] Clear error messages with context
- [ ] Performance benchmarks pass
- [ ] Integration tests cover all loop types
- [ ] Resume functionality tested

### Documentation Requirements

- [ ] `docs/loop-patterns.md` - Complete design guide
- [ ] `docs/loop-cookbook.md` - Real-world examples
- [ ] `docs/loop-tutorial.md` - Step-by-step tutorial
- [ ] `docs/loop-resume.md` - Checkpoint/resume guide
- [ ] API documentation for all new types
- [ ] Migration guide for existing workflows
- [ ] Security best practices documented

---

## Examples

### Example 1: Batch File Processing

```yaml
workflows:
  file_processor:
    tasks:
      scan_files:
        description: "Scan directory for files"
        agent: scanner
        definition_of_done:
          criteria:
            - type: state_exists
              key: file_list
              description: "File list stored in state"

      process_each:
        description: "Process file {{file}}"
        agent: processor
        depends_on: [scan_files]
        loop:
          type: for_each
          collection:
            source: state
            key: file_list
          iterator: file
          parallel: true
          max_parallel: 5
        loop_control:
          collect_results: true
          result_key: processed_files
          break_on_error: false
        definition_of_done:
          criteria:
            - type: file_exists
              path: "output/{{file}}"
              description: "Output file created"
```

### Example 2: API Polling

```yaml
workflows:
  api_poller:
    tasks:
      start_job:
        description: "Start background job"
        agent: initiator
        definition_of_done:
          criteria:
            - type: state_exists
              key: job_id

      poll_status:
        description: "Poll API until job complete (attempt {{poll_count}})"
        agent: poller
        depends_on: [start_job]
        loop:
          type: repeat_until
          condition:
            type: state_equals
            key: job_status
            value: "completed"
          min_iterations: 1
          max_iterations: 30
          iteration_variable: poll_count
          delay_between_secs: 10
        loop_control:
          break_condition:
            or:
              - type: state_equals
                key: job_status
                value: "failed"
              - type: state_equals
                key: job_status
                value: "completed"
        on_error:
          retry: 3
          retry_delay_secs: 5
```

### Example 3: Incremental Retry with Backoff

```yaml
workflows:
  resilient_task:
    tasks:
      retry_with_backoff:
        description: "Attempt {{attempt}} of risky operation"
        agent: worker
        loop:
          type: while
          condition:
            type: state_equals
            key: success
            value: false
          max_iterations: 5
          iteration_variable: attempt
          delay_between_secs: 2  # will be exponential with on_error
        loop_control:
          break_condition:
            type: state_equals
            key: success
            value: true
        on_error:
          retry: 1
          exponential_backoff: true
```

### Example 4: Nested Loops (Matrix Processing)

```yaml
workflows:
  matrix_processor:
    tasks:
      process_matrix:
        description: "Process row {{row}}"
        agent: row_processor
        loop:
          type: for_each
          collection:
            source: state
            key: rows
          iterator: row
          parallel: false

        subtasks:
          process_cell:
            description: "Process cell {{cell}} in row {{row}}"
            agent: cell_processor
            loop:
              type: for_each
              collection:
                source: state
                key: "row_{{row}}_cells"
              iterator: cell
              parallel: true
              max_parallel: 3
            definition_of_done:
              criteria:
                - type: state_exists
                  key: "result_{{row}}_{{cell}}"
```

### Example 5: Range-Based Generation

```yaml
workflows:
  report_generator:
    tasks:
      generate_reports:
        description: "Generate report {{report_id}}"
        agent: generator
        loop:
          type: for_each
          collection:
            source: range
            start: 1
            end: 100
            step: 1
          iterator: report_id
          parallel: true
          max_parallel: 10
        definition_of_done:
          criteria:
            - type: file_exists
              path: "reports/report_{{report_id}}.pdf"
```

---

## File Structure Summary

### New Files

```
src/dsl/loop_context.rs          ~150 lines
src/dsl/collection_sources.rs    ~200 lines
examples/foreach_demo.rs         ~100 lines
examples/repeat_demo.rs          ~80 lines
examples/while_demo.rs           ~100 lines
examples/polling_demo.rs         ~120 lines
examples/parallel_processing.rs  ~150 lines
examples/nested_loops.rs         ~150 lines
examples/loop_control.rs         ~120 lines
examples/resumable_loop.rs       ~100 lines
examples/file_collection.rs      ~120 lines
examples/range_loop.rs           ~80 lines
docs/loop-patterns.md            ~300 lines
docs/loop-cookbook.md            ~400 lines
docs/loop-tutorial.md            ~250 lines
docs/loop-resume.md              ~80 lines
benches/loop_performance.rs      ~200 lines
tests/integration/loop_tests.rs  ~300 lines
tests/integration/resume_tests.rs ~150 lines
```

### Modified Files

```
src/dsl/schema.rs          +200 lines
src/dsl/executor.rs        +690 lines (phases 2-5)
src/dsl/state.rs           +220 lines
src/dsl/task_graph.rs      +50 lines
src/dsl/validator.rs       +100 lines
src/dsl/parser.rs          +50 lines
```

**Estimated Total:** ~3,910 lines (implementation + tests + docs + examples)

---

## Next Steps

1. **Review this plan** with stakeholders
2. **Refine Phase 1 tasks** and create detailed tickets
3. **Set up development branch** (`feature/iterative-patterns`)
4. **Begin Phase 1 implementation**
5. **Establish testing framework** for loops
6. **Create tracking dashboard** for implementation progress

---

## Appendix: Design Decisions

### Why Dynamic Loop Handling?

**Decision:** Keep loop task as single node in graph, handle iterations in executor.

**Rationale:**
- Supports all loop types (including dynamic While/RepeatUntil)
- Simpler graph representation
- Easier state management
- Better performance for large loops
- Clearer dependency semantics

**Alternatives Considered:**
- Virtual task expansion: Complex for dynamic loops, graph explosion
- Hybrid approach: Added complexity without clear benefits

### Why Not Traditional Break/Continue Keywords?

**Decision:** Use `break_condition` and `continue_condition` in `loop_control`.

**Rationale:**
- Declarative DSL (YAML-based)
- No procedural control flow keywords
- Conditions evaluated by existing system
- More explicit and auditable
- Better for state-based workflows

### Why Multiple Loop Types?

**Decision:** Four distinct loop types (ForEach, While, RepeatUntil, Repeat).

**Rationale:**
- Different use cases have different semantics
- Clearer intent in workflow definitions
- Optimization opportunities (e.g., known total for Repeat)
- Better validation and error messages
- Familiar patterns from traditional programming

---

**End of Implementation Plan**
