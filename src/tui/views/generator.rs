//! AI Workflow Generator Interface
//!
//! Provides an interactive interface for generating DSL workflows from natural language.
//!
//! Features:
//! - Natural language input with multiline support
//! - Real-time workflow generation preview
//! - Diff view for comparing original and modified workflows
//! - Validation feedback integration
//! - Modification mode for existing workflows
//! - Stream-based generation progress tracking
//!
//! Modes:
//! - Create: Generate new workflow from description
//! - Modify: Modify existing workflow with NL instructions
//!
//! Navigation:
//! - Ctrl+G: Start generation
//! - Ctrl+A: Accept generated workflow
//! - Ctrl+R: Retry generation
//! - Tab: Switch between input/preview panels
//! - Esc: Cancel/Back

use crate::dsl::nl_generator::{generate_from_nl, modify_workflow_from_nl};
use crate::dsl::parser::{parse_workflow, write_workflow_file};
use crate::dsl::schema::DSLWorkflow;
use crate::dsl::validator::validate_workflow;
use crate::tui::theme::Theme;
use crate::AgentOptions;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use std::path::PathBuf;

/// Generator view modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneratorMode {
    /// Create new workflow from description
    Create,
    /// Modify existing workflow
    Modify,
}

/// Focus state for panels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPanel {
    /// Natural language input panel
    Input,
    /// Generated workflow preview panel
    Preview,
}

/// Generation status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerationStatus {
    /// Idle, waiting for user input
    Idle,
    /// Generation in progress
    InProgress { progress: String },
    /// Generation completed successfully
    Completed,
    /// Generation failed with error
    Failed { error: String },
    /// Validation in progress
    Validating,
    /// Validation completed
    Validated {
        is_valid: bool,
        errors: Vec<String>,
        warnings: Vec<String>,
    },
}

/// Generator state
#[derive(Debug, Clone)]
pub struct GeneratorState {
    /// Generator mode (Create or Modify)
    pub mode: GeneratorMode,

    /// Natural language description/instruction input
    pub nl_input: String,

    /// Cursor position in input field
    pub input_cursor: usize,

    /// Input scroll offset for multiline
    pub input_scroll: usize,

    /// Original workflow YAML (for Modify mode)
    pub original_yaml: Option<String>,

    /// Generated workflow YAML
    pub generated_yaml: Option<String>,

    /// Parsed workflow (if valid)
    pub generated_workflow: Option<DSLWorkflow>,

    /// Current generation status
    pub status: GenerationStatus,

    /// Currently focused panel
    pub focus: FocusPanel,

    /// Preview scroll offset
    pub preview_scroll: usize,

    /// Diff scroll offset (when comparing versions)
    pub diff_scroll: usize,

    /// Show diff view vs preview view
    pub show_diff: bool,

    /// Agent options for generation
    pub agent_options: Option<AgentOptions>,

    /// Output file path (optional)
    pub output_path: Option<PathBuf>,
}

impl GeneratorState {
    /// Create new generator state for workflow creation
    pub fn new_create() -> Self {
        Self {
            mode: GeneratorMode::Create,
            nl_input: String::new(),
            input_cursor: 0,
            input_scroll: 0,
            original_yaml: None,
            generated_yaml: None,
            generated_workflow: None,
            status: GenerationStatus::Idle,
            focus: FocusPanel::Input,
            preview_scroll: 0,
            diff_scroll: 0,
            show_diff: false,
            agent_options: None,
            output_path: None,
        }
    }

    /// Create new generator state for workflow modification
    pub fn new_modify(original_yaml: String) -> Self {
        Self {
            mode: GeneratorMode::Modify,
            nl_input: String::new(),
            input_cursor: 0,
            input_scroll: 0,
            original_yaml: Some(original_yaml),
            generated_yaml: None,
            generated_workflow: None,
            status: GenerationStatus::Idle,
            focus: FocusPanel::Input,
            preview_scroll: 0,
            diff_scroll: 0,
            show_diff: true, // Show diff by default in modify mode
            agent_options: None,
            output_path: None,
        }
    }

    /// Insert character at cursor position
    pub fn insert_char(&mut self, c: char) {
        self.nl_input.insert(self.input_cursor, c);
        self.input_cursor += 1;
    }

    /// Delete character before cursor
    pub fn delete_char(&mut self) {
        if self.input_cursor > 0 {
            self.nl_input.remove(self.input_cursor - 1);
            self.input_cursor -= 1;
        }
    }

    /// Move cursor left
    pub fn cursor_left(&mut self) {
        if self.input_cursor > 0 {
            self.input_cursor -= 1;
        }
    }

    /// Move cursor right
    pub fn cursor_right(&mut self) {
        if self.input_cursor < self.nl_input.len() {
            self.input_cursor += 1;
        }
    }

    /// Set generated workflow and parse it
    pub fn set_generated(&mut self, yaml: String) {
        // Parse the workflow
        match parse_workflow(&yaml) {
            Ok(workflow) => {
                self.generated_workflow = Some(workflow);
                self.generated_yaml = Some(yaml);
                self.status = GenerationStatus::Completed;
            }
            Err(e) => {
                self.generated_yaml = Some(yaml);
                self.generated_workflow = None;
                self.status = GenerationStatus::Failed {
                    error: format!("Parse error: {}", e),
                };
            }
        }
    }

    /// Validate the generated workflow
    pub fn validate_generated(&mut self) {
        if let Some(workflow) = &self.generated_workflow {
            match validate_workflow(workflow) {
                Ok(_) => {
                    self.status = GenerationStatus::Validated {
                        is_valid: true,
                        errors: Vec::new(),
                        warnings: Vec::new(),
                    };
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    let errors: Vec<String> = error_msg
                        .lines()
                        .filter(|l| {
                            !l.trim().is_empty() && !l.contains("Workflow validation failed")
                        })
                        .map(|l| l.to_string())
                        .collect();

                    self.status = GenerationStatus::Validated {
                        is_valid: false,
                        errors,
                        warnings: Vec::new(),
                    };
                }
            }
        } else {
            self.status = GenerationStatus::Failed {
                error: "No workflow to validate".to_string(),
            };
        }
    }

    /// Check if generation can be started
    pub fn can_generate(&self) -> bool {
        !self.nl_input.trim().is_empty()
            && matches!(
                self.status,
                GenerationStatus::Idle
                    | GenerationStatus::Completed
                    | GenerationStatus::Failed { .. }
                    | GenerationStatus::Validated { .. }
            )
    }

    /// Check if workflow can be accepted
    pub fn can_accept(&self) -> bool {
        self.generated_workflow.is_some()
            && matches!(
                self.status,
                GenerationStatus::Completed | GenerationStatus::Validated { is_valid: true, .. }
            )
    }

    /// Toggle diff view
    pub fn toggle_diff(&mut self) {
        if self.original_yaml.is_some() {
            self.show_diff = !self.show_diff;
        }
    }

    /// Switch focus to next panel
    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            FocusPanel::Input => FocusPanel::Preview,
            FocusPanel::Preview => FocusPanel::Input,
        };
    }
}

/// Render the generator view
pub fn render(frame: &mut Frame, area: Rect, state: &GeneratorState, theme: &Theme) {
    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(8), // Status panel
            Constraint::Length(1), // Shortcuts bar
        ])
        .split(area);

    // Render header
    render_header(frame, chunks[0], state, theme);

    // Split main content into input and preview
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40), // Input panel
            Constraint::Percentage(60), // Preview/Diff panel
        ])
        .split(chunks[1]);

    // Render input panel
    render_input_panel(frame, content_chunks[0], state, theme);

    // Render preview or diff panel
    if state.show_diff && state.original_yaml.is_some() {
        render_diff_panel(frame, content_chunks[1], state, theme);
    } else {
        render_preview_panel(frame, content_chunks[1], state, theme);
    }

    // Render status panel
    render_status_panel(frame, chunks[2], state, theme);

    // Render shortcuts bar
    render_shortcuts_bar(frame, chunks[3], state, theme);
}

/// Render header with mode and status
fn render_header(frame: &mut Frame, area: Rect, state: &GeneratorState, theme: &Theme) {
    let mode_text = match state.mode {
        GeneratorMode::Create => "Create New Workflow",
        GeneratorMode::Modify => "Modify Workflow",
    };

    let status_icon = match &state.status {
        GenerationStatus::Idle => Span::styled("○", Style::default().fg(theme.muted)),
        GenerationStatus::InProgress { .. } => Span::styled("◐", Style::default().fg(theme.accent)),
        GenerationStatus::Completed => Span::styled("✓", Style::default().fg(theme.success)),
        GenerationStatus::Failed { .. } => Span::styled("✗", Style::default().fg(theme.error)),
        GenerationStatus::Validating => Span::styled("◑", Style::default().fg(theme.warning)),
        GenerationStatus::Validated { is_valid, .. } => {
            if *is_valid {
                Span::styled("✓", Style::default().fg(theme.success))
            } else {
                Span::styled("⚠", Style::default().fg(theme.warning))
            }
        }
    };

    let header_line = Line::from(vec![
        Span::styled(
            "AI Workflow Generator",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" | Mode: ", Style::default().fg(theme.muted)),
        Span::styled(mode_text, Style::default().fg(theme.accent)),
        Span::styled(" | Status: ", Style::default().fg(theme.muted)),
        status_icon,
    ]);

    let header = Paragraph::new(header_line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border)),
    );

    frame.render_widget(header, area);
}

/// Render natural language input panel
fn render_input_panel(frame: &mut Frame, area: Rect, state: &GeneratorState, theme: &Theme) {
    let is_focused = state.focus == FocusPanel::Input;

    let border_style = if is_focused {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.border)
    };

    let title = match state.mode {
        GeneratorMode::Create => " Describe Your Workflow (Markdown) ",
        GeneratorMode::Modify => " Modification Instructions (Markdown) ",
    };

    let mut rendered_lines = Vec::new();

    if state.nl_input.is_empty() {
        // Show placeholder
        rendered_lines.push(Line::from(Span::styled(
            "Enter a natural language description of your workflow...",
            Style::default()
                .fg(theme.muted)
                .add_modifier(Modifier::ITALIC),
        )));
        rendered_lines.push(Line::from(""));
        rendered_lines.push(Line::from(Span::styled(
            "Markdown formatting: **bold**, *italic*, `code`, # Heading",
            Style::default()
                .fg(theme.muted)
                .add_modifier(Modifier::ITALIC),
        )));
    } else {
        // Calculate cursor line and column
        let (cursor_line, cursor_col) =
            calculate_cursor_position(&state.nl_input, state.input_cursor);

        // Render each line with markdown highlighting and cursor
        let lines: Vec<&str> = state.nl_input.lines().collect();
        for (idx, line) in lines.iter().enumerate() {
            let is_cursor_line = idx == cursor_line && is_focused;

            if is_cursor_line {
                // Render line with cursor
                let highlighted = highlight_markdown_with_cursor(line, cursor_col, theme);
                rendered_lines.push(Line::from(highlighted));
            } else {
                // Render line with markdown highlighting
                let highlighted = highlight_markdown(line, theme);
                rendered_lines.push(Line::from(highlighted));
            }
        }

        // If cursor is at the very end (past last newline), add an empty line with cursor
        if is_focused
            && state.input_cursor == state.nl_input.len()
            && state.nl_input.ends_with('\n')
        {
            rendered_lines.push(Line::from(vec![Span::styled(
                " ",
                Style::default()
                    .fg(ratatui::style::Color::Black)
                    .bg(ratatui::style::Color::White)
                    .add_modifier(Modifier::BOLD),
            )]));
        }
    }

    let text = Text::from(rendered_lines);
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(title),
        )
        .scroll((state.input_scroll as u16, 0))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Render workflow preview panel
fn render_preview_panel(frame: &mut Frame, area: Rect, state: &GeneratorState, theme: &Theme) {
    let is_focused = state.focus == FocusPanel::Preview;

    let border_style = if is_focused {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.border)
    };

    let title = if state.show_diff {
        " Workflow Diff "
    } else {
        " Generated Workflow "
    };

    let content = if let Some(ref yaml) = state.generated_yaml {
        // Show generated YAML with syntax highlighting
        let lines: Vec<&str> = yaml.lines().collect();
        let mut rendered_lines = Vec::new();

        for (line_idx, line) in lines.iter().enumerate() {
            let line_num = format!("{:4} │ ", line_idx + 1);
            let line_num_span = Span::styled(line_num, Style::default().fg(theme.muted));

            // Basic YAML highlighting
            let highlighted = highlight_yaml_line(line, theme);
            let mut spans = vec![line_num_span];
            spans.extend(highlighted);

            rendered_lines.push(Line::from(spans));
        }

        Text::from(rendered_lines)
    } else {
        // Show placeholder
        Text::from(vec![
            Line::from(""),
            Line::from(Span::styled(
                "No workflow generated yet.",
                Style::default()
                    .fg(theme.muted)
                    .add_modifier(Modifier::ITALIC),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press Ctrl+G to generate from your description.",
                Style::default().fg(theme.muted),
            )),
        ])
    };

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(title),
        )
        .scroll((state.preview_scroll as u16, 0))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Render diff view comparing original and generated workflows
fn render_diff_panel(frame: &mut Frame, area: Rect, state: &GeneratorState, theme: &Theme) {
    let is_focused = state.focus == FocusPanel::Preview;

    let border_style = if is_focused {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.border)
    };

    if state.original_yaml.is_none() {
        // No original to compare
        let text = Text::from(vec![
            Line::from(""),
            Line::from(Span::styled(
                "No original workflow to compare.",
                Style::default().fg(theme.muted),
            )),
        ]);

        let paragraph = Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(" Diff View "),
        );

        frame.render_widget(paragraph, area);
        return;
    }

    // Split area for side-by-side diff
    let diff_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Render original (left)
    render_diff_side(
        frame,
        diff_chunks[0],
        "Original",
        state.original_yaml.as_ref().unwrap(),
        theme,
        border_style,
        state.diff_scroll,
    );

    // Render generated (right)
    if let Some(ref generated) = state.generated_yaml {
        render_diff_side(
            frame,
            diff_chunks[1],
            "Generated",
            generated,
            theme,
            border_style,
            state.diff_scroll,
        );
    } else {
        let text = Text::from(vec![
            Line::from(""),
            Line::from(Span::styled(
                "No generated workflow yet.",
                Style::default().fg(theme.muted),
            )),
        ]);

        let paragraph = Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(" Generated "),
        );

        frame.render_widget(paragraph, diff_chunks[1]);
    }
}

/// Render one side of the diff view
fn render_diff_side(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    yaml: &str,
    theme: &Theme,
    border_style: Style,
    scroll: usize,
) {
    let lines: Vec<&str> = yaml.lines().collect();
    let mut rendered_lines = Vec::new();

    for line in lines.iter() {
        let highlighted = highlight_yaml_line(line, theme);
        rendered_lines.push(Line::from(highlighted));
    }

    let text = Text::from(rendered_lines);
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(format!(" {} ", title)),
        )
        .scroll((scroll as u16, 0))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Render status panel showing generation progress and validation
fn render_status_panel(frame: &mut Frame, area: Rect, state: &GeneratorState, theme: &Theme) {
    let mut items = Vec::new();

    match &state.status {
        GenerationStatus::Idle => {
            items.push(ListItem::new(Line::from(Span::styled(
                "Ready to generate workflow",
                Style::default().fg(theme.muted),
            ))));
            items.push(ListItem::new(Line::from("")));
            items.push(ListItem::new(Line::from(vec![
                Span::styled(
                    "💡 Tip: ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Give tasks enough context so they can be executed in new",
                    Style::default().fg(theme.muted),
                ),
            ])));
            items.push(ListItem::new(Line::from(Span::styled(
                "   conversations and are aware of previous achievements and the whole workflow.",
                Style::default().fg(theme.muted),
            ))));
        }
        GenerationStatus::InProgress { progress } => {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("⏳ ", Style::default().fg(theme.accent)),
                Span::styled("Generating workflow...", Style::default().fg(theme.accent)),
            ])));
            items.push(ListItem::new(Line::from(Span::styled(
                format!("  {}", progress),
                Style::default().fg(theme.muted),
            ))));
        }
        GenerationStatus::Completed => {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("✓ ", Style::default().fg(theme.success)),
                Span::styled(
                    "Workflow generated successfully",
                    Style::default().fg(theme.success),
                ),
            ])));
        }
        GenerationStatus::Failed { error } => {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("✗ ", Style::default().fg(theme.error)),
                Span::styled("Generation failed", Style::default().fg(theme.error)),
            ])));
            items.push(ListItem::new(Line::from(Span::styled(
                format!("  Error: {}", error),
                Style::default().fg(theme.muted),
            ))));
        }
        GenerationStatus::Validating => {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("◑ ", Style::default().fg(theme.warning)),
                Span::styled("Validating workflow...", Style::default().fg(theme.warning)),
            ])));
        }
        GenerationStatus::Validated {
            is_valid,
            errors,
            warnings,
        } => {
            if *is_valid {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("✓ ", Style::default().fg(theme.success)),
                    Span::styled(
                        "Workflow is valid and ready",
                        Style::default().fg(theme.success),
                    ),
                ])));
            } else {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("⚠ ", Style::default().fg(theme.warning)),
                    Span::styled(
                        format!(
                            "Validation issues found ({} errors, {} warnings)",
                            errors.len(),
                            warnings.len()
                        ),
                        Style::default().fg(theme.warning),
                    ),
                ])));

                for error in errors.iter().take(3) {
                    items.push(ListItem::new(Line::from(Span::styled(
                        format!("  • {}", error),
                        Style::default().fg(theme.error),
                    ))));
                }

                if errors.len() > 3 {
                    items.push(ListItem::new(Line::from(Span::styled(
                        format!("  ... and {} more", errors.len() - 3),
                        Style::default().fg(theme.muted),
                    ))));
                }
            }
        }
    }

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(" Status "),
    );

    frame.render_widget(list, area);
}

/// Render shortcuts bar
fn render_shortcuts_bar(frame: &mut Frame, area: Rect, state: &GeneratorState, theme: &Theme) {
    let shortcuts = match state.focus {
        FocusPanel::Input => {
            let mut parts = Vec::new();

            if state.can_generate() {
                parts.push("Ctrl+G: Generate");
            }
            parts.push("Ctrl+B: Bold");
            parts.push("Ctrl+I: Italic");
            parts.push("Ctrl+K: Code");
            parts.push("Ctrl+H: Heading");
            parts.push("Tab: Preview");
            parts.push("Esc: Cancel");

            parts.join(" | ")
        }
        FocusPanel::Preview => {
            let mut parts = Vec::new();

            if state.can_accept() {
                parts.push("Ctrl+A: Accept");
            }
            if state.can_generate() {
                parts.push("Ctrl+R: Retry");
            }
            if state.original_yaml.is_some() {
                parts.push("Ctrl+D: Toggle Diff");
            }
            parts.push("Tab: Input");
            parts.push("↑↓: Scroll");
            parts.push("Esc: Cancel");

            parts.join(" | ")
        }
    };

    let status = Paragraph::new(shortcuts).style(Style::default().fg(theme.muted).bg(theme.bg));

    frame.render_widget(status, area);
}

/// Calculate cursor position as (line, column) from linear cursor position
fn calculate_cursor_position(text: &str, cursor: usize) -> (usize, usize) {
    let mut chars_seen = 0;
    let mut line = 0;
    let mut col = 0;

    for (line_idx, line_text) in text.lines().enumerate() {
        let line_len = line_text.chars().count();
        let line_total = line_len + 1; // +1 for newline

        if chars_seen + line_total > cursor {
            // Cursor is on this line
            line = line_idx;
            col = cursor - chars_seen;
            break;
        }

        chars_seen += line_total;
        line = line_idx + 1;
        col = 0;
    }

    (line, col)
}

/// Highlight markdown text
fn highlight_markdown(line: &str, theme: &Theme) -> Vec<Span<'static>> {
    use ratatui::style::Color;

    let mut spans = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Headings
        if i == 0 && chars[i] == '#' {
            let mut hash_count = 0;
            while i < chars.len() && chars[i] == '#' {
                hash_count += 1;
                i += 1;
            }
            let heading: String = chars[i..].iter().collect();
            spans.push(Span::styled(
                "#".repeat(hash_count),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(
                heading,
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ));
            break;
        }

        // Bold **text**
        if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
            i += 2;
            let start = i;
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '*') {
                i += 1;
            }
            if i + 1 < chars.len() {
                let text: String = chars[start..i].iter().collect();
                spans.push(Span::styled(
                    format!("**{}**", text),
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ));
                i += 2;
                continue;
            } else {
                // Unclosed bold - treat as regular text
                let text: String = chars[start - 2..].iter().collect();
                spans.push(Span::raw(text));
                break;
            }
        }

        // Italic *text*
        if chars[i] == '*' && (i == 0 || i > 0 && chars[i - 1] != '*') {
            let star_pos = i;
            i += 1;
            let start = i;
            while i < chars.len() && chars[i] != '*' {
                i += 1;
            }
            if i < chars.len() {
                let text: String = chars[start..i].iter().collect();
                spans.push(Span::styled(
                    format!("*{}*", text),
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::ITALIC),
                ));
                i += 1;
                continue;
            } else {
                // Unclosed italic - treat as regular text
                i = star_pos;
                let text: String = chars[i..].iter().collect();
                spans.push(Span::raw(text));
                break;
            }
        }

        // Code `text`
        if i < chars.len() && chars[i] == '`' {
            let tick_pos = i;
            i += 1;
            let start = i;
            while i < chars.len() && chars[i] != '`' {
                i += 1;
            }
            if i < chars.len() {
                let text: String = chars[start..i].iter().collect();
                spans.push(Span::styled(
                    format!("`{}`", text),
                    Style::default().fg(Color::Cyan).bg(Color::Rgb(40, 40, 50)),
                ));
                i += 1;
                continue;
            } else {
                // Unclosed code - treat as regular text
                i = tick_pos;
                let text: String = chars[i..].iter().collect();
                spans.push(Span::raw(text));
                break;
            }
        }

        // Regular text
        let start = i;
        while i < chars.len() && chars[i] != '*' && chars[i] != '`' && chars[i] != '#' {
            i += 1;
        }
        if i > start {
            let text: String = chars[start..i].iter().collect();
            spans.push(Span::raw(text));
        }
    }

    if spans.is_empty() {
        spans.push(Span::raw(line.to_string()));
    }

    spans
}

/// Highlight markdown with cursor at specific position
fn highlight_markdown_with_cursor(
    line: &str,
    cursor_col: usize,
    theme: &Theme,
) -> Vec<Span<'static>> {
    use ratatui::style::Color;

    let chars: Vec<char> = line.chars().collect();
    let cursor_pos = cursor_col.min(chars.len());

    // Split into before, cursor char, and after
    let before: String = chars[..cursor_pos].iter().collect();
    let cursor_char = if cursor_pos < chars.len() {
        chars[cursor_pos]
    } else {
        ' '
    };
    let after: String = if cursor_pos < chars.len() {
        chars[cursor_pos + 1..].iter().collect()
    } else {
        String::new()
    };

    let mut spans = Vec::new();

    // Add highlighted text before cursor
    if !before.is_empty() {
        spans.extend(highlight_markdown(&before, theme));
    }

    // Add cursor
    spans.push(Span::styled(
        cursor_char.to_string(),
        Style::default()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::BOLD),
    ));

    // Add highlighted text after cursor
    if !after.is_empty() {
        spans.extend(highlight_markdown(&after, theme));
    }

    spans
}

/// Highlight a single YAML line (basic syntax highlighting)
fn highlight_yaml_line<'a>(line: &'a str, theme: &Theme) -> Vec<Span<'a>> {
    let trimmed = line.trim_start();

    if trimmed.starts_with('#') {
        // Comment
        vec![Span::styled(
            line,
            Style::default()
                .fg(theme.muted)
                .add_modifier(Modifier::ITALIC),
        )]
    } else if trimmed.starts_with("---") || trimmed.starts_with("...") {
        // Document separator
        vec![Span::styled(line, Style::default().fg(theme.border))]
    } else if trimmed.starts_with('-') && trimmed.len() > 1 && trimmed.chars().nth(1) == Some(' ') {
        // List item
        let indent_len = line.len() - trimmed.len();
        vec![
            Span::raw(&line[..indent_len]),
            Span::styled("- ", Style::default().fg(theme.accent)),
            Span::raw(&line[indent_len + 2..]),
        ]
    } else if let Some(colon_pos) = trimmed.find(':') {
        // Key-value pair
        let indent_len = line.len() - trimmed.len();
        let key = &trimmed[..colon_pos];
        let rest = &trimmed[colon_pos..];

        vec![
            Span::raw(&line[..indent_len]),
            Span::styled(
                key,
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(rest, Style::default().fg(theme.fg)),
        ]
    } else {
        // Plain line
        vec![Span::raw(line)]
    }
}

/// Start workflow generation (async operation - would be called from app loop)
pub async fn start_generation(state: &mut GeneratorState) -> Result<(), String> {
    state.status = GenerationStatus::InProgress {
        progress: "Initializing generation...".to_string(),
    };

    let result = match state.mode {
        GeneratorMode::Create => {
            generate_from_nl(&state.nl_input, state.agent_options.clone()).await
        }
        GeneratorMode::Modify => {
            if let Some(ref original) = state.original_yaml {
                modify_workflow_from_nl(&state.nl_input, original, state.agent_options.clone())
                    .await
            } else {
                return Err("No original workflow for modification".to_string());
            }
        }
    };

    match result {
        Ok(workflow) => {
            // Serialize workflow to YAML
            match serde_yaml::to_string(&workflow) {
                Ok(yaml) => {
                    state.generated_yaml = Some(yaml);
                    state.generated_workflow = Some(workflow);
                    state.status = GenerationStatus::Completed;

                    // Auto-validate
                    state.validate_generated();

                    Ok(())
                }
                Err(e) => {
                    state.status = GenerationStatus::Failed {
                        error: format!("Failed to serialize workflow: {}", e),
                    };
                    Err(format!("Serialization error: {}", e))
                }
            }
        }
        Err(e) => {
            state.status = GenerationStatus::Failed {
                error: e.to_string(),
            };
            Err(e.to_string())
        }
    }
}

/// Accept generated workflow and optionally save to file
pub fn accept_workflow(state: &GeneratorState) -> Result<DSLWorkflow, String> {
    if let Some(workflow) = &state.generated_workflow {
        // Save to file if output path is specified
        if let Some(ref path) = state.output_path {
            write_workflow_file(workflow, path.to_str().unwrap())
                .map_err(|e| format!("Failed to save workflow: {}", e))?;
        }

        Ok(workflow.clone())
    } else {
        Err("No workflow to accept".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_state_create() {
        let state = GeneratorState::new_create();
        assert_eq!(state.mode, GeneratorMode::Create);
        assert!(state.nl_input.is_empty());
        assert_eq!(state.focus, FocusPanel::Input);
        assert!(!state.show_diff);
    }

    #[test]
    fn test_generator_state_modify() {
        let original = "name: test\nversion: 1.0.0".to_string();
        let state = GeneratorState::new_modify(original.clone());
        assert_eq!(state.mode, GeneratorMode::Modify);
        assert_eq!(state.original_yaml, Some(original));
        assert!(state.show_diff);
    }

    #[test]
    fn test_input_editing() {
        let mut state = GeneratorState::new_create();

        state.insert_char('h');
        state.insert_char('i');
        assert_eq!(state.nl_input, "hi");
        assert_eq!(state.input_cursor, 2);

        state.delete_char();
        assert_eq!(state.nl_input, "h");
        assert_eq!(state.input_cursor, 1);
    }

    #[test]
    fn test_can_generate() {
        let mut state = GeneratorState::new_create();
        assert!(!state.can_generate());

        state.nl_input = "Create a workflow".to_string();
        assert!(state.can_generate());

        state.status = GenerationStatus::InProgress {
            progress: "Working...".to_string(),
        };
        assert!(!state.can_generate());
    }

    #[test]
    fn test_can_accept() {
        let mut state = GeneratorState::new_create();
        assert!(!state.can_accept());

        // Create a minimal workflow
        let workflow = DSLWorkflow {
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            dsl_version: "1.0.0".to_string(),
            provider: Default::default(),
            model: None,
            cwd: None,
            create_cwd: None,
            secrets: std::collections::HashMap::new(),
            agents: std::collections::HashMap::new(),
            tasks: std::collections::HashMap::new(),
            workflows: std::collections::HashMap::new(),
            inputs: std::collections::HashMap::new(),
            outputs: std::collections::HashMap::new(),
            tools: None,
            communication: None,
            mcp_servers: std::collections::HashMap::new(),
            subflows: std::collections::HashMap::new(),
            imports: std::collections::HashMap::new(),
            notifications: None,
            limits: None,
        };

        state.generated_workflow = Some(workflow);
        state.status = GenerationStatus::Completed;
        assert!(state.can_accept());
    }

    #[test]
    fn test_toggle_focus() {
        let mut state = GeneratorState::new_create();
        assert_eq!(state.focus, FocusPanel::Input);

        state.toggle_focus();
        assert_eq!(state.focus, FocusPanel::Preview);

        state.toggle_focus();
        assert_eq!(state.focus, FocusPanel::Input);
    }

    #[test]
    fn test_highlight_yaml_line() {
        let theme = Theme::default();

        // Test comment
        let spans = highlight_yaml_line("# comment", &theme);
        assert_eq!(spans.len(), 1);

        // Test key-value
        let spans = highlight_yaml_line("name: value", &theme);
        assert!(spans.len() >= 2);

        // Test list item
        let spans = highlight_yaml_line("- item", &theme);
        assert!(spans.len() >= 2);
    }
}
