# Task Group Examples

This directory contains examples demonstrating the task group features of the DSL workflow system.

## Overview

Task groups allow you to organize related tasks into logical units with configurable execution modes, error handling, and dependency management. Groups can be nested hierarchically, run sequentially or in parallel, and share context through variables.

## Examples

### simple-group.taskgroup.yaml

**Purpose:** Introduction to basic task group concepts

**Features Demonstrated:**
- Sequential task execution within a group
- Parallel task execution across groups
- Group-level configuration (timeouts, error handling)
- Basic group dependencies
- Variable passing between tasks and groups

**Use Case:** Simple code analysis followed by parallel documentation generation

**Structure:**
```
analysis_pipeline (sequential)
  ├── scan_files
  ├── check_syntax
  └── analyze_complexity

documentation_generation (parallel) [depends_on: analysis_pipeline]
  ├── generate_readme
  ├── generate_api_docs
  └── generate_changelog
```

**Run Example:**
```bash
cargo run --bin dsl-executor -- run examples/task-groups/simple-group.taskgroup.yaml \
  --input project_path=/path/to/project
```

**Validate Example:**
```bash
cargo run --bin dsl-executor -- validate examples/task-groups/simple-group.taskgroup.yaml
```

---

### advanced-group.taskgroup.yaml

**Purpose:** Showcase advanced task group features and patterns

**Features Demonstrated:**
- Hierarchical group nesting (3 levels deep)
- Cross-group dependencies and data flow
- Variable interpolation across group boundaries
- Conditional group execution based on quality thresholds
- Multi-agent collaboration patterns
- Parallel expert analysis with synthesis
- Custom error handling strategies (stop, continue, rollback)
- Group-level input/output variables
- Context sharing between groups

**Use Case:** Complete CI/CD pipeline with quality gates, multi-expert analysis, and conditional deployment

**Structure:**
```
full_pipeline (root)
  │
  ├── data_acquisition (parallel)
  │     ├── clone_repository
  │     ├── fetch_dependencies
  │     └── download_config
  │
  ├── quality_analysis (sequential)
  │     ├── parallel_analysis (parallel)
  │     │     ├── code_quality_check
  │     │     ├── security_audit
  │     │     └── performance_analysis
  │     │
  │     └── synthesis (sequential)
  │           ├── merge_findings
  │           └── calculate_score
  │
  └── build_and_deploy (conditional) [if quality_score >= threshold]
        ├── build_phase (sequential)
        │     ├── compile_code
        │     ├── run_tests
        │     └── create_artifacts
        │
        └── deployment_phase (sequential)
              ├── backup_current
              ├── deploy_new
              └── verify_deployment
```

**Run Example:**
```bash
cargo run --bin dsl-executor -- run examples/task-groups/advanced-group.taskgroup.yaml \
  --input repo_url=https://github.com/user/repo \
  --input environment=staging \
  --input quality_threshold=0.85
```

**Validate Example:**
```bash
cargo run --bin dsl-executor -- validate examples/task-groups/advanced-group.taskgroup.yaml
```

## Key Concepts

### Execution Modes

#### Sequential
Tasks run one after another in order:
```yaml
task_groups:
  my_group:
    execution_mode: "sequential"
    tasks: [task1, task2, task3]
```

#### Parallel
Tasks run concurrently:
```yaml
task_groups:
  my_group:
    execution_mode: "parallel"
    max_concurrency: 3  # Optional: limit concurrent tasks
    tasks: [task1, task2, task3]
```

### Group Dependencies

Groups can depend on other groups:
```yaml
task_groups:
  analysis:
    tasks: [analyze]

  deployment:
    depends_on: [analysis]  # Runs after analysis completes
    tasks: [deploy]
```

### Hierarchical Groups

Groups can contain other groups:
```yaml
task_groups:
  parent:
    groups: [child1, child2]

  child1:
    parent: "parent"
    tasks: [task1]

  child2:
    parent: "parent"
    tasks: [task2]
```

### Variable Interpolation

Groups can share data via variables:
```yaml
task_groups:
  producer:
    outputs:
      result: "${state.my_result}"

  consumer:
    depends_on: [producer]
    inputs:
      data: "${group.producer.result}"  # Access producer's output
```

Variable scoping:
- `${workflow.var}` - Workflow-level input variables
- `${group.group_id.var}` - Group-level output variables
- `${task.task_id.var}` - Task-level output variables
- `${var}` - Searches current scope upward

### Conditional Execution

Groups can execute conditionally:
```yaml
task_groups:
  conditional_group:
    condition: "${group.previous.status} == 'success'"
    tasks: [task1]
```

### Error Handling

Configure how groups handle errors:
```yaml
task_groups:
  my_group:
    on_error: "stop"      # Options: stop, continue, rollback
    timeout: 300          # Timeout in seconds
```

### Concurrency Control

Limit parallel task execution:
```yaml
task_groups:
  parallel_group:
    execution_mode: "parallel"
    max_concurrency: 3  # Run max 3 tasks at once
```

## Common Patterns

### Sequential Pipeline
Tasks must complete in order:
```yaml
task_groups:
  pipeline:
    execution_mode: "sequential"
    tasks: [step1, step2, step3]
```

### Parallel Fan-Out
Independent tasks run concurrently:
```yaml
task_groups:
  fan_out:
    execution_mode: "parallel"
    max_concurrency: 5
    tasks: [task1, task2, task3, task4, task5]
```

### Multi-Expert Analysis
Multiple agents analyze in parallel, then synthesize:
```yaml
task_groups:
  analysis:
    execution_mode: "parallel"
    tasks: [expert1_analysis, expert2_analysis, expert3_analysis]

  synthesis:
    depends_on: [analysis]
    execution_mode: "sequential"
    tasks: [merge_results]
```

### Quality Gate
Conditional progression based on metrics:
```yaml
task_groups:
  quality_check:
    tasks: [calculate_score]
    outputs:
      score: "${state.quality_score}"

  deployment:
    depends_on: [quality_check]
    condition: "${group.quality_check.score} >= 0.8"
    tasks: [deploy]
```

### Hierarchical Stages
Multi-level organization:
```yaml
task_groups:
  ci_cd:
    groups: [build_stage, test_stage, deploy_stage]

  build_stage:
    parent: "ci_cd"
    groups: [compile, package]
```

## Validation

The DSL validator checks:
- ✓ All group references are valid
- ✓ No circular dependencies between groups
- ✓ Parent-child relationships are consistent
- ✓ Variable references are valid
- ✓ Tasks are assigned to existing groups
- ✓ Execution modes are compatible with group structure

Run validation:
```bash
cargo run --bin dsl-executor -- validate path/to/workflow.yaml
```

## Best Practices

1. **Use meaningful group names**: Describe what the group does
2. **Set appropriate timeouts**: Prevent runaway processes
3. **Configure error handling**: Choose stop/continue based on failure impact
4. **Limit concurrency**: Prevent resource exhaustion
5. **Use hierarchical organization**: Break complex workflows into logical stages
6. **Pass data via variables**: Use group outputs/inputs for clean interfaces
7. **Add conditions for gates**: Implement quality gates and approvals
8. **Document group purpose**: Use description field

## Troubleshooting

**Group not executing:**
- Check `depends_on` dependencies are satisfied
- Verify `condition` evaluates to true
- Ensure parent group has completed

**Tasks in wrong order:**
- Verify `execution_mode` is correct
- Check task-level `depends_on` within group
- Review group dependencies

**Variable not found:**
- Ensure producing group/task has completed
- Check variable name spelling
- Verify scope (workflow/group/task)

**Validation errors:**
- Run `validate` command for detailed error messages
- Check for circular dependencies
- Verify all referenced groups/tasks exist

## Additional Resources

- DSL Documentation: `../../README.md`
- Schema Reference: `../../src/dsl/schema.rs`
- Validator: `../../src/dsl/validator.rs`
- Executor: `../../src/dsl/executor.rs`
- Task Graph: `../../src/dsl/task_graph.rs`
- More Examples: `../dsl/task_groups/`
