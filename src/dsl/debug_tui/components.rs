//! TUI components for rendering different panes
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::dsl::debugger::{DebuggerState, Inspector};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Render workflow tree pane
pub fn render_workflow_tree(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    _inspector: Option<&Arc<Inspector>>,
) {
    let title = if focused {
        " Workflow Tree [FOCUSED] "
    } else {
        " Workflow Tree "
    };

    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Gray)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    // TODO: Implement actual workflow tree rendering
    let content = [
        "📋 Workflow: Debug Example",
        "",
        "  ├─ 📝 Task: research [Ready]",
        "  ├─ 📝 Task: analyze [Pending]",
        "  └─ 📝 Task: write [Pending]",
        "",
        "Use ↑↓ to navigate, Enter to expand/collapse",
    ];

    let items: Vec<ListItem> = content.iter().map(|line| ListItem::new(*line)).collect();

    let list = List::new(items).block(block);

    frame.render_widget(list, area);
}

/// Render variables pane
pub async fn render_variables(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    inspector: Option<&Arc<Inspector>>,
) {
    let title = if focused {
        " Variables [FOCUSED] "
    } else {
        " Variables "
    };

    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Gray)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let mut lines = vec![Line::from("Workflow Variables:")];

    if let Some(insp) = inspector {
        let vars = insp.inspect_variables(None).await;

        if !vars.workflow_vars.is_empty() {
            for (name, value) in vars.workflow_vars.iter() {
                lines.push(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(name.clone(), Style::default().fg(Color::Yellow)),
                    Span::raw(" = "),
                    Span::styled(format!("{:?}", value), Style::default().fg(Color::Green)),
                ]));
            }
        } else {
            lines.push(Line::from("  (no variables)"));
        }

        if !vars.task_vars.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from("Task Variables:"));
            for (task_id, task_vars) in vars.task_vars.iter() {
                lines.push(Line::from(format!("  {}:", task_id)));
                for (name, value) in task_vars {
                    lines.push(Line::from(vec![
                        Span::raw("    "),
                        Span::styled(name.clone(), Style::default().fg(Color::Yellow)),
                        Span::raw(" = "),
                        Span::styled(format!("{:?}", value), Style::default().fg(Color::Green)),
                    ]));
                }
            }
        }
    } else {
        lines.push(Line::from("  (debugger not initialized)"));
    }

    let text = Text::from(lines);
    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Render timeline pane
pub async fn render_timeline(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    inspector: Option<&Arc<Inspector>>,
) {
    let title = if focused {
        " Timeline [FOCUSED] "
    } else {
        " Timeline "
    };

    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Gray)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let mut items = Vec::new();

    if let Some(insp) = inspector {
        let timeline = insp.timeline().await;
        let max_events = (area.height as usize)
            .saturating_sub(3)
            .min(timeline.events.len());

        for (i, event) in timeline.events.iter().take(max_events).enumerate() {
            use std::time::SystemTime;
            let elapsed = event
                .timestamp
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default();
            let timestamp = format!("[{:.2}s]", elapsed.as_secs_f64());
            let event_str = format!("{:?}", event.event_type);

            items.push(ListItem::new(vec![Line::from(vec![
                Span::styled(timestamp, Style::default().fg(Color::DarkGray)),
                Span::raw(" "),
                Span::styled(event_str, Style::default().fg(Color::White)),
            ])]));

            // Stop if we've filled the area
            if i >= max_events - 1 {
                break;
            }
        }

        if timeline.events.len() > max_events {
            items.push(ListItem::new(format!(
                "... {} more events",
                timeline.events.len() - max_events
            )));
        }
    } else {
        items.push(ListItem::new("(no timeline data)"));
    }

    let list = List::new(items).block(block);

    frame.render_widget(list, area);
}

/// Render REPL input pane
pub fn render_repl(
    frame: &mut Frame<'_>,
    area: Rect,
    focused: bool,
    input: &str,
    _cursor_position: usize,
) {
    let title = if focused {
        " REPL [FOCUSED] "
    } else {
        " REPL "
    };

    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::Gray)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let text = if focused {
        vec![Line::from(vec![
            Span::raw("debug> "),
            Span::styled(input, Style::default().fg(Color::White)),
            Span::styled(
                "█",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::RAPID_BLINK),
            ),
        ])]
    } else {
        vec![Line::from(vec![
            Span::raw("debug> "),
            Span::styled(input, Style::default().fg(Color::DarkGray)),
        ])]
    };

    let paragraph = Paragraph::new(text).block(block);

    frame.render_widget(paragraph, area);
}

/// Render status bar
pub async fn render_status(
    frame: &mut Frame<'_>,
    area: Rect,
    debugger: Option<&Arc<Mutex<DebuggerState>>>,
) {
    let status_text = if let Some(dbg) = debugger {
        let state = dbg.lock().await;
        // Count breakpoints by listing them
        let breakpoint_count = state.breakpoints.list_all().len();

        format!(
            " Mode: {:?} | Steps: {} | Breakpoints: {} ",
            state.mode, state.step_count, breakpoint_count
        )
    } else {
        " Debug TUI - Press ? for help, Tab to switch panes, q to quit ".to_string()
    };

    let paragraph =
        Paragraph::new(status_text).style(Style::default().bg(Color::DarkGray).fg(Color::White));

    frame.render_widget(paragraph, area);
}

/// Render help overlay
pub fn render_help(frame: &mut Frame<'_>, area: Rect) {
    let help_text = vec![
        Line::from(Span::styled(
            " Debug TUI - Keyboard Shortcuts ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Navigation:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from("  Tab / Shift+Tab  - Switch between panes"),
        Line::from("  ↑ / ↓            - Navigate within pane"),
        Line::from("  ← / →            - Expand/collapse tree nodes"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Execution Control:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from("  F5               - Continue execution"),
        Line::from("  F10              - Step over"),
        Line::from("  F11              - Step into"),
        Line::from("  Shift+F11        - Step out"),
        Line::from("  F9               - Toggle breakpoint"),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Other:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from("  ?                - Toggle this help"),
        Line::from("  q / Ctrl+C       - Quit"),
        Line::from(""),
        Line::from(Span::styled(
            "Press ? or Esc to close this help",
            Style::default().fg(Color::Yellow),
        )),
    ];

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .wrap(Wrap { trim: false });

    // Center the help dialog
    let help_area = Rect {
        x: area.width / 4,
        y: area.height / 6,
        width: area.width / 2,
        height: area.height * 2 / 3,
    };

    frame.render_widget(paragraph, help_area);
}
