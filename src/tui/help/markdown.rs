//! Markdown rendering for TUI
//!
//! Converts markdown to styled ratatui text with support for:
//! - Headers (h1-h6)
//! - **Bold** and *italic*
//! - `inline code`
//! - Code blocks with syntax highlighting
//! - Lists (ordered and unordered)
//! - Links
//! - Tables
//! - Blockquotes

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
};

/// Markdown renderer that converts markdown to ratatui Text
#[derive(Debug)]
pub struct MarkdownRenderer {
    /// Base style for normal text
    base_style: Style,
    /// Style for headers
    header_styles: [Style; 6],
    /// Style for code
    code_style: Style,
    /// Style for bold
    bold_style: Style,
    /// Style for italic
    italic_style: Style,
    /// Style for links
    link_style: Style,
    /// Style for blockquotes
    quote_style: Style,
}

impl MarkdownRenderer {
    /// Create a new markdown renderer with default styles
    pub fn new() -> Self {
        Self {
            base_style: Style::default().fg(Color::White),
            header_styles: [
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
                Style::default().fg(Color::Blue),
                Style::default().fg(Color::LightBlue),
                Style::default().fg(Color::LightBlue),
            ],
            code_style: Style::default().fg(Color::Green).bg(Color::DarkGray),
            bold_style: Style::default().add_modifier(Modifier::BOLD),
            italic_style: Style::default().add_modifier(Modifier::ITALIC),
            link_style: Style::default()
                .fg(Color::LightCyan)
                .add_modifier(Modifier::UNDERLINED),
            quote_style: Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::ITALIC),
        }
    }

    /// Render markdown text to ratatui Text
    pub fn render(&self, markdown: &str) -> Text<'static> {
        let mut lines = Vec::new();
        let mut in_code_block = false;
        let mut in_list = false;
        let mut table_mode = false;

        for line in markdown.lines() {
            // Handle code blocks
            if line.trim_start().starts_with("```") {
                in_code_block = !in_code_block;
                if !in_code_block {
                    lines.push(Line::from(""));
                }
                continue;
            }

            if in_code_block {
                lines.push(self.render_code_line(line));
                continue;
            }

            // Handle different line types
            if let Some(header_line) = self.parse_header(line) {
                if in_list {
                    lines.push(Line::from(""));
                    in_list = false;
                }
                lines.push(header_line);
                continue;
            }

            if let Some(list_line) = self.parse_list_item(line) {
                in_list = true;
                lines.push(list_line);
                continue;
            }

            if let Some(table_line) = self.parse_table_row(line) {
                table_mode = true;
                lines.push(table_line);
                continue;
            }

            if line.trim_start().starts_with('>') {
                lines.push(self.parse_blockquote(line));
                continue;
            }

            // Regular paragraph
            if !line.trim().is_empty() {
                if in_list {
                    lines.push(Line::from(""));
                    in_list = false;
                }
                if table_mode && !line.contains('|') {
                    lines.push(Line::from(""));
                    table_mode = false;
                }
                lines.push(self.parse_inline_styles(line));
            } else {
                lines.push(Line::from(""));
                in_list = false;
                table_mode = false;
            }
        }

        Text::from(lines)
    }

    /// Parse header lines (# Header)
    fn parse_header(&self, line: &str) -> Option<Line<'static>> {
        let trimmed = line.trim_start();
        let level = trimmed.chars().take_while(|&c| c == '#').count();

        if level > 0 && level <= 6 {
            let text = trimmed[level..].trim_start();
            let style = self.header_styles[level - 1];

            Some(Line::from(vec![
                Span::raw(""),
                Span::styled(text.to_string(), style),
            ]))
        } else {
            None
        }
    }

    /// Parse list items (- item or 1. item)
    fn parse_list_item(&self, line: &str) -> Option<Line<'static>> {
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();

        // Unordered list
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let text = &trimmed[2..];
            let prefix = " ".repeat(indent) + "• ";
            let mut spans = vec![Span::raw(prefix)];
            spans.extend(self.parse_inline_spans(text));
            return Some(Line::from(spans));
        }

        // Ordered list
        if let Some(pos) = trimmed.find(". ") {
            if trimmed[..pos].chars().all(|c| c.is_ascii_digit()) {
                let text = &trimmed[pos + 2..];
                let prefix = " ".repeat(indent) + &trimmed[..pos + 2];
                let mut spans = vec![Span::raw(prefix)];
                spans.extend(self.parse_inline_spans(text));
                return Some(Line::from(spans));
            }
        }

        None
    }

    /// Parse table rows
    fn parse_table_row(&self, line: &str) -> Option<Line<'static>> {
        if !line.contains('|') {
            return None;
        }

        let trimmed = line.trim();

        // Skip separator rows (|---|---|)
        if trimmed
            .chars()
            .all(|c| c == '|' || c == '-' || c == ':' || c.is_whitespace())
        {
            return Some(Line::from(Span::styled(
                trimmed.to_string(),
                Style::default().fg(Color::DarkGray),
            )));
        }

        let cells: Vec<&str> = trimmed
            .split('|')
            .filter(|s| !s.trim().is_empty())
            .collect();

        let mut spans = Vec::new();
        for (i, cell) in cells.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
            }
            spans.extend(self.parse_inline_spans(cell.trim()));
        }

        Some(Line::from(spans))
    }

    /// Parse blockquotes (> text)
    fn parse_blockquote(&self, line: &str) -> Line<'static> {
        let text = line.trim_start().trim_start_matches('>').trim_start();
        Line::from(vec![
            Span::raw("▌ "),
            Span::styled(text.to_string(), self.quote_style),
        ])
    }

    /// Render a code block line
    fn render_code_line(&self, line: &str) -> Line<'static> {
        Line::from(Span::styled(format!("  {}", line), self.code_style))
    }

    /// Parse inline styles like **bold**, *italic*, `code`, [links]
    fn parse_inline_styles(&self, line: &str) -> Line<'static> {
        Line::from(self.parse_inline_spans(line))
    }

    /// Parse inline spans with styles
    fn parse_inline_spans(&self, text: &str) -> Vec<Span<'static>> {
        let mut spans = Vec::new();
        let mut current = String::new();
        let mut chars = text.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '`' => {
                    // Inline code
                    if !current.is_empty() {
                        spans.push(Span::styled(current.clone(), self.base_style));
                        current.clear();
                    }
                    let mut code = String::new();
                    while let Some(&next) = chars.peek() {
                        if next == '`' {
                            chars.next();
                            break;
                        }
                        code.push(chars.next().unwrap());
                    }
                    spans.push(Span::styled(code, self.code_style));
                }
                '*' => {
                    if chars.peek() == Some(&'*') {
                        // Bold
                        chars.next(); // consume second *
                        if !current.is_empty() {
                            spans.push(Span::styled(current.clone(), self.base_style));
                            current.clear();
                        }
                        let mut bold = String::new();
                        let mut found_end = false;
                        while let Some(next) = chars.next() {
                            if next == '*' && chars.peek() == Some(&'*') {
                                chars.next();
                                found_end = true;
                                break;
                            }
                            bold.push(next);
                        }
                        if found_end {
                            spans.push(Span::styled(bold, self.bold_style));
                        } else {
                            current.push_str("**");
                            current.push_str(&bold);
                        }
                    } else {
                        // Italic
                        if !current.is_empty() {
                            spans.push(Span::styled(current.clone(), self.base_style));
                            current.clear();
                        }
                        let mut italic = String::new();
                        let mut found_end = false;
                        for next in chars.by_ref() {
                            if next == '*' {
                                found_end = true;
                                break;
                            }
                            italic.push(next);
                        }
                        if found_end {
                            spans.push(Span::styled(italic, self.italic_style));
                        } else {
                            current.push('*');
                            current.push_str(&italic);
                        }
                    }
                }
                '[' => {
                    // Link
                    if !current.is_empty() {
                        spans.push(Span::styled(current.clone(), self.base_style));
                        current.clear();
                    }
                    let mut link_text = String::new();
                    let mut found_bracket = false;
                    for next in chars.by_ref() {
                        if next == ']' {
                            found_bracket = true;
                            break;
                        }
                        link_text.push(next);
                    }
                    if found_bracket && chars.peek() == Some(&'(') {
                        chars.next(); // consume (
                        let mut _link_url = String::new();
                        for next in chars.by_ref() {
                            if next == ')' {
                                break;
                            }
                            _link_url.push(next);
                        }
                        spans.push(Span::styled(link_text, self.link_style));
                    } else {
                        current.push('[');
                        current.push_str(&link_text);
                        if found_bracket {
                            current.push(']');
                        }
                    }
                }
                _ => {
                    current.push(c);
                }
            }
        }

        if !current.is_empty() {
            spans.push(Span::styled(current, self.base_style));
        }

        if spans.is_empty() {
            spans.push(Span::raw(""));
        }

        spans
    }
}

impl Default for MarkdownRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_header() {
        let renderer = MarkdownRenderer::new();
        let text = renderer.render("# Header 1\n## Header 2");
        assert_eq!(text.lines.len(), 2);
    }

    #[test]
    fn test_render_bold_italic() {
        let renderer = MarkdownRenderer::new();
        let text = renderer.render("**bold** and *italic*");
        assert_eq!(text.lines.len(), 1);
    }

    #[test]
    fn test_render_code() {
        let renderer = MarkdownRenderer::new();
        let text = renderer.render("inline `code` here");
        assert_eq!(text.lines.len(), 1);
    }

    #[test]
    fn test_render_list() {
        let renderer = MarkdownRenderer::new();
        let text = renderer.render("- item 1\n- item 2");
        assert_eq!(text.lines.len(), 2);
    }

    #[test]
    fn test_render_code_block() {
        let renderer = MarkdownRenderer::new();
        let text = renderer.render("```\ncode\nblock\n```");
        assert_eq!(text.lines.len(), 3); // code + block + empty line after
    }
}
