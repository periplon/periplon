# Task Groups Documentation

## Table of Contents

1. [Overview](#overview)
2. [Core Concepts](#core-concepts)
3. [Configuration Reference](#configuration-reference)
4. [Execution Modes](#execution-modes)
5. [Dependencies and Ordering](#dependencies-and-ordering)
6. [Hierarchical Groups](#hierarchical-groups)
7. [Variable System](#variable-system)
8. [Error Handling](#error-handling)
9. [Advanced Patterns](#advanced-patterns)
10. [Examples](#examples)
11. [Best Practices](#best-practices)
12. [Troubleshooting](#troubleshooting)

## Overview

Task groups are a powerful feature of the DSL workflow system that allow you to organize, coordinate, and manage collections of related tasks with fine-grained control over execution, dependencies, and data flow.

### What Are Task Groups?

A task group is a logical collection of tasks (or other groups) that:
- Execute together as a coordinated unit
- Share common configuration (timeouts, error handling, permissions)
- Can run sequentially or in parallel
- Can depend on other groups
- Can be nested hierarchically
- Can share context through variables

### Why Use Task Groups?

**Organization**: Group related tasks together for clarity and maintainability

**Coordination**: Control execution order and parallelism across multiple tasks

**Error Handling**: Apply consistent error handling strategies to related tasks

**Reusability**: Define reusable workflow patterns that can be composed

**Performance**: Optimize execution through parallel task execution

**Visibility**: Track progress at the group level, not just individual tasks

### Key Benefits

- **Simplified Workflow Definition**: Express complex workflows concisely
- **Better Resource Management**: Control concurrency and resource usage
- **Improved Debugging**: Track failures at the group level
- **Enhanced Collaboration**: Multiple agents can work in coordinated groups
- **Flexible Composition**: Build complex pipelines from simple building blocks

## Core Concepts

### Task Group Anatomy

```yaml
task_groups:
  group_id:                          # Unique identifier
    description: "What this group does"
    execution_mode: "sequential"     # How tasks execute
    tasks: [task1, task2]            # Tasks in this group
    depends_on: [other_group]        # Group dependencies
    on_error: "stop"                 # Error handling strategy
    timeout: 300                     # Group timeout in seconds
    max_concurrency: 3               # Max parallel tasks

    # Advanced features
    parent: "parent_group"           # Hierarchical nesting
    groups: [child_group]            # Child groups
    condition: "${expr}"             # Conditional execution

    # Variables
    inputs:
      var: "${source}"               # Input variables
    outputs:
      result: "${source}"            # Output variables
```

### Group Lifecycle

1. **Validation**: Group structure, dependencies, and references are validated
2. **Scheduling**: Dependencies are resolved and execution order determined
3. **Initialization**: Input variables are resolved and group context is set up
4. **Execution**: Tasks/child groups execute according to execution mode
5. **Completion**: Output variables are captured and status is recorded
6. **Cleanup**: Resources are released and final state is persisted

### Group States

- **Pending**: Group is waiting for dependencies
- **Ready**: Dependencies satisfied, ready to execute
- **Running**: Group is actively executing
- **Completed**: All tasks completed successfully
- **Failed**: One or more tasks failed
- **Skipped**: Condition evaluated to false
- **Timeout**: Group exceeded timeout limit

## Configuration Reference

### Basic Properties

#### `description` (string, required)
Human-readable description of what the group does.

```yaml
description: "Build and test the application"
```

#### `execution_mode` (string, required)
How tasks within the group execute.

**Options:**
- `sequential`: Tasks run one after another
- `parallel`: Tasks run concurrently
- `auto`: Auto-determine based on dependencies

```yaml
execution_mode: "parallel"
```

#### `tasks` (array, optional)
List of task IDs that belong to this group.

```yaml
tasks:
  - analyze_code
  - check_tests
  - generate_docs
```

#### `groups` (array, optional)
List of child group IDs (for hierarchical groups).

```yaml
groups:
  - build_phase
  - test_phase
```

### Dependency Management

#### `depends_on` (array, optional)
Groups that must complete before this group starts.

```yaml
depends_on:
  - data_preparation
  - environment_setup
```

**Behavior:**
- All dependencies must complete successfully
- Dependencies are resolved in topological order
- Circular dependencies are detected and rejected

#### `condition` (string, optional)
Boolean expression that must evaluate to true for group to execute.

```yaml
condition: "${group.quality_check.score} >= 0.8"
```

**Use Cases:**
- Quality gates
- Environment-specific execution
- Feature flags
- Resource availability checks

### Error Handling

#### `on_error` (string, optional, default: "stop")
Strategy for handling task failures within the group.

**Options:**

**`stop`**: Stop group execution on first task failure
```yaml
on_error: "stop"
```
- Fail fast approach
- Prevents wasted work
- Good for critical pipelines

**`continue`**: Continue executing remaining tasks despite failures
```yaml
on_error: "continue"
```
- Collect all failures
- Useful for test suites
- Generate complete error reports

**`rollback`**: Execute rollback logic on failure (custom handler)
```yaml
on_error: "rollback"
```
- Attempt to undo changes
- Good for deployments
- Requires rollback task definition

#### `timeout` (integer, optional)
Maximum execution time in seconds for entire group.

```yaml
timeout: 600  # 10 minutes
```

**Behavior:**
- Applies to entire group execution
- Includes all child tasks/groups
- On timeout, group fails and running tasks are cancelled
- Inherits from parent if not specified

### Concurrency Control

#### `max_concurrency` (integer, optional)
Maximum number of tasks that can run in parallel.

```yaml
max_concurrency: 5
```

**Applies to:** Parallel execution mode only

**Use Cases:**
- Prevent resource exhaustion
- Rate limiting external API calls
- Control memory usage
- Respect system limits

**Default:** Unlimited (all tasks run concurrently)

### Hierarchical Structure

#### `parent` (string, optional)
ID of parent group (for nested groups).

```yaml
parent: "build_pipeline"
```

**Rules:**
- Creates parent-child relationship
- Child inherits parent's configuration
- Child cannot start until parent is running
- Parent completes when all children complete

### Variable System

#### `inputs` (object, optional)
Variables provided to the group from external sources.

```yaml
inputs:
  config_file: "${workflow.config_path}"
  previous_result: "${group.previous.output}"
```

**Sources:**
- Workflow-level inputs
- Other group outputs
- Task outputs
- State variables

#### `outputs` (object, optional)
Variables produced by the group for downstream consumption.

```yaml
outputs:
  build_artifact:
    source:
      type: file
      path: "./dist/app.zip"
  test_results:
    source:
      type: state
      key: "test.results"
```

**Output Types:**
- `file`: Read from file path
- `state`: Read from workflow state
- `task`: Read from specific task output

## Execution Modes

### Sequential Execution

Tasks execute one after another in order.

```yaml
task_groups:
  sequential_pipeline:
    execution_mode: "sequential"
    tasks:
      - step1
      - step2
      - step3
```

**Execution Order:**
1. step1 runs
2. step1 completes
3. step2 runs
4. step2 completes
5. step3 runs
6. step3 completes

**Characteristics:**
- Predictable order
- Each task waits for previous
- Task dependencies are honored
- Lower resource usage
- Longer total execution time

**Use Cases:**
- Database migrations (order matters)
- Build pipelines (each step needs previous output)
- Deployment steps (infrastructure → app → verification)

**Timeline:**
```
Time: 0s    5s    10s   15s
      │─────│─────│─────│
      step1 step2 step3
```

### Parallel Execution

Tasks execute concurrently.

```yaml
task_groups:
  parallel_tests:
    execution_mode: "parallel"
    max_concurrency: 3
    tasks:
      - test_backend
      - test_frontend
      - test_api
      - test_integration
```

**Execution Order:**
1. All tasks start simultaneously (up to max_concurrency)
2. Tasks complete independently
3. Group completes when all tasks finish

**Characteristics:**
- Maximum throughput
- Tasks run independently
- Higher resource usage
- Shorter total execution time
- Non-deterministic completion order

**Use Cases:**
- Independent tests
- Multi-agent analysis
- Parallel data processing
- Document generation

**Timeline (max_concurrency=3):**
```
Time: 0s              10s
      │───────────────│
      test_backend
      test_frontend
      test_api
                 ↓
           test_integration (starts after slot available)
```

### Auto Mode

Execution mode determined automatically based on task dependencies.

```yaml
task_groups:
  auto_mode:
    execution_mode: "auto"
    tasks:
      - task_a
      - task_b  # depends_on: [task_a]
      - task_c  # depends_on: [task_a]
      - task_d  # depends_on: [task_b, task_c]
```

**Behavior:**
- Analyzes task dependencies
- Creates optimal execution plan
- Parallelizes independent tasks
- Respects dependency order

**Execution Order:**
1. task_a runs first (no dependencies)
2. task_b and task_c run in parallel (both depend on task_a)
3. task_d runs last (depends on both task_b and task_c)

**Timeline:**
```
Time: 0s    5s         10s
      │─────│──────────│
      task_a
            ├─ task_b
            └─ task_c
                       └─ task_d
```

## Dependencies and Ordering

### Task-Level Dependencies

Dependencies defined on individual tasks.

```yaml
tasks:
  compile:
    description: "Compile code"
    group: "build"

  test:
    description: "Run tests"
    group: "build"
    depends_on: [compile]  # Task-level dependency
```

**Behavior:**
- Task waits for dependencies within group
- Works across execution modes
- Fine-grained control

### Group-Level Dependencies

Dependencies between entire groups.

```yaml
task_groups:
  build:
    tasks: [compile, link]

  test:
    depends_on: [build]  # Group-level dependency
    tasks: [unit_test, integration_test]

  deploy:
    depends_on: [test, security_scan]  # Multiple dependencies
    tasks: [deploy_staging]
```

**Behavior:**
- Entire group waits for all dependencies
- Provides coarse-grained orchestration
- Simplifies complex workflows

**Execution Order:**
1. `build` runs first (no dependencies)
2. `test` and `security_scan` can run in parallel (both depend only on `build`)
3. `deploy` runs last (depends on both `test` and `security_scan`)

### Dependency Resolution

The executor uses topological sort to determine execution order:

1. Build dependency graph
2. Detect cycles (error if found)
3. Sort groups topologically
4. Execute in resolved order

**Example Dependency Graph:**
```
    A
   ↙ ↘
  B   C
   ↘ ↙
    D
```

**Resolution:** A → (B, C in parallel) → D

**Cycle Detection:**
```
A → B → C → A  ❌ Error: circular dependency
```

### Conditional Dependencies

Dependencies can be conditional based on results.

```yaml
task_groups:
  quality_check:
    tasks: [analyze]
    outputs:
      pass: "${state.quality.pass}"

  deployment:
    depends_on: [quality_check]
    condition: "${group.quality_check.pass} == true"
    tasks: [deploy]
```

## Hierarchical Groups

### Parent-Child Relationships

Groups can contain other groups, creating a hierarchy.

```yaml
task_groups:
  # Root group
  full_pipeline:
    description: "Complete pipeline"
    groups:
      - phase1
      - phase2

  # Child groups
  phase1:
    parent: "full_pipeline"
    tasks: [task1, task2]

  phase2:
    parent: "full_pipeline"
    depends_on: [phase1]
    tasks: [task3, task4]
```

**Structure:**
```
full_pipeline
├── phase1
│   ├── task1
│   └── task2
└── phase2
    ├── task3
    └── task4
```

### Multi-Level Nesting

Groups can be nested multiple levels deep.

```yaml
task_groups:
  # Level 0: Root
  ci_cd:
    groups: [build_stage, test_stage, deploy_stage]

  # Level 1: Stages
  build_stage:
    parent: "ci_cd"
    groups: [compile, package]

  test_stage:
    parent: "ci_cd"
    depends_on: [build_stage]
    groups: [unit_tests, integration_tests]

  deploy_stage:
    parent: "ci_cd"
    depends_on: [test_stage]
    tasks: [deploy]

  # Level 2: Sub-stages
  compile:
    parent: "build_stage"
    tasks: [compile_backend, compile_frontend]

  package:
    parent: "build_stage"
    depends_on: [compile]
    tasks: [create_artifacts]

  unit_tests:
    parent: "test_stage"
    tasks: [test_unit]

  integration_tests:
    parent: "test_stage"
    depends_on: [unit_tests]
    tasks: [test_integration]
```

**Structure:**
```
ci_cd
├── build_stage
│   ├── compile
│   │   ├── compile_backend
│   │   └── compile_frontend
│   └── package
│       └── create_artifacts
├── test_stage
│   ├── unit_tests
│   │   └── test_unit
│   └── integration_tests
│       └── test_integration
└── deploy_stage
    └── deploy
```

### Configuration Inheritance

Child groups inherit configuration from parents.

```yaml
task_groups:
  parent:
    timeout: 600
    on_error: "stop"
    groups: [child]

  child:
    parent: "parent"
    # Inherits timeout: 600 and on_error: "stop"
    # Can override:
    timeout: 300  # Override parent's timeout
    tasks: [task1]
```

**Inheritance Rules:**
- Child inherits: `timeout`, `on_error`, `max_concurrency`
- Child can override any inherited property
- Explicit child value takes precedence
- Variables do NOT inherit (must be explicitly passed)

### Execution Flow

**Parent Group Lifecycle:**
1. Parent enters "running" state
2. Child groups are scheduled
3. Children execute according to their dependencies
4. Parent waits for all children to complete
5. Parent completes when all children are done

**Example:**
```yaml
task_groups:
  parent:
    execution_mode: "sequential"
    groups: [child1, child2]

  child1:
    parent: "parent"
    tasks: [task_a, task_b]

  child2:
    parent: "parent"
    depends_on: [child1]
    tasks: [task_c]
```

**Execution Order:**
1. `parent` starts
2. `child1` executes (task_a, task_b)
3. `child1` completes
4. `child2` executes (task_c)
5. `child2` completes
6. `parent` completes

## Variable System

### Variable Scoping

Variables are scoped hierarchically:

**Workflow Level:**
```yaml
inputs:
  project_name:
    type: string
    required: true

# Access: ${workflow.project_name}
```

**Group Level:**
```yaml
task_groups:
  my_group:
    outputs:
      result: "${state.my_result}"

# Access: ${group.my_group.result}
```

**Task Level:**
```yaml
tasks:
  my_task:
    outputs:
      output: "${state.task_output}"

# Access: ${task.my_task.output}
```

### Variable Interpolation

Variables are interpolated using `${scope.variable}` syntax.

**Explicit Scope:**
```yaml
description: "Process ${workflow.project_name}"
inputs:
  data: "${group.previous.output}"
  config: "${task.setup.config_path}"
```

**Implicit Scope:**
```yaml
description: "Process ${project_name}"  # Searches upward
```

**Resolution Order:**
1. Task scope (current task's variables)
2. Group scope (current group's variables)
3. Workflow scope (workflow inputs)
4. Error if not found

### Input Variables

Variables consumed by a group.

```yaml
task_groups:
  processor:
    inputs:
      source_file:
        type: string
        required: true
        default: "./data.json"
      threshold:
        type: number
        required: false
        default: 0.8
```

**Input Sources:**
- Workflow inputs: `${workflow.var}`
- Group outputs: `${group.group_id.var}`
- Task outputs: `${task.task_id.var}`
- Literal values: `"literal string"` or `42`

**Type Validation:**
```yaml
inputs:
  count:
    type: number      # Types: string, number, boolean, object, array
    required: true
    default: 10
    description: "Number of items to process"
```

### Output Variables

Variables produced by a group.

```yaml
task_groups:
  builder:
    outputs:
      artifact_path:
        source:
          type: file
          path: "./dist/app.zip"
      build_number:
        source:
          type: state
          key: "build.number"
      test_status:
        source:
          type: task
          task_id: "run_tests"
          variable: "status"
```

**Output Source Types:**

**File Source:**
```yaml
source:
  type: file
  path: "./output.json"  # Reads file contents as variable value
```

**State Source:**
```yaml
source:
  type: state
  key: "namespace.key"  # Reads from workflow state
```

**Task Source:**
```yaml
source:
  type: task
  task_id: "my_task"
  variable: "result"  # Reads from specific task output
```

### Variable Propagation

Variables flow through the workflow:

```yaml
task_groups:
  # Group 1: Produces data
  analysis:
    tasks: [analyze]
    outputs:
      result:
        source:
          type: state
          key: "analysis.result"

  # Group 2: Consumes from Group 1
  processing:
    depends_on: [analysis]
    inputs:
      data: "${group.analysis.result}"  # Consume Group 1 output
    tasks: [process]
    outputs:
      processed:
        source:
          type: state
          key: "processed.data"

  # Group 3: Consumes from Group 2
  reporting:
    depends_on: [processing]
    inputs:
      input_data: "${group.processing.processed}"  # Consume Group 2 output
    tasks: [report]
```

**Data Flow:**
```
analysis → [result] → processing → [processed] → reporting
```

### Advanced Variable Usage

**Conditional Execution:**
```yaml
condition: "${group.quality_check.score} >= ${workflow.threshold}"
```

**Dynamic Paths:**
```yaml
inputs:
  output_dir: "./results/${workflow.environment}/${workflow.timestamp}"
```

**Arithmetic (if supported):**
```yaml
condition: "${group.metrics.cpu_usage} * 100 > 80"
```

**String Concatenation:**
```yaml
description: "Deploy ${workflow.service_name} to ${workflow.environment}"
```

## Error Handling

### Error Handling Strategies

#### Stop on Error

Immediately stop group execution when a task fails.

```yaml
task_groups:
  critical_pipeline:
    on_error: "stop"
    tasks: [step1, step2, step3]
```

**Behavior:**
- First task failure stops the group
- Remaining tasks are cancelled
- Group status set to "failed"
- Error propagates to parent group

**Use Cases:**
- Critical pipelines where failures are unacceptable
- Dependencies between tasks (no point continuing)
- Fast failure detection

**Example:**
```
step1 ✓ → step2 ✗ → [stop] → step3 cancelled
Group status: Failed
```

#### Continue on Error

Continue executing remaining tasks despite failures.

```yaml
task_groups:
  test_suite:
    on_error: "continue"
    tasks: [test1, test2, test3, test4]
```

**Behavior:**
- Failed tasks don't stop execution
- All tasks attempt to run
- Group collects all failures
- Group status "failed" if any task failed

**Use Cases:**
- Test suites (want all test results)
- Independent validation checks
- Comprehensive error reporting

**Example:**
```
test1 ✓ → test2 ✗ → test3 ✓ → test4 ✗
All tests attempted
Group status: Failed (2/4 tasks failed)
```

#### Rollback on Error

Execute rollback logic when a task fails.

```yaml
task_groups:
  deployment:
    on_error: "rollback"
    tasks: [backup, deploy, verify]
```

**Behavior:**
- On failure, executes rollback tasks
- Attempts to restore previous state
- Rollback tasks defined separately
- Group status "failed" with rollback attempted

**Use Cases:**
- Deployments (restore previous version)
- Database migrations (rollback schema changes)
- Configuration updates (restore old config)

**Rollback Task Pattern:**
```yaml
tasks:
  backup:
    description: "Backup current state"
    outputs:
      backup_id: "${state.backup.id}"

  deploy:
    description: "Deploy new version"
    depends_on: [backup]

  rollback_deploy:
    description: "Rollback to ${task.backup.backup_id}"
    # Triggered on deployment failure
```

### Timeout Handling

Groups can have execution timeouts.

```yaml
task_groups:
  long_running:
    timeout: 3600  # 1 hour in seconds
    tasks: [task1, task2]
```

**Behavior:**
- Timer starts when group begins execution
- Includes all child tasks/groups
- On timeout:
  - Running tasks are cancelled
  - Group marked as "timeout"
  - Error propagates to parent

**Timeout Inheritance:**
```yaml
task_groups:
  parent:
    timeout: 600  # 10 minutes
    groups: [child]

  child:
    parent: "parent"
    # Inherits 600s timeout unless overridden
    timeout: 300  # Override to 5 minutes
```

### Error Propagation

Errors propagate up the group hierarchy.

```yaml
task_groups:
  root:
    on_error: "stop"
    groups: [child1]

  child1:
    parent: "root"
    on_error: "continue"
    tasks: [task_a, task_b]
```

**Scenario: task_a fails**
1. task_a fails
2. child1 continues (on_error: continue)
3. task_b executes
4. child1 completes with "failed" status
5. root sees child1 failed
6. root stops (on_error: stop)
7. root marked as "failed"

### Partial Failures

Handle scenarios where some tasks fail but others succeed.

```yaml
task_groups:
  parallel_processing:
    execution_mode: "parallel"
    on_error: "continue"
    tasks: [process_a, process_b, process_c]
    outputs:
      success_count:
        source:
          type: state
          key: "stats.success_count"

  conditional_next:
    depends_on: [parallel_processing]
    condition: "${group.parallel_processing.success_count} >= 2"
    tasks: [proceed]
```

**Behavior:**
- Collect all results (successes and failures)
- Make decision based on success rate
- Continue workflow conditionally

## Advanced Patterns

### Multi-Stage Pipeline

Organize complex workflows into stages.

```yaml
task_groups:
  pipeline:
    groups: [stage1, stage2, stage3, stage4]

  stage1:
    parent: "pipeline"
    description: "Source preparation"
    tasks: [checkout, fetch_deps]

  stage2:
    parent: "pipeline"
    depends_on: [stage1]
    description: "Build"
    tasks: [compile, test]

  stage3:
    parent: "pipeline"
    depends_on: [stage2]
    description: "Package"
    tasks: [create_artifacts]

  stage4:
    parent: "pipeline"
    depends_on: [stage3]
    description: "Deploy"
    tasks: [deploy_staging]
```

### Fan-Out / Fan-In

Parallel execution followed by aggregation.

```yaml
task_groups:
  # Fan-out: Multiple parallel tasks
  parallel_analysis:
    execution_mode: "parallel"
    tasks:
      - analyze_security
      - analyze_performance
      - analyze_quality
      - analyze_dependencies
    outputs:
      security_report: "${task.analyze_security.report}"
      perf_report: "${task.analyze_performance.report}"
      quality_report: "${task.analyze_quality.report}"
      deps_report: "${task.analyze_dependencies.report}"

  # Fan-in: Aggregate results
  aggregate_results:
    depends_on: [parallel_analysis]
    tasks: [merge_reports]
    inputs:
      security: "${group.parallel_analysis.security_report}"
      perf: "${group.parallel_analysis.perf_report}"
      quality: "${group.parallel_analysis.quality_report}"
      deps: "${group.parallel_analysis.deps_report}"
```

### Quality Gates

Conditional progression based on quality metrics.

```yaml
task_groups:
  quality_check:
    tasks: [run_linter, run_tests, calculate_coverage]
    outputs:
      coverage:
        source:
          type: state
          key: "test.coverage"
      linter_pass:
        source:
          type: state
          key: "lint.pass"

  deployment:
    depends_on: [quality_check]
    condition: |
      ${group.quality_check.coverage} >= 0.8 &&
      ${group.quality_check.linter_pass} == true
    tasks: [deploy]
```

### Environment-Specific Groups

Different groups for different environments.

```yaml
inputs:
  environment:
    type: string
    required: true

task_groups:
  deploy_dev:
    condition: "${workflow.environment} == 'dev'"
    tasks: [deploy_to_dev]

  deploy_staging:
    condition: "${workflow.environment} == 'staging'"
    tasks: [deploy_to_staging, smoke_test]

  deploy_prod:
    condition: "${workflow.environment} == 'prod'"
    tasks: [backup, deploy_to_prod, smoke_test, monitor]
```

### Blue-Green Deployment

Deployment pattern with rollback capability.

```yaml
task_groups:
  blue_green_deploy:
    groups: [prepare_green, switch_traffic, cleanup_blue]

  prepare_green:
    parent: "blue_green_deploy"
    tasks:
      - deploy_green
      - test_green
    outputs:
      green_healthy:
        source:
          type: state
          key: "green.healthy"

  switch_traffic:
    parent: "blue_green_deploy"
    depends_on: [prepare_green]
    condition: "${group.prepare_green.green_healthy} == true"
    tasks:
      - update_load_balancer
      - verify_traffic

  cleanup_blue:
    parent: "blue_green_deploy"
    depends_on: [switch_traffic]
    tasks:
      - decommission_blue
```

### Canary Deployment

Progressive rollout with monitoring.

```yaml
task_groups:
  canary_deploy:
    groups: [deploy_canary, monitor_canary, full_rollout]

  deploy_canary:
    parent: "canary_deploy"
    tasks:
      - deploy_to_canary_servers
      - route_traffic_10_percent

  monitor_canary:
    parent: "canary_deploy"
    depends_on: [deploy_canary]
    tasks:
      - monitor_error_rate
      - monitor_latency
    outputs:
      canary_healthy:
        source:
          type: state
          key: "canary.healthy"

  full_rollout:
    parent: "canary_deploy"
    depends_on: [monitor_canary]
    condition: "${group.monitor_canary.canary_healthy} == true"
    tasks:
      - deploy_to_all_servers
      - route_traffic_100_percent
```

### Multi-Expert Collaboration

Multiple specialized agents working together.

```yaml
task_groups:
  expert_review:
    execution_mode: "parallel"
    tasks:
      - architect_review
      - security_review
      - performance_review
    max_concurrency: 3
    outputs:
      arch_score: "${task.architect_review.score}"
      sec_score: "${task.security_review.score}"
      perf_score: "${task.performance_review.score}"

  synthesis:
    depends_on: [expert_review]
    tasks: [synthesize_recommendations]
    inputs:
      arch: "${group.expert_review.arch_score}"
      sec: "${group.expert_review.sec_score}"
      perf: "${group.expert_review.perf_score}"

  implementation:
    depends_on: [synthesis]
    tasks: [implement_changes]
```

### Resource Pooling

Control resource usage with concurrency limits.

```yaml
task_groups:
  # Heavy tasks - limit concurrency
  heavy_processing:
    execution_mode: "parallel"
    max_concurrency: 2  # Only 2 at a time
    tasks:
      - heavy_task_1
      - heavy_task_2
      - heavy_task_3
      - heavy_task_4
      - heavy_task_5

  # Light tasks - no limit
  light_processing:
    execution_mode: "parallel"
    tasks:
      - light_task_1
      - light_task_2
      - light_task_3
```

## Examples

See the [examples/task-groups](../../examples/task-groups/) directory for complete, runnable examples:

- **simple-group.taskgroup.yaml**: Basic sequential and parallel groups
- **advanced-group.taskgroup.yaml**: Hierarchical groups with multi-agent collaboration

Additional examples in [examples/dsl/task_groups](../../examples/dsl/task_groups/):
- **01_basic_groups.yaml**: Sequential, parallel, and mixed modes
- **02_dependency_chains.yaml**: Complex cross-group dependencies
- **03_hierarchical_groups.yaml**: Multi-level group nesting
- **04_variables_and_context.yaml**: Variable interpolation and context sharing
- **05_multi_agent_collaboration.yaml**: Expert agents working together
- **06_real_world_ci_cd.yaml**: Production-ready CI/CD pipeline

## Best Practices

### Organization

**✓ DO:**
- Group related tasks together
- Use meaningful group names
- Add descriptive documentation
- Keep groups focused and cohesive

**✗ DON'T:**
- Create overly large groups
- Mix unrelated tasks
- Use cryptic group names
- Create unnecessary nesting

### Execution Strategy

**✓ DO:**
- Use parallel for independent tasks
- Use sequential for ordered operations
- Set appropriate concurrency limits
- Consider resource constraints

**✗ DON'T:**
- Over-parallelize (resource exhaustion)
- Make everything sequential (performance)
- Ignore task dependencies
- Forget timeout settings

### Error Handling

**✓ DO:**
- Choose appropriate error strategy
- Set reasonable timeouts
- Plan for rollback scenarios
- Log errors comprehensively

**✗ DON'T:**
- Use same strategy everywhere
- Set unrealistic timeouts
- Ignore partial failures
- Swallow errors silently

### Variables

**✓ DO:**
- Use explicit scoping (${workflow.var})
- Validate input types
- Document expected variables
- Provide sensible defaults

**✗ DON'T:**
- Rely on implicit scoping
- Skip type validation
- Use undocumented variables
- Forget required inputs

### Hierarchies

**✓ DO:**
- Keep nesting shallow (2-3 levels)
- Create logical groupings
- Use inheritance wisely
- Document hierarchy structure

**✗ DON'T:**
- Nest too deeply (>4 levels)
- Create circular references
- Override everything
- Confuse hierarchy with dependencies

### Dependencies

**✓ DO:**
- Make dependencies explicit
- Validate dependency graph
- Use group dependencies for coarse control
- Use task dependencies for fine control

**✗ DON'T:**
- Create circular dependencies
- Over-specify dependencies
- Ignore implicit dependencies
- Forget about data dependencies

## Troubleshooting

### Validation Errors

**"Circular dependency detected"**
```
Error: Circular dependency: group_a → group_b → group_c → group_a
```

**Solution:** Review dependencies and break the cycle
```yaml
# Before (circular)
group_a:
  depends_on: [group_c]
group_b:
  depends_on: [group_a]
group_c:
  depends_on: [group_b]

# After (fixed)
group_a: {}
group_b:
  depends_on: [group_a]
group_c:
  depends_on: [group_b]
```

**"Unknown group reference"**
```
Error: Group 'my_group' references unknown group 'missing_group'
```

**Solution:** Ensure all referenced groups exist
```yaml
task_groups:
  my_group:
    depends_on: [other_group]  # Must exist

  other_group:  # ✓ Defined
    tasks: [task1]
```

**"Parent group not found"**
```
Error: Group 'child' specifies unknown parent 'parent'
```

**Solution:** Define parent before referencing
```yaml
task_groups:
  parent:  # ✓ Defined first
    groups: [child]

  child:
    parent: "parent"
```

### Execution Issues

**Group not starting**

**Symptoms:** Group remains in "pending" state

**Possible Causes:**
1. Dependencies not satisfied
2. Condition evaluates to false
3. Parent group not running
4. Resource constraints

**Solutions:**
- Check dependency status
- Verify condition expression
- Review parent group state
- Check resource availability

**Tasks executing in wrong order**

**Symptoms:** Tasks run before dependencies complete

**Possible Causes:**
1. Wrong execution mode
2. Missing task dependencies
3. Missing group dependencies

**Solutions:**
```yaml
# Ensure sequential mode if order matters
execution_mode: "sequential"

# Add explicit task dependencies
tasks:
  task2:
    depends_on: [task1]

# Add group dependencies
group2:
  depends_on: [group1]
```

**Group timeout**

**Symptoms:** Group fails with timeout error

**Possible Causes:**
1. Timeout too short
2. Slow task execution
3. Deadlock or hang

**Solutions:**
```yaml
# Increase timeout
timeout: 1800  # 30 minutes

# Add timeouts to individual tasks
tasks:
  slow_task:
    timeout: 900
```

### Variable Issues

**"Variable not found"**
```
Error: Variable '${group.missing.var}' not found
```

**Solutions:**
1. Check variable name spelling
2. Ensure producing group completed
3. Verify scope (workflow/group/task)
4. Check variable is in outputs

```yaml
# Ensure output is defined
task_groups:
  producer:
    outputs:
      my_var:  # ✓ Defined
        source:
          type: state
          key: "my_key"

  consumer:
    inputs:
      data: "${group.producer.my_var}"  # ✓ Correct reference
```

**"Type mismatch"**
```
Error: Expected number, got string for variable 'count'
```

**Solution:** Ensure type compatibility
```yaml
inputs:
  count:
    type: number
    # Don't pass string value like "10"
    # Pass number: 10
```

### Performance Issues

**Slow execution**

**Possible Causes:**
1. Sequential mode for independent tasks
2. Low concurrency limit
3. Inefficient task design

**Solutions:**
```yaml
# Use parallel mode
execution_mode: "parallel"

# Increase concurrency
max_concurrency: 10

# Break large tasks into smaller ones
```

**Resource exhaustion**

**Possible Causes:**
1. Too many parallel tasks
2. No concurrency limits
3. Memory-intensive tasks

**Solutions:**
```yaml
# Limit concurrency
max_concurrency: 3

# Use sequential for heavy tasks
execution_mode: "sequential"

# Add resource monitoring
```

## See Also

- [Architecture Documentation](./architecture.md) - Internal implementation details
- [API Reference](./api-reference.md) - Complete schema reference
- [Tutorial](./tutorial.md) - Step-by-step guide
- [Examples](../../examples/task-groups/) - Working examples
- [Main DSL Documentation](../../README.md) - Overall DSL overview
