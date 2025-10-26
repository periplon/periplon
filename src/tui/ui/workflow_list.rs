//! Workflow list view component
//!
//! Displays a list of available workflows with search and navigation.

use crate::tui::{state::WorkflowEntry, theme::Theme};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

/// Workflow list view component
pub struct WorkflowListView;

impl WorkflowListView {
    /// Create new workflow list view
    pub fn new() -> Self {
        Self
    }

    /// Render the workflow list
    pub fn render(
        frame: &mut ratatui::Frame,
        area: ratatui::layout::Rect,
        workflows: &[WorkflowEntry],
        selected: usize,
        theme: &Theme,
    ) {
        // Split into header, list, and footer with proper margins
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(5),    // List (min 5 lines)
                Constraint::Length(2), // Footer (2 lines for better visibility)
            ])
            .split(area);

        // Header with centered title
        let header = Paragraph::new(Line::from(vec![Span::styled(
            "DSL Workflow Manager",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )]))
        .alignment(ratatui::layout::Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border)),
        );
        frame.render_widget(header, chunks[0]);

        // Build workflow list
        let items: Vec<ListItem> = if workflows.is_empty() {
            vec![
                ListItem::new(Line::from("")),
                ListItem::new(Line::from(Span::styled(
                    "No workflows found in current directory",
                    Style::default().fg(theme.muted),
                ))),
                ListItem::new(Line::from("")),
                ListItem::new(Line::from(vec![
                    Span::raw("Press "),
                    Span::styled(
                        "n",
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" to create a new workflow"),
                ])),
                ListItem::new(Line::from(vec![
                    Span::raw("Press "),
                    Span::styled(
                        "q",
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" to quit"),
                ])),
            ]
        } else {
            workflows
                .iter()
                .enumerate()
                .map(|(idx, entry)| {
                    let is_selected = idx == selected;
                    let style = if is_selected {
                        Style::default()
                            .fg(theme.bg)
                            .bg(theme.highlight)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.fg)
                    };

                    let prefix = if is_selected { "▶ " } else { "  " };
                    ListItem::new(Line::from(vec![
                        Span::styled(prefix, style),
                        Span::styled(&entry.name, style),
                    ]))
                })
                .collect()
        };

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(" Workflows "),
        );
        frame.render_widget(list, chunks[1]);

        // Footer with keybindings (multi-line for better readability)
        let footer = Paragraph::new(vec![Line::from(vec![
            Span::styled(
                "↑↓",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Select │ "),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" View │ "),
            Span::styled(
                "e",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Edit │ "),
            Span::styled(
                "g",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Generate │ "),
            Span::styled(
                "n",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" New │ "),
            Span::styled(
                "s",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" States │ "),
            Span::styled(
                "?",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Help │ "),
            Span::styled(
                "q",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Quit"),
        ])])
        .style(Style::default().fg(theme.fg).bg(theme.bg))
        .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(footer, chunks[2]);
    }
}

impl Default for WorkflowListView {
    fn default() -> Self {
        Self::new()
    }
}
