# DSL TUI User Guide

Welcome to the DSL TUI (Terminal User Interface) - your interactive workspace for creating, editing, and executing AI agent workflows.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Main Views](#main-views)
3. [Creating Workflows](#creating-workflows)
4. [Editing Workflows](#editing-workflows)
5. [Executing Workflows](#executing-workflows)
6. [Using Help](#using-help)
7. [Keyboard Reference](#keyboard-reference)
8. [Tips & Tricks](#tips--tricks)

## Getting Started

### Launching the TUI

```bash
# Using the DSL executor
dsl-executor tui

# Or with cargo
cargo run --bin dsl-executor -- tui
```

### First Time Setup

When you first launch the TUI, you'll see the **Workflow List** view. This is your home base for managing workflows.

**Quick Start Steps:**
1. Press `?` or `F1` to open help (you're reading it now!)
2. Press `n` to create your first workflow
3. Press `g` to generate a workflow with AI assistance
4. Use arrow keys or `hjkl` to navigate

### Navigation Basics

The TUI uses keyboard navigation for speed and efficiency:

- **Arrow Keys** or **hjkl**: Move around (vim-style)
- **Enter**: Select or open items
- **Esc**: Go back or cancel
- **?** or **F1**: Get help (context-aware)
- **q**: Quit (with confirmation if needed)

## Main Views

The TUI has five main views, each designed for a specific purpose.

### 1. Workflow List

**Purpose**: Browse, search, and manage your workflows

**Features:**
- Searchable list of all workflows
- Quick actions (edit, execute, delete)
- Sort by name, date, or version
- Visual status indicators

**Key Actions:**
| Key | Action |
|-----|--------|
| `n` | Create new workflow |
| `g` | Generate with AI |
| `e` | Edit selected workflow |
| `x` | Execute workflow |
| `o` or `Enter` | Open in viewer |
| `/` | Search workflows |
| `d` | Delete workflow |

### 2. Workflow Viewer

**Purpose**: Read-only visualization of workflow structure

**Features:**
- Two view modes: Condensed summary and Full YAML
- Syntax highlighting
- Scrollable content
- Dependency visualization

**Key Actions:**
| Key | Action |
|-----|--------|
| `v` | Toggle view mode (condensed/full) |
| `e` | Switch to editor |
| `↑/↓` | Scroll content |
| `PageUp/PageDown` | Scroll by page |

### 3. Workflow Editor

**Purpose**: Edit workflow YAML with real-time validation

**Features:**
- Syntax-highlighted editing
- Real-time validation with inline errors
- Auto-indentation
- Undo/redo support
- Form mode for structured editing

**Key Actions:**
| Key | Action |
|-----|--------|
| `Ctrl+S` | Save workflow |
| `Ctrl+V` | Validate YAML |
| `Ctrl+Z` | Undo |
| `Ctrl+Y` | Redo |
| `Tab` | Toggle text/form mode |
| `Ctrl+Space` | Auto-complete |

### 4. Execution Monitor

**Purpose**: Watch workflows run in real-time

**Features:**
- Live task status updates
- Dependency graph visualization
- Real-time log streaming
- Progress indicators
- Pause/resume/stop controls

**Key Actions:**
| Key | Action |
|-----|--------|
| `Space` | Pause/Resume execution |
| `s` | Stop execution |
| `l` | Toggle log view |
| `f` | Follow mode (auto-scroll) |
| `/` | Filter logs |

### 5. AI Generator

**Purpose**: Create workflows from natural language descriptions

**Features:**
- Natural language input
- AI-powered generation
- Preview before saving
- Edit generated YAML
- Example prompts

**Example Prompts:**
- "Create a workflow that analyzes code quality"
- "Build a pipeline for processing customer feedback"
- "Generate a workflow to backup and sync files"

## Creating Workflows

### Method 1: Manual Creation

1. Press `n` from the workflow list
2. Enter a workflow name
3. Choose a starting point:
   - **Empty template**: Start from scratch
   - **Example workflow**: Start from a template
   - **Copy existing**: Duplicate and modify

### Method 2: AI Generation

1. Press `g` from the workflow list
2. Describe what you want in natural language
3. Review the generated YAML
4. Edit if needed
5. Save with `Ctrl+S`

**AI Generation Tips:**
- Be specific about your goals
- Mention any tools you want to use (Read, Write, WebSearch, etc.)
- Describe the workflow steps in order
- Include any dependencies between tasks

### Method 3: Import

1. Place YAML files in your workflows directory
2. The TUI auto-discovers them
3. Open and edit as needed

## Editing Workflows

### Text Mode (Default)

Direct YAML editing with syntax highlighting:

```yaml
name: "My Workflow"
version: "1.0.0"
description: "What this workflow does"

agents:
  researcher:
    description: "Research and analyze"
    tools: [Read, Write, WebSearch]

tasks:
  analyze:
    description: "Analyze the data"
    agent: "researcher"
```

**Editing Tips:**
- Use 2-space indentation
- Quote strings with special characters
- Use lists for tools and dependencies
- Add descriptions for clarity

### Form Mode

Toggle with `Tab` for structured editing:
- Field-by-field input
- Auto-validation
- Reduced syntax errors
- Guided workflow creation

### Real-Time Validation

The editor validates as you type:

✓ **Valid YAML syntax**
✓ **Required fields present**
✓ **Agent references exist**
✓ **No circular dependencies**
✓ **Valid tool names**

Errors appear inline with:
- Line number
- Error description
- Suggested fix (when available)

### Auto-Completion

Press `Ctrl+Space` for suggestions:
- Workflow field names
- Agent properties
- Tool names
- Task attributes
- Variable references

## Executing Workflows

### Starting Execution

1. Select a workflow in the list
2. Press `x` to execute
3. Execution monitor opens automatically

### Monitoring Progress

The execution monitor shows:

1. **Task Graph**: Visual dependency tree
2. **Task Status**: Current state of all tasks
   - 🔵 Pending
   - 🟡 Running
   - 🟢 Completed
   - 🔴 Failed
   - ⚪ Skipped
3. **Logs**: Real-time output from agents
4. **Progress**: Overall completion percentage

### Execution Controls

| Action | Key | Description |
|--------|-----|-------------|
| Pause | `Space` | Pause execution (can resume) |
| Resume | `Space` | Continue paused execution |
| Stop | `s` | Stop execution (confirmation required) |
| Follow logs | `f` | Auto-scroll to latest logs |
| Filter logs | `/` | Search/filter log entries |

### Log Filtering

Press `/` in execution monitor to filter by:
- Task name
- Log level (info, warning, error)
- Agent name
- Keyword search

Example filters:
- `task:analyze` - Only logs from "analyze" task
- `level:error` - Only error messages
- `agent:researcher` - Only from "researcher" agent

## Using Help

### Context-Aware Help

Press `?` or `F1` from any view to get relevant help:

- **In Workflow List**: Help about managing workflows
- **In Editor**: Help about editing and YAML syntax
- **In Execution Monitor**: Help about monitoring execution
- **In Generator**: Help about AI generation

### Browsing Help

Help has three modes:

1. **Browse Mode**: Navigate by category and topic
   - Left panel: Categories
   - Right panel: Topics
   - Press `Enter` to open a topic

2. **Topic Mode**: Read help content
   - Rich markdown formatting
   - Scrollable content
   - Related topics at bottom

3. **Search Mode**: Find help quickly
   - Press `/` to search
   - Type to filter results
   - Results ranked by relevance

### Search Tips

- Use specific keywords: "keyboard shortcuts editor"
- Try partial words: "work" finds "workflow"
- Search is case-insensitive
- Results show relevance percentage

## Keyboard Reference

### Global (Works Everywhere)

| Key | Action |
|-----|--------|
| `?` or `F1` | Context-aware help |
| `Esc` | Go back / Cancel |
| `q` | Quit (with confirmation) |
| `/` | Search / Filter |
| `Ctrl+L` | Refresh screen |
| `Ctrl+C` | Force quit |

### Navigation

| Key | Action |
|-----|--------|
| `↑` or `k` | Move up |
| `↓` or `j` | Move down |
| `←` or `h` | Move left |
| `→` or `l` | Move right |
| `Home` or `g` | Go to top |
| `End` or `G` | Go to bottom |
| `PageUp` | Scroll page up |
| `PageDown` | Scroll page down |

### View Switching

| Key | Action |
|-----|--------|
| `1` | Workflow list |
| `2` | Viewer |
| `3` | Editor |
| `4` | Execution monitor |
| `5` | AI generator |

### Workflow List

| Key | Action |
|-----|--------|
| `n` | New workflow |
| `g` | Generate with AI |
| `e` | Edit workflow |
| `x` | Execute workflow |
| `o` or `Enter` | Open in viewer |
| `d` | Delete workflow |
| `r` | Rename workflow |
| `c` | Copy workflow |

### Editor

| Key | Action |
|-----|--------|
| `Ctrl+S` | Save |
| `Ctrl+V` | Validate |
| `Ctrl+Z` | Undo |
| `Ctrl+Y` | Redo |
| `Tab` | Toggle text/form mode |
| `Ctrl+Space` | Auto-complete |
| `Ctrl+F` | Find |
| `Ctrl+H` | Find and replace |

### Execution Monitor

| Key | Action |
|-----|--------|
| `Space` | Pause/Resume |
| `s` | Stop execution |
| `l` | Toggle log view |
| `t` | Toggle task graph |
| `f` | Follow mode |
| `/` | Filter logs |

## Tips & Tricks

### Workflow Management

1. **Use Descriptive Names**: Make workflows easy to identify
2. **Add Descriptions**: Help future you understand the purpose
3. **Version Your Workflows**: Use semantic versioning (1.0.0, 1.1.0, etc.)
4. **Organize by Purpose**: Use prefixes like "prod-", "test-", "dev-"

### Editing Efficiency

1. **Validate Often**: Press `Ctrl+V` regularly
2. **Use Form Mode**: Toggle with `Tab` for complex workflows
3. **Leverage Auto-Complete**: `Ctrl+Space` saves time
4. **Save Frequently**: `Ctrl+S` after major changes
5. **Use Undo/Redo**: Don't be afraid to experiment

### Search Power

1. **Use `/` Everywhere**: Quick way to find what you need
2. **Partial Matches**: Type first few letters
3. **Filter by Tags**: Add keywords to workflows for easy searching
4. **Recent Items**: Most recently modified appear at top

### Execution Monitoring

1. **Use Follow Mode**: Press `f` to auto-scroll logs
2. **Filter Noise**: Use `/` to focus on relevant logs
3. **Check Task Graph**: Understand dependencies visually
4. **Monitor Resource Usage**: Watch for performance issues

### AI Generation

1. **Be Specific**: Detailed prompts generate better workflows
2. **Iterate**: Generate, review, edit, regenerate
3. **Learn from Examples**: Study generated YAML structure
4. **Customize After**: Always review and customize generated workflows

### Troubleshooting

**Problem: Validation errors**
- Solution: Read inline error messages carefully
- Check YAML indentation (use 2 spaces)
- Verify agent and task names match

**Problem: Execution fails**
- Solution: Check execution monitor logs
- Verify agent permissions
- Review task dependencies

**Problem: TUI not rendering correctly**
- Solution: Ensure terminal supports Unicode
- Try resizing terminal window
- Check color settings

**Problem: Search not finding workflows**
- Solution: Check search query spelling
- Try broader search terms
- Verify workflows have content to search

## Advanced Features

### Variable Interpolation

Use variables in workflows:

```yaml
inputs:
  project_name:
    type: string
    required: true

tasks:
  analyze:
    description: "Analyze ${workflow.project_name}"
```

Scopes:
- `${workflow.var}`: Workflow-level
- `${agent.var}`: Agent-level
- `${task.var}`: Task-level

### Task Dependencies

Create workflows with complex dependencies:

```yaml
tasks:
  fetch:
    description: "Fetch data"
    agent: "fetcher"

  process:
    description: "Process data"
    agent: "processor"
    depends_on: [fetch]

  report:
    description: "Generate report"
    agent: "reporter"
    depends_on: [process]
```

### Hierarchical Tasks

Break down complex tasks:

```yaml
tasks:
  analyze:
    description: "Main analysis"
    agent: "analyzer"
    subtasks:
      - analyze_code
      - analyze_docs
      - generate_summary
```

## Getting More Help

### Within the TUI
- Press `?` or `F1` for context-aware help
- Use `/` to search help topics
- Browse help categories systematically

### Documentation
- Check `docs/tui/` directory for detailed guides
- Read help system documentation
- Review example workflows

### Community
- Report issues on GitHub
- Share workflows and tips
- Contribute improvements

## Quick Reference Card

```
Navigation:     ↑↓←→ or hjkl    View Switch:    1-5
Help:           ? or F1         Search:         /
Select:         Enter           Back:           Esc
Quit:           q               Refresh:        Ctrl+L

Workflow List:                  Editor:
  n - New                         Ctrl+S - Save
  g - Generate (AI)               Ctrl+V - Validate
  e - Edit                        Ctrl+Z - Undo
  x - Execute                     Tab - Toggle mode
  d - Delete                      Ctrl+Space - Complete

Execution:                      Help:
  Space - Pause/Resume            ↑↓ - Navigate
  s - Stop                        / - Search
  f - Follow logs                 Enter - Open topic
  / - Filter                      Esc - Back
```

---

**Version**: 1.0.0
**Last Updated**: 2025-10-21
**Status**: Complete

*This guide is embedded in the TUI help system. Press `?` or `F1` for context-aware assistance!*
