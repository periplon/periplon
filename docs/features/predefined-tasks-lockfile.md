# Predefined Tasks Lock File System

## Overview

The lock file system ensures reproducible dependency resolution for predefined tasks across different environments and time periods. It captures exact versions, checksums, and source information for all resolved dependencies.

## Features

- **Reproducibility**: Ensures the same dependency versions are used across different machines
- **Integrity**: Verifies task content hasn't changed via SHA-256 checksums
- **Transparency**: Tracks where each task came from (local, git, registry)
- **Performance**: Skips resolution when lock file is valid
- **Version Compatibility**: Lock file format versioning with compatibility checks

## Lock File Format

Lock files are stored in YAML format at `.claude/tasks.lock.yaml`:

```yaml
version: "1.0.0"
generated_at: "2025-01-15T10:30:00Z"
generated_by: "periplon@0.1.0"

tasks:
  google-drive-upload:
    version: "1.2.0"
    checksum: "sha256:abc123..."
    source:
      type: "local"
      path: "./.claude/tasks/google-drive-upload.task.yaml"
    resolved_at: "2025-01-15T10:30:00Z"
    dependencies:
      auth-helper: "2.1.0"
    metadata:
      description: "Upload files to Google Drive"
      author: "Example Author"
      license: "MIT"

  auth-helper:
    version: "2.1.0"
    checksum: "sha256:def456..."
    source:
      type: "git"
      url: "https://github.com/org/tasks.git"
      ref: "v2.1.0"
      subpath: "auth-helper"
    resolved_at: "2025-01-15T10:30:00Z"
    dependencies: {}
    metadata:
      description: "Authentication helper"
      author: "Example Author"
      license: "MIT"

metadata:
  task_count: 2
```

## Usage

### Generating a Lock File

```rust
use periplon_sdk::dsl::predefined_tasks::{
    DependencyResolver, LocalSourceResolver, generate_lock_file,
    TaskReference, LOCK_FILE_NAME,
};

// Resolve dependencies
let mut resolver = DependencyResolver::new();
// ... add tasks to resolver ...

let task_ref = TaskReference::parse("my-task@1.0.0")?;
let resolved = resolver.resolve(&task_ref)?;

// Generate lock file
let source_resolver = LocalSourceResolver::new("./.claude/tasks".into());
let lock_file = generate_lock_file(&resolved, &source_resolver)?;

// Save to disk
lock_file.save(LOCK_FILE_NAME)?;
```

### Loading and Validating a Lock File

```rust
use periplon_sdk::dsl::predefined_tasks::{
    LockFile, validate_lock_file,
};

// Load existing lock file
let lock_file = LockFile::load(".claude/tasks.lock.yaml")?;

// Validate against current dependencies
let result = validate_lock_file(&lock_file, &resolved)?;

if result.is_valid() {
    println!("Lock file is valid!");
} else {
    println!("Issues found:");
    println!("{}", result.summary());
}
```

### Checking for Staleness

```rust
// Quick check if lock file needs regeneration
if lock_file.is_stale(&resolved) {
    println!("Lock file is stale - regenerating...");
    let new_lock_file = generate_lock_file(&resolved, &source_resolver)?;
    new_lock_file.save(LOCK_FILE_NAME)?;
}
```

### Verifying Task Integrity

```rust
// Verify a specific task's checksum
match lock_file.verify_task("my-task", &task) {
    Ok(_) => println!("Checksum verified!"),
    Err(e) => println!("Verification failed: {}", e),
}

// Verify all tasks
let tasks = /* HashMap of task name -> PredefinedTask */;
lock_file.verify_all(&tasks)?;
```

## Lock File Components

### LockFile

Main lock file structure containing:
- `version`: Lock file format version
- `generated_at`: Timestamp when generated
- `generated_by`: Tool and version that created it
- `tasks`: Map of task name to LockedTask
- `metadata`: Optional metadata

### LockedTask

Information about a locked task:
- `version`: Exact resolved version
- `checksum`: SHA-256 content checksum
- `source`: Source information (local/git/registry)
- `resolved_at`: When this task was resolved
- `dependencies`: Resolved dependency versions
- `metadata`: Optional task metadata (description, author, license)

### TaskSource

Source tracking for tasks:

**Local Source:**
```yaml
source:
  type: local
  path: "./.claude/tasks/my-task.task.yaml"
```

**Git Source:**
```yaml
source:
  type: git
  url: "https://github.com/org/repo.git"
  ref: "v1.0.0"
  subpath: "tasks/my-task"
```

**Registry Source:**
```yaml
source:
  type: registry
  url: "https://registry.example.com"
  package: "org/my-task"
```

## Validation

The validation system checks for:

1. **Version Mismatches**: Locked version differs from resolved version
2. **Dependency Mismatches**: Dependency graph changed
3. **Checksum Failures**: Task content modified
4. **Missing Tasks**: Task in dependencies but not in lock file
5. **Extra Tasks**: Task in lock file but not in dependencies

### ValidationResult

```rust
pub struct ValidationResult {
    pub issues: Vec<ValidationIssue>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool { /* ... */ }
    pub fn summary(&self) -> String { /* ... */ }
}
```

### ValidationIssue Types

```rust
pub enum ValidationIssue {
    MissingTask { name: String },
    ExtraTask { name: String },
    VersionMismatch { name: String, locked: String, resolved: String },
    DependencyMismatch { name: String },
    ChecksumFailed { name: String, error: String },
}
```

## Source Resolution

Implement the `SourceResolver` trait to customize source tracking:

```rust
use periplon_sdk::dsl::predefined_tasks::{
    SourceResolver, LockFileError,
    lockfile::TaskSource,
};

struct CustomSourceResolver {
    // ... custom fields ...
}

impl SourceResolver for CustomSourceResolver {
    fn resolve_source(
        &self,
        name: &str,
        version: &str,
    ) -> Result<TaskSource, LockFileError> {
        // Custom logic to determine source
        // e.g., check git, local, or registry
        Ok(TaskSource::Local {
            path: format!("./tasks/{}.task.yaml", name),
        })
    }
}
```

### Built-in Resolvers

**LocalSourceResolver**: Assumes all tasks are local files

```rust
use periplon_sdk::dsl::predefined_tasks::LocalSourceResolver;

let resolver = LocalSourceResolver::new("./.claude/tasks".into());
```

## Checksum Computation

Checksums are computed from the canonical YAML serialization of tasks:

```rust
use periplon_sdk::dsl::predefined_tasks::compute_task_checksum;

let checksum = compute_task_checksum(&task)?;
// Returns: "sha256:abc123..."
```

The checksum includes:
- All task metadata (name, version, description, etc.)
- Agent template configuration
- Input/output specifications
- Dependencies
- Examples

Any change to the task definition changes the checksum.

## Best Practices

### 1. Commit Lock Files

Always commit `tasks.lock.yaml` to version control:
```bash
git add .claude/tasks.lock.yaml
git commit -m "Update task dependencies"
```

### 2. Regenerate After Updates

Regenerate lock files when dependencies change:
```bash
# After updating a task version
periplon-executor lock --regenerate
```

### 3. Validate in CI/CD

Add lock file validation to your CI pipeline:
```rust
let lock_file = LockFile::load("tasks.lock.yaml")?;
let result = validate_lock_file(&lock_file, &resolved)?;

if !result.is_valid() {
    eprintln!("Lock file validation failed!");
    eprintln!("{}", result.summary());
    std::process::exit(1);
}
```

### 4. Review Lock File Changes

When reviewing PRs, check lock file changes for:
- Expected version updates
- New/removed dependencies
- Source changes

### 5. Handle Conflicts

Resolve lock file merge conflicts by regenerating:
```bash
# In case of conflict
git checkout --theirs .claude/tasks.lock.yaml
periplon-executor lock --regenerate
```

## Integration with Dependency Resolution

Lock files integrate seamlessly with the dependency resolver:

```rust
// 1. Load lock file if it exists
let lock_file = match LockFile::load(LOCK_FILE_NAME) {
    Ok(lf) => Some(lf),
    Err(_) => None,
};

// 2. Resolve dependencies
let resolved = resolver.resolve(&task_ref)?;

// 3. Check if lock file is valid
if let Some(ref lf) = lock_file {
    if !lf.is_stale(&resolved) {
        // Use locked versions
        println!("Using locked versions");
    } else {
        // Regenerate lock file
        let new_lf = generate_lock_file(&resolved, &source_resolver)?;
        new_lf.save(LOCK_FILE_NAME)?;
    }
} else {
    // Generate new lock file
    let new_lf = generate_lock_file(&resolved, &source_resolver)?;
    new_lf.save(LOCK_FILE_NAME)?;
}
```

## Example: Complete Workflow

```rust
use periplon_sdk::dsl::predefined_tasks::{
    DependencyResolver, LocalSourceResolver, LockFile,
    generate_lock_file, validate_lock_file, TaskReference,
    LOCK_FILE_NAME,
};

fn resolve_with_lockfile(
    task_ref: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Setup
    let mut resolver = DependencyResolver::new();
    // ... populate resolver ...

    let task_ref = TaskReference::parse(task_ref)?;
    let resolved = resolver.resolve(&task_ref)?;

    // Try to use existing lock file
    match LockFile::load(LOCK_FILE_NAME) {
        Ok(lock_file) => {
            println!("Found existing lock file");

            // Validate
            let result = validate_lock_file(&lock_file, &resolved)?;

            if result.is_valid() && !lock_file.is_stale(&resolved) {
                println!("✓ Lock file is valid and up-to-date");
                // Use locked versions
                for (name, locked) in &lock_file.tasks {
                    println!("  {} @ {}", name, locked.version);
                }
            } else {
                println!("⚠ Lock file needs regeneration");
                println!("{}", result.summary());

                // Regenerate
                let source_resolver = LocalSourceResolver::new(
                    "./.claude/tasks".into()
                );
                let new_lock = generate_lock_file(&resolved, &source_resolver)?;
                new_lock.save(LOCK_FILE_NAME)?;
                println!("✓ Lock file regenerated");
            }
        }
        Err(_) => {
            println!("No lock file found, generating...");

            // Generate new lock file
            let source_resolver = LocalSourceResolver::new(
                "./.claude/tasks".into()
            );
            let lock_file = generate_lock_file(&resolved, &source_resolver)?;
            lock_file.save(LOCK_FILE_NAME)?;
            println!("✓ Lock file created");
        }
    }

    Ok(())
}
```

## Error Handling

The lock file system provides detailed errors:

```rust
use periplon_sdk::dsl::predefined_tasks::LockFileError;

match lock_file.verify_task("my-task", &task) {
    Ok(_) => println!("Verified"),
    Err(LockFileError::ChecksumMismatch { task, expected, actual }) => {
        eprintln!("Checksum mismatch for {}:", task);
        eprintln!("  Expected: {}", expected);
        eprintln!("  Actual:   {}", actual);
    }
    Err(LockFileError::TaskNotFound(name)) => {
        eprintln!("Task '{}' not in lock file", name);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Running the Demo

See the complete working example:

```bash
cargo run --example lockfile_demo
```

This demonstrates:
1. Generating lock files from dependencies
2. Validating lock files
3. Checksum verification
4. Lock file persistence and loading

## API Reference

### Core Functions

- `generate_lock_file(resolved, source_resolver)` - Generate lock file from resolved tasks
- `validate_lock_file(lock_file, resolved)` - Validate lock file against dependencies
- `compute_task_checksum(task)` - Compute SHA-256 checksum of a task

### Types

- `LockFile` - Main lock file structure
- `LockedTask` - Individual locked task entry
- `TaskSource` - Source tracking (Local/Git/Registry)
- `ValidationResult` - Validation results with issues
- `ValidationIssue` - Individual validation issue
- `SourceResolver` - Trait for resolving task sources
- `LocalSourceResolver` - Built-in local file resolver

### Constants

- `LOCK_FILE_VERSION` - Current lock file format version ("1.0.0")
- `LOCK_FILE_NAME` - Default lock file name ("tasks.lock.yaml")

## See Also

- [Dependency Resolution](./predefined-tasks-deps.md)
- [Predefined Tasks Overview](./predefined-tasks.md)
- [Task Sources](./predefined-tasks-sources.md)
