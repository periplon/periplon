# DSL Workflow System Overview

## Introduction

The Periplon DSL (Domain-Specific Language) is a powerful YAML-based system for creating multi-agent AI workflows. It allows you to orchestrate complex agent interactions with minimal code.

## Quick Start

```bash
# Build the executor
cargo build --release --bin periplon-executor

# Generate a template to learn the DSL
./target/release/periplon-executor template > my_template.yaml

# Generate a workflow from natural language
./target/release/periplon-executor generate "Research a topic and write a report" -o research.yaml

# Validate the workflow
./target/release/periplon-executor validate research.yaml

# Run the workflow
./target/release/periplon-executor run research.yaml
```

## Basic Workflow Structure

```yaml
name: "My Workflow"
version: "1.0.0"
description: "Optional description"

agents:
  agent_id:
    description: "What this agent does"
    model: "claude-sonnet-4-5"  # optional
    tools: [Read, Write, WebSearch]
    permissions:
      mode: "acceptEdits"

tasks:
  task_id:
    description: "Task description"
    agent: "agent_id"
    depends_on: [other_task_id]  # optional
    subtasks: [child_task_id]    # optional
    output: "output.md"           # optional
```

## Key Features

### 1. Multi-Agent Collaboration

Define multiple specialized agents:

```yaml
agents:
  researcher:
    description: "Research information on topics"
    tools: [WebSearch, Read]

  writer:
    description: "Write comprehensive reports"
    tools: [Write, Read]

  reviewer:
    description: "Review and improve content"
    tools: [Read]
```

### 2. Task Dependencies

Create execution order with dependencies:

```yaml
tasks:
  research:
    description: "Gather information"
    agent: "researcher"

  write:
    description: "Create report"
    agent: "writer"
    depends_on: [research]

  review:
    description: "Review report"
    agent: "reviewer"
    depends_on: [write]
```

### 3. Hierarchical Tasks

Organize complex workflows with subtasks:

```yaml
tasks:
  project:
    description: "Complete project"
    subtasks: [planning, implementation, testing]

  planning:
    description: "Plan the work"
    agent: "planner"

  implementation:
    description: "Implement features"
    agent: "developer"

  testing:
    description: "Test implementation"
    agent: "tester"
```

### 4. Variable System

Use scoped variables with interpolation:

```yaml
inputs:
  project_name:
    type: string
    required: true
    default: "MyProject"

agents:
  researcher:
    description: "Research ${workflow.project_name}"
    inputs:
      api_key:
        type: string
        required: true

tasks:
  analyze:
    description: "Analyze project ${workflow.project_name}"
    outputs:
      result:
        source:
          type: file
          path: "./analysis.json"
```

### 5. State Management

Resume workflows from checkpoints:

```bash
# Run workflow
./target/release/periplon-executor run workflow.yaml

# If interrupted, resume from last checkpoint
./target/release/periplon-executor run workflow.yaml --resume
```

### 6. Loop System

Process collections and implement retry logic:

```yaml
tasks:
  process_items:
    description: "Process item {{item.name}}"
    agent: "processor"
    loop:
      type: for_each
      collection:
        source: file
        path: "items.json"
      iterator: "item"
      parallel: true
      max_parallel: 5
```

## CLI Commands

### Template Generation

```bash
# Generate template with all features
./target/release/periplon-executor template > template.yaml

# View template
cat template.yaml
```

### Natural Language Generation

```bash
# Generate from description
./target/release/periplon-executor generate "Analyze codebase and create documentation" -o analyze.yaml

# Specify model
./target/release/periplon-executor generate "Build ML pipeline" -o ml.yaml --model claude-sonnet-4-5
```

### Validation

```bash
# Validate workflow syntax and semantics
./target/release/periplon-executor validate workflow.yaml

# Validate with detailed output
./target/release/periplon-executor validate workflow.yaml --verbose
```

### Execution

```bash
# Run workflow
./target/release/periplon-executor run workflow.yaml

# Run with state persistence
./target/release/periplon-executor run workflow.yaml --state-file state.json

# Resume from checkpoint
./target/release/periplon-executor run workflow.yaml --resume --state-file state.json

# Run specific task only
./target/release/periplon-executor run workflow.yaml --task analyze
```

## Example Workflows

### Simple Research and Report

```yaml
name: "Research and Report"
version: "1.0.0"

agents:
  researcher:
    description: "Research information on topics"
    tools: [WebSearch, Read, Write]
    permissions:
      mode: "default"

  writer:
    description: "Write comprehensive reports"
    tools: [Read, Write]
    permissions:
      mode: "acceptEdits"

tasks:
  research:
    description: "Research the topic thoroughly"
    agent: "researcher"
    output: "findings.md"

  write_report:
    description: "Create final report from research"
    agent: "writer"
    depends_on: [research]
    output: "report.md"
```

### Code Analysis Pipeline

```yaml
name: "Code Analysis"
version: "1.0.0"

agents:
  analyzer:
    description: "Analyze code quality and patterns"
    tools: [Read, Grep, WebSearch]

  documenter:
    description: "Generate documentation"
    tools: [Read, Write]

tasks:
  analyze_structure:
    description: "Analyze codebase structure"
    agent: "analyzer"
    output: "structure.json"

  analyze_patterns:
    description: "Identify code patterns"
    agent: "analyzer"
    depends_on: [analyze_structure]
    output: "patterns.md"

  generate_docs:
    description: "Create documentation"
    agent: "documenter"
    depends_on: [analyze_structure, analyze_patterns]
    output: "README.md"
```

### API Processing with Loops

```yaml
name: "API Data Processing"
version: "1.0.0"

agents:
  processor:
    description: "Process API data"
    tools: [Read, Write]

tasks:
  fetch_and_process:
    description: "Processing user {{user.name}}"
    agent: "processor"
    loop:
      type: for_each
      collection:
        source: http
        url: "https://api.example.com/users"
        method: "GET"
        format: json
      iterator: "user"
      parallel: true
      max_parallel: 10
    loop_control:
      collect_results: true
      result_key: "processed_users"
```

## Advanced Features

### Notification System

Send notifications during workflow execution:

```yaml
notifications:
  - channel: ntfy
    topic: "workflow-updates"
    events: [on_complete, on_error]

  - channel: slack
    webhook: "${env.SLACK_WEBHOOK}"
    events: [on_error]
```

### Conditional Execution

Use conditions for dynamic workflow:

```yaml
tasks:
  deploy:
    description: "Deploy to production"
    agent: "deployer"
    condition:
      type: state_equals
      key: "tests_passed"
      value: true
```

### Parallel Execution

Execute tasks in parallel:

```yaml
tasks:
  test_unit:
    description: "Run unit tests"
    agent: "tester"

  test_integration:
    description: "Run integration tests"
    agent: "tester"

  lint:
    description: "Run linter"
    agent: "linter"

  # All three run in parallel (no dependencies)
```

## Documentation

- **[Loop Patterns Guide](../loop-patterns.md)** - Comprehensive loop reference
- **[Loop Cookbook](../loop-cookbook.md)** - 25 production-ready patterns
- **[DSL Implementation Details](../DSL_IMPLEMENTATION.md)** - Technical details
- **[Natural Language Generation](../DSL_NL_GENERATION.md)** - NL workflow generation

## Examples

Located in `examples/dsl_workflows/`:

- `research_report.yaml` - Basic research workflow
- `code_analysis.yaml` - Code analysis pipeline
- `api_processing.yaml` - API data processing
- `ml_pipeline.yaml` - Machine learning workflow

## Next Steps

1. Generate a template: `periplon-executor template`
2. Try natural language generation: `periplon-executor generate "your task"`
3. Validate your workflow: `periplon-executor validate workflow.yaml`
4. Run your first workflow: `periplon-executor run workflow.yaml`
5. Explore loop patterns in the cookbook
6. Check out the TUI for visual workflow management
