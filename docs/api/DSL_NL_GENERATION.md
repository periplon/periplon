# DSL Natural Language Generation

This document describes the natural language to DSL generation features added to the DSL Executor CLI.

## Overview

The DSL Executor now supports three powerful features:

1. **DSL Grammar Versioning** - Track DSL syntax versions for compatibility
2. **Template Generation** - Auto-generate comprehensive DSL templates with documentation
3. **Natural Language Generation** - Convert natural language descriptions into DSL workflows

## DSL Grammar Version

The DSL now includes a version field (`dsl_version`) that tracks the grammar/syntax version separately from individual workflow versions. This enables:

- **Breaking Change Management**: Major version bumps (e.g., 1.0.0 → 2.0.0) indicate breaking syntax changes
- **Compatibility Checking**: Tools can verify they support the DSL version
- **Future-Proofing**: Smooth migration paths when the DSL evolves

### Current Version

The current DSL grammar version is **1.0.0**.

### Usage in Workflows

```yaml
name: "My Workflow"
version: "1.0.0"
dsl_version: "1.0.0"  # Optional, defaults to current version

agents:
  # ...
```

## Template Generation

Generate a comprehensive DSL template with all available options and documentation.

### Command

```bash
# Print template to stdout
periplon-executor template

# Save template to file
periplon-executor template -o my_template.yaml
```

### Features

The generated template includes:

- **Complete Coverage**: All DSL fields and options
- **Documentation**: Inline comments explaining each field
- **Type Information**: Data types and expected formats
- **Examples**: Real-world usage examples
- **Required vs Optional**: Clear marking of mandatory fields
- **Auto-Generated**: Always reflects the current DSL schema

### Example Output

```yaml
# Agentic AI DSL Template
# DSL Grammar Version: 1.0.0
#
# This template shows all available DSL options with documentation.
# Required fields are marked with (REQUIRED)
# Optional fields are marked with (optional)
#

# (REQUIRED) Workflow name - identifies this workflow
name: "My Workflow"

# (REQUIRED) Workflow version - semantic versioning (e.g., 1.0.0)
version: "1.0.0"

# (optional) DSL grammar version - tracks DSL syntax version
# Defaults to current version: 1.0.0
dsl_version: "1.0.0"

# Agent definitions
agents:
  example_agent:
    # (REQUIRED) Description of what this agent does
    description: "Performs research and analysis"

    # (optional) Model to use (e.g., claude-sonnet-4-5, claude-opus-4)
    model: "claude-sonnet-4-5"

    # (optional) Tools this agent can use
    tools:
      - Read
      - Write
      - WebSearch

    # Permission settings
    permissions:
      mode: "default"  # default, acceptEdits, plan, bypassPermissions
```

## Natural Language Generation

Convert natural language descriptions into valid DSL workflows using AI.

### Command

```bash
# Generate workflow from description
periplon-executor generate "Create a workflow to research Rust async programming and write a tutorial" -o research_workflow.yaml

# With verbose output
periplon-executor generate "Build a web scraper that extracts article titles" -o scraper.yaml --verbose
```

### How It Works

1. **Analysis**: The AI analyzes your natural language description
2. **Structure Identification**: Identifies agents, tasks, dependencies, and tools
3. **DSL Generation**: Creates valid YAML following the DSL schema
4. **Validation**: Automatically validates the generated workflow
5. **Output**: Saves the workflow to the specified file

### Examples

#### Simple Research Workflow

**Input:**
```bash
periplon-executor generate "Research a topic and write a report" -o research.yaml
```

**Output (research.yaml):**
```yaml
name: "Research and Report"
version: "1.0.0"
dsl_version: "1.0.0"

agents:
  researcher:
    description: "Research information on the topic"
    tools:
      - WebSearch
      - WebFetch
      - Read
    permissions:
      mode: "default"

  writer:
    description: "Write and format the report"
    tools:
      - Read
      - Write
    permissions:
      mode: "acceptEdits"

tasks:
  research:
    description: "Research the topic and gather information"
    agent: "researcher"
    output: "research_findings.md"

  write_report:
    description: "Write a comprehensive report based on research"
    agent: "writer"
    depends_on:
      - research
    output: "final_report.md"
```

#### Complex Multi-Agent System

**Input:**
```bash
periplon-executor generate "Build a code review system with separate agents for linting, testing, security scanning, and generating a final report" -o code_review.yaml --verbose
```

The AI will create a workflow with:
- Multiple specialized agents (linter, tester, security_scanner, reporter)
- Parallel task execution where possible
- Proper dependency management
- Appropriate tools for each agent
- Error handling with retries

### Best Practices

1. **Be Specific**: More detail in your description leads to better results
   - Good: "Research Python async/await patterns, write example code, and create documentation"
   - Poor: "Make a Python thing"

2. **Mention Key Aspects**:
   - What agents/roles are needed
   - What tools they should use
   - Task dependencies and order
   - Output requirements

3. **Review and Refine**: Always review the generated workflow
   - Check agent permissions match your security requirements
   - Verify task dependencies are correct
   - Adjust tool selections if needed

4. **Iterative Approach**: Generate, test, refine
   ```bash
   # Generate initial version
   periplon-executor generate "..." -o workflow.yaml

   # Validate it
   periplon-executor validate workflow.yaml

   # Test run
   periplon-executor run workflow.yaml --dry-run

   # Execute
   periplon-executor run workflow.yaml
   ```

## Version Command

Check the DSL grammar version and compatibility information.

### Command

```bash
periplon-executor version
```

### Output

```
DSL Grammar Information
============================================================

  Version: 1.0.0
  Compatibility: Compatible with workflows using DSL v1.0.0

  Tip: Use 'periplon-executor template' to see all available options
  Tip: Use 'periplon-executor generate' to create workflows from natural language
```

## Integration with Existing Commands

All existing DSL Executor commands continue to work:

```bash
# Run a workflow
periplon-executor run workflow.yaml

# Validate a workflow
periplon-executor validate workflow.yaml

# List saved states
periplon-executor list

# Show workflow status
periplon-executor status "My Workflow"

# Clean saved states
periplon-executor clean
```

## Advanced Usage

### Custom System Prompts

The natural language generator uses a system prompt that includes the complete DSL grammar. You can extend this by:

1. Using the template as a reference guide
2. Providing more detailed natural language descriptions
3. Iterating on generated workflows

### Version Compatibility

When the DSL grammar evolves:

1. **Patch versions (1.0.x)**: Bug fixes, no breaking changes
2. **Minor versions (1.x.0)**: New optional features, backward compatible
3. **Major versions (x.0.0)**: Breaking changes, migration required

Tools can check `dsl_version` to ensure compatibility:

```rust
use periplon_sdk::dsl::{parse_workflow_file, DSL_GRAMMAR_VERSION};

let workflow = parse_workflow_file("workflow.yaml")?;

if workflow.dsl_version != DSL_GRAMMAR_VERSION {
    println!("Warning: Workflow uses DSL v{}, current version is v{}",
             workflow.dsl_version, DSL_GRAMMAR_VERSION);
}
```

## Troubleshooting

### Generated Workflow Invalid

If the generated workflow fails validation:

1. **Check the error message**: Validation errors are specific
2. **Use verbose mode**: See the generation steps
3. **Refine your description**: Be more specific about requirements
4. **Manual adjustment**: Edit the generated YAML

### Natural Language Generator Not Working

Requirements:
- CLI must be installed (`npm install -g @anthropic-ai/claude-code`)
- API credentials must be configured
- Internet connection required

### Template Missing Fields

If you notice missing fields in the template, this is a bug. The template is auto-generated from the schema and should always be complete. Please report issues at the project repository.

## Examples Gallery

### Data Processing Pipeline

```bash
periplon-executor generate "Create a data pipeline that fetches data from an API, validates it, transforms it, and stores it in a database" -o data_pipeline.yaml
```

### Testing Workflow

```bash
periplon-executor generate "Set up a testing workflow with unit tests, integration tests, and E2E tests running in parallel, then generate a coverage report" -o testing.yaml
```

### Documentation Generator

```bash
periplon-executor generate "Analyze code files, extract API documentation, generate markdown files, and create a documentation website" -o docs_gen.yaml
```

### CI/CD Pipeline

```bash
periplon-executor generate "Build a CI/CD pipeline that runs tests, builds Docker images, performs security scans, and deploys to staging" -o cicd.yaml
```

## API Reference

For programmatic access:

```rust
use periplon_sdk::dsl::{
    generate_from_nl,
    generate_template,
    DSL_GRAMMAR_VERSION,
};

// Generate template
let template = generate_template();
println!("{}", template);

// Generate from natural language
let workflow = generate_from_nl(
    "Create a research workflow",
    None  // Use default options
).await?;

// Check version
println!("DSL Version: {}", DSL_GRAMMAR_VERSION);
```

## Conclusion

These features make DSL workflows more accessible:

- **Template Generation**: Perfect for learning and reference
- **Natural Language**: Rapid prototyping and iteration
- **Versioning**: Long-term maintainability

Start experimenting:

```bash
# Learn the DSL
periplon-executor template > reference.yaml

# Generate your first workflow
periplon-executor generate "Your idea here" -o my_workflow.yaml

# Validate and run
periplon-executor validate my_workflow.yaml
periplon-executor run my_workflow.yaml
```
