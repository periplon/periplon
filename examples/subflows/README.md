# Subflow Examples

This directory contains examples demonstrating the subflow feature in the DSL.

## Overview

Subflows allow you to create reusable, composable workflow components. They support:

- **Inline Definition**: Define subflows directly in your main workflow file
- **Local Files**: Reference subflows from local YAML files
- **Git Repositories**: Fetch subflows from git repos with version pinning
- **HTTP Sources**: Download subflows from HTTP/HTTPS URLs with optional checksums

## Files

### `validation.yaml`
A reusable validation subflow that can be referenced by other workflows. It performs:
- Code syntax checking
- Linting
- Test execution

### `main_workflow.yaml`
Demonstrates a complete CI/CD pipeline using multiple subflows:
- **Inline build subflow** - Compiles and packages the application
- **External validation subflow** - References `validation.yaml`
- **Inline deployment subflow** - Deploys to staging/production

## Key Features Demonstrated

### 1. Inline Subflows
```yaml
subflows:
  build:
    description: "Build the project"
    agents:
      builder:
        description: "Builds the application"
        tools: [Read, Bash]
    tasks:
      compile:
        description: "Compile source code"
        agent: "builder"
```

### 2. External File References
```yaml
subflows:
  validation:
    description: "Validate code quality"
    source:
      type: file
      path: "./validation.yaml"
```

### 3. Subflow Inputs
```yaml
subflows:
  deploy:
    inputs:
      environment:
        type: "string"
        required: true
        description: "Target environment"
```

### 4. Referencing Subflows in Tasks
```yaml
tasks:
  build_project:
    description: "Execute build subflow"
    subflow: "build"
    inputs:
      build_type: "release"
```

## Usage

To run the main workflow with subflow resolution:

```bash
# Using the DSL executor
cargo run --bin dsl-executor -- run examples/subflows/main_workflow.yaml

# Or using the Rust API
use claude_agent_sdk::dsl::parse_workflow_with_subflows;

let workflow = parse_workflow_with_subflows("examples/subflows/main_workflow.yaml").await?;
```

## Remote Subflows

You can also reference subflows from remote sources:

### Git Repository
```yaml
subflows:
  deployment:
    source:
      type: git
      url: "https://github.com/org/workflows.git"
      path: "subflows/deploy.yaml"
      reference: "v1.0.0"  # tag, branch, or commit
```

### HTTP URL
```yaml
subflows:
  monitoring:
    source:
      type: http
      url: "https://example.com/workflows/monitoring.yaml"
      checksum: "sha256:abc123..."  # optional integrity check
```

## Benefits

1. **Reusability**: Define common workflows once, use them everywhere
2. **Modularity**: Break complex workflows into manageable pieces
3. **Versioning**: Pin subflows to specific versions via git tags
4. **Sharing**: Share subflows across teams via git or HTTP
5. **Composition**: Build complex pipelines by combining simple subflows
6. **Testing**: Test subflows independently before integration

## Validation

The DSL validator ensures:
- ✅ Subflow references are valid
- ✅ Required inputs are provided
- ✅ No circular subflow dependencies
- ✅ Agent and task references within subflows are correct
- ✅ Tasks specify either `agent` OR `subflow` (mutually exclusive)
