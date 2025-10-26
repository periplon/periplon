//! Workflow viewer component
//!
//! Provides read-only workflow visualization with two view modes:
//! - Condensed: Summary view with workflow metadata, agents, and tasks
//! - Full: Complete YAML content with syntax highlighting
//!
//! Navigation: Arrow keys, PageUp/PageDown, Home/End
//! View toggle: Tab key

use crate::dsl::{AgentSpec, DSLWorkflow, TaskSpec};
use crate::tui::state::{ViewerState, WorkflowViewMode};
use crate::tui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};

/// Render the workflow viewer
pub fn render(frame: &mut Frame, area: Rect, workflow: &DSLWorkflow, state: &ViewerState, theme: &Theme) {
    // Create main layout with header and content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    // Render header
    render_header(frame, chunks[0], workflow, state, theme);

    // Render content based on view mode
    match state.view_mode {
        WorkflowViewMode::Condensed => render_condensed_view(frame, chunks[1], workflow, state, theme),
        WorkflowViewMode::Full => render_full_view(frame, chunks[1], workflow, state, theme),
    }

    // Render status bar
    render_status_bar(frame, chunks[2], state, theme);
}

/// Render header with workflow metadata
fn render_header(frame: &mut Frame, area: Rect, workflow: &DSLWorkflow, state: &ViewerState, theme: &Theme) {
    let view_mode_text = match state.view_mode {
        WorkflowViewMode::Condensed => "Condensed",
        WorkflowViewMode::Full => "Full YAML",
    };

    let header_text = vec![
        Line::from(vec![
            Span::styled("Workflow: ", Style::default().fg(theme.muted)),
            Span::styled(&workflow.name, Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(" | Version: ", Style::default().fg(theme.muted)),
            Span::styled(&workflow.version, Style::default().fg(theme.accent)),
            Span::styled(" | View: ", Style::default().fg(theme.muted)),
            Span::styled(view_mode_text, Style::default().fg(theme.success)),
        ]),
    ];

    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border)));

    frame.render_widget(header, area);
}

/// Render condensed summary view
#[allow(clippy::vec_init_then_push)]
pub fn render_condensed_view(frame: &mut Frame, area: Rect, workflow: &DSLWorkflow, state: &ViewerState, theme: &Theme) {
    let mut lines = Vec::new();

    // Workflow metadata section
    lines.push(Line::from(vec![
        Span::styled("━━━ ", Style::default().fg(theme.border)),
        Span::styled("Workflow Metadata", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled(" ━━━", Style::default().fg(theme.border)),
    ]));
    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::styled("  Name:        ", Style::default().fg(theme.muted)),
        Span::styled(&workflow.name, Style::default().fg(theme.fg)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Version:     ", Style::default().fg(theme.muted)),
        Span::styled(&workflow.version, Style::default().fg(theme.fg)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  DSL Version: ", Style::default().fg(theme.muted)),
        Span::styled(&workflow.dsl_version, Style::default().fg(theme.fg)),
    ]));

    if let Some(ref cwd) = workflow.cwd {
        lines.push(Line::from(vec![
            Span::styled("  Working Dir: ", Style::default().fg(theme.muted)),
            Span::styled(cwd, Style::default().fg(theme.fg)),
        ]));
    }

    lines.push(Line::from(""));

    // Agents section
    if !workflow.agents.is_empty() {
        let agents_count = format!(" ({}) ", workflow.agents.len());
        lines.push(Line::from(vec![
            Span::styled("━━━ ", Style::default().fg(theme.border)),
            Span::styled("Agents", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(agents_count, Style::default().fg(theme.muted)),
            Span::styled("━━━", Style::default().fg(theme.border)),
        ]));
        lines.push(Line::from(""));

        for (agent_id, agent) in &workflow.agents {
            lines.extend(render_agent_summary(agent_id, agent, theme));
            lines.push(Line::from(""));
        }
    }

    // Tasks section
    if !workflow.tasks.is_empty() {
        let tasks_count = format!(" ({}) ", workflow.tasks.len());
        lines.push(Line::from(vec![
            Span::styled("━━━ ", Style::default().fg(theme.border)),
            Span::styled("Tasks", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(tasks_count, Style::default().fg(theme.muted)),
            Span::styled("━━━", Style::default().fg(theme.border)),
        ]));
        lines.push(Line::from(""));

        for (task_id, task) in &workflow.tasks {
            lines.extend(render_task_summary(task_id, task, theme));
            lines.push(Line::from(""));
        }
    }

    // Inputs section
    if !workflow.inputs.is_empty() {
        let inputs_count = format!(" ({}) ", workflow.inputs.len());
        lines.push(Line::from(vec![
            Span::styled("━━━ ", Style::default().fg(theme.border)),
            Span::styled("Inputs", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(inputs_count, Style::default().fg(theme.muted)),
            Span::styled("━━━", Style::default().fg(theme.border)),
        ]));
        lines.push(Line::from(""));

        for (name, spec) in &workflow.inputs {
            lines.push(Line::from(vec![
                Span::styled("  • ", Style::default().fg(theme.muted)),
                Span::styled(name, Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::styled(": ", Style::default().fg(theme.muted)),
                Span::styled(&spec.param_type, Style::default().fg(theme.success)),
                if spec.required {
                    Span::styled(" (required)", Style::default().fg(theme.warning))
                } else {
                    Span::styled(" (optional)", Style::default().fg(theme.muted))
                },
            ]));
        }
        lines.push(Line::from(""));
    }

    // Outputs section
    if !workflow.outputs.is_empty() {
        let outputs_count = format!(" ({}) ", workflow.outputs.len());
        lines.push(Line::from(vec![
            Span::styled("━━━ ", Style::default().fg(theme.border)),
            Span::styled("Outputs", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(outputs_count, Style::default().fg(theme.muted)),
            Span::styled("━━━", Style::default().fg(theme.border)),
        ]));
        lines.push(Line::from(""));

        for name in workflow.outputs.keys() {
            lines.push(Line::from(vec![
                Span::styled("  • ", Style::default().fg(theme.muted)),
                Span::styled(name, Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            ]));
        }
        lines.push(Line::from(""));
    }

    // Render with scrolling
    let content_height = area.height.saturating_sub(2) as usize; // Account for borders
    let total_lines = lines.len();
    let scroll_offset = (state.scroll as usize).min(total_lines.saturating_sub(content_height));

    let text = Text::from(lines);
    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border)))
        .scroll((scroll_offset as u16, 0))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);

    // Render scrollbar if needed
    if total_lines > content_height {
        render_scrollbar(frame, area, total_lines, scroll_offset, content_height, theme);
    }
}

/// Render agent summary
fn render_agent_summary(agent_id: &str, agent: &AgentSpec, theme: &Theme) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled("◆ ", Style::default().fg(theme.primary)),
        Span::styled(agent_id.to_string(), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
    ]));

    lines.push(Line::from(vec![
        Span::styled("    Description: ", Style::default().fg(theme.muted)),
        Span::styled(agent.description.clone(), Style::default().fg(theme.fg)),
    ]));

    if let Some(ref model) = agent.model {
        lines.push(Line::from(vec![
            Span::styled("    Model:       ", Style::default().fg(theme.muted)),
            Span::styled(model.clone(), Style::default().fg(theme.success)),
        ]));
    }

    if !agent.tools.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("    Tools:       ", Style::default().fg(theme.muted)),
            Span::styled(agent.tools.join(", "), Style::default().fg(theme.fg)),
        ]));
    }

    if let Some(max_turns) = agent.max_turns {
        lines.push(Line::from(vec![
            Span::styled("    Max Turns:   ", Style::default().fg(theme.muted)),
            Span::styled(max_turns.to_string(), Style::default().fg(theme.warning)),
        ]));
    }

    lines
}

/// Render task summary
fn render_task_summary(task_id: &str, task: &TaskSpec, theme: &Theme) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled("▶ ", Style::default().fg(theme.success)),
        Span::styled(task_id.to_string(), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
    ]));

    lines.push(Line::from(vec![
        Span::styled("    Description: ", Style::default().fg(theme.muted)),
        Span::styled(task.description.clone(), Style::default().fg(theme.fg)),
    ]));

    if let Some(ref agent) = task.agent {
        lines.push(Line::from(vec![
            Span::styled("    Agent:       ", Style::default().fg(theme.muted)),
            Span::styled(agent.clone(), Style::default().fg(theme.primary)),
        ]));
    }

    if !task.depends_on.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("    Depends On:  ", Style::default().fg(theme.muted)),
            Span::styled(task.depends_on.join(", "), Style::default().fg(theme.warning)),
        ]));
    }

    if !task.subtasks.is_empty() {
        let subtasks_text = format!("{} tasks", task.subtasks.len());
        lines.push(Line::from(vec![
            Span::styled("    Subtasks:    ", Style::default().fg(theme.muted)),
            Span::styled(subtasks_text, Style::default().fg(theme.fg)),
        ]));
    }

    lines
}

/// Render full YAML view with syntax highlighting
pub fn render_full_view(frame: &mut Frame, area: Rect, workflow: &DSLWorkflow, state: &ViewerState, theme: &Theme) {
    // Serialize workflow to YAML
    let yaml_content = match serde_yaml::to_string(workflow) {
        Ok(yaml) => yaml,
        Err(e) => format!("Error serializing workflow: {}", e),
    };

    // Apply syntax highlighting
    let highlighted_lines = highlight_yaml(&yaml_content, theme);

    // Calculate scroll
    let content_height = area.height.saturating_sub(2) as usize;
    let total_lines = highlighted_lines.len();
    let scroll_offset = (state.scroll as usize).min(total_lines.saturating_sub(content_height));

    let text = Text::from(highlighted_lines);
    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border)))
        .scroll((scroll_offset as u16, 0))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);

    // Render scrollbar if needed
    if total_lines > content_height {
        render_scrollbar(frame, area, total_lines, scroll_offset, content_height, theme);
    }
}

/// Apply syntax highlighting to YAML content
fn highlight_yaml(yaml: &str, theme: &Theme) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    for line in yaml.lines() {
        let trimmed = line.trim_start();
        let indent_len = line.len() - trimmed.len();
        let indent = " ".repeat(indent_len);

        let highlighted_line = if trimmed.starts_with('#') {
            // Comment
            Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(theme.muted).add_modifier(Modifier::ITALIC),
            ))
        } else if trimmed.starts_with("---") || trimmed.starts_with("...") {
            // Document separator
            Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(theme.border),
            ))
        } else if trimmed.starts_with('-') && trimmed.len() > 1 && trimmed.chars().nth(1) == Some(' ') {
            // List item
            Line::from(vec![
                Span::styled(indent, Style::default()),
                Span::styled("- ", Style::default().fg(theme.accent)),
                Span::styled(
                    trimmed[2..].to_string(),
                    Style::default().fg(theme.fg),
                ),
            ])
        } else if let Some(colon_pos) = trimmed.find(':') {
            // Key-value pair
            let key = &trimmed[..colon_pos];
            let value = &trimmed[colon_pos + 1..].trim_start();

            let value_style = if value.starts_with('"') || value.starts_with('\'') {
                // String value
                Style::default().fg(theme.success)
            } else if value.parse::<f64>().is_ok() {
                // Number value
                Style::default().fg(theme.warning)
            } else if *value == "true" || *value == "false" || *value == "null" {
                // Boolean or null
                Style::default().fg(theme.primary)
            } else {
                // Default
                Style::default().fg(theme.fg)
            };

            Line::from(vec![
                Span::styled(indent, Style::default()),
                Span::styled(
                    key.to_string(),
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                ),
                Span::styled(": ", Style::default().fg(theme.muted)),
                Span::styled(value.to_string(), value_style),
            ])
        } else {
            // Plain line
            Line::from(Span::styled(line.to_string(), Style::default().fg(theme.fg)))
        };

        lines.push(highlighted_line);
    }

    lines
}

/// Render status bar with navigation hints
fn render_status_bar(frame: &mut Frame, area: Rect, state: &ViewerState, theme: &Theme) {
    let status_text = match state.view_mode {
        WorkflowViewMode::Condensed => {
            "Tab: Full YAML | ↑↓: Scroll | PgUp/PgDn: Page | Home/End: Top/Bottom | Esc: Back"
        }
        WorkflowViewMode::Full => {
            "Tab: Condensed | ↑↓: Scroll | PgUp/PgDn: Page | Home/End: Top/Bottom | Esc: Back"
        }
    };

    let status = Paragraph::new(status_text)
        .style(Style::default().fg(theme.muted).bg(theme.bg));

    frame.render_widget(status, area);
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
