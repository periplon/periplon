# TUI State Browser Guide

## Overview

The State Browser is an interactive TUI component for browsing, viewing, and resuming workflow execution states. It provides comprehensive state management features including filtering, sorting, detailed inspection, and workflow resumption.

## Features

### State List View

- **Browse States**: View all available workflow states in a scrollable list
- **Status Indicators**: Color-coded status badges (Running, Completed, Failed, Paused)
- **Progress Bars**: Visual progress indicators for each workflow
- **Task Summary**: Quick overview of completed/total tasks
- **Time Information**: Last checkpoint time displayed for each state
- **Filtering**: Real-time search/filter by workflow name or status
- **Sorting**: Multiple sort modes (name, modified time, progress)

### State Details View

- **Status Overview**: Current workflow status with color coding
- **Progress Display**: Percentage and visual progress bar
- **Execution Duration**: Time elapsed since workflow start
- **Task Breakdown**: Categorized task lists:
  - Completed tasks with results
  - Failed tasks with error messages
  - Pending tasks
- **Loop States**: Detailed iteration progress for loop tasks
- **Metadata**: Custom workflow metadata display
- **Scrollable Content**: Full navigation through detailed state information

### Controls and Navigation

#### List View Controls

| Key       | Action                          |
|-----------|---------------------------------|
| ↑/↓       | Navigate through state list     |
| Enter     | View detailed state information |
| s         | Cycle through sort modes        |
| /         | Filter states (search)          |
| r         | Resume selected workflow        |
| d         | Delete selected state           |
| ?         | Show help                       |
| q/Esc     | Return to workflow list         |

#### Details View Controls

| Key         | Action                     |
|-------------|----------------------------|
| ↑/↓         | Scroll content             |
| PgUp/PgDn   | Page up/down               |
| r           | Resume workflow (if resumable) |
| d           | Delete state               |
| Esc/q       | Return to list view        |

## Usage

### Accessing State Browser

From the main workflow list, press `s` to open the State Browser.

### Browsing States

1. Navigate through the list using arrow keys (↑/↓)
2. View status, progress, and task counts at a glance
3. Sort states by pressing `s` to cycle through sort modes:
   - Name (ascending/descending)
   - Modified time (ascending/descending)
   - Progress (ascending/descending)

### Filtering States

1. Press `/` to activate filter mode
2. Type to filter by workflow name or status
3. Filtered results update in real-time

### Viewing State Details

1. Select a workflow state in the list
2. Press Enter to view detailed information
3. Scroll through details using arrow keys or PgUp/PgDn
4. View:
   - Overall status and progress
   - Execution duration
   - Completed tasks and their results
   - Failed tasks with error messages
   - Pending tasks
   - Loop iteration progress (if applicable)
   - Custom metadata

### Resuming Workflows

Workflows in "Running" or "Paused" status can be resumed:

1. Select the workflow state
2. Press `r` to resume execution
3. The workflow will continue from its last checkpoint

### Deleting States

To clean up old or unwanted states:

1. Select the state to delete
2. Press `d`
3. Confirm deletion in the modal dialog

## Architecture Integration

### File Location

- **Implementation**: `src/tui/views/state_browser.rs`
- **State Management**: Integrated into `AppState` as `state_browser` field
- **View Mode**: Added `StateBrowser` variant to `ViewMode` enum

### State Persistence

The State Browser integrates with the DSL state persistence system:

- **State Directory**: `.workflow_states` by default
- **File Format**: JSON files with `.state.json` extension
- **State Loading**: Uses `StatePersistence` for all I/O operations

### Rendering Pipeline

1. **App::render()** routes to `StateBrowser` view mode
2. **render_state_browser()** dispatches to list or details view
3. List view renders state entries with progress bars
4. Details view renders comprehensive state information

### Event Handling

1. **App::handle_event()** routes key events based on view mode
2. **handle_state_browser_key()** processes navigation and actions
3. State browser updates its internal state
4. Re-render triggered automatically

## Data Structures

### StateBrowserState

Main state container for the browser:

```rust
pub struct StateBrowserState {
    states: Vec<StateEntry>,           // Loaded state entries
    selected_index: usize,             // Current selection
    current_state: Option<WorkflowState>, // Detailed state
    view_mode: StateBrowserViewMode,   // List or Details
    filter_query: String,              // Search filter
    sort_mode: StateSortMode,          // Current sort
    details_scroll: usize,             // Scroll position
    state_dir: PathBuf,                // State directory
}
```

### StateEntry

Compact state representation for list display:

```rust
pub struct StateEntry {
    workflow_name: String,
    workflow_version: String,
    status: WorkflowStatus,
    progress: f64,
    checkpoint_at: SystemTime,
    total_tasks: usize,
    completed_tasks: usize,
    failed_tasks: usize,
}
```

## Visual Design

### Color Scheme

- **Running**: Yellow status indicator
- **Completed**: Green status indicator
- **Failed**: Red status indicator
- **Paused**: Cyan status indicator
- **Progress Bars**:
  - < 50%: Red
  - 50-99%: Yellow
  - 100%: Green

### Layout

#### List View Layout

```
┌─────────────────────────────────────────────┐
│        Workflow State Browser               │ Header
├─────────────────────────────────────────────┤
│ Filter: (type to search)  Sort: Modified ↓  │ Filter/Sort
├─────────────────────────────────────────────┤
│ ► [Running] workflow1 v1.0 [████░░] (3/5)   │
│   [Completed] workflow2 v1.0 [█████] (5/5)  │ State List
│   [Failed] workflow3 v1.0 [██░░░] (2/5)     │
├─────────────────────────────────────────────┤
│ ↑/↓: Navigate | Enter: Details | q: Back    │ Status Bar
└─────────────────────────────────────────────┘
```

#### Details View Layout

```
┌─────────────────────────────────────────────┐
│     Workflow State: workflow1 v1.0          │ Header
├─────────────────────────────────────────────┤
│ Status: Running                             │
│ Progress: 60%  [███████░░░]                 │
│ Duration: 5m 30s                            │
│                                             │
│ Task Status:                                │ Details
│   Completed: 3                              │
│     ✓ task1                                 │
│     ✓ task2                                 │
│   Pending: 2                                │
│     ○ task3                                 │
├─────────────────────────────────────────────┤
│ r: Resume | d: Delete | Esc: Back           │ Controls
└─────────────────────────────────────────────┘
```

## Performance Considerations

- **Lazy Loading**: States loaded only when browser is opened
- **Filtered Display**: Only filtered states rendered in list
- **Buffered I/O**: State persistence uses buffered readers/writers
- **Pagination**: Details view supports large state files via scrolling
- **Minimal Re-renders**: Only update on state changes

## Testing

### Unit Tests

- `test_state_browser_creation`: Verify initialization
- `test_state_entry_from_state`: State entry conversion
- `test_sort_mode_cycle`: Sort mode cycling
- `test_progress_bar_rendering`: Visual elements
- `test_duration_formatting`: Time display
- `test_state_filtering`: Filter functionality

### Integration Points

1. State persistence system (`src/dsl/state.rs`)
2. Workflow executor for resume functionality
3. Theme system for consistent styling
4. Help system for context-sensitive help

## Future Enhancements

- [ ] Search history for filters
- [ ] Bulk state operations (delete multiple)
- [ ] State export/import
- [ ] Workflow state comparison
- [ ] State statistics and analytics
- [ ] Auto-refresh for running workflows
- [ ] State backup/restore
- [ ] Custom state directory configuration

## Related Documentation

- [TUI Architecture](architecture_analysis.md)
- [State Persistence](../../docs/DSL_IMPLEMENTATION.md#state-persistence)
- [Workflow Execution](../../docs/DSL_IMPLEMENTATION.md#execution)
- [Help System](../tui/help/docs/user_guide.md)
