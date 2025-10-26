# DSL Executor CLI Guide

## Overview

The `periplon-executor` is a command-line tool for executing multi-agent DSL workflows. It provides a simple interface for running, validating, and managing workflow executions with support for state persistence, progress tracking, and resume functionality.

## Installation

### Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/periplon
cd periplon

# Build the CLI tool
cargo build --release --bin periplon-executor

# The binary will be at: target/release/periplon-executor
```

### Install Locally

```bash
cargo install --path . --bin periplon-executor
```

## Commands

### `run` - Execute a Workflow

Execute a workflow from a YAML file.

**Usage:**
```bash
periplon-executor run <WORKFLOW_FILE> [OPTIONS]
```

**Options:**
- `-s, --state-dir <DIR>` - Directory to store workflow state (default: `.workflow_states`)
- `-r, --resume` - Resume from saved state if available
- `-c, --clean` - Clean state before execution (delete existing state)
- `-v, --verbose` - Enable verbose output
- `--dry-run` - Validate workflow without executing

**Examples:**

```bash
# Run a workflow
periplon-executor run examples/dsl/simple_file_organizer.yaml

# Run with verbose output
periplon-executor run examples/dsl/research_pipeline.yaml --verbose

# Run with state persistence
periplon-executor run workflow.yaml --state-dir ./my-states

# Resume a previously interrupted workflow
periplon-executor run workflow.yaml --resume

# Clean and restart
periplon-executor run workflow.yaml --clean

# Validate without running
periplon-executor run workflow.yaml --dry-run
```

**Output:**
```
============================================================
DSL Workflow Executor
============================================================

Parsing workflow...  ✓
Validating workflow...  ✓

Initializing workflow...  ✓

Executing workflow...
------------------------------------------------------------

Executing task: organize_downloads - Sort files in Downloads folder
  Message: ...
Task completed: organize_downloads

------------------------------------------------------------
Shutting down...  ✓

✓ Workflow completed successfully!
  Duration: 2.45s
  Completed: 5/5
```

---

### `validate` - Validate a Workflow

Validate a workflow file without executing it.

**Usage:**
```bash
periplon-executor validate <WORKFLOW_FILE> [OPTIONS]
```

**Options:**
- `-v, --verbose` - Show detailed validation information

**Examples:**

```bash
# Validate a workflow
periplon-executor validate workflow.yaml

# Validate with details
periplon-executor validate workflow.yaml --verbose
```

**Output (verbose):**
```
Validating workflow...

  Parsing YAML...  ✓
    Workflow: File Organizer
    Version: 1.0.0
  Checking semantics...  ✓
    Agents: 1
    Tasks: 1

  Agents:
    • organizer - Organize files by type and date
      Tools: Read, Bash, Glob

  Tasks:
    • organize_downloads - Sort files in Downloads folder

✓ Workflow is valid
```

---

### `list` - List Saved Workflow States

List all saved workflow states in the state directory.

**Usage:**
```bash
periplon-executor list [OPTIONS]
```

**Options:**
- `-s, --state-dir <DIR>` - Directory containing workflow states (default: `.workflow_states`)

**Examples:**

```bash
# List all saved states
periplon-executor list

# List from custom directory
periplon-executor list --state-dir ./my-states
```

**Output:**
```
Saved Workflow States:
  Directory: .workflow_states

  • File Organizer
    Status: Completed
    Progress: 100.0%
    Tasks: 5/5
    Finished

  • Research Pipeline
    Status: Running
    Progress: 60.0%
    Tasks: 3/5
    In Progress

Total: 2 workflows
```

---

### `clean` - Clean Saved Workflow States

Delete saved workflow states.

**Usage:**
```bash
periplon-executor clean [WORKFLOW_NAME] [OPTIONS]
```

**Options:**
- `-s, --state-dir <DIR>` - Directory containing workflow states (default: `.workflow_states`)
- `-y, --yes` - Skip confirmation prompt

**Examples:**

```bash
# Clean a specific workflow (with confirmation)
periplon-executor clean "File Organizer"

# Clean all workflows (with confirmation)
periplon-executor clean

# Clean without confirmation
periplon-executor clean "Research Pipeline" --yes

# Clean all from custom directory
periplon-executor clean --state-dir ./my-states --yes
```

**Output:**
```
Delete state for workflow 'File Organizer'? [y/N] y
✓ Deleted state for 'File Organizer'
```

---

### `status` - Show Workflow Status

Display detailed status and progress information for a workflow.

**Usage:**
```bash
periplon-executor status <WORKFLOW_NAME> [OPTIONS]
```

**Options:**
- `-s, --state-dir <DIR>` - Directory containing workflow states (default: `.workflow_states`)

**Examples:**

```bash
# Show workflow status
periplon-executor status "Research Pipeline"

# From custom directory
periplon-executor status "My Workflow" --state-dir ./my-states
```

**Output:**
```
Workflow Status
============================================================

  Name: Research Pipeline
  Version: 1.0.0
  Status: Running

  Progress: 60.0%
  Total Tasks: 5
  Completed: 3
  Failed: 0

  Started: SystemTime { ... }
  Ended: In Progress
```

---

### `group` - Manage Task Groups

Manage task groups - collections of reusable predefined tasks that can be imported into workflows.

#### `group list` - List Available Task Groups

List all available task groups from configured search paths.

**Usage:**
```bash
periplon-executor group list [OPTIONS]
```

**Options:**
- `-v, --verbose` - Show detailed information about each group
- `-j, --json` - Output results in JSON format with syntax coloring
- `-p, --path <DIR>` - Search directory (default: standard search paths)

**Examples:**

```bash
# List all available task groups
periplon-executor group list

# List with detailed information
periplon-executor group list --verbose

# List from a specific directory
periplon-executor group list --path ~/.claude/task-groups

# JSON output for scripting
periplon-executor group list --json
```

**Output:**
```
Task Groups
============================================================

  Search Paths:
    • /Users/you/.claude/task-groups
    • ./.claude/task-groups

  Found Available Groups:

  • google-workspace@1.0.0
    3 tasks | /Users/you/.claude/task-groups/google-workspace.taskgroup.yaml

  • aws-tools@2.1.0
    5 tasks | /Users/you/.claude/task-groups/aws-tools.taskgroup.yaml

Total: 2 groups
```

---

#### `group install` - Install (Validate) a Task Group

Install and validate a task group by reference. This verifies that the group definition is valid and all required tasks are available.

**Usage:**
```bash
periplon-executor group install <GROUP_REF> [OPTIONS]
```

**Arguments:**
- `<GROUP_REF>` - Task group reference (format: `name@version`)

**Options:**
- `-v, --verbose` - Show detailed installation steps
- `-j, --json` - Output results in JSON format

**Examples:**

```bash
# Install a task group
periplon-executor group install google-workspace@1.0.0

# Install with verbose output
periplon-executor group install aws-tools@2.1.0 --verbose

# Install with JSON output
periplon-executor group install slack-integration@1.0.0 --json
```

**Output:**
```
Installing Task Group
============================================================

  Group: google-workspace
  Version: 1.0.0

  Resolving task group...  ✓
  Validating tasks...  ✓

✓ Task group installed successfully!
  Group: google-workspace
  Version: 1.0.0
  Tasks: 3
  Description: Google Workspace integration tasks

  Source: /Users/you/.claude/task-groups/google-workspace.taskgroup.yaml
```

---

#### `group update` - Update Task Group Cache

Update task group cache and refresh information. Use `--force` to clear the cache and reload all groups.

**Usage:**
```bash
periplon-executor group update [GROUP_REF] [OPTIONS]
```

**Arguments:**
- `[GROUP_REF]` - Specific group to update (optional, updates all if omitted)

**Options:**
- `-f, --force` - Force cache refresh
- `-v, --verbose` - Show detailed update information
- `-j, --json` - Output results in JSON format

**Examples:**

```bash
# Update all groups
periplon-executor group update

# Update specific group
periplon-executor group update google-workspace@1.0.0

# Force cache refresh for all groups
periplon-executor group update --force

# Update with verbose output
periplon-executor group update --verbose

# JSON output
periplon-executor group update --json
```

**Output:**
```
Updating Task Groups
============================================================

  ✓ Cache cleared
  Found 3 groups

  Updating: google-workspace@1.0.0
  Updating: aws-tools@2.1.0
  Updating: slack-integration@1.0.0

✓ Updated 3 groups
```

---

#### `group validate` - Validate a Task Group File

Validate a task group definition file to ensure it's correctly formatted and all referenced tasks exist.

**Usage:**
```bash
periplon-executor group validate <GROUP_FILE> [OPTIONS]
```

**Arguments:**
- `<GROUP_FILE>` - Path to the task group YAML file

**Options:**
- `-v, --verbose` - Show detailed validation information
- `-j, --json` - Output results in JSON format

**Examples:**

```bash
# Validate a task group file
periplon-executor group validate my-group.taskgroup.yaml

# Validate with verbose output
periplon-executor group validate my-group.taskgroup.yaml --verbose

# JSON output
periplon-executor group validate my-group.taskgroup.yaml --json
```

**Output (verbose):**
```
Validating Task Group
============================================================

  Loading group file...  ✓
    Name: my-custom-group
    Version: 1.0.0
    Tasks: 3

  Resolving tasks...  ✓
    ✓ task1 v1.0.0
    ✓ task2 v1.0.0
    ✓ task3 v2.0.0

✓ Task group is valid!
  Group: my-custom-group
  Version: 1.0.0
  Tasks: 3

  Tasks:
    • task1 v1.0.0 (required)
    • task2 v1.0.0
    • task3 v2.0.0 (required)
```

**Validation Errors:**
```
Validating Task Group
============================================================

✗ Task group validation failed!

  Errors:
    • Task 'missing-task' v1.0.0 not found
    • Task 'another-task' version mismatch: required 2.0.0, found 1.0.0
```

---

### Task Groups Overview

Task groups allow you to:

1. **Organize reusable tasks** into logical collections
2. **Share configuration** across multiple tasks (API keys, permissions, environment variables)
3. **Version and distribute** task collections
4. **Import into workflows** using namespace references

**Example Task Group File:**

```yaml
apiVersion: "taskgroup/v1"
kind: "TaskGroup"
metadata:
  name: "google-workspace"
  version: "1.0.0"
  description: "Google Workspace integration tasks"
  author: "Your Name"
  license: "MIT"
  tags:
    - "google"
    - "productivity"

spec:
  tasks:
    - name: "upload-to-drive"
      version: "1.0.0"
      required: true
      description: "Upload files to Google Drive"

    - name: "create-doc"
      version: "1.0.0"
      required: true
      description: "Create a Google Doc"

    - name: "send-gmail"
      version: "1.0.0"
      required: false
      description: "Send email via Gmail"

  shared_config:
    inputs:
      api_key:
        type: string
        required: true
        description: "Google API key"

    permissions:
      mode: "acceptEdits"
      allowed_directories:
        - "/tmp"

    environment:
      GOOGLE_API_URL: "https://www.googleapis.com"

    max_turns: 10
```

**Using in Workflows:**

```yaml
name: "My Workflow"
version: "1.0.0"

# Import task groups
imports:
  google: "google-workspace@1.0.0"
  aws: "aws-tools@2.1.0"

agents: {}

tasks:
  upload_files:
    description: "Upload processed files to Google Drive"
    uses_workflow: "google:upload-to-drive"
    inputs:
      files: "./output/*.pdf"
      folder_id: "abc123"
```

**Search Paths:**

Task groups are discovered from these locations (in priority order):

1. `./.claude/task-groups/` - Project-local groups (highest priority)
2. `~/.claude/task-groups/` - User-global groups

Place your `.taskgroup.yaml` files in these directories to make them available for import.

---

## Workflow State Persistence

### How It Works

When you run a workflow with state persistence enabled (using `--resume`, `--clean`, or `--state-dir`), the executor:

1. **Saves state automatically** after each task completion
2. **Tracks progress** including completed, failed, and pending tasks
3. **Records errors** for failed tasks with full error messages
4. **Timestamps execution** to track task duration
5. **Stores in JSON format** for easy debugging

### Resume Functionality

Use `--resume` to continue a workflow that was interrupted:

```bash
# Start a workflow
periplon-executor run long-workflow.yaml

# Interrupt with Ctrl+C

# Resume from where it left off
periplon-executor run long-workflow.yaml --resume
```

The executor will:
- ✓ Load the saved state
- ✓ Skip already-completed tasks
- ✓ Resume from the next pending task
- ✓ Show progress from the checkpoint

### State Files

State files are stored as JSON in the state directory:

```
.workflow_states/
├── File Organizer.state.json
├── Research Pipeline.state.json
└── Data Processing.state.json
```

**Example state file:**
```json
{
  "workflow_name": "Research Pipeline",
  "workflow_version": "1.0.0",
  "task_statuses": {
    "gather_data": "Completed",
    "process_data": "Running",
    "write_report": "Pending"
  },
  "task_start_times": {
    "gather_data": { "secs_since_epoch": 1698765432, "nanos_since_epoch": 0 }
  },
  "task_end_times": {
    "gather_data": { "secs_since_epoch": 1698765532, "nanos_since_epoch": 0 }
  },
  "task_attempts": {
    "gather_data": 1,
    "process_data": 2
  },
  "task_errors": {},
  "status": "Running",
  "started_at": { "secs_since_epoch": 1698765400, "nanos_since_epoch": 0 },
  "ended_at": null,
  "checkpoint_at": { "secs_since_epoch": 1698765550, "nanos_since_epoch": 0 },
  "metadata": {}
}
```

---

## Workflow Examples

### Simple File Organization

```yaml
name: "File Organizer"
version: "1.0.0"

agents:
  organizer:
    description: "Organize files by type and date"
    tools:
      - Read
      - Bash
      - Glob
    permissions:
      mode: "default"

tasks:
  organize_downloads:
    description: "Sort files in Downloads folder"
    agent: "organizer"
```

**Run it:**
```bash
periplon-executor run file-organizer.yaml --verbose
```

### Research Pipeline with State

```yaml
name: "Research Pipeline"
version: "1.0.0"

agents:
  researcher:
    description: "Research and gather information"
    tools: [Read, WebSearch]
    permissions:
      mode: "default"

  writer:
    description: "Write documentation"
    tools: [Read, Write, Edit]
    permissions:
      mode: "acceptEdits"

tasks:
  gather_data:
    description: "Research the topic"
    agent: "researcher"

  write_report:
    description: "Write research report"
    agent: "writer"
    depends_on: ["gather_data"]
    on_error:
      retry: 3
```

**Run with state:**
```bash
# Start the workflow
periplon-executor run research.yaml --state-dir ./states

# If interrupted, resume later
periplon-executor run research.yaml --state-dir ./states --resume

# Check status
periplon-executor status "Research Pipeline" --state-dir ./states
```

---

## Error Handling

### Retry on Failure

The executor supports automatic retry for failed tasks:

```yaml
tasks:
  fetch_data:
    agent: "fetcher"
    on_error:
      retry: 3  # Retry up to 3 times
```

### Fallback Agents

Specify a fallback agent if the primary fails:

```yaml
tasks:
  critical_task:
    agent: "primary_agent"
    on_error:
      retry: 2
      fallback_agent: "backup_agent"
```

### Error Messages

Failed tasks show detailed error information:

```bash
✗ Workflow failed!
  Error: Task 'fetch_data' failed after 3 attempts
  Progress: 40.0%
  Completed: 2/5
  Failed: fetch_data
```

---

## Tips and Best Practices

### 1. Use Validation First

Always validate your workflow before running:

```bash
periplon-executor validate workflow.yaml --verbose
```

### 2. Enable State for Long Workflows

For workflows that take a long time:

```bash
periplon-executor run long-workflow.yaml --state-dir ./states
```

### 3. Clean State When Needed

If a workflow changes significantly, clean the old state:

```bash
periplon-executor run workflow.yaml --clean
```

### 4. Check Status During Execution

In another terminal, check progress:

```bash
periplon-executor status "My Workflow"
```

### 5. Use Verbose Mode for Debugging

When things don't work as expected:

```bash
periplon-executor run workflow.yaml --verbose
```

### 6. List States Regularly

Keep track of your workflows:

```bash
periplon-executor list
```

---

## Troubleshooting

### Workflow Validation Fails

**Problem:** `Error: Agent 'unknown_agent' not found`

**Solution:** Check that all agents referenced in tasks are defined in the `agents` section.

### State Won't Resume

**Problem:** Resume flag doesn't seem to work

**Solution:**
1. Check that the workflow name matches exactly
2. Ensure the state directory is correct
3. Verify the state file exists: `periplon-executor list`

### Permission Errors

**Problem:** `Permission denied` when running workflow

**Solution:** Ensure the `permissions` mode in your workflow allows the required operations:
- `default`: Read-only
- `acceptEdits`: Allows file modifications

### State File Corruption

**Problem:** Unable to load state file

**Solution:**
1. Check the JSON file manually (`.workflow_states/<name>.state.json`)
2. If corrupted, clean the state: `periplon-executor clean "<workflow_name>" --yes`
3. Restart the workflow fresh

---

## Advanced Usage

### Custom State Directory

Organize states by project:

```bash
mkdir -p ./project-states
periplon-executor run workflow.yaml --state-dir ./project-states
```

### Scripting with periplon-executor

Use in shell scripts:

```bash
#!/bin/bash

# Run workflow and check exit code
if periplon-executor run workflow.yaml; then
    echo "Workflow succeeded"
    periplon-executor status "My Workflow"
else
    echo "Workflow failed"
    exit 1
fi
```

### CI/CD Integration

Use in continuous integration:

```bash
# .github/workflows/workflow.yml
- name: Run DSL Workflow
  run: |
    periplon-executor validate workflow.yaml
    periplon-executor run workflow.yaml --dry-run
```

---

## Exit Codes

The CLI uses standard exit codes:

- `0` - Success
- `1` - Failure (validation error, execution error, etc.)

Check exit codes in scripts:

```bash
periplon-executor run workflow.yaml
if [ $? -eq 0 ]; then
    echo "Success"
fi
```

---

## Environment Variables

Currently, the CLI does not use environment variables, but workflows can access environment variables through hooks.

---

## Colored Output

The CLI uses colored output for better readability:

- 🟢 **Green** - Success, completed tasks
- 🔴 **Red** - Errors, failed tasks
- 🟡 **Yellow** - Warnings, in-progress tasks
- 🔵 **Cyan** - Headers, separators
- ⚪ **White** - Normal text
- **Bold** - Important information

To disable colors (for logging):

```bash
NO_COLOR=1 periplon-executor run workflow.yaml
```

---

## Version Information

Check the version:

```bash
periplon-executor --version
```

Get help:

```bash
periplon-executor --help
periplon-executor run --help
periplon-executor validate --help
```

---

## Support

For issues, feature requests, or questions:

- GitHub Issues: https://github.com/yourusername/periplon/issues
- Documentation: See `DSL_IMPLEMENTATION.md` for technical details
- Examples: Check `examples/dsl/` directory

---

**Last Updated:** 2025-10-18
**CLI Version:** 0.1.0
