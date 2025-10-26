# Task Groups Module

Collections of related predefined tasks that work together as cohesive units.

## Overview

The Task Groups module enables bundling multiple predefined tasks into reusable suites, such as:

- **Integration Suites**: Tasks for API integrations (e.g., Google Workspace suite)
- **Feature Bundles**: Related functionality (e.g., data pipeline tasks)
- **Multi-Step Workflows**: Pre-configured task sequences (e.g., backup workflows)

## Module Structure

```
groups/
├── mod.rs          # Module exports
├── schema.rs       # Type definitions (TaskGroup, SharedConfig, etc.)
├── parser.rs       # YAML parsing for .taskgroup.yaml files
└── loader.rs       # Discovery, loading, and resolution
```

## Key Components

### Schema (`schema.rs`)

Defines the structure of task groups:

- **`TaskGroup`**: Complete group definition
- **`TaskGroupSpec`**: Tasks, shared config, workflows, dependencies
- **`SharedConfig`**: Configuration applied to all tasks
- **`TaskGroupReference`**: References to groups (`"name@version"`)

### Parser (`parser.rs`)

Parses `.taskgroup.yaml` files into structured data:

- Validates YAML format
- Deserializes to `TaskGroup` structs
- Provides detailed error messages

### Loader (`loader.rs`)

Discovers and resolves task groups:

- **Discovery**: Finds groups in search paths
- **Loading**: Parses group definitions
- **Resolution**: Loads all referenced tasks
- **Configuration**: Applies shared config
- **Caching**: Performance optimization

## Quick Start

### Define a Task Group

Create `.claude/task-groups/my-suite.taskgroup.yaml`:

```yaml
apiVersion: "taskgroup/v1"
kind: "TaskGroup"

metadata:
  name: "my-suite"
  version: "1.0.0"
  description: "My task suite"
  tags: ["integration", "api"]

spec:
  # Tasks in the group
  tasks:
    - name: "task1"
      version: "1.0.0"
      required: true
    - name: "task2"
      version: "2.0.0"
      required: false

  # Shared configuration for all tasks
  shared_config:
    inputs:
      api_key:
        type: string
        required: true
    permissions:
      mode: "acceptEdits"
    max_turns: 10
```

### Load a Task Group

```rust
use claude_agent_sdk::dsl::predefined_tasks::{
    TaskGroupLoader, TaskGroupReference
};

// Create loader
let mut loader = TaskGroupLoader::new();

// Load group
let group_ref = TaskGroupReference::parse("my-suite@1.0.0")?;
let resolved = loader.load(&group_ref)?;

// Access tasks
for name in resolved.task_names() {
    let task = resolved.get_task(&name).unwrap();
    println!("Task: {} v{}", task.metadata.name, task.metadata.version);
}
```

## Features

### Discovery

- Automatic discovery from `.claude/task-groups/`
- User global (`~/.claude/task-groups/`) and project local
- Priority-based resolution
- Custom search paths

### Resolution

- Loads all referenced tasks
- Validates version constraints
- Checks required tasks exist
- Applies shared configuration

### Shared Configuration

Apply common settings to all tasks:

- **Inputs**: Shared input parameters
- **Permissions**: Default permission mode
- **Environment**: Shared environment variables
- **Max Turns**: Default turn limit

Task-specific configuration takes precedence.

### Caching

- In-memory cache for resolved groups
- Leverages task loader cache
- Configurable cache management

## File Format

Task groups use the `.taskgroup.yaml` extension:

```yaml
apiVersion: "taskgroup/v1"    # API version
kind: "TaskGroup"             # Resource type

metadata:
  name: string                # Unique identifier
  version: string             # Semantic version
  description: string         # Optional description
  author: string              # Optional author
  tags: [string]              # Optional tags

spec:
  tasks:                      # Tasks in the group
    - name: string            # Task name
      version: string         # Task version
      required: bool          # Is required?
      description: string     # Optional description

  shared_config:              # Optional shared config
    inputs: {...}             # Shared inputs
    permissions: {...}        # Shared permissions
    environment: {...}        # Shared env vars
    max_turns: number         # Shared max turns

  workflows:                  # Optional pre-built workflows
    - name: string
      description: string
      tasks: {...}
      inputs: {...}
      outputs: {...}

  dependencies:               # Optional dependencies
    - name: string
      version: string
      optional: bool
```

## Directory Structure

Expected layout:

```
.claude/
├── tasks/              # Individual tasks (TaskLoader)
│   ├── task1.task.yaml
│   └── task2.task.yaml
│
└── task-groups/        # Task groups (TaskGroupLoader)
    ├── my-suite.taskgroup.yaml
    └── other-suite.taskgroup.yaml

~/.claude/
├── tasks/              # User global tasks
└── task-groups/        # User global groups
```

## API Reference

### TaskGroupLoader

Main loader interface:

```rust
// Constructors
TaskGroupLoader::new()                          // Default paths
TaskGroupLoader::with_paths(Vec<PathBuf>)       // Custom paths
TaskGroupLoader::with_task_loader(TaskLoader)   // Custom task loader

// Loading
load(&mut self, &TaskGroupReference) -> Result<ResolvedTaskGroup>
load_from_file(&mut self, &Path) -> Result<ResolvedTaskGroup>
discover_all(&self) -> Result<HashMap<String, PathBuf>>

// Utilities
add_path(&mut self, PathBuf)
clear_cache(&mut self)
cached_groups(&self) -> Vec<String>
task_loader_mut(&mut self) -> &mut TaskLoader
```

### ResolvedTaskGroup

Resolved group with all tasks loaded:

```rust
pub struct ResolvedTaskGroup {
    pub group: TaskGroup,                       // Original definition
    pub tasks: HashMap<String, PredefinedTask>, // Resolved tasks
    pub source_path: PathBuf,                   // Source file path
}

// Methods
get_task(&self, &str) -> Option<&PredefinedTask>
task_names(&self) -> Vec<String>
contains_task(&self, &str) -> bool
```

### TaskGroupReference

Reference to a task group:

```rust
TaskGroupReference::parse("group-name@version")?
TaskGroupReference::parse("group@1.0.0#workflow")?  // With workflow

reference.to_string()  // Format as string
```

### GroupLoadError

Error variants:

- `IoError` - File system errors
- `ParseError` - YAML parsing errors
- `GroupNotFound` - Group not found
- `TaskNotFound` - Referenced task not found
- `VersionMismatch` - Version mismatch
- `RequiredTaskMissing` - Required task missing

## Examples

See `examples/task_groups_loader.rs` for comprehensive examples.

## Documentation

- [Schema Documentation](./schema.rs) - Type definitions
- [Parser Documentation](./parser.rs) - YAML parsing
- [Loader Documentation](./loader.rs) - Discovery and loading
- [External Guide](../../../docs/task-groups-loader.md) - Comprehensive guide

## Testing

Run tests:

```bash
cargo test groups::loader
cargo test groups::parser
cargo test groups::schema
```

## Future Phases

### Phase 2: Advanced Features

- Remote sources (Git repositories)
- Version ranges (semver constraints)
- Dependency resolution
- Workflow templates

### Phase 3: Optimization

- Persistent cache
- Lazy loading
- Parallel resolution
- Watch mode

## See Also

- [Predefined Tasks](../README.md) - Individual task system
- [Task Resolution](../resolver.rs) - How groups integrate with workflows
- [Implementation Plan](../../../../implement-predefined-tasks.yaml) - Overall roadmap
