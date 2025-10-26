# AI Workflow Generator View

## Overview

The Generator View provides an interactive interface for creating and modifying DSL workflows using natural language descriptions. It leverages the agent SDK itself to convert natural language into structured YAML workflows with real-time validation and preview capabilities.

**Location**: `src/tui/views/generator.rs`

## Features

### Core Capabilities

1. **Natural Language Input**
   - Multiline text input for workflow descriptions
   - Cursor-based editing with visual feedback
   - Auto-scroll to keep cursor visible

2. **Dual Operation Modes**
   - **Create Mode**: Generate new workflows from scratch
   - **Modify Mode**: Edit existing workflows with NL instructions

3. **Real-time Preview**
   - Live YAML preview with syntax highlighting
   - Line-numbered display
   - Scrollable content view

4. **Diff Comparison**
   - Side-by-side diff view (Modify mode)
   - Original vs Generated comparison
   - Synchronized scrolling

5. **Validation Integration**
   - Automatic validation after generation
   - Error and warning display
   - Detailed feedback on validation issues

6. **Progress Tracking**
   - Generation status indicators
   - Progress messages during AI processing
   - Error handling and retry support

## Architecture

### State Management

```rust
pub struct GeneratorState {
    // Mode configuration
    pub mode: GeneratorMode,                    // Create or Modify

    // Input state
    pub nl_input: String,                       // User's NL description
    pub input_cursor: usize,                    // Cursor position
    pub input_scroll: usize,                    // Scroll offset

    // Workflow data
    pub original_yaml: Option<String>,          // Original (for Modify mode)
    pub generated_yaml: Option<String>,         // Generated YAML
    pub generated_workflow: Option<DSLWorkflow>, // Parsed workflow

    // Status tracking
    pub status: GenerationStatus,               // Current status
    pub focus: FocusPanel,                      // Active panel

    // View control
    pub preview_scroll: usize,                  // Preview scroll position
    pub diff_scroll: usize,                     // Diff scroll position
    pub show_diff: bool,                        // Diff vs preview toggle

    // Configuration
    pub agent_options: Option<AgentOptions>,    // Agent config
    pub output_path: Option<PathBuf>,           // Save destination
}
```

### Generation Status States

```rust
pub enum GenerationStatus {
    Idle,                                       // Ready for input
    InProgress { progress: String },            // Generating
    Completed,                                  // Generation succeeded
    Failed { error: String },                   // Generation failed
    Validating,                                 // Running validation
    Validated {                                 // Validation complete
        is_valid: bool,
        errors: Vec<String>,
        warnings: Vec<String>,
    },
}
```

## Layout Structure

```
┌────────────────────────────────────────────────────────────────┐
│ AI Workflow Generator | Mode: Create | Status: ✓              │
├──────────────────────────┬─────────────────────────────────────┤
│ Describe Your Workflow   │ Generated Workflow                  │
│                          │                                     │
│ [Natural language input] │    1 │ name: "My Workflow"         │
│ [with multiline support] │    2 │ version: "1.0.0"            │
│ [and cursor indicator]   │    3 │ agents:                     │
│                          │    4 │   researcher:               │
│                          │    5 │     description: "..."      │
│                          │                                     │
├──────────────────────────┴─────────────────────────────────────┤
│ Status                                                         │
│ ✓ Workflow generated successfully                             │
│ ✓ Workflow is valid and ready                                 │
├────────────────────────────────────────────────────────────────┤
│ Ctrl+G: Generate | Ctrl+A: Accept | Tab: Switch | Esc: Cancel │
└────────────────────────────────────────────────────────────────┘
```

### Diff View Layout (Modify Mode)

```
┌────────────────────────────────────────────────────────────────┐
│ AI Workflow Generator | Mode: Modify | Status: ✓              │
├──────────────────────────┬──────────────────┬──────────────────┤
│ Modification Instr.      │ Original         │ Generated        │
│                          │                  │                  │
│ [NL instructions for     │ name: "Old"      │ name: "New"      │
│  modifying the workflow] │ version: "1.0"   │ version: "1.0"   │
│                          │ agents: ...      │ agents: ...      │
│                          │                  │ tasks: ...       │
├──────────────────────────┴──────────────────┴──────────────────┤
│ Status: 2 modifications applied successfully                   │
└────────────────────────────────────────────────────────────────┘
```

## Usage Flow

### Creating a New Workflow

1. **Input Description**
   ```
   User: "Create a workflow to research Rust async programming,
          summarize key concepts, and write a tutorial blog post"
   ```

2. **Trigger Generation** (Ctrl+G)
   - Status changes to `InProgress`
   - Calls `nl_generator::generate_from_nl()`
   - Displays progress messages

3. **Preview Generated YAML**
   - Parsed workflow displayed with syntax highlighting
   - Automatic validation runs
   - Errors/warnings shown in status panel

4. **Accept or Retry**
   - Ctrl+A: Accept and save (if valid)
   - Ctrl+R: Retry with same or modified description
   - Tab: Switch between panels to review

### Modifying Existing Workflow

1. **Load Original**
   ```rust
   let state = GeneratorState::new_modify(original_yaml);
   ```

2. **Input Modifications**
   ```
   User: "Add a validation task that checks the output files
          before publishing"
   ```

3. **Generate & Compare**
   - Diff view shows original vs modified
   - Highlights changes made
   - Validates modified workflow

4. **Accept Changes**
   - Review diff carefully
   - Accept if changes are correct
   - Retry if not what was intended

## Key Functions

### Starting Generation

```rust
pub async fn start_generation(
    state: &mut GeneratorState
) -> Result<(), String>
```

**Process**:
1. Set status to `InProgress`
2. Call appropriate generator function:
   - `generate_from_nl()` for Create mode
   - `modify_workflow_from_nl()` for Modify mode
3. Update state with generated workflow
4. Auto-validate result
5. Update status to `Completed` or `Failed`

### Accepting Workflow

```rust
pub fn accept_workflow(
    state: &GeneratorState
) -> Result<DSLWorkflow, String>
```

**Actions**:
- Validates workflow exists
- Optionally saves to file
- Returns workflow for further use

### State Management

```rust
// Input editing
state.insert_char('a');
state.delete_char();
state.cursor_left();
state.cursor_right();

// View control
state.toggle_focus();      // Switch input/preview
state.toggle_diff();       // Toggle diff view

// Validation
state.validate_generated(); // Run validation
state.can_generate();      // Check if ready
state.can_accept();        // Check if acceptable
```

## Integration Points

### nl_generator Module

The view integrates directly with the `nl_generator` module:

```rust
use crate::dsl::nl_generator::{
    generate_from_nl,
    modify_workflow_from_nl,
};
```

**Automatic Retry Logic**:
- Handles validation errors automatically
- Retries generation with error feedback
- Max 3 retry attempts
- Shows progress to user

### Validation System

Uses the standard DSL validator:

```rust
use crate::dsl::validator::validate_workflow;
```

**Feedback Integration**:
- Errors displayed in status panel
- Line numbers for error locations
- Warning support
- Real-time validation status

### Theme System

Consistent styling with TUI theme:

```rust
use crate::tui::theme::Theme;

// Syntax highlighting
highlight_yaml_line(line, theme)

// Status colors
theme.success  // Valid workflows
theme.error    // Errors
theme.warning  // Warnings
theme.accent   // UI highlights
```

## Keyboard Shortcuts

### Input Panel (Focused)

- **Type**: Enter natural language description
- **Backspace**: Delete character
- **Ctrl+G**: Start generation
- **Tab**: Switch to preview panel
- **Esc**: Cancel and return

### Preview Panel (Focused)

- **Ctrl+A**: Accept generated workflow
- **Ctrl+R**: Retry generation
- **Ctrl+D**: Toggle diff view (Modify mode)
- **↑/↓**: Scroll preview
- **Tab**: Switch to input panel
- **Esc**: Cancel and return

## Error Handling

### Generation Failures

```rust
GenerationStatus::Failed { error }
```

**Causes**:
- YAML parsing errors
- Invalid agent responses
- Network/communication issues
- Validation failures (after max retries)

**User Experience**:
- Clear error messages
- Retry option available
- Previous input preserved
- Suggested fixes when possible

### Validation Errors

```rust
GenerationStatus::Validated {
    is_valid: false,
    errors: vec![...],
    warnings: vec![...],
}
```

**Display**:
- List of specific errors
- Line numbers when available
- Warnings separate from errors
- Accept disabled until valid

## Testing

### Unit Tests

```rust
#[test]
fn test_generator_state_create() {
    let state = GeneratorState::new_create();
    assert_eq!(state.mode, GeneratorMode::Create);
}

#[test]
fn test_input_editing() {
    let mut state = GeneratorState::new_create();
    state.insert_char('h');
    state.insert_char('i');
    assert_eq!(state.nl_input, "hi");
}

#[test]
fn test_can_generate() {
    let mut state = GeneratorState::new_create();
    state.nl_input = "Create workflow".to_string();
    assert!(state.can_generate());
}
```

### Integration Testing

Test with real workflow generation:

```rust
#[tokio::test]
async fn test_generate_simple_workflow() {
    let mut state = GeneratorState::new_create();
    state.nl_input = "Create a simple research workflow".to_string();

    let result = start_generation(&mut state).await;
    assert!(result.is_ok());
    assert!(state.generated_workflow.is_some());
}
```

## Best Practices

### For Users

1. **Be Specific**: Provide detailed descriptions
   - ✓ "Create a workflow with agents for research, analysis, and documentation"
   - ✗ "Make a workflow"

2. **Iterative Refinement**: Use modify mode to improve
   - Generate initial version
   - Review and identify issues
   - Use modify mode for adjustments

3. **Validate Early**: Check validation before accepting
   - Review all errors
   - Understand warnings
   - Test workflow before use

### For Developers

1. **State Consistency**: Always update status with workflow
   ```rust
   state.set_generated(yaml); // Handles both
   ```

2. **Error Handling**: Provide actionable messages
   ```rust
   GenerationStatus::Failed {
       error: format!("Parse error at line {}: {}", line, msg)
   }
   ```

3. **Async Operations**: Handle generation in event loop
   ```rust
   // In app event handler
   match event {
       KeyCode::Char('g') if ctrl_pressed => {
           tokio::spawn(async move {
               start_generation(&mut state).await
           });
       }
   }
   ```

## Future Enhancements

### Planned Features

1. **Template Selection**
   - Pre-built workflow templates
   - Quick-start options
   - Common patterns library

2. **Interactive Refinement**
   - Ask clarifying questions
   - Suggest improvements
   - Show alternative approaches

3. **History Tracking**
   - Previous generations
   - Modification history
   - Undo/redo support

4. **Export Options**
   - Multiple format support
   - Documentation generation
   - Diagram export

5. **Collaborative Features**
   - Share workflows
   - Import from library
   - Version control integration

## Examples

### Example 1: Research Workflow

**Input**:
```
Create a workflow to research quantum computing basics,
compile findings into a structured report, and create
a presentation deck.
```

**Generated** (excerpt):
```yaml
name: "Quantum Computing Research"
version: "1.0.0"
agents:
  researcher:
    description: "Research quantum computing fundamentals"
    tools: [WebSearch, Read, Write]
  analyst:
    description: "Compile and structure findings"
    tools: [Read, Write]
  presenter:
    description: "Create presentation materials"
    tools: [Read, Write]
tasks:
  research:
    agent: researcher
    description: "Research quantum computing basics"
    output: "research_notes.md"
  compile:
    agent: analyst
    description: "Create structured report"
    depends_on: [research]
    output: "report.md"
  present:
    agent: presenter
    description: "Create presentation deck"
    depends_on: [compile]
    output: "presentation.md"
```

### Example 2: Workflow Modification

**Original**:
```yaml
name: "Simple Test"
version: "1.0.0"
tasks:
  test:
    description: "Run tests"
    agent: tester
```

**Modification Input**:
```
Add a task to validate the test results and
send a notification if tests fail
```

**Modified**:
```yaml
name: "Simple Test"
version: "1.0.0"
tasks:
  test:
    description: "Run tests"
    agent: tester
    output: "test_results.json"
  validate:
    description: "Validate test results"
    agent: validator
    depends_on: [test]
    inputs:
      results: "${workflow.test.output}"
notifications:
  test_failure:
    channels:
      - type: email
        recipients: ["dev@example.com"]
    triggers:
      - condition: "task_failed"
        task: "test"
```

## See Also

- [Editor View](./editor_view.md) - Manual YAML editing
- [Viewer View](./viewer_view.md) - Workflow visualization
- [NL Generator Module](../dsl/nl_generator.md) - Backend generation logic
- [DSL Schema](../dsl/schema.md) - Workflow structure
- [Validation](../dsl/validation.md) - Workflow validation rules
