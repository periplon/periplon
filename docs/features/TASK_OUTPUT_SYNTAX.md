# Task Output Syntax in DSL

## Overview

Tasks in the DSL have **TWO ways** to specify outputs:

1. **Simple output** - Single output file path (string)
2. **Complex outputs** - Named outputs with typed sources (map)

## 1. Simple Output (Single File)

Use `output:` for a single output file path.

### Syntax
```yaml
tasks:
  task_name:
    description: "Task description"
    agent: "agent_id"
    output: "path/to/output.txt"  # Simple string path
```

### Examples

#### Basic Output File
```yaml
tasks:
  generate_report:
    description: "Generate analysis report"
    agent: "analyst"
    output: "report.md"  # Output saved to report.md
```

#### Output with Path
```yaml
tasks:
  process_data:
    description: "Process CSV data"
    agent: "processor"
    output: "results/processed_data.csv"  # Output in subdirectory
```

#### Script Task with Output
```yaml
tasks:
  run_tests:
    description: "Run test suite"
    script:
      language: bash
      content: |
        pytest tests/ > test_results.txt 2>&1
        echo "Tests complete"
    output: "test_results.txt"  # Capture test output
```

### How It Works

- The executor **does not automatically write** to this file
- **You or the agent** must write to the file
- The path is available to the agent as context
- Useful for documentation and DoD checks

### Use Cases

✅ Simple file outputs
✅ Single result file per task
✅ Documentation of expected output
✅ DoD criteria checking (file_exists)

## 2. Complex Outputs (Named with Sources)

Use `outputs:` for multiple named outputs with explicit sources.

### Syntax
```yaml
tasks:
  task_name:
    description: "Task description"
    agent: "agent_id"
    outputs:
      output_name:
        source:
          type: file | state | task_output  # REQUIRED
          # type-specific fields below
        description: "Optional description"
```

### Source Types

#### A. File Source
Read output from a file path.

```yaml
outputs:
  result:
    source:
      type: file
      path: "./output/result.json"
    description: "Task execution result"
```

#### B. State Source
Read output from workflow state.

```yaml
outputs:
  processing_status:
    source:
      type: state
      key: "status"
    description: "Current processing status"
```

#### C. Task Output Source
Read output from another task's result.

```yaml
outputs:
  previous_result:
    source:
      type: task_output
      task: "fetch_data"
    description: "Data from previous task"
```

### Complete Examples

#### Multiple Outputs
```yaml
tasks:
  analyze_data:
    description: "Analyze dataset and generate multiple outputs"
    agent: "analyst"
    outputs:
      summary:
        source:
          type: file
          path: "./analysis/summary.json"
        description: "Analysis summary"

      full_report:
        source:
          type: file
          path: "./analysis/full_report.md"
        description: "Complete analysis report"

      status:
        source:
          type: state
          key: "analysis_status"
        description: "Analysis completion status"
```

#### Chaining Task Outputs
```yaml
tasks:
  fetch_data:
    description: "Fetch raw data"
    agent: "fetcher"
    outputs:
      raw_data:
        source:
          type: file
          path: "./data/raw.csv"

  process_data:
    description: "Process the fetched data"
    agent: "processor"
    depends_on: [fetch_data]
    outputs:
      processed:
        source:
          type: task_output
          task: "fetch_data"  # Use output from fetch_data
        description: "Processed data from previous task"

      final_result:
        source:
          type: file
          path: "./data/processed.json"
```

### Auto-Injection into Agent Context

When you define `outputs:`, the DSL executor **automatically injects** this information into the agent's prompt:

```
## Required Outputs
This task MUST produce the following outputs:

- `summary`: File: `./analysis/summary.json` - Analysis summary
- `full_report`: File: `./analysis/full_report.md` - Complete analysis report
- `status`: Workflow state key: `analysis_status` - Analysis completion status
```

This helps the agent understand what files/data it needs to produce.

### Use Cases

✅ Multiple output files per task
✅ Reading from workflow state
✅ Chaining task outputs
✅ Subflow inputs/outputs
✅ Complex data flow between tasks

## Comparison

| Feature | `output:` (Simple) | `outputs:` (Complex) |
|---------|-------------------|---------------------|
| **Type** | String path | Map of OutputSpec |
| **Count** | Single file | Multiple outputs |
| **Sources** | File only | File, State, TaskOutput |
| **Auto-injection** | No | Yes (to agent prompt) |
| **Use with** | Basic tasks | Complex workflows |
| **Named** | No | Yes |
| **Subflows** | No | Yes |

## Can You Use Both?

**Yes!** You can use both in the same task:

```yaml
tasks:
  complex_task:
    description: "Task with both output types"
    agent: "worker"
    output: "main_result.txt"  # Simple output
    outputs:  # Complex outputs
      details:
        source:
          type: file
          path: "./details.json"
      status:
        source:
          type: state
          key: "task_status"
```

However, this is rarely needed. Choose one based on your needs.

## Variable Interpolation

Both output types support variable interpolation:

### Simple Output
```yaml
inputs:
  project_name:
    type: string
    default: "myproject"

tasks:
  build:
    description: "Build project"
    agent: "builder"
    output: "${workflow.project_name}/build.log"  # Interpolated
```

### Complex Outputs
```yaml
inputs:
  output_dir:
    type: string
    default: "./results"

tasks:
  analyze:
    description: "Analyze data"
    agent: "analyst"
    outputs:
      report:
        source:
          type: file
          path: "${workflow.output_dir}/report.md"  # Interpolated
```

## Real-World Examples

### Example 1: Log Processing
```yaml
tasks:
  process_logs:
    description: "Process application logs"
    script:
      language: bash
      content: |
        grep ERROR app.log > errors.txt
        grep WARN app.log > warnings.txt
        wc -l errors.txt warnings.txt > summary.txt
    outputs:
      errors:
        source:
          type: file
          path: "./errors.txt"
        description: "Error log entries"

      warnings:
        source:
          type: file
          path: "./warnings.txt"
        description: "Warning log entries"

      summary:
        source:
          type: file
          path: "./summary.txt"
        description: "Log summary statistics"
```

### Example 2: Data Pipeline
```yaml
tasks:
  extract:
    description: "Extract data from source"
    agent: "extractor"
    output: "raw_data.csv"

  transform:
    description: "Transform extracted data"
    agent: "transformer"
    depends_on: [extract]
    outputs:
      cleaned_data:
        source:
          type: file
          path: "./cleaned_data.csv"

      validation_report:
        source:
          type: file
          path: "./validation.txt"

  load:
    description: "Load data into database"
    agent: "loader"
    depends_on: [transform]
    inputs:
      data_file: "./cleaned_data.csv"
    output: "load_log.txt"
```

### Example 3: Workflow with State
```yaml
tasks:
  initialize:
    description: "Initialize workflow state"
    script:
      language: bash
      content: |
        echo "initialized" > state.txt
    outputs:
      status:
        source:
          type: state
          key: "workflow_status"

  process:
    description: "Process data"
    agent: "processor"
    depends_on: [initialize]
    outputs:
      previous_status:
        source:
          type: task_output
          task: "initialize"  # Get status from initialize task
```

## Best Practices

### When to Use `output:`
- ✅ Single output file
- ✅ Simple workflows
- ✅ Script tasks with one result
- ✅ Quick prototypes

### When to Use `outputs:`
- ✅ Multiple output files
- ✅ Reading from workflow state
- ✅ Chaining task results
- ✅ Subflow integration
- ✅ Complex data flow
- ✅ Need auto-injection into agent prompts

### General Tips

1. **Be specific with paths**: Use relative paths from workflow CWD
2. **Document outputs**: Use `description` field
3. **Use variables**: Interpolate paths for flexibility
4. **Check with DoD**: Verify outputs with Definition of Done criteria

### Example with DoD
```yaml
tasks:
  generate_report:
    description: "Generate analysis report"
    agent: "analyst"
    output: "analysis_report.md"
    definition_of_done:
      criteria:
        - type: file_exists
          path: "analysis_report.md"
          description: "Report file must exist"

        - type: file_contains
          path: "analysis_report.md"
          pattern: "## Conclusion"
          description: "Report must have conclusion section"

      fail_on_unmet: true
```

## Summary

**Simple Output (`output:`):**
```yaml
output: "result.txt"
```

**Complex Outputs (`outputs:`):**
```yaml
outputs:
  result:
    source:
      type: file  # or state, task_output
      path: "./result.json"
    description: "Task result"
```

Choose based on your workflow complexity and data flow requirements!
