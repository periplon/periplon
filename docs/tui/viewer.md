# TUI Workflow Viewer Component

## Overview

The workflow viewer is a read-only visualization component for DSL workflows in the TUI REPL. It provides two distinct view modes for examining workflow structure and content.

## Architecture

Located in `src/tui/ui/viewer.rs`, the component follows the hexagonal architecture pattern:

```
┌─────────────────────────────────────────┐
│         Workflow Viewer (UI)            │
│  ┌───────────────────────────────────┐  │
│  │   Header (metadata & view mode)   │  │
│  ├───────────────────────────────────┤  │
│  │                                   │  │
│  │   Content Area                    │  │
│  │   ┌─────────────────────────┐     │  │
│  │   │ Condensed View          │     │  │
│  │   │ - Workflow metadata     │     │  │
│  │   │ - Agent summaries       │     │  │
│  │   │ - Task summaries        │     │  │
│  │   │ - Input/Output specs    │     │  │
│  │   └─────────────────────────┘     │  │
│  │              OR                    │  │
│  │   ┌─────────────────────────┐     │  │
│  │   │ Full YAML View          │     │  │
│  │   │ - Syntax highlighted    │     │  │
│  │   │ - Complete workflow     │     │  │
│  │   └─────────────────────────┘     │  │
│  │                                   │  │
│  ├───────────────────────────────────┤  │
│  │   Status Bar (keybindings)        │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

## View Modes

### Condensed View

A structured summary view displaying:

1. **Workflow Metadata**
   - Name
   - Version
   - DSL Version
   - Working Directory (if set)

2. **Agents Section**
   - Agent ID with diamond marker (`◆`)
   - Description
   - Model (if specified)
   - Tools list
   - Max turns (if limited)

3. **Tasks Section**
   - Task ID with arrow marker (`▶`)
   - Description
   - Assigned agent
   - Dependencies
   - Subtask count

4. **Inputs Section**
   - Variable name
   - Type (string, number, boolean, etc.)
   - Required/Optional status

5. **Outputs Section**
   - Variable names

**Visual Features:**
- Section headers with decorative borders
- Color-coded elements:
  - Primary: Section headers
  - Accent: IDs and variable names
  - Success: Types and models
  - Warning: Required flags and dependencies
  - Muted: Labels and optional markers

### Full YAML View

Complete YAML representation with syntax highlighting:

- **Comments**: Dimmed italic text
- **Keys**: Bold accent color
- **String values**: Success color (green)
- **Numbers**: Warning color (yellow)
- **Booleans/Null**: Primary color (cyan)
- **List markers**: Accent color
- **Document separators**: Border color

## State Management

### ViewerState

Located in `src/tui/state.rs`:

```rust
pub struct ViewerState {
    pub view_mode: WorkflowViewMode,  // Condensed or Full
    pub scroll_offset: usize,          // Current scroll position
    pub page_size: usize,              // Lines per page (dynamic)
}
```

**Methods:**
- `toggle_view_mode()`: Switch between condensed and full views
- `scroll_up()`: Move up one line
- `scroll_down(max_lines)`: Move down one line (bounded)
- `page_up()`: Move up one page
- `page_down(max_lines)`: Move down one page (bounded)
- `scroll_to_top()`: Jump to beginning
- `scroll_to_bottom(max_lines)`: Jump to end
- `update_page_size(height)`: Adjust for terminal resize
- `reset()`: Reset to initial state

## Navigation

### Keybindings

| Key | Action |
|-----|--------|
| `Tab` | Toggle between condensed and full view |
| `↑` / `k` | Scroll up one line |
| `↓` / `j` | Scroll down one line |
| `PgUp` | Scroll up one page |
| `PgDn` | Scroll down one page |
| `Home` | Jump to top |
| `End` | Jump to bottom |
| `e` | Switch to editor mode |
| `Esc` | Return to workflow list |

### Scrolling

- Automatic scrollbar when content exceeds viewport
- Scroll position indicators (↑/↓) on scrollbar
- Smooth line-by-line scrolling
- Page-based navigation for quick jumps
- Boundary detection prevents over-scrolling

## Integration

### App Integration (src/tui/app.rs)

**State:**
```rust
pub struct AppState {
    pub viewer_state: ViewerState,
    pub current_workflow: Option<DSLWorkflow>,
    // ...
}
```

**View Mode:**
```rust
pub enum ViewMode {
    Viewer,  // Added for read-only visualization
    // ...
}
```

**Handler:**
```rust
async fn handle_viewer_key(&mut self, key: KeyEvent) -> Result<()> {
    // Navigation and view mode switching
}
```

**Rendering:**
```rust
fn render_viewer_static(frame, area, state, theme) {
    if let Some(workflow) = &state.current_workflow {
        viewer::render(frame, area, workflow, &state.viewer_state, theme);
    }
}
```

### Workflow List Integration

Opening a workflow from the list:
1. Loads workflow from file
2. Sets `current_workflow` in state
3. Resets viewer state
4. Switches to `ViewMode::Viewer`

## Rendering Pipeline

### Condensed View Rendering

1. **Header Section** (3 lines)
   - Workflow name, version, DSL version
   - Current view mode indicator
   - Styled with theme colors

2. **Content Section** (dynamic)
   - Build text lines from workflow structure
   - Apply color coding and formatting
   - Calculate total lines for scrolling

3. **Scrolling**
   - Apply scroll offset to content
   - Render visible portion only
   - Show scrollbar if needed

4. **Status Bar** (1 line)
   - Context-sensitive keybinding hints
   - Different hints per view mode

### Full YAML View Rendering

1. **Serialization**
   - Convert `DSLWorkflow` to YAML string
   - Handle serialization errors gracefully

2. **Syntax Highlighting**
   - Parse each line
   - Detect line type (comment, key-value, list, etc.)
   - Apply appropriate styling

3. **Rendering**
   - Same scrolling mechanism as condensed view
   - Preserve YAML indentation
   - No word wrapping

## Theme Integration

Uses `Theme` from `src/tui/theme.rs`:

```rust
pub struct Theme {
    pub primary: Color,    // Section headers, main highlights
    pub accent: Color,     // IDs, keys, markers
    pub success: Color,    // Types, models, string values
    pub warning: Color,    // Required flags, numbers
    pub error: Color,      // Error states (unused in viewer)
    pub muted: Color,      // Labels, optional markers, comments
    pub fg: Color,         // Normal text
    pub bg: Color,         // Background
    pub border: Color,     // Borders, separators
    pub highlight: Color,  // Selections (unused in viewer)
}
```

## Performance Considerations

1. **Lazy Rendering**
   - Only visible lines are rendered
   - Scrollbar calculations are efficient

2. **String Allocation**
   - YAML is serialized once per render
   - Line-by-line processing minimizes memory

3. **Scroll Bounds**
   - Max lines calculated dynamically
   - Prevents rendering off-screen content

4. **View Mode Toggle**
   - Scroll position resets on toggle
   - Prevents disorientation

## Future Enhancements

Potential improvements:

1. **Search/Filter**
   - Find text in condensed or full view
   - Jump to next/previous match
   - Highlight matches

2. **Collapse/Expand**
   - Fold sections in condensed view
   - Hide/show agents, tasks, etc.

3. **Copy to Clipboard**
   - Copy selected sections
   - Export to file

4. **Diff View**
   - Compare workflow versions
   - Show changes since last save

5. **Validation Indicators**
   - Highlight validation errors inline
   - Show warnings and suggestions

6. **Cross-References**
   - Jump to agent definition from task
   - Navigate dependency graph

## Testing

Manual testing checklist:

- [ ] Condensed view renders all sections correctly
- [ ] Full YAML view syntax highlighting works
- [ ] Tab toggles between views
- [ ] Scroll navigation (↑↓, PgUp/PgDn, Home/End)
- [ ] Scrollbar appears when content exceeds viewport
- [ ] Terminal resize updates page size
- [ ] Esc returns to workflow list
- [ ] 'e' key switches to editor mode
- [ ] Theme colors applied consistently
- [ ] Long workflows scroll correctly
- [ ] Empty sections don't render

## References

- Main implementation: `src/tui/ui/viewer.rs`
- State management: `src/tui/state.rs` (ViewerState, WorkflowViewMode)
- App integration: `src/tui/app.rs` (handle_viewer_key, render_viewer_static)
- Theme: `src/tui/theme.rs`
- Workflow schema: `src/dsl/schema.rs` (DSLWorkflow, AgentSpec, TaskSpec)
