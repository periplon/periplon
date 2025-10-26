//! Modal dialog component
//!
//! Provides overlay modals for confirmations, inputs, and messages.

use crate::tui::state::Modal;
use crate::tui::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

// Re-export modal types from state
pub use crate::tui::state::{ConfirmAction, InputAction};

/// Modal view component
pub struct ModalView;

impl ModalView {
    /// Create new modal view
    pub fn new() -> Self {
        Self
    }

    /// Render a modal dialog (static method)
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        modal: &Modal,
        input_buffer: &str,
        theme: &Theme,
    ) {
        render_modal_impl(frame, area, modal, input_buffer, theme);
    }
}

impl Default for ModalView {
    fn default() -> Self {
        Self::new()
    }
}

/// Legacy render function for backward compatibility
pub fn render(
    frame: &mut Frame,
    area: Rect,
    modal: &Modal,
    input_buffer: &str,
    theme: &Theme,
) {
    render_modal_impl(frame, area, modal, input_buffer, theme);
}

/// Internal modal rendering implementation
fn render_modal_impl(
    frame: &mut Frame,
    area: Rect,
    modal: &Modal,
    input_buffer: &str,
    theme: &Theme,
) {
    // Create centered modal area with better proportions
    let modal_area = {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(25),  // Top margin
                Constraint::Min(10),         // Modal content (min 10 lines)
                Constraint::Percentage(25),  // Bottom margin
            ])
            .split(area);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(15),  // Left margin
                Constraint::Min(40),         // Modal content (min 40 chars)
                Constraint::Percentage(15),  // Right margin
            ])
            .split(vertical[1])[1]
    };

    // Clear the background
    frame.render_widget(Clear, modal_area);

    match modal {
        Modal::Confirm { title, message, .. } => {
            let block = Block::default()
                .title(title.as_str())
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.warning));

            let text = vec![
                Line::from(""),
                Line::from(message.as_str()),
                Line::from(""),
                Line::from(vec![
                    Span::styled("y", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
                    Span::raw(": Yes  "),
                    Span::styled("n", Style::default().fg(theme.error).add_modifier(Modifier::BOLD)),
                    Span::raw(": No"),
                ]),
            ];

            let paragraph = Paragraph::new(text)
                .block(block)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });

            frame.render_widget(paragraph, modal_area);
        }
        Modal::Input { title, prompt, .. } => {
            let block = Block::default()
                .title(title.as_str())
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent));

            let cursor = "▌";

            let text = vec![
                Line::from(""),
                Line::from(prompt.as_str()),
                Line::from(""),
                Line::from(vec![
                    Span::styled("> ", Style::default().fg(theme.accent)),
                    Span::styled(input_buffer, Style::default().fg(theme.fg)),
                    Span::styled(cursor, Style::default().fg(theme.accent).add_modifier(Modifier::SLOW_BLINK)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Enter", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
                    Span::raw(": Submit  "),
                    Span::styled("Esc", Style::default().fg(theme.error).add_modifier(Modifier::BOLD)),
                    Span::raw(": Cancel"),
                ]),
            ];

            let paragraph = Paragraph::new(text)
                .block(block)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });

            frame.render_widget(paragraph, modal_area);
        }
        Modal::Error { title, message } => {
            let block = Block::default()
                .title(title.as_str())
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.error));

            let text = vec![
                Line::from(""),
                Line::from(Span::styled(message.as_str(), Style::default().fg(theme.error))),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Enter", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
                    Span::raw(": Close"),
                ]),
            ];

            let paragraph = Paragraph::new(text)
                .block(block)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });

            frame.render_widget(paragraph, modal_area);
        }
        Modal::Info { title, message } => {
            let block = Block::default()
                .title(title.as_str())
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.primary));

            let text = vec![
                Line::from(""),
                Line::from(message.as_str()),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Enter", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
                    Span::raw(": Close"),
                ]),
            ];

            let paragraph = Paragraph::new(text)
                .block(block)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });

            frame.render_widget(paragraph, modal_area);
        }
        Modal::Success { title, message } => {
            let block = Block::default()
                .title(title.as_str())
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.success));

            let text = vec![
                Line::from(""),
                Line::from(Span::styled(message.as_str(), Style::default().fg(theme.success))),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Enter", Style::default().fg(theme.success).add_modifier(Modifier::BOLD)),
                    Span::raw(": Close"),
                ]),
            ];

            let paragraph = Paragraph::new(text)
                .block(block)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });

            frame.render_widget(paragraph, modal_area);
        }
    }
}
