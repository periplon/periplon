# Task Group Examples

This directory contains comprehensive examples demonstrating the task group features of the DSL workflow system.

## Overview

Task groups allow you to organize related tasks, control execution flow, and manage complex multi-agent workflows with hierarchical organization and dependency management.

## Examples

### 01_basic_groups.yaml
**Demonstrates:** Basic task group features
- Sequential execution groups
- Parallel execution groups
- Group-level configuration (timeouts, error handling)
- Mixed execution modes

**Use case:** Simple code analysis and documentation generation

**Key features:**
```yaml
task_groups:
  sequential_analysis:
    execution_mode: "sequential"  # Tasks run one after another
    on_error: "stop"              # Stop on first error

  parallel_docs:
    execution_mode: "parallel"    # Tasks run concurrently
    max_concurrency: 3            # Limit concurrent tasks
```

**Run:**
```bash
cargo run --bin dsl-executor -- run examples/dsl/task_groups/01_basic_groups.yaml \
  --input project_path=/path/to/project
```

### 02_dependency_chains.yaml
**Demonstrates:** Complex dependency patterns
- Cross-group dependencies
- Group-level dependency resolution
- Conditional group execution
- Multi-stage pipeline (acquisition → processing → validation → cleanup)

**Use case:** Data processing pipeline with validation

**Key features:**
```yaml
task_groups:
  data_processing:
    depends_on: [data_acquisition]  # Group-level dependency

  cleanup_and_finalize:
    depends_on: [validation_suite]
    condition: "${group.validation_suite.status} == 'success'"  # Conditional
```

**Run:**
```bash
cargo run --bin dsl-executor -- run examples/dsl/task_groups/02_dependency_chains.yaml \
  --input repo_url=https://github.com/user/repo
```

### 03_hierarchical_groups.yaml
**Demonstrates:** Hierarchical group organization
- Nested task groups (groups within groups)
- Parent-child relationships
- Property inheritance
- Multi-level deployment pipeline

**Use case:** Complete CI/CD deployment with build, test, and deploy stages

**Key features:**
```yaml
task_groups:
  full_deployment:
    groups:
      - build_and_test      # Child group
      - deploy_and_verify   # Child group

  build_and_test:
    parent: "full_deployment"
    groups:
      - build_phase         # Grandchild group
      - test_phase          # Grandchild group
```

**Run:**
```bash
cargo run --bin dsl-executor -- run examples/dsl/task_groups/03_hierarchical_groups.yaml \
  --input service_name=my-service \
  --input environment=staging
```

### 04_variables_and_context.yaml
**Demonstrates:** Advanced variable usage
- Group-level input/output variables
- Variable interpolation across groups
- Context sharing between groups
- Dynamic variable resolution

**Use case:** Code analysis with data transformation and reporting

**Key features:**
```yaml
task_groups:
  code_analysis:
    outputs:
      total_files: "${state.analysis.total_files}"
      avg_complexity: "${state.analysis.avg_complexity}"

  data_processing:
    inputs:
      file_count: "${group.code_analysis.total_files}"
      complexity: "${group.code_analysis.avg_complexity}"
```

**Run:**
```bash
cargo run --bin dsl-executor -- run examples/dsl/task_groups/04_variables_and_context.yaml \
  --input source_dir=/path/to/code \
  --input output_format=json \
  --input quality_threshold=0.8
```

### 05_multi_agent_collaboration.yaml
**Demonstrates:** Multi-agent collaboration patterns
- Specialized agents working in parallel
- Agent handoffs and context passing
- Expert consultation patterns
- Collaborative problem-solving

**Use case:** Code refactoring with multiple expert agents

**Key features:**
```yaml
agents:
  architect:
    description: "System architect specializing in design patterns"
  security_expert:
    description: "Security specialist focused on vulnerabilities"
  performance_expert:
    description: "Performance optimization specialist"

task_groups:
  expert_analysis:
    execution_mode: "parallel"
    tasks:
      - architecture_review   # Uses architect agent
      - security_audit        # Uses security_expert agent
      - performance_analysis  # Uses performance_expert agent
```

**Run:**
```bash
cargo run --bin dsl-executor -- run examples/dsl/task_groups/05_multi_agent_collaboration.yaml \
  --input codebase_path=/path/to/code \
  --input target_language=rust
```

### 06_real_world_ci_cd.yaml
**Demonstrates:** Production-ready CI/CD pipeline
- Multi-stage build and test
- Environment-specific deployments
- Rollback capabilities
- Monitoring and verification
- Production safeguards

**Use case:** Complete CI/CD pipeline for real-world deployment

**Key features:**
```yaml
task_groups:
  build_pipeline:
    groups:
      - compile_and_build
      - security_scanning

  deployment:
    groups:
      - deploy_infrastructure
      - deploy_application
      - post_deploy_validation

  rollback:
    condition: "${group.monitoring.monitoring_status} == 'failed'"
```

**Run:**
```bash
cargo run --bin dsl-executor -- run examples/dsl/task_groups/06_real_world_ci_cd.yaml \
  --input repo_url=https://github.com/user/repo \
  --input branch=main \
  --input target_env=staging
```

## Common Patterns

### Sequential Execution
Tasks run one after another:
```yaml
task_groups:
  my_group:
    execution_mode: "sequential"
    tasks: [task1, task2, task3]
```

### Parallel Execution
Tasks run concurrently:
```yaml
task_groups:
  my_group:
    execution_mode: "parallel"
    max_concurrency: 3
    tasks: [task1, task2, task3]
```

### Group Dependencies
Groups can depend on other groups:
```yaml
task_groups:
  group_a:
    tasks: [task1]

  group_b:
    depends_on: [group_a]  # Runs after group_a completes
    tasks: [task2]
```

### Hierarchical Groups
Groups can contain other groups:
```yaml
task_groups:
  parent_group:
    groups: [child_group_1, child_group_2]

  child_group_1:
    parent: "parent_group"
    tasks: [task1]
```

### Variable Sharing
Groups can pass data via variables:
```yaml
task_groups:
  producer:
    outputs:
      result: "${state.my_result}"

  consumer:
    inputs:
      data: "${group.producer.result}"
```

### Conditional Execution
Groups can execute conditionally:
```yaml
task_groups:
  conditional_group:
    condition: "${group.previous_group.status} == 'success'"
```

### Error Handling
Groups have configurable error handling:
```yaml
task_groups:
  my_group:
    on_error: "stop"      # stop, continue, or rollback
    timeout: 300          # seconds
```

## Validation

Validate any workflow before running:

```bash
cargo run --bin dsl-executor -- validate examples/dsl/task_groups/01_basic_groups.yaml
```

The validator checks:
- Group reference validity
- Circular dependencies in groups
- Parent-child relationship consistency
- Variable reference validity
- Task assignment to groups
- Execution mode compatibility

## Features Summary

| Feature | Example File |
|---------|-------------|
| Sequential groups | 01, 02, 03, 06 |
| Parallel groups | 01, 02, 05, 06 |
| Group dependencies | 02, 03, 04, 06 |
| Hierarchical groups | 03, 06 |
| Variable interpolation | 04, 05, 06 |
| Multi-agent coordination | 05, 06 |
| Conditional execution | 02, 04, 06 |
| Error handling | All |
| Timeouts | 03, 06 |
| Concurrency limits | 01, 03, 05, 06 |

## Best Practices

1. **Use sequential groups when order matters**: Database migrations, deployment steps
2. **Use parallel groups for independent tasks**: Testing, analysis, documentation
3. **Organize complex workflows hierarchically**: Break down into logical stages
4. **Pass data via variables**: Use group-level outputs/inputs for clean interfaces
5. **Set appropriate timeouts**: Prevent runaway processes
6. **Configure error handling**: Choose stop/continue based on failure impact
7. **Limit concurrency**: Prevent resource exhaustion with max_concurrency
8. **Use conditional execution**: Skip unnecessary work based on previous results

## Advanced Topics

### Cycle Detection
The validator automatically detects cycles in group dependencies:
```
Error: Circular dependency detected: A → B → C → A
```

### Variable Scoping
Variables have hierarchical scoping:
- `${workflow.var}` - Workflow-level input
- `${group.group_id.var}` - Group-level output
- `${task.task_id.var}` - Task-level output
- `${var}` - Searches current scope upward

### State Persistence
Groups can checkpoint state for resumption:
```yaml
task_groups:
  my_group:
    checkpoint: true  # Save state after completion
```

### Dynamic Group Generation
Groups can be generated programmatically (see natural language generator).

## Troubleshooting

**Group not executing:**
- Check `depends_on` is satisfied
- Verify `condition` evaluates to true
- Check parent group has completed

**Tasks running in wrong order:**
- Verify `execution_mode` is set correctly
- Check task-level `depends_on` within group
- Review group-level dependencies

**Variable not found:**
- Ensure producing group has completed
- Check variable name spelling
- Verify scope (workflow/group/task)

**Validation errors:**
- Run `validate` command for detailed errors
- Check for circular dependencies
- Verify all referenced groups/tasks exist

## Further Reading

- Main DSL documentation: `../../README.md`
- Schema reference: `../../../src/dsl/schema.rs`
- Validator implementation: `../../../src/dsl/validator.rs`
- Executor implementation: `../../../src/dsl/executor.rs`
- Task graph management: `../../../src/dsl/task_graph.rs`
