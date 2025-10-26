//! Terminal utilities for TUI testing
//!
//! Provides helper functions for creating test terminals, rendering components,
//! and inspecting terminal buffers.

use ratatui::backend::{Backend, TestBackend};
use ratatui::Terminal;

/// Create a test terminal with specified dimensions
///
/// # Examples
///
/// ```ignore
/// let terminal = create_terminal(80, 24);
/// ```
pub fn create_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).expect("Failed to create test terminal")
}

/// Render a component using a closure and return the terminal
///
/// # Examples
///
/// ```ignore
/// let terminal = render_with_terminal(120, 40, |f| {
///     MyView::render(f, f.area(), &state, &theme);
/// });
/// ```
pub fn render_with_terminal<F>(width: u16, height: u16, render_fn: F) -> Terminal<TestBackend>
where
    F: FnOnce(&mut ratatui::Frame),
{
    let mut terminal = create_terminal(width, height);
    terminal
        .draw(render_fn)
        .expect("Failed to render to terminal");
    terminal
}

/// Check if terminal buffer contains the specified text
///
/// This is a case-sensitive search across the entire buffer content.
///
/// # Examples
///
/// ```ignore
/// assert!(buffer_contains(&terminal, "Hello World"));
/// ```
pub fn buffer_contains(terminal: &Terminal<TestBackend>, text: &str) -> bool {
    buffer_content(terminal).contains(text)
}

/// Get the full buffer content as a single string
///
/// This collects all cell symbols from the buffer into a single string,
/// useful for debugging and custom assertions.
///
/// # Examples
///
/// ```ignore
/// let content = buffer_content(&terminal);
/// assert!(content.contains("Expected text"));
/// ```
pub fn buffer_content(terminal: &Terminal<TestBackend>) -> String {
    let buffer = terminal.backend().buffer();
    buffer
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<Vec<_>>()
        .join("")
}

/// Get buffer content as lines
///
/// Returns the buffer content split into lines, useful for line-by-line
/// assertions and debugging.
pub fn buffer_lines(terminal: &Terminal<TestBackend>) -> Vec<String> {
    let buffer = terminal.backend().buffer();
    let width = buffer.area().width as usize;
    let content = buffer_content(terminal);

    content
        .chars()
        .collect::<Vec<_>>()
        .chunks(width)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect()
}

/// Count occurrences of text in buffer
///
/// # Examples
///
/// ```ignore
/// assert_eq!(count_in_buffer(&terminal, "task"), 3);
/// ```
pub fn count_in_buffer(terminal: &Terminal<TestBackend>, text: &str) -> usize {
    buffer_content(terminal).matches(text).count()
}

/// Check if buffer contains text at a specific line
///
/// Line numbers are 0-indexed.
pub fn buffer_line_contains(terminal: &Terminal<TestBackend>, line: usize, text: &str) -> bool {
    buffer_lines(terminal)
        .get(line)
        .map(|l| l.contains(text))
        .unwrap_or(false)
}

/// Get terminal dimensions
pub fn terminal_size(terminal: &Terminal<TestBackend>) -> (u16, u16) {
    let size = terminal.backend().size().expect("Failed to get terminal size");
    (size.width, size.height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_terminal() {
        let terminal = create_terminal(80, 24);
        let (width, height) = terminal_size(&terminal);
        assert_eq!(width, 80);
        assert_eq!(height, 24);
    }

    #[test]
    fn test_buffer_content() {
        let terminal = render_with_terminal(40, 10, |f| {
            use ratatui::widgets::{Block, Borders};
            let block = Block::default()
                .title("Test")
                .borders(Borders::ALL);
            f.render_widget(block, f.area());
        });

        let content = buffer_content(&terminal);
        assert!(!content.is_empty());
    }

    #[test]
    fn test_buffer_contains() {
        let terminal = render_with_terminal(40, 10, |f| {
            use ratatui::widgets::Paragraph;
            let text = Paragraph::new("Hello World");
            f.render_widget(text, f.area());
        });

        assert!(buffer_contains(&terminal, "Hello"));
        assert!(buffer_contains(&terminal, "World"));
        assert!(!buffer_contains(&terminal, "Goodbye"));
    }

    #[test]
    fn test_count_in_buffer() {
        let terminal = render_with_terminal(40, 10, |f| {
            use ratatui::widgets::Paragraph;
            let text = Paragraph::new("test test test");
            f.render_widget(text, f.area());
        });

        assert_eq!(count_in_buffer(&terminal, "test"), 3);
    }
}
