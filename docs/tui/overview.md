# DSL TUI Overview

Welcome to the DSL Workflow Terminal User Interface (TUI)!

## What is the DSL TUI?

The DSL TUI is an interactive terminal application for creating, managing, and executing AI agent workflows. It provides a user-friendly interface for working with DSL (Domain-Specific Language) workflows without needing to manually edit YAML files.

## Key Features

### 🎨 Interactive Workflow Management
- Browse and search workflows in a searchable list
- Create new workflows from templates or natural language
- Edit workflows with real-time validation
- Execute workflows and monitor progress

### 🤖 AI-Powered Generation
- Generate workflows from natural language descriptions
- Get intelligent suggestions and auto-completion
- Preview generated workflows before saving

### ✏️ Advanced Editor
- Syntax-highlighted YAML editing
- Real-time validation with inline error messages
- Form-based editing mode for structured input
- Undo/redo support

### 📊 Execution Monitoring
- Real-time task status visualization
- Dependency graph view
- Live log streaming with filtering
- Pause, resume, and stop controls

### 📚 Comprehensive Help System
- Context-aware help based on current view
- Searchable documentation
- Keyboard shortcut reference
- Markdown-formatted guides

## Architecture

The TUI follows hexagonal architecture patterns:

```
Primary Adapters (User Interface)
    ↓
Application Layer (TUI Application)
    ↓
Domain Layer (DSL Core)
    ↓
Secondary Adapters (File System, AI Service)
```

### Components

- **Workflow List**: Main hub for browsing and managing workflows
- **Viewer**: Read-only workflow visualization
- **Editor**: Interactive YAML editor with validation
- **Execution Monitor**: Real-time workflow execution tracking
- **AI Generator**: Natural language workflow creation
- **Help System**: Searchable documentation and guides

## Getting Started

1. **Launch**: `periplon-executor tui` or `cargo run --bin periplon-executor -- tui`
2. **Navigate**: Use arrow keys or vim-style navigation (hjkl)
3. **Get Help**: Press `?` or `F1` for context-sensitive help
4. **Create**: Press `n` to create a new workflow or `g` for AI generation
5. **Execute**: Select a workflow and press `x` to run it

## Navigation

- **Arrow Keys** or **hjkl**: Move up/down/left/right
- **Enter**: Select/Open
- **Esc**: Go back/Cancel
- **q**: Quit
- **?** or **F1**: Help

## Views

### 1. Workflow List
Your starting point for managing workflows.

**Actions:**
- `n` - New workflow
- `e` - Edit selected workflow
- `x` - Execute workflow
- `o` or `Enter` - Open in viewer
- `/` - Search workflows

### 2. Workflow Viewer
Read-only view with two modes:

- **Condensed**: Summary view showing key information
- **Full**: Complete YAML with syntax highlighting

**Toggle:** Press `v` to switch between modes

### 3. Workflow Editor
Full-featured YAML editor with:

- Real-time validation
- Syntax highlighting
- Auto-completion (Ctrl+Space)
- Undo/Redo (Ctrl+Z/Ctrl+Y)
- Form mode toggle (Tab)

**Save:** Ctrl+S

### 4. Execution Monitor
Track running workflows with:

- Task dependency graph
- Real-time status updates
- Log streaming
- Progress indicators

**Controls:**
- `Space` - Pause/Resume
- `s` - Stop execution
- `l` - Toggle log view
- `f` - Follow mode (auto-scroll)

### 5. AI Generator
Create workflows from natural language:

1. Describe what you want
2. Review generated YAML
3. Edit if needed
4. Save to workflow directory

## Best Practices

1. **Use Descriptive Names**: Make workflow and task names clear
2. **Validate Before Executing**: Use Ctrl+V in the editor
3. **Start Simple**: Test with basic workflows first
4. **Use Search**: Quick way to find workflows in large collections
5. **Leverage AI Generation**: Great for getting started quickly

## Technical Details

### File Locations
- Workflows: `./workflows/*.yaml`
- State: `./.dsl-state/*`
- Logs: Available in execution monitor

### Requirements
- Terminal with Unicode support
- Minimum 80x24 characters
- True color support recommended

### Performance
- Handles hundreds of workflows efficiently
- Real-time search with sub-100ms latency
- Streaming execution logs

## Troubleshooting

### TUI doesn't render correctly
- Ensure terminal supports Unicode
- Try resizing terminal
- Check terminal color settings

### Workflow validation errors
- Review inline error messages
- Check YAML syntax
- Verify agent and task references

### Execution issues
- Check execution monitor logs
- Verify agent permissions
- Review task dependencies

## Next Steps

- Read the [Getting Started Guide](#getting_started)
- Learn about [Editing Workflows](#editing_workflows)
- Explore [Keyboard Shortcuts](#keyboard_shortcuts_global)
- Try [AI Workflow Generation](#generating_workflows)

## Support

For issues and questions:
- Press `?` for context-sensitive help
- Search documentation with `/`
- Review troubleshooting guide
- Check project repository

---

**Version**: 1.0.0
**Last Updated**: 2025-10-21
