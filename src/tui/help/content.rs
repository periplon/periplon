//! Help content database
//!
//! Structured help content with topics, sections, and embedded documentation.

use std::collections::HashMap;

/// Help topic with title, content, and metadata
#[derive(Debug, Clone)]
pub struct HelpTopic {
    /// Unique topic identifier
    pub id: String,
    /// Topic title
    pub title: String,
    /// Topic content (markdown format)
    pub content: String,
    /// Related topics
    pub related: Vec<String>,
    /// Search keywords
    pub keywords: Vec<String>,
    /// Topic category
    pub category: HelpCategory,
}

/// Help section grouping related topics
#[derive(Debug, Clone)]
pub struct HelpSection {
    /// Section title
    pub title: String,
    /// Section topics
    pub topics: Vec<HelpTopic>,
}

/// Help category enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HelpCategory {
    GettingStarted,
    WorkflowManagement,
    Editing,
    Execution,
    KeyboardShortcuts,
    Advanced,
    Troubleshooting,
}

impl HelpCategory {
    /// Get category display name
    pub fn name(&self) -> &'static str {
        match self {
            HelpCategory::GettingStarted => "Getting Started",
            HelpCategory::WorkflowManagement => "Workflow Management",
            HelpCategory::Editing => "Editing Workflows",
            HelpCategory::Execution => "Workflow Execution",
            HelpCategory::KeyboardShortcuts => "Keyboard Shortcuts",
            HelpCategory::Advanced => "Advanced Features",
            HelpCategory::Troubleshooting => "Troubleshooting",
        }
    }
}

/// Complete help content database
#[derive(Debug)]
pub struct HelpContent {
    /// All help topics indexed by ID
    topics: HashMap<String, HelpTopic>,
    /// Topics grouped by category
    categories: HashMap<HelpCategory, Vec<String>>,
}

impl HelpContent {
    /// Create new help content database
    pub fn new() -> Self {
        let mut content = Self {
            topics: HashMap::new(),
            categories: HashMap::new(),
        };
        content.initialize_content();
        content
    }

    /// Initialize all help content
    fn initialize_content(&mut self) {
        // Getting Started
        self.add_topic(HelpTopic {
            id: "overview".to_string(),
            title: "TUI Overview".to_string(),
            content: include_str!("../../../docs/tui/overview.md").to_string(),
            related: vec!["getting_started".to_string(), "navigation".to_string()],
            keywords: vec![
                "intro".to_string(),
                "introduction".to_string(),
                "overview".to_string(),
            ],
            category: HelpCategory::GettingStarted,
        });

        self.add_topic(HelpTopic {
            id: "getting_started".to_string(),
            title: "Getting Started".to_string(),
            content: r#"# Getting Started with DSL TUI

Welcome to the DSL Workflow TUI (Terminal User Interface)! This interactive tool helps you create, edit, and execute AI agent workflows.

## Quick Start

1. **Launch the TUI**: Run `dsl-executor tui` or `cargo run --bin dsl-executor -- tui`
2. **Navigate**: Use arrow keys or vim keys (hjkl) to move around
3. **Create a workflow**: Press `n` to create a new workflow
4. **Edit a workflow**: Select a workflow and press `Enter` or `e`
5. **Execute a workflow**: Press `x` to run the selected workflow
6. **Get help**: Press `?` or `F1` at any time

## Main Views

- **Workflow List**: Browse and manage your workflows
- **Viewer**: Read-only workflow visualization
- **Editor**: Edit workflow YAML with validation
- **Execution Monitor**: Watch workflows run in real-time
- **AI Generator**: Create workflows from natural language

## Navigation Basics

- `↑/↓` or `k/j`: Move up/down
- `Enter`: Select/Open
- `Esc`: Go back/Cancel
- `?` or `F1`: Context-sensitive help
- `q`: Quit (with confirmation)

## Next Steps

- Read about [Workflow Management](#navigating_workflows)
- Learn [Keyboard Shortcuts](#keyboard_shortcuts_global)
- Explore the [Editor](#editing_workflows)
"#.to_string(),
            related: vec!["overview".to_string(), "keyboard_shortcuts_global".to_string()],
            keywords: vec!["start".to_string(), "begin".to_string(), "intro".to_string(), "tutorial".to_string()],
            category: HelpCategory::GettingStarted,
        });

        // Workflow Management
        self.add_topic(HelpTopic {
            id: "navigating_workflows".to_string(),
            title: "Navigating Workflows".to_string(),
            content: r#"# Navigating Workflows

The Workflow List is your main hub for managing DSL workflows.

## Workflow List View

- **Up/Down**: Navigate through workflows using arrow keys or `k/j`
- **Search**: Press `/` to search workflows by name or description
- **Filter**: Type to filter the list in real-time
- **Clear search**: Press `Esc` to clear the search query

## Workflow Actions

| Key | Action |
|-----|--------|
| `Enter` or `o` | Open workflow in viewer |
| `e` | Edit workflow in editor |
| `x` | Execute workflow |
| `n` | Create new workflow |
| `d` | Delete selected workflow |
| `r` | Rename workflow |
| `c` | Copy workflow |

## Workflow Details

Each workflow entry shows:
- **Name**: Workflow identifier
- **Description**: Brief summary (if available)
- **Version**: Workflow version number
- **Modified**: Last modification time
- **Status**: Execution status indicator

## Tips

- Use search (`/`) to quickly find workflows in large lists
- Press `?` for context-specific help
- Workflows are auto-discovered from your workflows directory
"#
            .to_string(),
            related: vec![
                "creating_workflows".to_string(),
                "keyboard_shortcuts_list".to_string(),
            ],
            keywords: vec![
                "browse".to_string(),
                "list".to_string(),
                "manage".to_string(),
                "search".to_string(),
            ],
            category: HelpCategory::WorkflowManagement,
        });

        self.add_topic(HelpTopic {
            id: "creating_workflows".to_string(),
            title: "Creating Workflows".to_string(),
            content: r#"# Creating Workflows

There are multiple ways to create workflows in the TUI.

## Method 1: AI Generator (Recommended)

1. Press `g` from the workflow list
2. Describe your workflow in natural language
3. Review the generated YAML
4. Edit if needed and save

Example prompt:
> "Create a workflow that analyzes a codebase for security issues, generates a report, and sends it via email"

## Method 2: Manual Creation

1. Press `n` from the workflow list
2. Enter a workflow name
3. Choose to start from:
   - Empty template
   - Example workflow
   - Copy existing workflow

## Method 3: Import

1. Place YAML file in workflows directory
2. TUI auto-discovers it
3. Edit as needed

## Workflow Structure

```yaml
name: "My Workflow"
version: "1.0.0"
description: "Optional description"

agents:
  agent_id:
    description: "What this agent does"
    tools: [Read, Write, WebSearch]

tasks:
  task_id:
    description: "Task description"
    agent: "agent_id"
    depends_on: []
```

## Best Practices

- Use descriptive names for workflows, agents, and tasks
- Add descriptions to help others understand intent
- Test workflows with simple tasks first
- Use the validator before executing

See [Editing Workflows](#editing_workflows) for more details.
"#.to_string(),
            related: vec!["editing_workflows".to_string(), "generating_workflows".to_string()],
            keywords: vec!["new".to_string(), "create".to_string(), "generate".to_string(), "template".to_string()],
            category: HelpCategory::WorkflowManagement,
        });

        // Editing
        self.add_topic(HelpTopic {
            id: "editing_workflows".to_string(),
            title: "Editing Workflows".to_string(),
            content: r#"# Editing Workflows

The workflow editor provides real-time validation and syntax highlighting.

## Editor Modes

### Text Mode (Default)
- Direct YAML editing
- Syntax highlighting
- Real-time validation
- Auto-indentation

### Form Mode
- Structured forms for workflow components
- Field validation
- Auto-completion
- Toggle with `Tab`

## Editing Commands

| Key | Action |
|-----|--------|
| `Ctrl+S` | Save workflow |
| `Ctrl+V` | Validate YAML |
| `Ctrl+Z` | Undo |
| `Ctrl+Y` | Redo |
| `Tab` | Toggle text/form mode |
| `Esc` | Cancel (prompts if unsaved) |

## Navigation

- **Arrow keys**: Move cursor
- **Home/End**: Start/end of line
- **PageUp/PageDown**: Scroll page
- **Ctrl+Home/End**: Start/end of document

## Validation

The editor validates:
- ✓ YAML syntax
- ✓ Required fields
- ✓ Agent references
- ✓ Task dependencies
- ✓ Circular dependency detection
- ✓ Variable references

Validation errors appear inline with:
- Line number
- Error description
- Suggested fix (when available)

## Auto-completion

Press `Ctrl+Space` for context-aware suggestions:
- Workflow fields
- Agent properties
- Tool names
- Task attributes

## Tips

- Save frequently with `Ctrl+S`
- Use validation (`Ctrl+V`) before executing
- Form mode helps prevent syntax errors
- Undo history preserved across saves
"#
            .to_string(),
            related: vec![
                "yaml_syntax".to_string(),
                "validation".to_string(),
                "keyboard_shortcuts_editor".to_string(),
            ],
            keywords: vec![
                "edit".to_string(),
                "modify".to_string(),
                "change".to_string(),
                "yaml".to_string(),
            ],
            category: HelpCategory::Editing,
        });

        // Execution
        self.add_topic(HelpTopic {
            id: "monitoring_execution".to_string(),
            title: "Monitoring Execution".to_string(),
            content: r#"# Monitoring Workflow Execution

The execution monitor provides real-time visibility into running workflows.

## Execution View

The monitor displays:

1. **Task Graph**: Visual representation of task dependencies
2. **Current Status**: Which tasks are running, completed, or failed
3. **Logs**: Real-time output from agents and tasks
4. **Progress**: Overall completion percentage
5. **Timing**: Elapsed time and estimated completion

## Task States

- 🔵 **Pending**: Waiting to start
- 🟡 **Running**: Currently executing
- 🟢 **Completed**: Successfully finished
- 🔴 **Failed**: Error occurred
- ⚪ **Skipped**: Dependency failed

## Controls

| Key | Action |
|-----|--------|
| `Space` | Pause/Resume execution |
| `s` | Stop execution (with confirmation) |
| `l` | Toggle log view |
| `t` | Toggle task graph view |
| `f` | Follow mode (auto-scroll logs) |
| `↑/↓` | Scroll logs |

## Log Filtering

Press `/` to filter logs by:
- Task name
- Log level (info, warning, error)
- Agent name
- Keyword search

## Execution States

- **Running**: Workflow is executing
- **Paused**: Execution paused (can resume)
- **Completed**: All tasks finished successfully
- **Failed**: One or more tasks failed
- **Cancelled**: Stopped by user

## Tips

- Use follow mode (`f`) to auto-scroll logs
- Filter logs to focus on specific tasks
- Check task graph for dependency issues
- Review failed task logs for debugging
"#
            .to_string(),
            related: vec![
                "task_status".to_string(),
                "keyboard_shortcuts_monitor".to_string(),
            ],
            keywords: vec![
                "execute".to_string(),
                "run".to_string(),
                "monitor".to_string(),
                "logs".to_string(),
                "status".to_string(),
            ],
            category: HelpCategory::Execution,
        });

        // Keyboard Shortcuts
        self.add_topic(HelpTopic {
            id: "keyboard_shortcuts_global".to_string(),
            title: "Global Keyboard Shortcuts".to_string(),
            content: r#"# Global Keyboard Shortcuts

These shortcuts work in all views.

## Navigation

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

## General

| Key | Action |
|-----|--------|
| `?` or `F1` | Help (context-aware) |
| `Esc` | Back/Cancel |
| `q` | Quit (with confirmation) |
| `/` | Search/Filter |
| `Ctrl+C` | Force quit |
| `Ctrl+L` | Refresh screen |

## View Switching

| Key | Action |
|-----|--------|
| `1` | Workflow list |
| `2` | Viewer |
| `3` | Editor |
| `4` | Execution monitor |
| `5` | AI generator |

See also:
- [Workflow List Shortcuts](#keyboard_shortcuts_list)
- [Editor Shortcuts](#keyboard_shortcuts_editor)
- [Monitor Shortcuts](#keyboard_shortcuts_monitor)
"#
            .to_string(),
            related: vec![
                "keyboard_shortcuts_list".to_string(),
                "keyboard_shortcuts_editor".to_string(),
            ],
            keywords: vec![
                "shortcuts".to_string(),
                "keys".to_string(),
                "hotkeys".to_string(),
                "bindings".to_string(),
            ],
            category: HelpCategory::KeyboardShortcuts,
        });

        self.add_topic(HelpTopic {
            id: "keyboard_shortcuts_list".to_string(),
            title: "Workflow List Shortcuts".to_string(),
            content: r#"# Workflow List Keyboard Shortcuts

Shortcuts specific to the workflow list view.

## Navigation

| Key | Action |
|-----|--------|
| `↑/↓` or `k/j` | Navigate workflows |
| `Enter` or `o` | Open in viewer |
| `/` | Search workflows |

## Actions

| Key | Action |
|-----|--------|
| `n` | New workflow |
| `e` | Edit workflow |
| `x` | Execute workflow |
| `d` | Delete workflow |
| `r` | Rename workflow |
| `c` | Copy workflow |
| `g` | Generate with AI |

## Sorting

| Key | Action |
|-----|--------|
| `s` | Sort menu |
| `sn` | Sort by name |
| `sm` | Sort by modified date |
| `sv` | Sort by version |

All global shortcuts also apply. Press `?` for context-specific help.
"#
            .to_string(),
            related: vec![
                "navigating_workflows".to_string(),
                "keyboard_shortcuts_global".to_string(),
            ],
            keywords: vec![
                "list".to_string(),
                "shortcuts".to_string(),
                "workflow list".to_string(),
            ],
            category: HelpCategory::KeyboardShortcuts,
        });

        self.add_topic(HelpTopic {
            id: "keyboard_shortcuts_editor".to_string(),
            title: "Editor Keyboard Shortcuts".to_string(),
            content: r#"# Editor Keyboard Shortcuts

Shortcuts for the workflow editor.

## File Operations

| Key | Action |
|-----|--------|
| `Ctrl+S` | Save |
| `Ctrl+Q` | Save and quit |
| `Esc` | Cancel (confirm if modified) |

## Editing

| Key | Action |
|-----|--------|
| `Ctrl+Z` | Undo |
| `Ctrl+Y` or `Ctrl+Shift+Z` | Redo |
| `Ctrl+X` | Cut line |
| `Ctrl+C` | Copy line |
| `Ctrl+V` | Paste (or Validate if no clipboard) |
| `Tab` | Indent / Toggle mode |
| `Shift+Tab` | Unindent |

## Navigation

| Key | Action |
|-----|--------|
| `Arrow keys` | Move cursor |
| `Home` | Start of line |
| `End` | End of line |
| `Ctrl+Home` | Start of document |
| `Ctrl+End` | End of document |
| `Ctrl+G` | Go to line |

## Features

| Key | Action |
|-----|--------|
| `Ctrl+Space` | Auto-complete |
| `Ctrl+V` | Validate YAML |
| `Ctrl+F` | Find |
| `Ctrl+H` | Find and replace |

All global shortcuts also apply.
"#
            .to_string(),
            related: vec![
                "editing_workflows".to_string(),
                "keyboard_shortcuts_global".to_string(),
            ],
            keywords: vec![
                "editor".to_string(),
                "shortcuts".to_string(),
                "edit".to_string(),
                "keys".to_string(),
            ],
            category: HelpCategory::KeyboardShortcuts,
        });

        // Advanced
        self.add_topic(HelpTopic {
            id: "yaml_syntax".to_string(),
            title: "YAML Syntax Reference".to_string(),
            content: r#"# DSL YAML Syntax Reference

Complete reference for DSL workflow YAML syntax.

## Top-Level Structure

```yaml
name: string              # Required: Workflow name
version: string           # Required: Semantic version
description: string       # Optional: Workflow description

inputs:                   # Optional: Workflow-level inputs
  input_name:
    type: string|number|boolean|array|object
    required: boolean
    default: any

agents:                   # Required: Agent definitions
  agent_id: AgentDef

tasks:                    # Required: Task definitions
  task_id: TaskDef

hooks:                    # Optional: Lifecycle hooks
  on_start: HookDef
  on_complete: HookDef
  on_error: HookDef
```

## Agent Definition

```yaml
agents:
  researcher:
    description: string   # Required: What this agent does
    model: string        # Optional: AI model (default: claude-sonnet-4-5)
    tools:               # Optional: Tool allowlist
      - Read
      - Write
      - WebSearch
    permissions:         # Optional: Permission settings
      mode: default|acceptEdits|plan|bypassPermissions
      max_turns: number
    inputs:              # Optional: Agent-specific inputs
      api_key:
        type: string
        required: true
```

## Task Definition

```yaml
tasks:
  analyze:
    description: string         # Required: Task description
    agent: string              # Required: Agent ID reference
    depends_on:                # Optional: Task dependencies
      - other_task_id
    subtasks:                  # Optional: Child tasks
      - child_task_id
    output: string             # Optional: Output file path
    inputs:                    # Optional: Task inputs
      config: value
    outputs:                   # Optional: Output variables
      result:
        source:
          type: file|state|result
          path: string
```

## Variable Interpolation

Use `${scope.variable}` or `${variable}` syntax:

```yaml
tasks:
  analyze:
    description: "Analyze ${workflow.project_name}"
    inputs:
      config: "${workflow.project_name}/config.yaml"
```

Scopes:
- `workflow.*`: Workflow-level variables
- `agent.*`: Agent-level variables
- `task.*`: Task-level variables

## Data Types

- **string**: Text values
- **number**: Numeric values
- **boolean**: true/false
- **array**: Lists of values
- **object**: Key-value maps

See official DSL documentation for complete specification.
"#
            .to_string(),
            related: vec!["editing_workflows".to_string(), "validation".to_string()],
            keywords: vec![
                "yaml".to_string(),
                "syntax".to_string(),
                "format".to_string(),
                "schema".to_string(),
            ],
            category: HelpCategory::Advanced,
        });

        // Initialize category index
        for topic in self.topics.values() {
            self.categories
                .entry(topic.category)
                .or_default()
                .push(topic.id.clone());
        }
    }

    /// Add a topic to the database
    fn add_topic(&mut self, topic: HelpTopic) {
        let id = topic.id.clone();
        self.topics.insert(id, topic);
    }

    /// Get a topic by ID
    pub fn get_topic(&self, id: &str) -> Option<&HelpTopic> {
        self.topics.get(id)
    }

    /// Get all topics in a category
    pub fn get_category_topics(&self, category: HelpCategory) -> Vec<&HelpTopic> {
        self.categories
            .get(&category)
            .map(|ids| ids.iter().filter_map(|id| self.topics.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get all topics
    pub fn all_topics(&self) -> Vec<&HelpTopic> {
        self.topics.values().collect()
    }

    /// Get all categories with their topics
    pub fn all_categories(&self) -> Vec<(HelpCategory, Vec<&HelpTopic>)> {
        let mut categories: Vec<_> = self
            .categories
            .keys()
            .map(|&cat| (cat, self.get_category_topics(cat)))
            .collect();

        // Sort by category enum order
        categories.sort_by_key(|(cat, _)| *cat as u8);
        categories
    }
}

impl Default for HelpContent {
    fn default() -> Self {
        Self::new()
    }
}
