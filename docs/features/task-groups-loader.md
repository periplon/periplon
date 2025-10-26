# Task Groups Loader

The Task Groups Loader module (`src/dsl/predefined_tasks/groups/loader.rs`) provides functionality for discovering, loading, and resolving task groups from the filesystem.

## Overview

The loader is responsible for:

1. **Discovery**: Finding task group files in configured search paths
2. **Loading**: Parsing task group definitions from `.taskgroup.yaml` files
3. **Resolution**: Loading all tasks referenced by a group
4. **Configuration**: Applying shared configuration to all tasks in a group
5. **Caching**: Maintaining a cache of resolved groups for performance

## Architecture

### Key Components

#### `TaskGroupLoader`

The main loader that manages task group discovery and resolution:

```rust
pub struct TaskGroupLoader {
    search_paths: Vec<PathBuf>,      // Directories to search for groups
    task_loader: TaskLoader,          // Loader for individual tasks
    cache: HashMap<String, ResolvedTaskGroup>,  // Resolved group cache
}
```

**Key Methods:**
- `new()` - Create loader with default search paths
- `with_paths()` - Create loader with custom search paths
- `with_task_loader()` - Create loader with custom task loader
- `load()` - Load and resolve a task group by reference
- `discover_all()` - Find all available task groups
- `clear_cache()` - Clear the resolution cache

#### `ResolvedTaskGroup`

A fully resolved task group with all tasks loaded:

```rust
pub struct ResolvedTaskGroup {
    pub group: TaskGroup,                    // Original group definition
    pub tasks: HashMap<String, PredefinedTask>,  // Resolved tasks
    pub source_path: PathBuf,                // Where the group was loaded from
}
```

**Key Methods:**
- `get_task()` - Get a task by name
- `task_names()` - List all task names
- `contains_task()` - Check if a task exists

#### `GroupLoadError`

Comprehensive error types for group loading failures:

```rust
pub enum GroupLoadError {
    IoError { path, source },           // File system errors
    ParseError { path, source },        // YAML parsing errors
    GroupNotFound(String),              // Group not found
    TaskNotFound { group, task, version },  // Referenced task not found
    VersionMismatch { task, required, found },  // Version mismatch
    RequiredTaskMissing { group, task },  // Required task missing
    // ... more variants
}
```

## Discovery Process

### Search Paths

Default search paths (in priority order):

1. **Project Local**: `./.claude/task-groups/` (highest priority)
2. **User Global**: `~/.claude/task-groups/`

Custom paths can be configured via `with_paths()`.

### File Naming Convention

Task group files must follow the naming pattern:
- `{group-name}.taskgroup.yaml`

Example: `google-workspace-suite.taskgroup.yaml`

### Discovery Algorithm

```
For each search path (in priority order):
  1. Check if directory exists
  2. Scan for *.taskgroup.yaml files
  3. Parse each file to extract metadata
  4. Build map of group references to file paths
  5. Later paths override earlier paths (priority)
```

## Resolution Process

When loading a task group, the loader:

### 1. Locate the Group File

```rust
let group_ref = TaskGroupReference::parse("google-workspace-suite@2.0.0")?;
let resolved = loader.load(&group_ref)?;
```

Searches in priority order through configured paths.

### 2. Parse the Group Definition

Load and parse the YAML file using the parser module.

### 3. Resolve All Referenced Tasks

For each task in `spec.tasks`:

```yaml
tasks:
  - name: "gmail-search"
    version: "1.0.0"
    required: true
  - name: "drive-upload"
    version: "2.1.0"
    required: false
```

The loader:
1. Creates a `TaskReference` for each task
2. Uses the `TaskLoader` to find and load the task
3. Validates the version matches
4. Checks required tasks are present

### 4. Apply Shared Configuration

If the group defines shared configuration:

```yaml
spec:
  shared_config:
    inputs:
      api_key:
        type: string
        required: true
    permissions:
      mode: "acceptEdits"
    max_turns: 10
```

The loader applies these settings to all tasks:

```rust
fn apply_shared_config(task: &mut PredefinedTask, shared_config: &SharedConfig) {
    // Merge shared inputs (task-specific take precedence)
    for (key, value) in &shared_config.inputs {
        task.spec.inputs.entry(key.clone()).or_insert_with(...);
    }

    // Apply shared permissions if not set
    if task.spec.agent_template.permissions.mode.is_none() {
        task.spec.agent_template.permissions = shared_perms.clone();
    }

    // Apply shared max_turns if not set
    if task.spec.agent_template.max_turns.is_none() {
        task.spec.agent_template.max_turns = shared_config.max_turns;
    }
}
```

**Configuration Precedence:**
- Task-specific settings > Shared configuration
- Shared configuration only fills in missing values

### 5. Cache the Result

The resolved group is cached using the reference as key:
- Key format: `"group-name@version"`
- Cache persists across multiple loads
- Can be cleared with `clear_cache()`

## Usage Examples

### Basic Loading

```rust
use periplon_sdk::dsl::predefined_tasks::{
    TaskGroupLoader, TaskGroupReference
};

// Create loader with default search paths
let mut loader = TaskGroupLoader::new();

// Load a task group
let group_ref = TaskGroupReference::parse("google-workspace-suite@2.0.0")?;
let resolved = loader.load(&group_ref)?;

// Access resolved tasks
if let Some(task) = resolved.get_task("gmail-search") {
    println!("Found task: {}", task.metadata.name);
}

// List all tasks in the group
for name in resolved.task_names() {
    println!("Task: {}", name);
}
```

### Custom Search Paths

```rust
use std::path::PathBuf;

let custom_paths = vec![
    PathBuf::from("/opt/claude-tasks/groups"),
    PathBuf::from("./my-groups"),
];

let mut loader = TaskGroupLoader::with_paths(custom_paths);
let resolved = loader.load(&group_ref)?;
```

### Custom Task Loader

```rust
// Create a custom task loader with specific search paths
let task_loader = TaskLoader::with_paths(vec![
    PathBuf::from("./custom-tasks"),
]);

// Use it with the group loader
let mut loader = TaskGroupLoader::with_task_loader(task_loader);
let resolved = loader.load(&group_ref)?;
```

### Discovery and Inspection

```rust
// Discover all available groups
let discovered = loader.discover_all()?;

for (group_ref, path) in discovered {
    println!("Found group: {} at {}", group_ref, path.display());
}

// Load from a specific file
let resolved = loader.load_from_file(&PathBuf::from(
    ".claude/task-groups/my-suite.taskgroup.yaml"
))?;
```

### Error Handling

```rust
use periplon_sdk::dsl::predefined_tasks::GroupLoadError;

match loader.load(&group_ref) {
    Ok(resolved) => {
        println!("Loaded {} tasks", resolved.tasks.len());
    }
    Err(GroupLoadError::GroupNotFound(ref_str)) => {
        eprintln!("Group not found: {}", ref_str);
    }
    Err(GroupLoadError::TaskNotFound { group, task, version }) => {
        eprintln!("Task {}@{} required by {} not found", task, version, group);
    }
    Err(GroupLoadError::VersionMismatch { task, required, found }) => {
        eprintln!("Version mismatch for {}: need {}, got {}", task, required, found);
    }
    Err(e) => {
        eprintln!("Error loading group: {}", e);
    }
}
```

## Performance Considerations

### Caching

The loader maintains two levels of cache:

1. **Group Cache**: Resolved task groups
   - Key: `"group-name@version"`
   - Avoids re-parsing YAML and re-resolving tasks

2. **Task Cache**: Individual tasks (via `TaskLoader`)
   - Key: `"task-name@version"`
   - Shared across multiple group loads

### Cache Management

```rust
// Check what's cached
let cached = loader.cached_groups();
println!("Cached groups: {:?}", cached);

// Clear cache to force reload
loader.clear_cache();

// Access underlying task loader cache
loader.task_loader_mut().clear_cache();
```

### Discovery Optimization

Discovery is lazy - groups are only discovered when:
1. `discover_all()` is explicitly called
2. A group is loaded and not found in cache

## Integration with Task Resolution

The group loader integrates with the broader task resolution system:

```
Workflow
    ↓
TaskResolver
    ↓
TaskGroupLoader ←→ TaskLoader
    ↓                    ↓
ResolvedTaskGroup    PredefinedTask
```

The resolver uses the group loader when it encounters:

```yaml
tasks:
  backup_suite:
    uses_group: "google-workspace-suite@2.0.0"
```

## Testing

The module includes comprehensive tests:

### Unit Tests

```bash
cargo test groups::loader
```

Tests cover:
- ✅ File type detection (`.taskgroup.yaml`)
- ✅ Group discovery in directories
- ✅ Loading with task resolution
- ✅ Missing task handling
- ✅ Caching behavior
- ✅ Shared configuration application
- ✅ Priority-based path resolution

### Integration Tests

Create test fixtures:

```rust
use tempfile::TempDir;

let group_dir = TempDir::new()?;
let task_dir = TempDir::new()?;

// Create test tasks
create_test_task_file(task_dir.path(), "task1", "1.0.0");

// Create test group
create_test_group_file(
    group_dir.path(),
    "my-group",
    "1.0.0",
    vec![("task1", "1.0.0")],
);

// Test loading
let task_loader = TaskLoader::with_paths(vec![task_dir.path().to_path_buf()]);
let mut loader = TaskGroupLoader::with_task_loader(task_loader);
loader.add_path(group_dir.path().to_path_buf());

let group_ref = TaskGroupReference::parse("my-group@1.0.0")?;
let resolved = loader.load(&group_ref)?;

assert!(resolved.contains_task("task1"));
```

## File System Layout

Expected directory structure:

```
.claude/
├── tasks/                    # Individual tasks (for TaskLoader)
│   ├── gmail-search.task.yaml
│   ├── drive-upload.task.yaml
│   └── calendar-sync.task.yaml
│
└── task-groups/              # Task groups (for TaskGroupLoader)
    ├── google-workspace-suite.taskgroup.yaml
    ├── data-pipeline-suite.taskgroup.yaml
    └── integration-tests.taskgroup.yaml
```

## Error Recovery

The loader is fault-tolerant during discovery:

```rust
match load_task_group(&path) {
    Ok(group) => {
        // Add to discovered groups
    }
    Err(e) => {
        // Log warning but continue discovery
        eprintln!("Warning: Failed to load task group from {}: {}", path.display(), e);
    }
}
```

This ensures one malformed group file doesn't break discovery of valid groups.

## Future Enhancements

### Phase 2+

1. **Remote Sources**: Support Git repositories and registries
2. **Version Constraints**: Use semver ranges (`^2.0.0`, `~1.5.0`)
3. **Dependency Resolution**: Auto-install dependent groups
4. **Lockfile Integration**: Track resolved versions
5. **Parallel Loading**: Load multiple groups concurrently
6. **Incremental Updates**: Detect and reload changed groups

## Best Practices

1. **Use Descriptive Names**: Name groups clearly (`api-integration-suite`, not `suite1`)
2. **Version Properly**: Follow semantic versioning
3. **Cache Management**: Clear cache during development, keep in production
4. **Error Handling**: Always handle `TaskNotFound` and `VersionMismatch` errors
5. **Path Priority**: Put project-specific paths last (highest priority)
6. **Required Tasks**: Only mark truly essential tasks as required
7. **Shared Config**: Use sparingly - prefer task-specific configuration

## See Also

- [Task Groups Schema](./task-groups-schema.md) - Group definition format
- [Task Groups Parser](./task-groups-parser.md) - YAML parsing
- [Task Loader](./predefined-tasks-loader.md) - Individual task loading
- [Task Resolution](./task-resolution.md) - How groups integrate with workflows
