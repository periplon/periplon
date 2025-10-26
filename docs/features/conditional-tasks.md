# Conditional Task Execution

The DSL supports powerful conditional task execution, allowing you to control which tasks run based on the state of other tasks or workflow metadata.

## Overview

Conditional tasks allow you to:
- Run tasks only when previous tasks succeed or fail
- Skip tasks based on workflow state (e.g., environment variables)
- Create complex branching logic with AND, OR, and NOT operators
- Build flexible CI/CD pipelines with conditional deployments

## Condition Types

### 1. Task Status Conditions

Check if a task has a specific status:

```yaml
tasks:
  deploy:
    description: "Deploy to production"
    agent: "deployer"
    depends_on:
      - "run_tests"
    condition:
      type: "task_status"
      task: "run_tests"
      status: "completed"
```

**Available statuses:**
- `completed` - Task finished successfully
- `failed` - Task encountered an error
- `skipped` - Task was skipped due to unmet condition
- `running` - Task is currently executing
- `pending` - Task hasn't started yet

### 2. State-Based Conditions

#### State Equals

Check if a workflow state variable equals a specific value:

```yaml
tasks:
  deploy_prod:
    description: "Deploy to production"
    agent: "deployer"
    condition:
      type: "state_equals"
      key: "environment"
      value: "production"
```

#### State Exists

Check if a workflow state variable exists:

```yaml
tasks:
  use_cache:
    description: "Use cached build artifacts"
    agent: "builder"
    condition:
      type: "state_exists"
      key: "cache_key"
```

### 3. Always/Never Conditions

#### Always
Task always runs (useful for testing):

```yaml
tasks:
  always_run:
    description: "This task always executes"
    agent: "worker"
    condition:
      type: "always"
```

#### Never
Task never runs (useful for disabling features):

```yaml
tasks:
  experimental:
    description: "Experimental feature - disabled"
    agent: "worker"
    condition:
      type: "never"
```

## Logical Operators

### AND Operator

All conditions must be true:

```yaml
tasks:
  deploy_prod:
    description: "Deploy to production if tests pass and env is prod"
    agent: "deployer"
    condition:
      and:
        - type: "task_status"
          task: "run_tests"
          status: "completed"
        - type: "state_equals"
          key: "environment"
          value: "production"
```

### OR Operator

At least one condition must be true:

```yaml
tasks:
  send_notification:
    description: "Notify on build or test failure"
    agent: "notifier"
    condition:
      or:
        - type: "task_status"
          task: "build"
          status: "failed"
        - type: "task_status"
          task: "test"
          status: "failed"
```

### NOT Operator

Inverts the condition:

```yaml
tasks:
  deploy_staging:
    description: "Deploy to staging (non-production)"
    agent: "deployer"
    condition:
      not:
        type: "state_equals"
        key: "environment"
        value: "production"
```

## Complex Conditions

You can nest logical operators for complex logic:

```yaml
tasks:
  conditional_deploy:
    description: "Deploy if (tests passed AND prod) OR force_deploy is set"
    agent: "deployer"
    condition:
      or:
        - and:
            - type: "task_status"
              task: "tests"
              status: "completed"
            - type: "state_equals"
              key: "environment"
              value: "production"
        - type: "state_exists"
          key: "force_deploy"
```

## Task Execution Behavior

### Skipped Tasks

When a task's condition is not met:
1. The task is marked as `skipped`
2. Dependent tasks can still run (skipped tasks are treated as completed for dependency purposes)
3. The workflow continues normally

### Dependencies

Conditional tasks still respect the `depends_on` field:
- Dependencies must complete before the condition is evaluated
- If a dependency fails, the conditional task won't run (unless you handle the failure with conditions)

Example:

```yaml
tasks:
  analyze:
    description: "Analyze code"
    agent: "analyzer"

  test:
    description: "Run tests"
    agent: "tester"
    depends_on:
      - "analyze"

  deploy:
    description: "Deploy only if tests pass"
    agent: "deployer"
    depends_on:
      - "test"
    condition:
      type: "task_status"
      task: "test"
      status: "completed"
```

In this example:
1. `analyze` runs first
2. `test` waits for `analyze` to complete
3. `deploy` waits for `test` to complete, then checks if `test` succeeded
4. If `test` failed, `deploy` is skipped

## Common Patterns

### 1. Conditional Deployment

```yaml
tasks:
  deploy_prod:
    condition:
      and:
        - type: "task_status"
          task: "tests"
          status: "completed"
        - type: "state_equals"
          key: "branch"
          value: "main"
```

### 2. Failure Handling

```yaml
tasks:
  rollback:
    description: "Rollback on deployment failure"
    condition:
      type: "task_status"
      task: "deploy"
      status: "failed"
```

### 3. Environment-Specific Tasks

```yaml
tasks:
  debug_mode:
    condition:
      type: "state_equals"
      key: "environment"
      value: "development"

  performance_mode:
    condition:
      type: "state_equals"
      key: "environment"
      value: "production"
```

### 4. Feature Flags

```yaml
tasks:
  new_feature:
    description: "New feature - enabled via flag"
    condition:
      type: "state_exists"
      key: "enable_new_feature"
```

## Setting Workflow State

To use state-based conditions, you can set workflow state metadata programmatically:

```rust
use periplon_sdk::dsl::{DSLExecutor, parse_workflow_file};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workflow = parse_workflow_file("workflow.yaml")?;
    let mut executor = DSLExecutor::new(workflow)?;

    // Enable state persistence
    executor.enable_state_persistence(None)?;

    // Initialize executor
    executor.initialize().await?;

    // Set workflow state before execution
    if let Some(state) = executor.get_state_mut() {
        state.add_metadata(
            "environment".to_string(),
            serde_json::json!("production")
        );
        state.add_metadata(
            "branch".to_string(),
            serde_json::json!("main")
        );
    }

    // Execute workflow
    executor.execute().await?;
    executor.shutdown().await?;

    Ok(())
}
```

## Best Practices

1. **Use descriptive task names** - Make it clear what the task does and when it runs
2. **Document complex conditions** - Add comments in your YAML to explain complex logic
3. **Test your conditions** - Verify tasks skip/run as expected in different scenarios
4. **Keep conditions simple** - Break complex logic into multiple tasks when possible
5. **Handle failures gracefully** - Use conditions to implement proper error handling and rollback

## Example: Complete CI/CD Pipeline

See `examples/workflows/conditional_tasks.yaml` for a complete example demonstrating:
- Conditional deployment based on test results
- Environment-specific deployments
- Failure notifications
- Feature flag support
- Complex conditional logic with AND/OR/NOT operators
