# Subflows - Composable Workflow Components

## Overview

Subflows are reusable workflow components that enable modularity and composition in the DSL. They allow you to:

- **Break down complex workflows** into manageable pieces
- **Reuse common patterns** across multiple workflows
- **Share workflows** via git repositories or HTTP
- **Version control** workflow components independently
- **Test workflows** in isolation before integration

## Features

### 1. Multiple Definition Methods

#### Inline Subflows
Define subflows directly in your main workflow file:

```yaml
subflows:
  validation:
    description: "Validate data quality"
    agents:
      validator:
        description: "Validation agent"
        tools: [Read, Bash]
    tasks:
      check:
        description: "Run checks"
        agent: "validator"
```

#### Local File References
Reference subflows from local files:

```yaml
subflows:
  testing:
    source:
      type: file
      path: "./subflows/testing.yaml"
```

#### Git Repository References
Fetch subflows from git repos with version pinning:

```yaml
subflows:
  deployment:
    source:
      type: git
      url: "https://github.com/org/workflows.git"
      path: "subflows/deploy.yaml"
      reference: "v1.0.0"  # tag, branch, or commit
```

#### HTTP References
Download subflows from HTTP/HTTPS URLs:

```yaml
subflows:
  monitoring:
    source:
      type: http
      url: "https://example.com/workflows/monitoring.yaml"
      checksum: "sha256:abc123..."  # optional integrity check
```

### 2. Input Parameters

Subflows can accept typed inputs with validation:

```yaml
subflows:
  process:
    inputs:
      data_file:
        type: "string"
        required: true
        description: "Path to data file"
      mode:
        type: "string"
        required: false
        default: "normal"
        description: "Processing mode"
```

### 3. Output Definitions

Subflows can declare outputs from multiple sources:

```yaml
subflows:
  process:
    outputs:
      result_file:
        source:
          type: file
          path: "result.json"
        description: "Processing result"

      state_value:
        source:
          type: state
          key: "final_state"

      task_output:
        source:
          type: task_output
          task: "process_task"
```

### 4. Task References

Tasks can execute subflows instead of agents:

```yaml
tasks:
  run_validation:
    description: "Execute validation subflow"
    subflow: "validation"  # Reference to subflow
    inputs:
      data_file: "./data.json"
      mode: "strict"
    depends_on: ["prepare"]
```

**Note**: Tasks must specify either `agent` OR `subflow`, not both.

## Usage

### Parsing Workflows with Subflows

```rust
use periplon_sdk::dsl::parse_workflow_with_subflows;

// Automatically resolves all subflow references
let workflow = parse_workflow_with_subflows("workflow.yaml").await?;
```

### Manual Subflow Resolution

```rust
use periplon_sdk::dsl::{parse_workflow_file, SubflowCache, fetch_subflow};

let mut workflow = parse_workflow_file("workflow.yaml")?;
let cache = SubflowCache::new();

// Fetch specific subflow
if let Some(subflow_spec) = workflow.subflows.get("deployment") {
    if let Some(source) = &subflow_spec.source {
        let subflow = fetch_subflow(source, Some(Path::new(".")), &cache).await?;
        // Use the fetched subflow...
    }
}
```

### Merging Subflows Inline

```rust
use periplon_sdk::dsl::merge_subflow_inline;

// Merge a subflow into the main workflow with namespacing
merge_subflow_inline(&mut workflow, "validation")?;

// Agents and tasks are now prefixed: "validation.validator", "validation.check"
```

## Validation

The DSL validator ensures:

- ✅ **Subflow references exist**: Tasks reference defined subflows
- ✅ **Required inputs provided**: All required inputs are supplied
- ✅ **No circular dependencies**: Subflows don't reference each other in cycles
- ✅ **Mutual exclusivity**: Tasks have either `agent` OR `subflow`, not both
- ✅ **Valid agent references**: Agents within subflows exist
- ✅ **Tool validation**: Tools used by subflow agents are valid
- ✅ **Permission validation**: Permission modes are correct

## Caching

Remote subflows are cached automatically to avoid redundant network requests:

```rust
let cache = SubflowCache::new();

// First fetch - downloads from remote
let subflow1 = fetch_subflow(&source, None, &cache).await?;

// Second fetch - uses cache
let subflow2 = fetch_subflow(&source, None, &cache).await?;
```

## Examples

### CI/CD Pipeline

```yaml
name: "CI/CD Pipeline"
version: "1.0.0"

subflows:
  # Inline build subflow
  build:
    agents:
      builder:
        description: "Builds the application"
        tools: [Read, Bash]
    tasks:
      compile:
        description: "Compile code"
        agent: "builder"
    inputs:
      build_type:
        type: "string"
        default: "release"

  # External validation subflow
  validation:
    source:
      type: file
      path: "./validation.yaml"

  # Git-based deployment subflow
  deploy:
    source:
      type: git
      url: "https://github.com/org/workflows.git"
      path: "deploy.yaml"
      reference: "v1.0.0"

tasks:
  build_project:
    subflow: "build"
    inputs:
      build_type: "release"

  validate_code:
    subflow: "validation"
    depends_on: ["build_project"]

  deploy_staging:
    subflow: "deploy"
    depends_on: ["validate_code"]
```

## Best Practices

1. **Keep subflows focused**: Each subflow should have a single, clear purpose
2. **Define clear interfaces**: Use inputs/outputs to make dependencies explicit
3. **Version remote subflows**: Pin to specific versions for reproducibility
4. **Use checksums for HTTP**: Verify integrity of downloaded workflows
5. **Document inputs/outputs**: Add descriptions to all parameters
6. **Test independently**: Validate subflows work standalone before integration
7. **Namespace appropriately**: Use meaningful subflow names to avoid conflicts

## Architecture

### Components

- **`src/dsl/schema.rs`**: Subflow type definitions
- **`src/dsl/fetcher.rs`**: Remote subflow fetching with caching
- **`src/dsl/parser.rs`**: Subflow resolution and merging
- **`src/dsl/validator.rs`**: Subflow validation logic
- **`src/dsl/template.rs`**: Template generation with subflow examples

### Execution Flow

1. Parse main workflow YAML
2. Identify subflow references
3. Fetch external subflows (local, git, HTTP)
4. Recursively resolve nested subflows
5. Detect circular dependencies
6. Validate all subflow references
7. Merge or execute subflows as needed

## API Reference

### Types

- `SubflowSpec`: Subflow definition with agents, tasks, inputs, outputs
- `SubflowSource`: Source location (file, git, HTTP)
- `InputSpec`: Input parameter specification
- `OutputSpec`: Output specification
- `OutputDataSource`: Output source (file, state, task_output)
- `SubflowCache`: Caching layer for remote subflows

### Functions

- `fetch_subflow()`: Fetch a subflow from any source
- `parse_workflow_with_subflows()`: Parse workflow and resolve all subflows
- `merge_subflow_inline()`: Merge subflow into main workflow with namespacing
- `validate_workflow()`: Validate workflow including subflow references

## Limitations

- **No runtime parameters**: Inputs are defined at workflow parse time
- **Git requires shell**: Git fetching uses `git clone` command
- **HTTP requires network**: No offline mode for HTTP subflows
- **Checksum format**: Only SHA-256 checksums supported
- **Synchronous fetching**: Subflows fetched sequentially, not in parallel

## Future Enhancements

- [ ] Parallel subflow fetching
- [ ] HTTP caching with ETags
- [ ] Subflow registry/marketplace
- [ ] Runtime input binding
- [ ] Conditional subflow loading
- [ ] Subflow output consumption
- [ ] Encrypted subflow sources
