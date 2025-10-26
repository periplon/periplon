# Predefined Tasks - Quick Start Guide

## What are Predefined Tasks?

Predefined tasks are reusable task definitions that can be shared across multiple workflows. They provide:

- **Reusability**: Define once, use everywhere
- **Consistency**: Standardized task behavior
- **Discoverability**: Auto-discover tasks from local directories
- **Type Safety**: Well-defined input/output contracts

## Quick Start

### 1. Create a Predefined Task

Create a `.task.yaml` file in `.claude/tasks/`:

```yaml
# .claude/tasks/my-task.task.yaml
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "my-task"
  version: "1.0.0"
  description: "My reusable task"

spec:
  agent_template:
    description: "Process ${input.file}"
    tools: ["Read", "Write"]
    permissions:
      mode: "default"

  inputs:
    file:
      type: string
      required: true
      description: "File to process"

  outputs:
    result:
      type: string
      source:
        type: state
        key: "output"
```

### 2. Use in a Workflow

Reference the task in your workflow:

```yaml
# workflow.yaml
name: "My Workflow"
version: "1.0.0"

tasks:
  process:
    uses: "my-task@1.0.0"
    inputs:
      file: "./input.txt"
```

### 3. Task Discovery Locations

Tasks are discovered from (in priority order):

1. **Project**: `./.claude/tasks/`
2. **User**: `~/.claude/tasks/`

Higher priority locations override lower priority ones.

## Task Definition Format

### Metadata

```yaml
metadata:
  name: "task-name"              # Required: lowercase-with-hyphens
  version: "1.0.0"               # Required: version string
  author: "author-name"          # Optional
  description: "Description"     # Optional
  license: "MIT"                 # Optional
  repository: "https://..."      # Optional
  tags: ["tag1", "tag2"]        # Optional
```

### Agent Template

```yaml
spec:
  agent_template:
    description: "Agent description with ${input.variable}"
    model: "claude-sonnet-4-5"   # Optional, defaults to sonnet
    system_prompt: "..."         # Optional
    tools: ["Read", "Write"]     # Optional
    permissions:
      mode: "default"            # default, acceptEdits, plan, bypassPermissions
    max_turns: 10                # Optional
```

### Inputs

```yaml
spec:
  inputs:
    input_name:
      type: string               # string, number, boolean, object, array, secret
      required: true             # true or false
      default: "value"           # Optional default value
      description: "..."         # Optional description
      validation:                # Optional validation rules
        pattern: "^[a-z]+$"      # Regex for strings
        min: 0                   # Min value for numbers
        max: 100                 # Max value for numbers
        min_length: 1            # Min length for strings/arrays
        max_length: 100          # Max length for strings/arrays
        allowed_values:          # Enum values
          - "value1"
          - "value2"
      source: "${env.VAR}"       # Optional source for default
```

### Outputs

```yaml
spec:
  outputs:
    output_name:
      type: string               # Optional type for documentation
      description: "..."         # Optional description
      source:
        type: state              # state, file, or task_output
        key: "state_key"         # State key for state type
```

## Variable Interpolation

Use `${input.name}` in agent templates:

```yaml
agent_template:
  description: "Process ${input.file} with ${input.mode} mode"
  system_prompt: "Maximum length: ${input.max_length}"
```

## Input Validation

### String Validation

```yaml
inputs:
  filename:
    type: string
    validation:
      pattern: "^[a-zA-Z0-9_.-]+$"
      min_length: 1
      max_length: 255
```

### Number Validation

```yaml
inputs:
  count:
    type: number
    validation:
      min: 1
      max: 1000
```

### Enum Validation

```yaml
inputs:
  mode:
    type: string
    validation:
      allowed_values:
        - "fast"
        - "thorough"
        - "balanced"
```

## Workflow Usage

### Basic Usage

```yaml
tasks:
  my_task:
    uses: "task-name@1.0.0"
    inputs:
      input1: "value1"
      input2: 42
```

### With Dependencies

```yaml
tasks:
  task1:
    uses: "task-a@1.0.0"
    inputs:
      file: "./input.txt"

  task2:
    uses: "task-b@2.0.0"
    depends_on: [task1]
    inputs:
      previous_result: "${task.task1.output}"
```

### Embed vs Uses

**Uses** (recommended): References the task definition

```yaml
tasks:
  my_task:
    uses: "task-name@1.0.0"
```

**Embed** (for customization): Copies the task definition

```yaml
tasks:
  my_task:
    embed: "task-name@1.0.0"
    overrides:
      agent_template:
        model: "claude-opus-4"  # Override model
```

## Examples

### File Processing Task

```yaml
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "file-analyzer"
  version: "1.0.0"

spec:
  agent_template:
    description: "Analyze ${input.file_path} for ${input.analysis_type}"
    tools: ["Read", "Write"]

  inputs:
    file_path:
      type: string
      required: true
      validation:
        pattern: "^[^<>:\"|?*]+$"

    analysis_type:
      type: string
      default: "general"
      validation:
        allowed_values: ["general", "security", "performance"]

  outputs:
    report:
      type: string
      source:
        type: file
        path: "./analysis-report.md"
```

### API Integration Task

```yaml
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "api-caller"
  version: "1.0.0"

spec:
  agent_template:
    description: "Call ${input.endpoint} API"
    tools: ["WebFetch", "Bash"]

  inputs:
    endpoint:
      type: string
      required: true
      validation:
        pattern: "^https?://.+"

    api_key:
      type: secret
      required: true
      source: "${env.API_KEY}"

    method:
      type: string
      default: "GET"
      validation:
        allowed_values: ["GET", "POST", "PUT", "DELETE"]

  outputs:
    response:
      type: string
      source:
        type: state
        key: "api_response"
```

## Error Handling

Common errors and solutions:

### Task Not Found

```
Error: Task not found: my-task@1.0.0
```

**Solution**: Ensure the task file exists in `.claude/tasks/` with the correct name and version.

### Invalid Reference Format

```
Error: Invalid task reference 'my-task'. Expected format: 'task-name@version'
```

**Solution**: Always use the format `task-name@version` (e.g., `my-task@1.0.0`).

### Missing Required Input

```
Error: Missing required input 'file' for task 'my-task'
```

**Solution**: Provide all required inputs in the workflow task definition.

### Type Mismatch

```
Error: Invalid type for input 'count': expected number, got string
```

**Solution**: Ensure input values match the expected types.

### Validation Failed

```
Error: Input validation failed for 'filename': Value does not match pattern
```

**Solution**: Check validation rules and ensure inputs meet the requirements.

## Best Practices

### 1. Naming Conventions

- **Task names**: lowercase-with-hyphens (e.g., `google-drive-upload`)
- **Input names**: lowercase_with_underscores (e.g., `file_path`)
- **Output names**: lowercase_with_underscores (e.g., `result_data`)

### 2. Versioning

- Use semantic versioning (e.g., `1.0.0`)
- Increment versions when changing inputs/outputs
- Document breaking changes

### 3. Documentation

- Always provide clear descriptions
- Include examples for common use cases
- Document all inputs and outputs

### 4. Validation

- Add validation rules for all inputs
- Use patterns for file paths and URLs
- Use allowed_values for enums

### 5. Security

- Use `secret` type for sensitive data
- Source secrets from environment variables
- Never hardcode credentials

## CLI Commands (Future)

```bash
# List available tasks
periplon-executor tasks list

# Show task details
periplon-executor tasks show my-task

# Validate task definition
periplon-executor tasks validate ./my-task.task.yaml

# Create new task from template
periplon-executor tasks new my-task
```

## Programmatic API

```rust
use periplon_sdk::dsl::predefined_tasks::{
    TaskLoader, TaskResolver, TaskReference
};

// Load a task
let mut loader = TaskLoader::new();
let task_ref = TaskReference::parse("my-task@1.0.0")?;
let task = loader.load(&task_ref)?;

// Resolve and instantiate
let mut resolver = TaskResolver::new();
let (agent_id, agent_spec, task_spec) = resolver.resolve(
    "my-task@1.0.0",
    &inputs,
    &outputs
)?;
```

## Next Steps

- **Phase 2**: Git repository support for remote tasks
- **Phase 3**: Semantic versioning and dependency resolution
- **Phase 4**: Task groups and bundles
- **Phase 5**: Marketplace support

## Resources

- [Implementation Specification](./predefined-tasks-implementation.md)
- [Phase 1 Summary](./predefined-tasks-phase1-summary.md)
- [Example Tasks](./.claude/tasks/)
- [Example Workflow](../examples/workflows/predefined-tasks-demo.yaml)
