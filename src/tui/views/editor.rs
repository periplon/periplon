//! Interactive workflow editor with real-time validation
//!
//! Provides two editing modes:
//! - Text Mode: YAML editor with syntax highlighting and validation
//! - Form Mode: Structured form-based editing with auto-completion
//!
//! Features:
//! - Real-time YAML validation with inline error markers
//! - Auto-completion for DSL keywords and structures
//! - Smart indentation and formatting
//! - Quick fixes for common validation errors
//! - Live validation feedback panel
//!
//! Navigation:
//! - Arrow keys: Move cursor
//! - Tab: Toggle between text/form mode
//! - Ctrl+S: Save workflow
//! - Ctrl+V: Validate
//! - Esc: Cancel/Back

use crate::dsl::{schema::DSLWorkflow, validator::validate_workflow};
use crate::tui::state::EditorState;
use crate::tui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};

/// Editor view modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    /// Text editing mode (YAML)
    Text,
    /// Form-based structured editing
    Form,
}

/// Validation feedback
#[derive(Debug, Clone)]
pub struct ValidationFeedback {
    /// Validation errors with line numbers
    pub errors: Vec<(usize, String)>,
    /// Validation warnings with line numbers
    pub warnings: Vec<(usize, String)>,
    /// Last validation timestamp
    pub validated_at: Option<std::time::Instant>,
}

impl Default for ValidationFeedback {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationFeedback {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            validated_at: None,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }
}

/// Auto-completion suggestion
#[derive(Debug, Clone)]
pub struct AutoCompletionSuggestion {
    /// Suggestion text
    pub text: String,
    /// Description/documentation
    pub description: String,
    /// Category (e.g., "keyword", "agent", "task")
    pub category: String,
}

/// DSL keywords for auto-completion
const DSL_KEYWORDS: &[(&str, &str)] = &[
    ("name:", "Workflow name"),
    ("version:", "Workflow version"),
    ("dsl_version:", "DSL schema version"),
    ("agents:", "Agent definitions"),
    ("tasks:", "Task definitions"),
    ("workflows:", "Workflow orchestration"),
    ("inputs:", "Input variable definitions"),
    ("outputs:", "Output variable definitions"),
    ("tools:", "Tool configuration"),
    ("communication:", "Communication settings"),
    ("mcp_servers:", "MCP server configuration"),
    ("subflows:", "Subflow definitions"),
    ("imports:", "Task group imports"),
    ("notifications:", "Notification settings"),
    ("description:", "Description text"),
    ("model:", "AI model name"),
    ("system_prompt:", "System prompt text"),
    ("cwd:", "Working directory"),
    ("create_cwd:", "Create directory flag"),
    ("permissions:", "Permission settings"),
    ("max_turns:", "Maximum conversation turns"),
    ("agent:", "Task agent reference"),
    ("depends_on:", "Task dependencies"),
    ("subtasks:", "Subtask definitions"),
    ("output:", "Output file path"),
    ("parallel:", "Parallel execution flag"),
];

/// Render the workflow editor
pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &EditorState,
    feedback: &ValidationFeedback,
    theme: &Theme,
) {
    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Editor content
            Constraint::Length(1),  // Status bar with validation
        ])
        .split(area);

    // Render header
    render_header(frame, chunks[0], state, feedback, theme);

    // Render editor based on mode
    match state.mode {
        EditorMode::Text => render_text_editor(frame, chunks[1], state, feedback, theme),
        EditorMode::Form => render_form_editor(frame, chunks[1], state, theme),
    }

    // Render status bar with integrated validation
    render_status_bar_with_validation(frame, chunks[2], state, feedback, theme);

    // Render expanded validation modal if enabled
    if state.validation_expanded && (!feedback.errors.is_empty() || !feedback.warnings.is_empty()) {
        render_validation_modal(frame, area, feedback, theme);
    }
}

/// Render header with file info and validation status
fn render_header(
    frame: &mut Frame,
    area: Rect,
    state: &EditorState,
    feedback: &ValidationFeedback,
    theme: &Theme,
) {
    let file_name = state
        .file_path
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("<new workflow>");

    let mode_text = match state.mode {
        EditorMode::Text => "Text",
        EditorMode::Form => "Form",
    };

    let validation_status = if feedback.is_valid() {
        Span::styled(" ✓ Valid", Style::default().fg(theme.success))
    } else {
        Span::styled(
            format!(" ✗ {} errors", feedback.error_count()),
            Style::default().fg(theme.error),
        )
    };

    let modified_marker = if state.modified {
        Span::styled(" [Modified]", Style::default().fg(theme.warning))
    } else {
        Span::styled("", Style::default())
    };

    let header_text = vec![Line::from(vec![
        Span::styled("Editing: ", Style::default().fg(theme.muted)),
        Span::styled(
            file_name,
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
        modified_marker,
        Span::styled(" | Mode: ", Style::default().fg(theme.muted)),
        Span::styled(mode_text, Style::default().fg(theme.accent)),
        Span::styled(" |", Style::default().fg(theme.muted)),
        validation_status,
    ])];

    let header = Paragraph::new(header_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border)),
    );

    frame.render_widget(header, area);
}

/// Render text editor with YAML syntax highlighting and inline validation
fn render_text_editor(
    frame: &mut Frame,
    area: Rect,
    state: &EditorState,
    feedback: &ValidationFeedback,
    theme: &Theme,
) {
    let content_height = area.height.saturating_sub(2) as usize;
    let lines: Vec<&str> = state.content.lines().collect();
    let total_lines = lines.len();

    // Calculate scroll to keep cursor visible
    let scroll_offset = calculate_scroll_offset(state.cursor_line(), state.scroll_offset(), content_height);

    // Build error/warning map for inline markers
    let mut line_markers: std::collections::HashMap<usize, Vec<String>> = std::collections::HashMap::new();
    for (line_num, error) in &feedback.errors {
        line_markers
            .entry(*line_num)
            .or_default()
            .push(format!("❌ {}", error));
    }
    for (line_num, warning) in &feedback.warnings {
        line_markers
            .entry(*line_num)
            .or_default()
            .push(format!("⚠️  {}", warning));
    }

    // Render lines with highlighting and markers
    let mut rendered_lines = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        let line_num = idx + 1;
        let is_cursor_line = idx == state.cursor_line();

        // Highlight current line
        let line_style = if is_cursor_line {
            Style::default().bg(Color::Rgb(40, 40, 50))
        } else {
            Style::default()
        };

        // Line number
        let line_num_text = format!("{:4} │ ", line_num);
        let line_num_style = if is_cursor_line {
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.muted)
        };

        // Syntax highlight the line with cursor
        let highlighted = if is_cursor_line {
            highlight_yaml_line_with_cursor(line, state.cursor.1, theme)
        } else {
            highlight_yaml_line(line, theme)
        };

        // Build complete line
        let mut spans = vec![Span::styled(line_num_text, line_num_style)];
        spans.extend(highlighted);

        rendered_lines.push(Line::from(spans).style(line_style));

        // Add inline validation markers
        if let Some(markers) = line_markers.get(&line_num) {
            for marker in markers {
                rendered_lines.push(Line::from(vec![
                    Span::styled("     │ ", Style::default().fg(theme.muted)),
                    Span::styled(
                        marker.clone(),
                        Style::default()
                            .fg(if marker.starts_with("❌") {
                                theme.error
                            } else {
                                theme.warning
                            })
                            .add_modifier(Modifier::ITALIC),
                    ),
                ]));
            }
        }
    }

    // Show cursor position
    if state.cursor_line() < lines.len() {
        // We'll just highlight the line for now; cursor rendering would need custom widget
    }

    let text = Text::from(rendered_lines);
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(" YAML Editor "),
        )
        .scroll((scroll_offset as u16, 0))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);

    // Render scrollbar
    if total_lines > content_height {
        render_scrollbar(frame, area, total_lines, scroll_offset, content_height, theme);
    }
}

/// Render form-based structured editor
fn render_form_editor(frame: &mut Frame, area: Rect, state: &EditorState, theme: &Theme) {
    // Parse current workflow to populate form fields
    let workflow = parse_workflow_content(&state.content);

    let form_text = match workflow {
        Ok(wf) => render_form_fields(&wf, theme),
        Err(e) => vec![
            Line::from(""),
            Line::from(Span::styled(
                "⚠️  Unable to parse workflow for form mode",
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!("Error: {}", e),
                Style::default().fg(theme.error),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Switch to Text mode to fix syntax errors.",
                Style::default().fg(theme.muted).add_modifier(Modifier::ITALIC),
            )),
        ],
    };

    let paragraph = Paragraph::new(form_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(" Form Editor "),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Render status bar with integrated validation and keyboard shortcuts
fn render_status_bar_with_validation(
    frame: &mut Frame,
    area: Rect,
    state: &EditorState,
    feedback: &ValidationFeedback,
    theme: &Theme,
) {
    // Build validation status
    let validation_text = if feedback.errors.is_empty() && feedback.warnings.is_empty() {
        Span::styled("✓ Valid", Style::default().fg(theme.success))
    } else {
        let mut parts = Vec::new();
        if !feedback.errors.is_empty() {
            parts.push(format!("{} error{}", feedback.errors.len(), if feedback.errors.len() == 1 { "" } else { "s" }));
        }
        if !feedback.warnings.is_empty() {
            parts.push(format!("{} warning{}", feedback.warnings.len(), if feedback.warnings.len() == 1 { "" } else { "s" }));
        }
        let message = format!("✗ {}", parts.join(", "));
        Span::styled(message, Style::default().fg(theme.error))
    };

    // Build keyboard shortcuts with highlighted keys
    let shortcuts = build_keybinding_display(state.mode, theme);

    let modified_indicator = if state.modified {
        Span::styled(" [Modified]", Style::default().fg(theme.warning))
    } else {
        Span::styled("", Style::default())
    };

    // Combine all elements
    let mut status_spans = vec![validation_text];
    status_spans.push(Span::styled(" | ", Style::default().fg(theme.muted)));
    status_spans.extend(shortcuts);
    status_spans.push(modified_indicator);

    let status_line = Line::from(status_spans);
    let status = Paragraph::new(status_line).style(Style::default().bg(theme.bg));

    frame.render_widget(status, area);
}

/// Build keybinding display with highlighted keys
fn build_keybinding_display(mode: EditorMode, theme: &Theme) -> Vec<Span<'static>> {
    let mut spans = Vec::new();

    let keybindings = match mode {
        EditorMode::Text => vec![
            ("^S", "Save"),
            ("^R", "Run"),
            ("v", "Valid"),
            ("Tab", "Form"),
            ("Esc", "Back"),
        ],
        EditorMode::Form => vec![
            ("Tab", "Text"),
            ("^S", "Save"),
            ("v", "Valid"),
            ("Esc", "Back"),
        ],
    };

    for (idx, (key, action)) in keybindings.iter().enumerate() {
        if idx > 0 {
            spans.push(Span::styled(" │ ", Style::default().fg(theme.muted)));
        }

        // Highlight the key
        spans.push(Span::styled(
            (*key).to_string(),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(":", Style::default().fg(theme.muted)));
        spans.push(Span::styled(
            (*action).to_string(),
            Style::default().fg(theme.fg),
        ));
    }

    spans
}

/// Render validation modal with detailed error and warning information
fn render_validation_modal(
    frame: &mut Frame,
    area: Rect,
    feedback: &ValidationFeedback,
    theme: &Theme,
) {
    // Calculate modal size (centered, 80% of screen)
    let modal_width = (area.width as f32 * 0.8) as u16;
    let modal_height = (area.height as f32 * 0.6) as u16;
    let modal_x = (area.width.saturating_sub(modal_width)) / 2;
    let modal_y = (area.height.saturating_sub(modal_height)) / 2;

    let modal_area = Rect {
        x: area.x + modal_x,
        y: area.y + modal_y,
        width: modal_width,
        height: modal_height,
    };

    // Clear the area behind the modal (semi-transparent effect)
    let clear_block = Block::default()
        .style(Style::default().bg(Color::Black));
    frame.render_widget(clear_block, area);

    // Build validation items
    let mut items = Vec::new();

    items.push(ListItem::new(Line::from(Span::styled(
        "Validation Details",
        Style::default()
            .fg(theme.primary)
            .add_modifier(Modifier::BOLD),
    ))));
    items.push(ListItem::new(Line::from("")));

    // Show errors
    if !feedback.errors.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            format!("Errors ({})", feedback.errors.len()),
            Style::default()
                .fg(theme.error)
                .add_modifier(Modifier::BOLD),
        ))));
        items.push(ListItem::new(Line::from("")));

        for (line_num, error) in &feedback.errors {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("  ❌ Line ", Style::default().fg(theme.error)),
                Span::styled(
                    format!("{}: ", line_num),
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(error.clone(), Style::default().fg(theme.fg)),
            ])));
        }
        items.push(ListItem::new(Line::from("")));
    }

    // Show warnings
    if !feedback.warnings.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            format!("Warnings ({})", feedback.warnings.len()),
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        ))));
        items.push(ListItem::new(Line::from("")));

        for (line_num, warning) in &feedback.warnings {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("  ⚠️  Line ", Style::default().fg(theme.warning)),
                Span::styled(
                    format!("{}: ", line_num),
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(warning.clone(), Style::default().fg(theme.fg)),
            ])));
        }
    }

    items.push(ListItem::new(Line::from("")));
    items.push(ListItem::new(Line::from(Span::styled(
        "Press 'v' or 'Esc' to close",
        Style::default()
            .fg(theme.muted)
            .add_modifier(Modifier::ITALIC),
    ))));

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent))
            .title(" Validation Details ")
            .style(Style::default().bg(theme.bg)),
    );

    frame.render_widget(list, modal_area);
}

/// Highlight a single YAML line
fn highlight_yaml_line<'a>(line: &'a str, theme: &Theme) -> Vec<Span<'a>> {
    let trimmed = line.trim_start();

    if trimmed.starts_with('#') {
        // Comment
        vec![Span::styled(
            line,
            Style::default().fg(theme.muted).add_modifier(Modifier::ITALIC),
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

/// Highlight a YAML line with a visible cursor
fn highlight_yaml_line_with_cursor(line: &str, cursor_col: usize, theme: &Theme) -> Vec<Span<'static>> {
    // Convert cursor column to byte position (handle multi-byte chars)
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

    // Add text before cursor (with syntax highlighting)
    if !before.is_empty() {
        let highlighted_before = highlight_yaml_line(&before, theme);
        for span in highlighted_before {
            spans.push(Span::styled(span.content.to_string(), span.style));
        }
    }

    // Add cursor character with inverted colors (bright and visible)
    spans.push(Span::styled(
        cursor_char.to_string(),
        Style::default()
            .fg(Color::Black)
            .bg(Color::White)
            .add_modifier(Modifier::BOLD),
    ));

    // Add text after cursor (with syntax highlighting)
    if !after.is_empty() {
        let highlighted_after = highlight_yaml_line(&after, theme);
        for span in highlighted_after {
            spans.push(Span::styled(span.content.to_string(), span.style));
        }
    }

    spans
}

/// Calculate scroll offset to keep cursor visible
fn calculate_scroll_offset(cursor_line: usize, current_offset: usize, viewport_height: usize) -> usize {
    if cursor_line < current_offset {
        // Cursor above viewport, scroll up
        cursor_line
    } else if cursor_line >= current_offset + viewport_height {
        // Cursor below viewport, scroll down
        cursor_line.saturating_sub(viewport_height - 1)
    } else {
        // Cursor in viewport, keep current offset
        current_offset
    }
}

/// Parse workflow content from YAML string
fn parse_workflow_content(content: &str) -> Result<DSLWorkflow, String> {
    serde_yaml::from_str(content).map_err(|e| e.to_string())
}

/// Render form fields for structured editing
#[allow(clippy::vec_init_then_push)]
fn render_form_fields(workflow: &DSLWorkflow, theme: &Theme) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    // Workflow metadata
    lines.push(Line::from(Span::styled(
        "━━━ Workflow Metadata ━━━",
        Style::default()
            .fg(theme.primary)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::styled("Name:        ", Style::default().fg(theme.muted)),
        Span::styled(
            workflow.name.clone(),
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ),
    ]));

    lines.push(Line::from(vec![
        Span::styled("Version:     ", Style::default().fg(theme.muted)),
        Span::styled(workflow.version.clone(), Style::default().fg(theme.fg)),
    ]));

    lines.push(Line::from(vec![
        Span::styled("DSL Version: ", Style::default().fg(theme.muted)),
        Span::styled(workflow.dsl_version.clone(), Style::default().fg(theme.fg)),
    ]));

    if let Some(ref cwd) = workflow.cwd {
        lines.push(Line::from(vec![
            Span::styled("Working Dir: ", Style::default().fg(theme.muted)),
            Span::styled(cwd.clone(), Style::default().fg(theme.fg)),
        ]));
    }

    lines.push(Line::from(""));

    // Agents
    if !workflow.agents.is_empty() {
        lines.push(Line::from(Span::styled(
            format!("━━━ Agents ({}) ━━━", workflow.agents.len()),
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        for (id, agent) in &workflow.agents {
            lines.push(Line::from(vec![
                Span::styled("◆ ", Style::default().fg(theme.accent)),
                Span::styled(
                    id.clone(),
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  Description: ", Style::default().fg(theme.muted)),
                Span::styled(agent.description.clone(), Style::default().fg(theme.fg)),
            ]));
            if let Some(ref model) = agent.model {
                lines.push(Line::from(vec![
                    Span::styled("  Model:       ", Style::default().fg(theme.muted)),
                    Span::styled(model.clone(), Style::default().fg(theme.success)),
                ]));
            }
            lines.push(Line::from(""));
        }
    }

    // Tasks
    if !workflow.tasks.is_empty() {
        lines.push(Line::from(Span::styled(
            format!("━━━ Tasks ({}) ━━━", workflow.tasks.len()),
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        for (id, task) in &workflow.tasks {
            lines.push(Line::from(vec![
                Span::styled("▶ ", Style::default().fg(theme.success)),
                Span::styled(
                    id.clone(),
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  Description: ", Style::default().fg(theme.muted)),
                Span::styled(task.description.clone(), Style::default().fg(theme.fg)),
            ]));
            if let Some(ref agent) = task.agent {
                lines.push(Line::from(vec![
                    Span::styled("  Agent:       ", Style::default().fg(theme.muted)),
                    Span::styled(agent.clone(), Style::default().fg(theme.primary)),
                ]));
            }
            lines.push(Line::from(""));
        }
    }

    lines
}

/// Render scrollbar
fn render_scrollbar(
    frame: &mut Frame,
    area: Rect,
    total_lines: usize,
    scroll_offset: usize,
    content_height: usize,
    theme: &Theme,
) {
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .style(Style::default().fg(theme.border))
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));

    let mut scrollbar_state = ScrollbarState::new(total_lines)
        .position(scroll_offset)
        .viewport_content_length(content_height);

    frame.render_stateful_widget(
        scrollbar,
        area.inner(ratatui::layout::Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
}

/// Get auto-completion suggestions based on context
pub fn get_autocomplete_suggestions(line: &str, cursor_col: usize) -> Vec<AutoCompletionSuggestion> {
    let mut suggestions = Vec::new();

    // Get word at cursor
    let before_cursor = &line[..cursor_col.min(line.len())];
    let word_start = before_cursor
        .rfind(|c: char| c.is_whitespace() || c == ':')
        .map(|i| i + 1)
        .unwrap_or(0);
    let prefix = &before_cursor[word_start..];

    // Match against DSL keywords
    for (keyword, description) in DSL_KEYWORDS {
        if keyword.starts_with(prefix) {
            suggestions.push(AutoCompletionSuggestion {
                text: keyword.to_string(),
                description: description.to_string(),
                category: "keyword".to_string(),
            });
        }
    }

    suggestions
}

/// Validate workflow and return feedback
pub fn validate_and_get_feedback(content: &str) -> ValidationFeedback {
    let mut feedback = ValidationFeedback::new();

    // Parse workflow
    match parse_workflow_content(content) {
        Ok(workflow) => {
            // Run semantic validation
            if let Err(e) = validate_workflow(&workflow) {
                // Extract error messages (simplified - would need better parsing)
                let error_msg = e.to_string();
                for (idx, line) in error_msg.lines().enumerate() {
                    if !line.is_empty() && !line.contains("Workflow validation failed") {
                        feedback.errors.push((idx + 1, line.to_string()));
                    }
                }
            }
        }
        Err(e) => {
            // Parse error - try to extract line number
            let error_msg = e.to_string();
            if let Some(line_num) = extract_line_number(&error_msg) {
                feedback.errors.push((line_num, error_msg));
            } else {
                feedback.errors.push((1, error_msg));
            }
        }
    }

    feedback.validated_at = Some(std::time::Instant::now());
    feedback
}

/// Extract line number from error message
fn extract_line_number(error: &str) -> Option<usize> {
    // Try to find "line X" pattern
    error
        .split_whitespace()
        .skip_while(|&w| w != "line")
        .nth(1)
        .and_then(|s| s.trim_end_matches(&[',', ':'][..]).parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autocomplete_suggestions() {
        let suggestions = get_autocomplete_suggestions("na", 2);
        assert!(suggestions.iter().any(|s| s.text == "name:"));

        let suggestions = get_autocomplete_suggestions("age", 3);
        assert!(suggestions.iter().any(|s| s.text == "agents:"));
    }

    #[test]
    fn test_validate_valid_workflow() {
        let yaml = r#"
name: "Test Workflow"
version: "1.0.0"
agents:
  test_agent:
    description: "Test agent"
tasks:
  test_task:
    description: "Test task"
    agent: "test_agent"
"#;

        let feedback = validate_and_get_feedback(yaml);
        assert!(feedback.is_valid());
    }

    #[test]
    fn test_validate_invalid_workflow() {
        let yaml = r#"
name: "Test Workflow"
version: "1.0.0"
tasks:
  test_task:
    description: "Test task"
    agent: "nonexistent_agent"
"#;

        let feedback = validate_and_get_feedback(yaml);
        assert!(!feedback.is_valid());
        assert!(feedback.error_count() > 0);
    }

    #[test]
    fn test_highlight_yaml_line() {
        let theme = Theme::default();

        // Test comment
        let spans = highlight_yaml_line("# This is a comment", &theme);
        assert_eq!(spans.len(), 1);

        // Test key-value
        let spans = highlight_yaml_line("name: value", &theme);
        assert!(spans.len() >= 2);

        // Test list item
        let spans = highlight_yaml_line("- item", &theme);
        assert!(spans.len() >= 2);
    }
}
