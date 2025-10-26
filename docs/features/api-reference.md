# Task Groups API Reference

Complete reference for task group configuration schema.

## Table of Contents

1. [TaskGroup Schema](#taskgroup-schema)
2. [ExecutionMode](#executionmode)
3. [ErrorStrategy](#errorstrategy)
4. [VariableDefinition](#variabledefinition)
5. [VariableOutput](#variableoutput)
6. [Complete Example](#complete-example)

## TaskGroup Schema

### Fields

#### `description` (required)

**Type:** String

**Description:** Human-readable description of what this group does.

**Example:**
```yaml
description: "Build and test the application"
```

**Constraints:**
- Must not be empty
- Should be concise and descriptive
- Used in logs and UI

---

#### `execution_mode` (required)

**Type:** [ExecutionMode](#executionmode)

**Description:** Determines how tasks within the group execute.

**Values:**
- `sequential`: Tasks run one after another
- `parallel`: Tasks run concurrently
- `auto`: Automatically determine based on dependencies

**Example:**
```yaml
execution_mode: "parallel"
```

**Default:** None (must be specified)

---

#### `tasks` (optional)

**Type:** Array of Strings

**Description:** List of task IDs that belong to this group.

**Example:**
```yaml
tasks:
  - compile_code
  - run_tests
  - generate_docs
```

**Constraints:**
- Each task ID must reference an existing task
- Tasks can only belong to one group
- Required if `groups` is not specified

**Mutually Exclusive:** Either `tasks` or `groups` (or both) must be specified

---

#### `groups` (optional)

**Type:** Array of Strings

**Description:** List of child group IDs for hierarchical organization.

**Example:**
```yaml
groups:
  - build_phase
  - test_phase
  - deploy_phase
```

**Constraints:**
- Each group ID must reference an existing group
- Child groups must have `parent` field pointing to this group
- Creates parent-child hierarchy

**Mutually Exclusive:** Either `tasks` or `groups` (or both) must be specified

---

#### `depends_on` (optional)

**Type:** Array of Strings

**Description:** Group IDs that must complete before this group starts.

**Example:**
```yaml
depends_on:
  - build_group
  - test_group
```

**Constraints:**
- All dependency IDs must reference existing groups
- Cannot create circular dependencies
- All dependencies must complete successfully (unless condition allows failure)

**Default:** `[]` (no dependencies)

---

#### `parent` (optional)

**Type:** String

**Description:** ID of parent group for hierarchical nesting.

**Example:**
```yaml
parent: "ci_cd_pipeline"
```

**Constraints:**
- Must reference an existing group
- Parent must list this group in its `groups` field
- Creates parent-child relationship

**Default:** `null` (root-level group)

---

#### `condition` (optional)

**Type:** String (expression)

**Description:** Boolean expression that must evaluate to true for group to execute.

**Example:**
```yaml
condition: "${group.quality_check.score} >= 0.8"
```

**Expression Syntax:**
- Variable references: `${scope.variable}`
- Comparison operators: `==`, `!=`, `>`, `>=`, `<`, `<=`
- Logical operators: `&&`, `||`, `!`
- Parentheses for grouping: `(expr)`

**Supported Scopes:**
- `workflow`: Workflow-level inputs
- `group.group_id`: Group outputs
- `task.task_id`: Task outputs

**Examples:**
```yaml
# Simple comparison
condition: "${workflow.environment} == 'prod'"

# Numeric threshold
condition: "${group.tests.coverage} >= 0.8"

# Logical operators
condition: "${workflow.deploy} == true && ${group.tests.passed} == true"

# Complex expression
condition: "(${group.quality.score} >= 0.8 && ${workflow.environment} == 'prod') || ${workflow.force_deploy} == true"
```

**Default:** `null` (always execute)

---

#### `on_error` (optional)

**Type:** [ErrorStrategy](#errorstrategy)

**Description:** How to handle task failures within the group.

**Values:**
- `stop`: Stop group execution on first failure
- `continue`: Continue executing remaining tasks
- `rollback`: Execute rollback logic on failure

**Example:**
```yaml
on_error: "continue"
```

**Default:** `"stop"`

---

#### `timeout` (optional)

**Type:** Integer (seconds)

**Description:** Maximum execution time for the entire group.

**Example:**
```yaml
timeout: 600  # 10 minutes
```

**Constraints:**
- Must be positive integer
- Applies to entire group (including all child groups/tasks)
- On timeout, group fails and running tasks are cancelled

**Default:** No timeout (infinite)

**Inheritance:** Child groups inherit parent timeout if not specified

---

#### `max_concurrency` (optional)

**Type:** Integer

**Description:** Maximum number of tasks that can run in parallel.

**Example:**
```yaml
max_concurrency: 5
```

**Constraints:**
- Must be positive integer
- Only applies when `execution_mode: "parallel"`
- Limits concurrent task execution

**Default:** Unlimited (all tasks run concurrently)

**Use Cases:**
- Prevent resource exhaustion
- Rate limit external API calls
- Control memory/CPU usage

---

#### `inputs` (optional)

**Type:** Object (Map of String to [VariableDefinition](#variabledefinition))

**Description:** Variables consumed by this group.

**Example:**
```yaml
inputs:
  config_file:
    type: string
    required: true
    description: "Path to configuration file"
  threshold:
    type: number
    required: false
    default: 0.8
```

**See:** [VariableDefinition](#variabledefinition) for field details

**Default:** `{}` (no inputs)

---

#### `outputs` (optional)

**Type:** Object (Map of String to [VariableOutput](#variableoutput))

**Description:** Variables produced by this group.

**Example:**
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

**See:** [VariableOutput](#variableoutput) for field details

**Default:** `{}` (no outputs)

---

## ExecutionMode

Enum specifying how tasks execute within a group.

### Values

#### `sequential`

Tasks execute one after another in order.

**Characteristics:**
- Predictable order
- Lower resource usage
- Tasks wait for previous to complete
- Task dependencies honored

**Use Cases:**
- Build pipelines (order matters)
- Database migrations
- Deployment steps

**Example:**
```yaml
execution_mode: "sequential"
tasks:
  - step1
  - step2
  - step3
```

**Execution:**
```
step1 → complete → step2 → complete → step3 → complete
```

---

#### `parallel`

Tasks execute concurrently.

**Characteristics:**
- Maximum throughput
- Higher resource usage
- Tasks run independently
- Can limit with `max_concurrency`

**Use Cases:**
- Independent tests
- Parallel data processing
- Multi-agent analysis

**Example:**
```yaml
execution_mode: "parallel"
max_concurrency: 3
tasks:
  - test1
  - test2
  - test3
  - test4
```

**Execution:**
```
test1 ─┐
test2 ─┼─→ (up to 3 concurrent)
test3 ─┤
test4 ─┘ (waits for slot)
```

---

#### `auto`

Automatically determine execution mode based on task dependencies.

**Behavior:**
- Analyzes task dependencies
- Parallelizes independent tasks
- Respects dependency order
- Optimizes execution plan

**Use Cases:**
- Complex workflows with mixed dependencies
- When optimal strategy unclear
- Dynamic task sets

**Example:**
```yaml
execution_mode: "auto"
tasks:
  - task_a          # No deps: runs first
  - task_b          # depends_on: [task_a]
  - task_c          # depends_on: [task_a]
  - task_d          # depends_on: [task_b, task_c]
```

**Execution:**
```
task_a → (task_b, task_c in parallel) → task_d
```

---

## ErrorStrategy

Enum specifying how to handle task failures.

### Values

#### `stop`

Stop group execution on first task failure.

**Behavior:**
- First failure stops the group immediately
- Remaining tasks are cancelled
- Group marked as failed
- Error propagates to parent

**Use Cases:**
- Critical pipelines
- When failures are unacceptable
- Fast failure detection

**Example:**
```yaml
on_error: "stop"
```

**Execution:**
```
task1 ✓ → task2 ✗ → [STOP] → task3 cancelled
Group status: Failed
```

---

#### `continue`

Continue executing remaining tasks despite failures.

**Behavior:**
- Failures don't stop execution
- All tasks attempt to run
- Collect all failures
- Group marked failed if any task failed

**Use Cases:**
- Test suites (want all results)
- Independent validations
- Comprehensive error reports

**Example:**
```yaml
on_error: "continue"
```

**Execution:**
```
task1 ✓ → task2 ✗ → task3 ✓ → task4 ✗
Group status: Failed (2/4 tasks failed)
```

---

#### `rollback`

Execute rollback logic on failure.

**Behavior:**
- On failure, execute rollback tasks
- Attempt to restore previous state
- Group marked failed with rollback attempted

**Use Cases:**
- Deployments (restore previous version)
- Database migrations
- Configuration updates

**Example:**
```yaml
on_error: "rollback"
```

**Note:** Requires rollback task definition (implementation-specific)

---

## VariableDefinition

Defines an input variable for a group.

### Fields

#### `type` (required)

**Type:** String

**Description:** Variable data type.

**Values:**
- `string`: Text value
- `number`: Numeric value (integer or float)
- `boolean`: True/false value
- `object`: JSON object
- `array`: JSON array

**Example:**
```yaml
type: string
```

---

#### `required` (optional)

**Type:** Boolean

**Description:** Whether variable must be provided.

**Example:**
```yaml
required: true
```

**Default:** `false`

**Behavior:**
- If `true` and variable not provided: validation error
- If `false` and variable not provided: use default (if specified)

---

#### `default` (optional)

**Type:** Any (must match `type`)

**Description:** Default value if variable not provided.

**Example:**
```yaml
default: "production"
```

**Constraints:**
- Type must match specified `type`
- Only used if `required: false`

---

#### `description` (optional)

**Type:** String

**Description:** Human-readable description of variable purpose.

**Example:**
```yaml
description: "Environment to deploy to (dev, staging, prod)"
```

---

### Complete Example

```yaml
inputs:
  environment:
    type: string
    required: true
    description: "Target deployment environment"

  replicas:
    type: number
    required: false
    default: 3
    description: "Number of replicas to deploy"

  enable_monitoring:
    type: boolean
    required: false
    default: true
    description: "Enable monitoring and alerting"

  config:
    type: object
    required: false
    description: "Additional configuration options"
```

---

## VariableOutput

Defines an output variable produced by a group.

### Fields

#### `source` (required)

**Type:** Object (VariableOutputSource)

**Description:** Source of the output variable value.

**Structure:**
```yaml
source:
  type: <source_type>
  # Additional fields based on type
```

---

### Source Types

#### File Source

Read variable value from file.

**Schema:**
```yaml
source:
  type: file
  path: <file_path>
```

**Fields:**
- `type`: Must be `"file"`
- `path`: Path to file (supports variable interpolation)

**Example:**
```yaml
outputs:
  build_artifact:
    source:
      type: file
      path: "./dist/app-${workflow.version}.zip"
```

**Behavior:**
- File is read when group completes
- File contents become variable value
- Supports text and JSON files
- Variable interpolation in path

---

#### State Source

Read variable value from workflow state.

**Schema:**
```yaml
source:
  type: state
  key: <state_key>
```

**Fields:**
- `type`: Must be `"state"`
- `key`: Key in workflow state (dot-notation for nested)

**Example:**
```yaml
outputs:
  test_coverage:
    source:
      type: state
      key: "test.coverage.percent"
```

**Behavior:**
- Reads from in-memory workflow state
- Supports nested keys with dot notation
- Must be set during task execution

---

#### Task Source

Read variable value from specific task output.

**Schema:**
```yaml
source:
  type: task
  task_id: <task_id>
  variable: <variable_name>
```

**Fields:**
- `type`: Must be `"task"`
- `task_id`: ID of task to read from
- `variable`: Name of task's output variable

**Example:**
```yaml
outputs:
  deployment_id:
    source:
      type: task
      task_id: "deploy_task"
      variable: "deployment_id"
```

**Behavior:**
- Reads from specific task's outputs
- Task must have completed
- Task must define the output variable

---

### Complete Example

```yaml
outputs:
  # File source
  build_artifact:
    source:
      type: file
      path: "./dist/app.zip"

  # State source
  build_number:
    source:
      type: state
      key: "build.number"

  # Nested state
  test_coverage:
    source:
      type: state
      key: "test.coverage.percent"

  # Task source
  deployment_url:
    source:
      type: task
      task_id: "deploy_prod"
      variable: "url"
```

---

## Complete Example

Full task group configuration demonstrating all features:

```yaml
task_groups:
  # Root group
  ci_cd_pipeline:
    description: "Complete CI/CD pipeline"
    execution_mode: "sequential"
    groups:
      - build_and_test
      - deploy

    timeout: 1800  # 30 minutes
    on_error: "stop"

    inputs:
      repo_url:
        type: string
        required: true
        description: "Repository URL"

      environment:
        type: string
        required: true
        description: "Target environment"

    outputs:
      deployment_url:
        source:
          type: state
          key: "deployment.url"

  # Child group 1
  build_and_test:
    description: "Build and test application"
    parent: "ci_cd_pipeline"
    execution_mode: "sequential"
    groups:
      - build
      - test

    timeout: 900  # 15 minutes

  # Grandchild group 1.1
  build:
    description: "Build application"
    parent: "build_and_test"
    execution_mode: "parallel"
    tasks:
      - compile_backend
      - compile_frontend
      - build_docker

    max_concurrency: 3
    on_error: "stop"

    outputs:
      docker_image:
        source:
          type: state
          key: "docker.image_tag"

  # Grandchild group 1.2
  test:
    description: "Run test suite"
    parent: "build_and_test"
    execution_mode: "parallel"
    tasks:
      - run_unit_tests
      - run_integration_tests
      - run_e2e_tests

    depends_on: [build]
    max_concurrency: 3
    on_error: "continue"  # Collect all test failures

    outputs:
      test_coverage:
        source:
          type: state
          key: "test.coverage"

  # Child group 2
  deploy:
    description: "Deploy to ${workflow.environment}"
    parent: "ci_cd_pipeline"
    execution_mode: "sequential"
    tasks:
      - backup_current
      - deploy_new
      - verify_deployment

    depends_on: [build_and_test]
    condition: "${group.test.test_coverage} >= 0.8"
    timeout: 600
    on_error: "rollback"

    inputs:
      image:
        type: string
        required: true
        default: "${group.build.docker_image}"

    outputs:
      deployment_url:
        source:
          type: task
          task_id: "deploy_new"
          variable: "url"
```

This example demonstrates:
- ✓ 3-level hierarchy (pipeline → phases → stages)
- ✓ Sequential and parallel execution modes
- ✓ Group dependencies
- ✓ Task dependencies (implicit via group ordering)
- ✓ Conditional execution based on test coverage
- ✓ Variable inputs and outputs
- ✓ Variable interpolation in descriptions
- ✓ Different error handling strategies
- ✓ Timeouts at multiple levels
- ✓ Concurrency limits

---

## Schema Validation Rules

### Group References

1. All group IDs in `depends_on` must exist
2. All group IDs in `groups` must exist
3. Parent group ID must exist
4. No circular dependencies allowed

### Hierarchy

1. Child groups must have `parent` field
2. Parent must list child in `groups` field
3. No cycles in parent-child relationships

### Tasks

1. All task IDs in `tasks` must exist
2. Tasks can only belong to one group
3. Task dependencies must be within same group or across groups

### Variables

1. Input variable types must be valid
2. Output source types must be valid
3. Variable references must use valid scopes
4. Referenced groups/tasks must exist

### Execution

1. `execution_mode` must be valid enum value
2. `on_error` must be valid enum value
3. `timeout` must be positive integer
4. `max_concurrency` must be positive integer

---

## See Also

- [Main Documentation](./README.md) - High-level overview and concepts
- [Architecture](./architecture.md) - Implementation details
- [Tutorial](./tutorial.md) - Step-by-step guide
- [Examples](../../examples/task-groups/) - Working examples
