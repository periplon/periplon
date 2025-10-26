# DSL TUI User Guide

Welcome to the DSL TUI (Text User Interface) - an interactive terminal application for creating, managing, and executing AI workflow definitions.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Interface Overview](#interface-overview)
3. [Working with Workflows](#working-with-workflows)
4. [Executing Workflows](#executing-workflows)
5. [Managing State](#managing-state)
6. [Tips and Tricks](#tips-and-tricks)

## Getting Started

### Installation

Build the TUI binary:

```bash
cargo build --release --bin periplon-tui --features tui
```

### Launching the TUI

```bash
# Launch with default workflow directory (~/.claude-sdk/workflows)
./target/release/periplon-tui

# Launch with custom workflow directory
./target/release/periplon-tui --workflow-dir ./my-workflows

# Launch with a specific workflow
./target/release/periplon-tui --workflow ./workflow.yaml

# Launch in readonly mode (prevents editing)
./target/release/periplon-tui --readonly

# Launch with custom theme
./target/release/periplon-tui --theme dark

# Enable debug logging
./target/release/periplon-tui --debug
```

### Command-line Options

- `--workflow-dir <PATH>`: Set the workflow directory (default: `~/.claude-sdk/workflows`)
- `--workflow <PATH>`: Load a specific workflow on startup
- `--readonly`: Launch in read-only mode (no editing or execution)
- `--theme <THEME>`: Set color theme (light, dark, solarized, monokai)
- `--debug`: Enable debug logging to `~/.claude-sdk/tui-debug.log`
- `-h, --help`: Show help information
- `-V, --version`: Show version information

## Interface Overview

### Main Screen

The TUI is organized into several main views accessible via tabs:

```
┌─ DSL TUI ─────────────────────────────────────────────────────┐
│ [Workflows] [Execution] [State] [Help]                         │
├────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Current view content appears here                             │
│                                                                 │
│                                                                 │
├────────────────────────────────────────────────────────────────┤
│ Status: Ready │ <F1> Help │ <Tab> Switch View │ <q> Quit      │
└────────────────────────────────────────────────────────────────┘
```

### Navigation

- **Tab / Shift+Tab**: Switch between views
- **Arrow Keys**: Navigate within lists and menus
- **Enter**: Select/activate item
- **Esc**: Go back/cancel
- **q**: Quit application
- **F1**: Show help

### Views

1. **Workflows View**: Browse, create, edit, and manage workflow files
2. **Execution View**: Monitor running workflows and view execution logs
3. **State View**: Browse saved workflow states and resume executions
4. **Help View**: Access documentation and keyboard shortcuts

## Working with Workflows

### File Browser

The Workflows view shows all workflow files in your workflow directory:

```
┌─ Workflows ────────────────────────────────────────────────────┐
│ 📁 my-workflows/                                                │
│   📄 research-workflow.yaml                                     │
│   📄 code-review.yaml                                           │
│   📁 templates/                                                 │
│     📄 basic-template.yaml                                      │
└────────────────────────────────────────────────────────────────┘
```

**Navigation**:
- **↑/↓**: Navigate files and folders
- **Enter**: Open workflow or expand folder
- **Backspace**: Go to parent directory
- **n**: Create new workflow
- **e**: Edit selected workflow
- **d**: Delete selected workflow (with confirmation)
- **r**: Rename selected workflow
- **v**: View workflow details (read-only)

### Creating Workflows

There are three ways to create workflows:

#### 1. From Template

Press `n` in the Workflows view and select "From Template":

1. Choose a template from the list
2. Enter a name for your new workflow
3. The template will be copied and opened in the editor

#### 2. Generate from Natural Language

Press `n` and select "Generate with AI":

1. Enter a description of what you want the workflow to do
2. The AI will generate a complete workflow
3. Review the generated workflow
4. Edit if needed and save

Example prompts:
- "Create a workflow that analyzes code quality and generates a report"
- "Build a multi-agent research workflow with summarization"
- "Create a workflow for automated code review with multiple specialized agents"

#### 3. Create Blank

Press `n` and select "Blank Workflow":

1. Enter a name for your workflow
2. A minimal workflow template is created
3. Edit the workflow to add your agents and tasks

### Editing Workflows

The workflow editor provides syntax highlighting and real-time validation:

```
┌─ Editor: research-workflow.yaml ──────────────────────────────┐
│ 1  name: "Research Workflow"                                   │
│ 2  version: "1.0.0"                                            │
│ 3  description: "Multi-agent research and analysis"           │
│ 4                                                              │
│ 5  agents:                                                     │
│ 6    researcher:                                               │
│ 7      description: "Research web sources"                     │
│ 8      tools: [WebSearch, WebFetch]                           │
│ 9                                                              │
│ 10 tasks:                                                      │
│ 11   research:                                                 │
│ 12     description: "Research ${workflow.topic}"               │
│ 13     agent: "researcher"                                     │
├────────────────────────────────────────────────────────────────┤
│ ✓ Valid YAML │ ⚠ 1 Warning: Missing output file              │
│ <Ctrl+S> Save │ <Esc> Cancel │ <F1> Help                      │
└────────────────────────────────────────────────────────────────┘
```

**Editor Commands**:
- **Ctrl+S**: Save changes
- **Esc**: Cancel and return to file browser
- **Ctrl+F**: Find text
- **Ctrl+H**: Find and replace
- **Ctrl+Z**: Undo
- **Ctrl+Y**: Redo
- **Tab**: Indent
- **Shift+Tab**: Unindent

**Editor Features**:
- **Syntax Highlighting**: YAML syntax is highlighted for readability
- **Real-time Validation**: Errors and warnings appear in the status bar
- **Auto-completion**: Press `Ctrl+Space` for suggestions (agents, tasks, tools)
- **Line Numbers**: Always visible for easy navigation
- **Bracket Matching**: Matching brackets are highlighted

### Workflow Validation

Workflows are validated in real-time as you edit. Validation checks include:

- ✓ **YAML Syntax**: Proper YAML structure
- ✓ **Schema**: Required fields present (name, agents, tasks)
- ✓ **Agent References**: All task agents exist
- ✓ **Dependencies**: Task dependencies are valid and acyclic
- ✓ **Tools**: Tool names are recognized
- ✓ **Variables**: Variable references are defined
- ✓ **Permissions**: Permission modes are valid

Validation messages appear in the status bar:
- **Green ✓**: Workflow is valid
- **Yellow ⚠**: Warnings (non-blocking)
- **Red ✗**: Errors (must be fixed)

### Viewing Workflow Details

Press `v` on a workflow to view its details without editing:

```
┌─ Workflow Details: research-workflow.yaml ────────────────────┐
│ Name: Research Workflow                                        │
│ Version: 1.0.0                                                 │
│ Description: Multi-agent research and analysis                │
│                                                                 │
│ Agents: 3                                                      │
│   • researcher (tools: WebSearch, WebFetch)                    │
│   • analyzer (tools: Read, Write)                              │
│   • summarizer (tools: Write)                                  │
│                                                                 │
│ Tasks: 5                                                       │
│   • research (agent: researcher)                               │
│   • analyze (agent: analyzer, depends on: research)            │
│   • summarize (agent: summarizer, depends on: analyze)         │
│                                                                 │
│ Variables: 2 inputs, 1 output                                  │
│   Inputs: topic (required), depth (default: "medium")          │
│   Outputs: summary_file                                        │
└────────────────────────────────────────────────────────────────┘
```

## Executing Workflows

### Starting Execution

From the Workflows view:

1. Select a workflow
2. Press `x` to execute
3. Configure execution options (if workflow has inputs)
4. Press Enter to start

The Execution view will automatically open showing live progress.

### Execution Monitor

```
┌─ Execution Monitor ────────────────────────────────────────────┐
│ Workflow: research-workflow.yaml                               │
│ Status: Running (Task 2/5)                                     │
│ Started: 2025-10-21 10:30:15                                   │
│                                                                 │
│ Task Progress:                                                 │
│ ✓ research      [========================================] 100%│
│ ► analyze       [==================>                   ]  45% │
│ ○ summarize     [                                      ]   0% │
│                                                                 │
│ Current Agent: analyzer                                        │
│ Current Task: analyze                                          │
│                                                                 │
│ Logs:                                                          │
│ [10:30:16] [researcher] Starting web search for topic...       │
│ [10:30:22] [researcher] Found 15 relevant sources              │
│ [10:30:25] [researcher] Task completed successfully            │
│ [10:30:26] [analyzer] Starting analysis of research data...    │
│ [10:30:30] [analyzer] Processing document 1 of 15...           │
│                                                                 │
│ <p> Pause │ <s> Stop │ <Esc> Background │ <F1> Help           │
└────────────────────────────────────────────────────────────────┘
```

**Execution Commands**:
- **p**: Pause execution (can resume later)
- **s**: Stop execution (cannot resume)
- **r**: Resume paused execution
- **Esc**: Move execution to background and return to browser
- **↑/↓**: Scroll through logs
- **PageUp/PageDown**: Fast scroll through logs

### Execution Status

Workflows can be in one of these states:

- **Running**: Currently executing (green)
- **Paused**: Execution paused by user (yellow)
- **Completed**: All tasks finished successfully (green)
- **Failed**: Execution failed with errors (red)
- **Stopped**: Manually stopped by user (gray)

### Background Execution

Press `Esc` during execution to background the workflow. It continues running while you browse other workflows or views. Background executions show in the status bar:

```
┌────────────────────────────────────────────────────────────────┐
│ Status: 1 running workflow │ <Tab> Execution View │ <q> Quit  │
└────────────────────────────────────────────────────────────────┘
```

Switch to the Execution view to monitor background workflows.

### Execution History

All executions are logged. View past executions in the Execution view by pressing `h`:

```
┌─ Execution History ────────────────────────────────────────────┐
│ Date       │ Workflow              │ Status    │ Duration      │
│────────────┼──────────────────────┼───────────┼───────────────│
│ 2025-10-21 │ research-workflow    │ Completed │ 5m 42s        │
│ 2025-10-21 │ code-review          │ Failed    │ 2m 18s        │
│ 2025-10-20 │ research-workflow    │ Completed │ 6m 05s        │
│                                                                 │
│ <Enter> View Details │ <d> Delete │ <Esc> Back                │
└────────────────────────────────────────────────────────────────┘
```

## Managing State

### State Persistence

Workflow state is automatically saved during execution, allowing you to:

- Resume paused workflows
- Recover from crashes
- Review execution history
- Debug failed workflows

### State Browser

The State view shows all saved workflow states:

```
┌─ Workflow States ──────────────────────────────────────────────┐
│ research-workflow.yaml                                         │
│   📝 2025-10-21 10:30:15 - Running (Task 2/5)                  │
│   ✓ 2025-10-21 09:15:22 - Completed                            │
│   ✗ 2025-10-20 16:45:10 - Failed (Task 3/5)                    │
│                                                                 │
│ code-review.yaml                                               │
│   ⏸ 2025-10-21 08:20:30 - Paused (Task 1/3)                   │
│   ✓ 2025-10-20 14:30:45 - Completed                            │
│                                                                 │
│ <Enter> View Details │ <r> Resume │ <d> Delete │ <Esc> Back   │
└────────────────────────────────────────────────────────────────┘
```

**State Commands**:
- **Enter**: View state details
- **r**: Resume paused or failed execution from last checkpoint
- **d**: Delete state (with confirmation)
- **e**: Export state to JSON file
- **↑/↓**: Navigate states

### State Details

View detailed information about a saved state:

```
┌─ State Details ────────────────────────────────────────────────┐
│ Workflow: research-workflow.yaml                               │
│ Status: Paused                                                 │
│ Created: 2025-10-21 10:30:15                                   │
│ Updated: 2025-10-21 10:35:42                                   │
│                                                                 │
│ Progress: 2/5 tasks completed (40%)                            │
│   ✓ research                                                   │
│   ✓ analyze                                                    │
│   ⏸ summarize (paused)                                         │
│   ○ review                                                     │
│   ○ publish                                                    │
│                                                                 │
│ Variables:                                                     │
│   workflow.topic = "AI agents"                                 │
│   task.analyze.result = "./analysis.json"                      │
│                                                                 │
│ <r> Resume │ <e> Export │ <Esc> Back                           │
└────────────────────────────────────────────────────────────────┘
```

### Resuming Execution

To resume a paused or failed workflow:

1. Navigate to the State view
2. Select the state to resume
3. Press `r`
4. Execution continues from the last checkpoint

The executor will:
- Restore all workflow variables
- Skip completed tasks
- Resume from the last incomplete task
- Use the same agents and configuration

## Tips and Tricks

### Keyboard Shortcuts

Learn the most useful shortcuts by pressing `F1` or see the [Shortcuts Reference](shortcuts.md).

### Quick Actions

- **Double-click** on workflow name to open in editor
- **Ctrl+X** from anywhere to execute current/selected workflow
- **Ctrl+N** to quickly create new workflow
- **/** to search workflows by name

### Working with Large Workflows

For workflows with many tasks:

1. Use the **Task Graph View** (`g` in Execution view) to visualize dependencies
2. Use **filters** (`f`) to show only running/failed tasks
3. **Collapse** completed task groups to reduce clutter

### Debugging Failed Workflows

When a workflow fails:

1. Check the **error message** in the execution log
2. View the **state details** to see which task failed
3. Review **task inputs** and **agent configuration**
4. **Edit workflow** to fix the issue
5. **Resume from state** to continue from last checkpoint

### Organizing Workflows

Best practices for organizing workflows:

- Use **subdirectories** for different projects or categories
- Name workflows descriptively: `project-feature-action.yaml`
- Use **templates** for common patterns
- Add **comments** in YAML for complex logic
- Use **version numbers** in workflow definitions

### Performance Tips

- Use `--theme light` for better performance on slow terminals
- Limit log verbosity in production workflows
- Use **background execution** for long-running workflows
- Clean up old states periodically with `d` in State view

### Customization

Create a config file at `~/.claude-sdk/tui-config.yaml`:

```yaml
# TUI Configuration
theme: dark
workflow_dir: ~/my-workflows
auto_save: true
log_level: info
max_log_lines: 1000
```

### Integration with CLI

The TUI uses the same workflow files as the CLI executor:

```bash
# Execute workflow from CLI
./target/release/periplon-executor run workflow.yaml

# Validate workflow from CLI
./target/release/periplon-executor validate workflow.yaml
```

Both tools share the same state directory, so you can:
- Start execution in TUI, monitor in CLI
- Resume TUI execution from CLI state
- Use CLI for automation, TUI for development

## Getting Help

- Press **F1** anywhere for context-sensitive help
- Visit the [Architecture Documentation](architecture.md) for technical details
- Check the [Troubleshooting Guide](troubleshooting.md) for common issues
- See [Shortcuts Reference](shortcuts.md) for all keyboard shortcuts

## Next Steps

- Read the [Architecture Documentation](architecture.md) to understand the design
- Check out [Example Workflows](../../examples/workflows/) for inspiration
- Join our community forum for tips and support

---

**Version**: 1.0.0
**Last Updated**: 2025-10-21
