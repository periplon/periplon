//! Common test assertions for TUI testing
//!
//! Provides reusable assertion functions for terminal and UI testing.

use super::terminal::{buffer_contains, buffer_content, terminal_size};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

/// Assert that terminal buffer contains the specified text
///
/// # Panics
///
/// Panics if the text is not found in the buffer.
///
/// # Examples
///
/// ```ignore
/// assert_buffer_contains(&terminal, "Hello World");
/// ```
pub fn assert_buffer_contains(terminal: &Terminal<TestBackend>, text: &str) {
    assert!(
        buffer_contains(terminal, text),
        "Buffer does not contain expected text: '{}'\nBuffer content:\n{}",
        text,
        buffer_content(terminal)
    );
}

/// Assert that terminal buffer does NOT contain the specified text
///
/// # Panics
///
/// Panics if the text IS found in the buffer.
///
/// # Examples
///
/// ```ignore
/// assert_buffer_not_contains(&terminal, "Error");
/// ```
pub fn assert_buffer_not_contains(terminal: &Terminal<TestBackend>, text: &str) {
    assert!(
        !buffer_contains(terminal, text),
        "Buffer contains unexpected text: '{}'\nBuffer content:\n{}",
        text,
        buffer_content(terminal)
    );
}

/// Assert terminal has expected dimensions
///
/// # Panics
///
/// Panics if terminal size doesn't match expected dimensions.
///
/// # Examples
///
/// ```ignore
/// assert_terminal_size(&terminal, 80, 24);
/// ```
pub fn assert_terminal_size(terminal: &Terminal<TestBackend>, width: u16, height: u16) {
    let (actual_width, actual_height) = terminal_size(terminal);
    assert_eq!(
        (actual_width, actual_height),
        (width, height),
        "Terminal size mismatch: expected {}x{}, got {}x{}",
        width,
        height,
        actual_width,
        actual_height
    );
}

/// Assert terminal width matches expected value
pub fn assert_terminal_width(terminal: &Terminal<TestBackend>, width: u16) {
    let (actual_width, _) = terminal_size(terminal);
    assert_eq!(
        actual_width, width,
        "Terminal width mismatch: expected {}, got {}",
        width, actual_width
    );
}

/// Assert terminal height matches expected value
pub fn assert_terminal_height(terminal: &Terminal<TestBackend>, height: u16) {
    let (_, actual_height) = terminal_size(terminal);
    assert_eq!(
        actual_height, height,
        "Terminal height mismatch: expected {}, got {}",
        height, actual_height
    );
}

/// Assert terminal buffer contains all of the specified texts
///
/// # Panics
///
/// Panics if any of the texts are not found.
pub fn assert_buffer_contains_all(terminal: &Terminal<TestBackend>, texts: &[&str]) {
    let content = buffer_content(terminal);
    for text in texts {
        assert!(
            content.contains(text),
            "Buffer does not contain expected text: '{}'\nBuffer content:\n{}",
            text, content
        );
    }
}

/// Assert terminal buffer contains at least one of the specified texts
///
/// # Panics
///
/// Panics if none of the texts are found.
pub fn assert_buffer_contains_any(terminal: &Terminal<TestBackend>, texts: &[&str]) {
    let content = buffer_content(terminal);
    let found = texts.iter().any(|text| content.contains(text));
    assert!(
        found,
        "Buffer does not contain any of the expected texts: {:?}\nBuffer content:\n{}",
        texts, content
    );
}

/// Assert buffer contains text appearing in order
///
/// This checks that the texts appear in the specified order in the buffer,
/// though they don't need to be adjacent.
pub fn assert_buffer_contains_in_order(terminal: &Terminal<TestBackend>, texts: &[&str]) {
    let content = buffer_content(terminal);
    let mut last_pos = 0;

    for text in texts {
        if let Some(pos) = content[last_pos..].find(text) {
            last_pos += pos + text.len();
        } else {
            panic!(
                "Expected text '{}' not found after position {} in buffer.\nBuffer content:\n{}",
                text, last_pos, content
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::terminal::render_with_terminal;

    #[test]
    fn test_assert_buffer_contains() {
        let terminal = render_with_terminal(40, 10, |f| {
            use ratatui::widgets::Paragraph;
            let text = Paragraph::new("Hello World");
            f.render_widget(text, f.area());
        });

        assert_buffer_contains(&terminal, "Hello");
        assert_buffer_contains(&terminal, "World");
    }

    #[test]
    #[should_panic(expected = "Buffer does not contain expected text")]
    fn test_assert_buffer_contains_panic() {
        let terminal = render_with_terminal(40, 10, |f| {
            use ratatui::widgets::Paragraph;
            let text = Paragraph::new("Hello World");
            f.render_widget(text, f.area());
        });

        assert_buffer_contains(&terminal, "Goodbye");
    }

    #[test]
    fn test_assert_buffer_not_contains() {
        let terminal = render_with_terminal(40, 10, |f| {
            use ratatui::widgets::Paragraph;
            let text = Paragraph::new("Hello World");
            f.render_widget(text, f.area());
        });

        assert_buffer_not_contains(&terminal, "Goodbye");
    }

    #[test]
    fn test_assert_terminal_size() {
        let terminal = render_with_terminal(80, 24, |_f| {});
        assert_terminal_size(&terminal, 80, 24);
    }

    #[test]
    fn test_assert_buffer_contains_all() {
        let terminal = render_with_terminal(40, 10, |f| {
            use ratatui::widgets::Paragraph;
            let text = Paragraph::new("Hello World Test");
            f.render_widget(text, f.area());
        });

        assert_buffer_contains_all(&terminal, &["Hello", "World", "Test"]);
    }

    #[test]
    fn test_assert_buffer_contains_any() {
        let terminal = render_with_terminal(40, 10, |f| {
            use ratatui::widgets::Paragraph;
            let text = Paragraph::new("Hello World");
            f.render_widget(text, f.area());
        });

        assert_buffer_contains_any(&terminal, &["Goodbye", "Hello", "Test"]);
    }

    #[test]
    fn test_assert_buffer_contains_in_order() {
        let terminal = render_with_terminal(40, 10, |f| {
            use ratatui::widgets::Paragraph;
            let text = Paragraph::new("First Second Third");
            f.render_widget(text, f.area());
        });

        assert_buffer_contains_in_order(&terminal, &["First", "Second", "Third"]);
    }
}
