# DSL TUI Documentation

Comprehensive documentation for the DSL TUI (Text User Interface) application.

## Quick Links

### User Documentation
- **[User Guide](user-guide.md)** - Complete guide for using the TUI
- **[Keyboard Shortcuts](shortcuts.md)** - Full keyboard shortcuts reference
- **[Troubleshooting](troubleshooting.md)** - Common issues and solutions

### Developer Documentation
- **[Architecture](architecture.md)** - Technical architecture and design
- **[Developer Guide](developer-guide.md)** - Development workflow and contributing

## Getting Started

### Installation

```bash
# Build the TUI binary
cargo build --release --bin periplon-tui --features tui
```

### Launching

```bash
# Launch with default settings
./target/release/periplon-tui

# Launch with custom workflow directory
./target/release/periplon-tui --workflow-dir ./my-workflows

# Launch with specific workflow
./target/release/periplon-tui --workflow ./workflow.yaml

# Launch in readonly mode
./target/release/periplon-tui --readonly

# Launch with debug logging
./target/release/periplon-tui --debug
```

## What is the DSL TUI?

The DSL TUI is an interactive terminal application for creating, managing, and executing AI workflow definitions. It provides:

- **Visual Workflow Management**: Browse and organize workflows in a file tree
- **Interactive Editor**: Edit workflows with syntax highlighting and validation
- **AI-Powered Generation**: Create workflows from natural language
- **Real-time Monitoring**: Watch workflow execution with live logs
- **State Persistence**: Save and resume workflow executions
- **Help System**: Context-sensitive help and documentation

## Main Features

### 1. Workflow Browser
- Browse workflow files in directory tree
- Create new workflows (template, AI, or blank)
- Edit existing workflows
- View workflow details
- Execute workflows
- Delete and rename files

### 2. Workflow Editor
- YAML syntax highlighting
- Real-time validation
- Auto-completion for agents, tasks, tools
- Find and replace
- Undo/redo
- Error navigation

### 3. Execution Monitor
- Real-time progress tracking
- Task status visualization
- Live log streaming
- Pause/resume/stop controls
- Background execution
- Execution history

### 4. State Browser
- View saved workflow states
- Resume from checkpoints
- Export states to JSON
- Compare states
- Delete old states

### 5. Help System
- Context-sensitive help (F1)
- Embedded documentation
- Keyboard shortcuts reference
- Searchable help content

## Quick Start Tutorial

### 1. Create Your First Workflow

1. Launch TUI: `./target/release/periplon-tui`
2. Press `n` to create new workflow
3. Select "Generate with AI"
4. Enter description: "Analyze code quality and generate report"
5. Review generated workflow
6. Press `Ctrl+S` to save

### 2. Execute the Workflow

1. Select your workflow in browser
2. Press `x` to execute
3. Enter any required inputs
4. Monitor execution in Execution view
5. View results when complete

### 3. Resume from State

1. Switch to State view (Tab)
2. Select a saved state
3. Press `r` to resume
4. Execution continues from checkpoint

## Documentation Overview

### User Guide
Complete guide covering:
- Interface overview
- Navigation and keyboard shortcuts
- Creating and editing workflows
- Executing workflows
- Managing state
- Tips and tricks

### Keyboard Shortcuts
Comprehensive reference including:
- Global shortcuts
- View-specific shortcuts
- Editor commands
- Execution controls
- Customization options

### Architecture
Technical documentation covering:
- Hexagonal architecture design
- Component breakdown
- Data flow
- Extension points
- Testing strategy
- Performance considerations

### Developer Guide
Development guide including:
- Setup and tools
- Development workflow
- Adding features
- Testing approaches
- Debugging techniques
- Contributing guidelines

### Troubleshooting
Solutions for:
- Installation issues
- Startup problems
- Display issues
- Workflow issues
- Execution issues
- Performance issues
- Configuration issues

## Keyboard Shortcuts (Most Common)

| Key | Action |
|-----|--------|
| `F1` | Show help |
| `Tab` | Switch views |
| `q` | Quit |
| `Esc` | Go back |
| `↑↓` | Navigate |
| `Enter` | Select |
| `n` | New workflow |
| `e` | Edit workflow |
| `x` | Execute workflow |
| `Ctrl+S` | Save |

See [Keyboard Shortcuts](shortcuts.md) for complete reference.

## Views

The TUI has four main views accessible via Tab:

1. **Workflows** - File browser and workflow management
2. **Execution** - Real-time execution monitoring
3. **State** - Saved state browsing and resumption
4. **Help** - Documentation and help content

## Configuration

Optional configuration file at `~/.claude-sdk/tui-config.yaml`:

```yaml
# Theme
theme: dark  # light, dark, solarized, monokai

# Directories
workflow_dir: ~/.claude-sdk/workflows
state_dir: ~/.claude-sdk/states

# Behavior
auto_save: true
refresh_rate_ms: 100
max_log_lines: 1000

# Editor
editor:
  tab_size: 2
  auto_indent: true
  syntax_highlighting: true

# Keybindings
keybindings:
  quit: q
  help: F1
  save: Ctrl+S
```

## Architecture Highlights

The TUI follows **Hexagonal Architecture** (Ports & Adapters):

- **Domain Layer**: Pure business logic (workflows, tasks, agents)
- **Ports Layer**: Interface definitions (traits)
- **Application Layer**: Use case orchestration
- **Adapters Layer**: External system implementations
- **UI Layer**: Presentation and rendering

This design ensures:
- Clean separation of concerns
- Testability at all layers
- Easy extension and maintenance
- Independence from external systems

See [Architecture](architecture.md) for detailed explanation.

## Development

### Requirements
- Rust 1.70+
- Cargo
- Tokio async runtime

### Building
```bash
cargo build --features tui
```

### Testing
```bash
# All tests
cargo test --features tui

# TUI-specific tests
cargo test --test tui_unit_tests --features tui
cargo test --test tui_integration_tests --features tui

# With output
cargo test --features tui -- --nocapture
```

### Running
```bash
# Development mode
cargo run --bin periplon-tui --features tui -- --debug

# Auto-rebuild on changes
cargo watch -x 'run --bin periplon-tui --features tui'
```

See [Developer Guide](developer-guide.md) for complete development documentation.

## Contributing

Contributions welcome! Please:

1. Read the [Developer Guide](developer-guide.md)
2. Follow hexagonal architecture patterns
3. Add tests for new features
4. Update documentation
5. Submit pull request

## Support

- **Issues**: Report bugs and request features
- **Documentation**: Check guides in this directory
- **Help**: Press F1 in TUI for context help

## License

MIT OR Apache-2.0

---

**Version**: 1.0.0
**Last Updated**: 2025-10-21
