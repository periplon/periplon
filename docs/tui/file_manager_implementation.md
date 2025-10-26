# File Manager Implementation

## Overview

The workflow file manager provides an interactive TUI for browsing, previewing, and managing DSL workflow files. It follows the same architectural patterns as the state browser for consistency.

**Location**: `src/tui/views/file_manager.rs`

## Features Implemented

### 1. **File Tree Navigation**
- Hierarchical directory tree with expandable/collapsible folders
- Visual tree indentation showing directory depth
- Directory expansion tracking with state persistence
- Navigate into directories with Enter key
- Navigate to parent directory with Backspace
- Icons for different file types:
  - 📁 Directories
  - ⚙️ Workflow files (.yaml, .yml)
  - 📄 Regular files

### 2. **Preview Pane**
- View file contents with syntax highlighting
- YAML syntax highlighting with color-coded:
  - Keys (bold, primary color)
  - Values (text color)
  - Comments (muted color)
  - List items (yellow)
- **Workflow Validation Integration**:
  - Automatic parsing of workflow files (.yaml, .yml)
  - Real-time validation using DSL validator
  - Visual validation status indicators:
    - ✅ Valid Workflow (green)
    - ❌ Validation Failed (red with error details)
    - ⚠️ Not a workflow file (yellow)
- Scrollable preview with:
  - Arrow keys for line-by-line scrolling
  - PgUp/PgDn for page scrolling
  - Visual scrollbar indicator
- Graceful error handling for unreadable files

### 3. **File Operations**

#### Quick Actions:
- **Open (o)**: Open workflow file in editor/viewer
- **Preview (p)**: View file contents with syntax highlighting
- **Delete (d)**: Remove file or directory
- **Rename (r)**: Rename file with inline input
- **Copy (c)**: Copy file to new name

#### Action Modes:
- Interactive input buffer for rename/copy operations
- Visual feedback showing current action
- Enter to confirm, Esc to cancel

### 4. **Search and Filter**
- Real-time filtering as you type
- Filter by filename or path
- Case-insensitive search
- Visual indication of filter query
- Filter persists across navigation

### 5. **Sorting Capabilities**

Seven sort modes with cycling:
- **Name ↑/↓**: Alphabetical sorting
- **Modified ↑/↓**: Sort by last modified time
- **Size ↑/↓**: Sort by file size
- **Type**: Group by file type (directories, workflows, files)

Press 's' to cycle through sort modes.

### 6. **Hidden Files**
- Toggle hidden files visibility with 'h' key
- Files starting with '.' are hidden by default
- Visual indicator showing hidden file status
- Persists across directory navigation

### 7. **Metadata Display**

For each file entry:
- File size in human-readable format (B, KB, MB, GB)
- Last modified time as relative time ("5m ago", "2h ago", "3d ago")
- File type indication through icons and colors
- Tree depth visualization

## Architecture

### State Management

```rust
pub struct FileManagerState {
    current_dir: PathBuf,              // Current directory path
    entries: Vec<FileEntry>,           // Flattened file tree
    selected_index: usize,             // Currently selected entry
    list_state: ListState,             // Ratatui list state
    view_mode: FileManagerViewMode,    // Tree or Preview
    preview_content: Option<String>,   // Cached preview
    preview_scroll: usize,             // Preview scroll position
    filter_query: String,              // Search query
    show_hidden: bool,                 // Show hidden files
    action_mode: FileActionMode,       // Current action
    input_buffer: String,              // Action input
    expanded_dirs: Vec<PathBuf>,       // Expanded directories
    sort_mode: FileSortMode,           // Current sort mode
    loaded_workflow: Option<DSLWorkflow>, // Loaded workflow (if valid)
    validation_errors: Vec<String>,    // Validation errors
}
```

### View Modes

```rust
pub enum FileManagerViewMode {
    Tree,    // File listing with tree navigation
    Preview, // File content preview with syntax highlighting
}
```

### Action Modes

```rust
pub enum FileActionMode {
    None,   // No action in progress
    Rename, // Renaming file (with input)
    Copy,   // Copying file (with input)
}
```

### File Entry

```rust
pub struct FileEntry {
    path: PathBuf,              // Full file path
    name: String,               // Display name
    is_dir: bool,               // Directory flag
    is_workflow: bool,          // Workflow file flag
    size: u64,                  // File size in bytes
    modified: SystemTime,       // Last modified time
    depth: usize,               // Tree depth level
}
```

## User Interface Layout

### Tree View
```
┌────────────────────────────────────────────────────────┐
│      Workflow File Manager - /path/to/workflows        │
├────────────────────────────────────────────────────────┤
│ Filter: (type to search) | Sort: Name ↑ | Hidden: Off  │
├────────────────────────────────────────────────────────┤
│ ► 📁 workflows/          -        5m ago               │
│   ► ⚙️ workflow1.yaml    2.5 KB   2h ago               │
│   ► ⚙️ workflow2.yml     1.8 KB   1d ago               │
│ ► 📁 templates/          -        3d ago               │
│ ► 📄 readme.txt          1.2 KB   1w ago               │
├────────────────────────────────────────────────────────┤
│         Actions: o: Open | p: Preview | d: Delete      │
│                  r: Rename | c: Copy                   │
├────────────────────────────────────────────────────────┤
│ ↑/↓: Navigate | Enter: Open | Backspace: Parent        │
│ s: Sort | h: Toggle Hidden | /: Filter | q: Back       │
└────────────────────────────────────────────────────────┘
```

### Preview View (with Validation)
```
┌────────────────────────────────────────────────────────┐
│         Preview: /path/to/workflows/test.yaml          │
├────────────────────────────────────────────────────────┤
│              Validation                                │
│              ✅ Valid Workflow                          │
├────────────────────────────────────────────────────────┤
│ name: Test Workflow                                    │
│ version: 1.0.0                                         │
│ description: A test workflow                           │
│                                                        │
│ agents:                                                │
│   researcher:                                          │
│     description: Research and gather information       │
│     tools: [WebSearch, Read]                          │
│                                                        │
│ tasks:                                                 │
│   research_topic:                                      │
│     agent: researcher                                  │
│     description: Research the given topic              │
│                                              ↑         │
│                                              █         │
│                                              ↓         │
├────────────────────────────────────────────────────────┤
│      Esc/q: Back | ↑/↓: Scroll | PgUp/PgDn: Page       │
└────────────────────────────────────────────────────────┘
```

## Key Bindings

| Key | Action |
|-----|--------|
| ↑/↓ | Navigate up/down in file list |
| Enter | Open directory / Preview file |
| Backspace | Navigate to parent directory |
| o | Open selected file |
| p | Preview selected file |
| d | Delete selected file/directory |
| r | Rename selected file |
| c | Copy selected file |
| s | Cycle sort mode |
| h | Toggle hidden files |
| / | Start/modify filter |
| Esc/q | Back to previous view / Cancel action |
| PgUp/PgDn | Page up/down (in preview) |

## Implementation Details

### Workflow Loading and Validation

The file manager integrates directly with the DSL parser and validator:

```rust
/// Load and validate a workflow file
pub fn load_workflow(&mut self, path: &Path) -> Result<()> {
    self.validation_errors.clear();

    match parse_workflow_file(path) {
        Ok(workflow) => {
            // Validate the workflow
            match validate_workflow(&workflow) {
                Ok(_) => {
                    self.loaded_workflow = Some(workflow);
                }
                Err(e) => {
                    self.loaded_workflow = Some(workflow);
                    self.validation_errors.push(format!("Validation error: {}", e));
                }
            }
        }
        Err(e) => {
            self.loaded_workflow = None;
            self.validation_errors.push(format!("Parse error: {}", e));
        }
    }

    Ok(())
}
```

**Key Features**:
- Automatic workflow parsing when previewing .yaml/.yml files
- Real-time validation against DSL schema
- Stores both workflow and validation errors
- Provides query methods: `get_loaded_workflow()`, `has_validation_errors()`, `get_validation_errors()`

### Directory Loading
- Recursive directory traversal with depth tracking
- Lazy loading of subdirectories (only when expanded)
- Efficient filtering during directory read
- Error handling for permission issues

### Sorting Algorithm
- Stable sort with directories always first
- Secondary sort by selected criteria
- Preserves tree structure during sort

### Preview Rendering
- Line-by-line syntax analysis
- YAML-specific highlighting rules:
  - Comment detection (#)
  - Key-value pair parsing (:)
  - List item detection (-)
- Maintains indentation for readability
- Scrollable with visual scrollbar

### File Type Detection
```rust
fn is_workflow_file(path: &Path) -> bool {
    matches!(
        path.extension().map(|e| e.to_string_lossy().to_lowercase()),
        Some("yaml") | Some("yml")
    )
}
```

### Human-Readable Formatting

**File Sizes**:
- Bytes (< 1 KB): "500 B"
- Kilobytes (< 1 MB): "2.5 KB"
- Megabytes (< 1 GB): "1.8 MB"
- Gigabytes: "2.1 GB"

**Time Ago**:
- Seconds (< 1 min): "30s ago"
- Minutes (< 1 hour): "15m ago"
- Hours (< 1 day): "5h ago"
- Days: "3d ago"

## Testing

Comprehensive integration test suite in `tests/file_manager_integration_test.rs`:

1. **Basic Operations**: File/directory detection, counting
2. **Navigation**: Selection, movement, parent navigation
3. **Directory Expansion**: Tree expansion/collapse
4. **Filtering**: Search functionality, filter accuracy
5. **Sorting**: All sort modes, order verification
6. **Preview**: Content loading, scrolling
7. **Hidden Files**: Toggle visibility
8. **Metadata**: Icons, sizes, times
9. **Large Directories**: Performance with 100+ files
10. **Empty Directories**: Edge case handling

All tests use temporary directories for isolation.

## Integration Points

### With Other TUI Components
- Uses shared `Theme` for consistent styling
- Compatible with `Layout` system from other views
- Follows same navigation patterns as state browser
- Integrates with workflow editor for opening files

### With DSL System
- **Full Integration**: Uses `parse_workflow_file()` from DSL parser
- **Validation**: Integrates with `validate_workflow()` for real-time validation
- **Workflow Detection**: Automatically detects .yaml/.yml workflow files
- **Execution Ready**: Loaded workflows can be passed to executor
- **Editor Integration**: Can open workflows in editor with validation context

## Future Enhancements

### Planned Features
1. **File Creation**: New workflow from template
2. **Batch Operations**: Multi-select with space bar
3. **Clipboard**: Copy/paste file paths
4. **Bookmarks**: Quick access to common directories
5. **Search History**: Recent searches
6. **Validation Preview**: Show validation errors inline
7. **Git Integration**: Show file status indicators
8. **Diff View**: Compare workflow versions
9. **Tree Collapse All**: Collapse all expanded directories
10. **Keyboard Shortcuts**: Customizable keybindings

### Performance Optimizations
- Virtual scrolling for very large directories
- Incremental directory loading
- Cached file metadata
- Background file scanning

## Usage Example

```rust
use periplon_sdk::tui::views::file_manager::{
    FileManagerState, render_file_manager
};
use periplon_sdk::tui::theme::Theme;
use std::path::PathBuf;

// Initialize file manager
let mut state = FileManagerState::new(
    PathBuf::from("./workflows")
).unwrap();

// In your TUI render loop
render_file_manager(&mut frame, area, &mut state, &theme);

// Handle events
match key_event.code {
    KeyCode::Down => state.select_next(),
    KeyCode::Up => state.select_previous(),
    KeyCode::Enter => {
        if let Some(entry) = state.selected_entry() {
            if entry.is_dir {
                state.toggle_directory().unwrap();
            } else {
                state.load_preview().unwrap();
            }
        }
    }
    KeyCode::Char('p') => state.load_preview().unwrap(),
    KeyCode::Char('s') => state.next_sort_mode(),
    KeyCode::Char('h') => state.toggle_hidden().unwrap(),
    KeyCode::Esc => state.back_to_tree(),
    // ... more key handlers
}
```

## Design Patterns

### Hexagonal Architecture Compliance
- Pure view logic, no business rules
- State management separated from rendering
- File system operations abstracted
- Easy to test with temporary directories

### Ratatui Best Practices
- Stateful widgets for list management
- Efficient rendering with scroll optimization
- Proper scrollbar state management
- Theme-based styling

### Error Handling
- Result types for all I/O operations
- Graceful degradation on permission errors
- User-friendly error messages
- No panics in user-facing code

## Conclusion

The file manager provides a robust, user-friendly interface for browsing and managing DSL workflow files. It integrates seamlessly with the existing TUI architecture while providing powerful features like real-time filtering, multiple sort modes, and syntax-highlighted previews.

**Key Achievements**:
- ✅ Full file tree navigation with expansion
- ✅ Syntax-highlighted YAML preview
- ✅ Comprehensive file operations (delete, rename, copy)
- ✅ Advanced filtering and sorting
- ✅ Hidden file support
- ✅ Human-readable metadata display
- ✅ Extensive test coverage
- ✅ Consistent with existing TUI patterns
