# Workflow Editor Design

## Overview

The interactive workflow editor provides real-time validation and two editing modes:
- **Text Mode**: YAML editor with syntax highlighting and inline validation markers
- **Form Mode**: Structured form-based editing with auto-completion

## Architecture

### Components

```
src/tui/views/editor.rs
├── EditorMode          # Text or Form editing mode
├── ValidationFeedback  # Real-time validation results
├── AutoCompletionSuggestion  # Completion suggestions
└── render()            # Main render function
```

### State Management

The `EditorState` (in `src/tui/state.rs`) maintains:
- Current editing mode (text/form)
- Workflow content (YAML string)
- Cursor position (line, column)
- Scroll offset for viewport navigation
- Modified flag for unsaved changes
- File path being edited
- Undo/redo history (up to 100 snapshots)

### Features

#### Text Mode Features

1. **Syntax Highlighting**
   - YAML key-value pairs highlighted
   - Comments in muted italic style
   - List items with accent color
   - Document separators

2. **Real-time Validation**
   - Inline error markers (❌) at error lines
   - Inline warning markers (⚠️) at warning lines
   - Validation panel showing all issues
   - Line number references for quick navigation

3. **Auto-completion** (Ctrl+Space)
   - DSL keyword completion
   - Context-aware suggestions
   - Categories: keyword, agent, task
   - All 20+ DSL keywords supported

4. **Editing Operations**
   - Character insertion/deletion
   - Newline handling with smart indentation
   - Cursor navigation (up/down/left/right)
   - Undo/Redo (Ctrl+Z / Ctrl+Y)

#### Form Mode Features

1. **Structured View**
   - Workflow metadata section
   - Agents section with details
   - Tasks section with dependencies
   - Inputs/outputs sections

2. **Parse Error Handling**
   - Graceful fallback when YAML invalid
   - Helpful error messages
   - Suggestion to switch to text mode

### Keyboard Shortcuts

**Common**
- `Tab`: Toggle between Text/Form mode
- `Ctrl+S`: Save workflow
- `Ctrl+V`: Validate workflow
- `Esc`: Back to previous view

**Text Mode**
- `Arrow keys`: Move cursor
- `Ctrl+Space`: Auto-completion
- `Ctrl+Z`: Undo
- `Ctrl+Y`: Redo
- `Backspace`: Delete character
- `Enter`: New line

**Form Mode**
- `↑↓`: Navigate fields
- `Enter`: Edit field

### Validation Integration

The editor integrates with the DSL validator:

```rust
// Validation pipeline
parse_workflow_content(yaml)  // Parse YAML
  ↓
validate_workflow(workflow)   // Semantic validation
  ↓
ValidationFeedback            // Display errors/warnings
```

Validation checks:
- YAML syntax correctness
- Agent reference validity
- Task dependency cycles
- Variable reference resolution
- Tool availability
- Permission mode correctness

### Visual Layout

```
┌─────────────────────────────────────────────────────────┐
│ Editing: workflow.yaml [Modified] | Mode: Text | ✓ Valid│
├─────────────────────────────────────────────────────────┤
│   1 │ name: "Example Workflow"                          │
│   2 │ version: "1.0.0"                                  │
│   3 │ agents:                                           │
│   4 │   researcher:                                     │
│   5 │     description: "Research agent"                 │
│     │ ❌ Line 5: Missing 'agent' reference              │
│   6 │                                                   │
├─────────────────────────────────────────────────────────┤
│ Validation                                              │
│ Errors (1)                                              │
│   ❌ Line 5: Task 'analyze' references non-existent... │
├─────────────────────────────────────────────────────────┤
│ Tab: Form | Ctrl+S: Save | Ctrl+V: Validate | Esc: Back│
└─────────────────────────────────────────────────────────┘
```

### DSL Keywords for Auto-completion

All major DSL keywords are supported:
- Workflow metadata: `name:`, `version:`, `dsl_version:`
- Definitions: `agents:`, `tasks:`, `workflows:`, `inputs:`, `outputs:`
- Configuration: `tools:`, `communication:`, `mcp_servers:`
- Advanced: `subflows:`, `imports:`, `notifications:`
- Agent fields: `description:`, `model:`, `system_prompt:`, `permissions:`
- Task fields: `agent:`, `depends_on:`, `subtasks:`, `output:`

### Testing

The editor includes comprehensive unit tests:
- Auto-completion suggestion matching
- YAML syntax highlighting
- Validation feedback for valid/invalid workflows
- Line number extraction from error messages

## Usage Example

```rust
use crate::tui::views::editor::{render, validate_and_get_feedback, EditorMode};
use crate::tui::state::EditorState;
use crate::tui::theme::Theme;

let mut state = EditorState::new();
state.load_content(yaml_content, Some(PathBuf::from("workflow.yaml")));

let feedback = validate_and_get_feedback(&state.content);

// Render editor
render(&mut frame, area, &state, &feedback, &theme);
```

## Future Enhancements

1. **Advanced Auto-completion**
   - Context-aware agent name suggestions
   - Task dependency auto-completion from existing tasks
   - Variable reference auto-completion

2. **Quick Fixes**
   - One-click fixes for common errors
   - Automatic formatting
   - Missing field insertion

3. **Advanced Form Mode**
   - Interactive field editing
   - Dropdown selections for enums
   - In-place validation

4. **Diff View**
   - Show changes before saving
   - Compare with saved version
   - Selective undo for specific changes

5. **Multi-file Editing**
   - Split panes for subflows
   - Cross-file navigation
   - Workspace-wide search/replace
