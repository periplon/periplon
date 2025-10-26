# DSL Quick Start Guide

Welcome to the Agentic AI DSL! This guide will help you create your first multi-agent workflow in just a few minutes.

## What is the DSL?

The Domain-Specific Language (DSL) for periplon allows you to define complex multi-agent workflows using simple YAML files. Key features include:

- 🌲 **Hierarchical Tasks**: Nested subtasks with automatic dependency management
- ⚡ **Parallel Execution**: True concurrent task execution
- 🤖 **Multi-Agent**: Multiple specialized agents working together
- ✅ **Type-Safe**: Compile-time validation via Rust
- 📊 **Dependency Resolution**: Automatic topological sorting

## 5-Minute Tutorial

### Step 1: Create a Simple Workflow

Create a file `my_workflow.yaml`:

```yaml
name: "My First Workflow"
version: "1.0.0"

agents:
  researcher:
    description: "Research specialist"
    model: "claude-sonnet-4-5"
    tools:
      - Read
      - WebSearch
    permissions:
      mode: "default"

tasks:
  research_topic:
    description: "Research AI and machine learning trends"
    agent: "researcher"
```

### Step 2: Run the Workflow

```rust
use periplon_sdk::{parse_workflow_file, validate_workflow, DSLExecutor};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load and validate
    let workflow = parse_workflow_file("my_workflow.yaml")?;
    validate_workflow(&workflow)?;

    // Execute
    let mut executor = DSLExecutor::new(workflow)?;
    executor.initialize().await?;
    executor.execute().await?;
    executor.shutdown().await?;

    Ok(())
}
```

## Hierarchical Tasks Example

Create complex workflows with nested subtasks:

```yaml
name: "Project Builder"
version: "1.0.0"

agents:
  architect:
    description: "System architect"
    tools: [Read, Write]
    permissions:
      mode: "acceptEdits"

  coder:
    description: "Code implementer"
    tools: [Read, Write, Edit, Bash]
    permissions:
      mode: "acceptEdits"

tasks:
  build_project:
    description: "Build complete project"
    agent: "architect"

    subtasks:
      - design_architecture:
          description: "Design system architecture"
          agent: "architect"
          output: "architecture.md"

      - implement_backend:
          description: "Implement backend"
          agent: "coder"
          depends_on:
            - design_architecture

          subtasks:
            - setup_database:
                description: "Set up database schema"
                agent: "coder"

            - create_api:
                description: "Create REST API"
                agent: "coder"
                depends_on:
                  - setup_database
```

**How it works:**
1. Tasks are flattened: `build_project`, `build_project.design_architecture`, `build_project.implement_backend`, etc.
2. Subtasks automatically depend on their parent
3. Dependencies are resolved using topological sort
4. Execution follows the dependency order

## Parallel Execution Example

Run tasks concurrently using `parallel_with`:

```yaml
tasks:
  build_app:
    description: "Build application"

    subtasks:
      - build_backend:
          description: "Build backend services"
          agent: "backend_dev"

      - build_frontend:
          description: "Build user interface"
          agent: "frontend_dev"
          parallel_with:
            - build_backend  # These run concurrently!

      - run_tests:
          description: "Run all tests"
          agent: "qa_engineer"
          depends_on:
            - build_backend
            - build_frontend
```

**How it works:**
1. `build_backend` and `build_frontend` execute in parallel using tokio::spawn
2. `run_tests` waits for both to complete
3. Thread-safe state management with Arc<Mutex<>>

## Agent Configuration

### Basic Agent

```yaml
agents:
  simple_agent:
    description: "A simple agent"
    tools:
      - Read
      - Write
```

### Advanced Agent

```yaml
agents:
  advanced_agent:
    description: "Advanced agent with all options"
    model: "claude-sonnet-4-5"
    system_prompt: |
      You are an expert software engineer.
      Write clean, well-documented code.
    tools:
      - Read
      - Write
      - Edit
      - Bash
      - Grep
      - Glob
    permissions:
      mode: "acceptEdits"
      allowed_directories:
        - "./src"
        - "./tests"
    max_turns: 20
```

## Task Dependencies

### Simple Dependency

```yaml
tasks:
  task1:
    description: "First task"
    agent: "agent1"

  task2:
    description: "Second task"
    agent: "agent2"
    depends_on:
      - task1  # Runs after task1 completes
```

### Complex Dependencies

```yaml
tasks:
  task1:
    description: "Independent task 1"

  task2:
    description: "Independent task 2"

  task3:
    description: "Depends on both"
    depends_on:
      - task1
      - task2

  task4:
    description: "Depends on task3"
    depends_on:
      - task3
```

**Execution order:** `task1` and `task2` run first (can be parallel), then `task3`, then `task4`

## Validation

The DSL automatically validates:

✅ All agent references exist
✅ No circular dependencies
✅ All tools are valid
✅ Permission modes are correct
✅ Task dependencies exist

```rust
use periplon_sdk::{parse_workflow, validate_workflow};

let yaml = r#"
name: "Test"
version: "1.0.0"

agents:
  agent1:
    tools:
      - InvalidTool  # ❌ This will fail validation!
"#;

let workflow = parse_workflow(yaml)?;
let result = validate_workflow(&workflow);
// Result will be Err with detailed error message
```

## Common Patterns

### Pattern 1: Sequential Pipeline

```yaml
tasks:
  pipeline:
    subtasks:
      - step1:
          description: "Collect data"
      - step2:
          description: "Process data"
          depends_on: [step1]
      - step3:
          description: "Generate report"
          depends_on: [step2]
```

### Pattern 2: Parallel Fan-out, Serial Fan-in

```yaml
tasks:
  parallel_work:
    subtasks:
      - prepare:
          description: "Prepare work"

      - work1:
          description: "Parallel work 1"
          depends_on: [prepare]

      - work2:
          description: "Parallel work 2"
          depends_on: [prepare]
          parallel_with: [work1]

      - work3:
          description: "Parallel work 3"
          depends_on: [prepare]
          parallel_with: [work1, work2]

      - combine:
          description: "Combine results"
          depends_on: [work1, work2, work3]
```

### Pattern 3: Map-Reduce

```yaml
tasks:
  map_reduce:
    subtasks:
      - map_task1:
          description: "Process partition 1"
          parallel_with: [map_task2, map_task3]

      - map_task2:
          description: "Process partition 2"

      - map_task3:
          description: "Process partition 3"

      - reduce:
          description: "Combine results"
          depends_on: [map_task1, map_task2, map_task3]
```

## Examples

Check out the `examples/dsl/` directory for complete examples:

1. **simple_file_organizer.yaml** - Basic file organization with hierarchical tasks
2. **research_pipeline.yaml** - Research and documentation with parallel tasks
3. **data_pipeline.yaml** - Data processing with sequential stages

Run the example parser:

```bash
cargo run --example test_dsl_parser
```

## API Reference

### Main Functions

```rust
// Parse YAML string
pub fn parse_workflow(yaml: &str) -> Result<DSLWorkflow>

// Parse YAML file
pub fn parse_workflow_file<P: AsRef<Path>>(path: P) -> Result<DSLWorkflow>

// Validate workflow
pub fn validate_workflow(workflow: &DSLWorkflow) -> Result<()>

// Execute workflow
let mut executor = DSLExecutor::new(workflow)?;
executor.initialize().await?;
executor.execute().await?;
executor.shutdown().await?;
```

### Key Types

- `DSLWorkflow`: Root workflow structure
- `AgentSpec`: Agent configuration
- `TaskSpec`: Task definition
- `DSLExecutor`: Execution engine

## Troubleshooting

### "Agent not found" error

Make sure all agents referenced in tasks are defined:

```yaml
agents:
  my_agent:  # ✅ Defined
    description: "My agent"

tasks:
  my_task:
    agent: "my_agent"  # ✅ Matches
```

### "Circular dependency detected"

Check your `depends_on` relationships for cycles:

```yaml
# ❌ BAD: Circular dependency
task1:
  depends_on: [task2]
task2:
  depends_on: [task1]

# ✅ GOOD: Linear dependency
task1:
  description: "First"
task2:
  description: "Second"
  depends_on: [task1]
```

### "Invalid tool" error

Use only supported tools:

✅ Valid: `Read`, `Write`, `Edit`, `Bash`, `Grep`, `Glob`, `WebSearch`, `WebFetch`, `Task`, `TodoWrite`, `Skill`, `SlashCommand`

```yaml
agents:
  agent1:
    tools:
      - Read      # ✅ Valid
      - MyTool    # ❌ Invalid
```

## Next Steps

1. ✅ Read the [DSL Implementation Guide](DSL_IMPLEMENTATION.md) for detailed architecture
2. ⚡ Try the examples in `examples/dsl/`
3. 🚀 Build your own multi-agent workflows
4. 📖 Check [dsl-plan.md](dsl-plan.md) for the complete roadmap

## Need Help?

- Check the examples: `examples/dsl/*.yaml`
- Read the tests: `tests/hierarchical_tests.rs`
- Review the implementation: `DSL_IMPLEMENTATION.md`

Happy building! 🎉
