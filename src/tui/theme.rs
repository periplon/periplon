//! Theme and styling for TUI
//!
//! Defines color schemes and styling constants for consistent UI appearance.

use ratatui::style::{Color, Modifier, Style};

/// Application theme
#[derive(Debug, Clone)]
pub struct Theme {
    /// Primary color
    pub primary: Color,

    /// Secondary color
    pub secondary: Color,

    /// Accent color
    pub accent: Color,

    /// Background color
    pub bg: Color,

    /// Foreground/text color
    pub fg: Color,

    /// Success color
    pub success: Color,

    /// Warning color
    pub warning: Color,

    /// Error color
    pub error: Color,

    /// Muted/dim color
    pub muted: Color,

    /// Border color
    pub border: Color,

    /// Selected/highlight color
    pub highlight: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary: Color::Cyan,
            secondary: Color::Blue,
            accent: Color::Magenta,
            bg: Color::Black,
            fg: Color::White,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            muted: Color::DarkGray,
            border: Color::Gray,
            highlight: Color::LightCyan,
        }
    }
}

impl Theme {
    /// Create light theme
    pub fn light() -> Self {
        Self {
            primary: Color::Blue,
            secondary: Color::DarkGray,
            accent: Color::Magenta,
            bg: Color::White,
            fg: Color::Black,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            muted: Color::Gray,
            border: Color::DarkGray,
            highlight: Color::LightBlue,
        }
    }

    /// Create monokai theme
    pub fn monokai() -> Self {
        Self {
            primary: Color::Rgb(102, 217, 239),  // cyan
            secondary: Color::Rgb(249, 38, 114), // pink
            accent: Color::Rgb(166, 226, 46),    // green
            bg: Color::Rgb(39, 40, 34),          // dark bg
            fg: Color::Rgb(248, 248, 242),       // light fg
            success: Color::Rgb(166, 226, 46),   // green
            warning: Color::Rgb(230, 219, 116),  // yellow
            error: Color::Rgb(249, 38, 114),     // pink
            muted: Color::Rgb(117, 113, 94),     // gray
            border: Color::Rgb(73, 72, 62),      // border
            highlight: Color::Rgb(73, 72, 62),   // selection
        }
    }

    /// Create solarized dark theme
    pub fn solarized() -> Self {
        Self {
            primary: Color::Rgb(38, 139, 210),   // blue
            secondary: Color::Rgb(42, 161, 152), // cyan
            accent: Color::Rgb(211, 54, 130),    // magenta
            bg: Color::Rgb(0, 43, 54),           // base03
            fg: Color::Rgb(131, 148, 150),       // base0
            success: Color::Rgb(133, 153, 0),    // green
            warning: Color::Rgb(181, 137, 0),    // yellow
            error: Color::Rgb(220, 50, 47),      // red
            muted: Color::Rgb(88, 110, 117),     // base01
            border: Color::Rgb(7, 54, 66),       // base02
            highlight: Color::Rgb(7, 54, 66),    // base02
        }
    }

    /// Get primary style
    pub fn primary(&self) -> Style {
        Style::default().fg(self.primary)
    }

    /// Get secondary style
    pub fn secondary(&self) -> Style {
        Style::default().fg(self.secondary)
    }

    /// Get accent style
    pub fn accent(&self) -> Style {
        Style::default().fg(self.accent)
    }

    /// Get success style
    pub fn success(&self) -> Style {
        Style::default().fg(self.success)
    }

    /// Get warning style
    pub fn warning(&self) -> Style {
        Style::default().fg(self.warning)
    }

    /// Get error style
    pub fn error(&self) -> Style {
        Style::default().fg(self.error)
    }

    /// Get muted style
    pub fn muted(&self) -> Style {
        Style::default().fg(self.muted)
    }

    /// Get border style
    pub fn border(&self) -> Style {
        Style::default().fg(self.border)
    }

    /// Get highlight style (for selections)
    pub fn highlight(&self) -> Style {
        Style::default()
            .fg(self.bg)
            .bg(self.highlight)
            .add_modifier(Modifier::BOLD)
    }

    /// Get title style
    pub fn title(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }

    /// Get subtitle style
    pub fn subtitle(&self) -> Style {
        Style::default()
            .fg(self.secondary)
            .add_modifier(Modifier::ITALIC)
    }

    /// Get bold style
    pub fn bold(&self) -> Style {
        Style::default().fg(self.fg).add_modifier(Modifier::BOLD)
    }

    /// Get dim style
    pub fn dim(&self) -> Style {
        Style::default().fg(self.muted).add_modifier(Modifier::DIM)
    }

    /// Get normal text style
    pub fn normal(&self) -> Style {
        Style::default().fg(self.fg).bg(self.bg)
    }

    /// Get modal background style
    pub fn modal_bg(&self) -> Style {
        Style::default().fg(self.fg).bg(Color::DarkGray)
    }

    /// Get modal border style
    pub fn modal_border(&self) -> Style {
        Style::default().fg(self.primary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme() {
        let theme = Theme::default();
        assert_eq!(theme.primary, Color::Cyan);
        assert_eq!(theme.bg, Color::Black);
        assert_eq!(theme.fg, Color::White);
        assert_eq!(theme.success, Color::Green);
        assert_eq!(theme.error, Color::Red);
    }

    #[test]
    fn test_light_theme() {
        let theme = Theme::light();
        assert_eq!(theme.primary, Color::Blue);
        assert_eq!(theme.bg, Color::White);
        assert_eq!(theme.fg, Color::Black);
    }

    #[test]
    fn test_monokai_theme() {
        let theme = Theme::monokai();
        assert_eq!(theme.primary, Color::Rgb(102, 217, 239));
        assert_eq!(theme.bg, Color::Rgb(39, 40, 34));
        assert_eq!(theme.fg, Color::Rgb(248, 248, 242));
    }

    #[test]
    fn test_solarized_theme() {
        let theme = Theme::solarized();
        assert_eq!(theme.primary, Color::Rgb(38, 139, 210));
        assert_eq!(theme.bg, Color::Rgb(0, 43, 54));
    }

    #[test]
    fn test_style_methods() {
        let theme = Theme::default();

        let primary_style = theme.primary();
        assert!(matches!(primary_style.fg, Some(Color::Cyan)));

        let success_style = theme.success();
        assert!(matches!(success_style.fg, Some(Color::Green)));

        let error_style = theme.error();
        assert!(matches!(error_style.fg, Some(Color::Red)));
    }

    #[test]
    fn test_highlight_style() {
        let theme = Theme::default();
        let highlight = theme.highlight();

        assert_eq!(highlight.fg, Some(theme.bg));
        assert_eq!(highlight.bg, Some(theme.highlight));
    }

    #[test]
    fn test_title_style_has_bold() {
        let theme = Theme::default();
        let title = theme.title();

        assert_eq!(title.fg, Some(theme.primary));
        assert!(title.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn test_subtitle_style_has_italic() {
        let theme = Theme::default();
        let subtitle = theme.subtitle();

        assert_eq!(subtitle.fg, Some(theme.secondary));
        assert!(subtitle.add_modifier.contains(Modifier::ITALIC));
    }

    #[test]
    fn test_normal_style() {
        let theme = Theme::default();
        let normal = theme.normal();

        assert_eq!(normal.fg, Some(theme.fg));
        assert_eq!(normal.bg, Some(theme.bg));
    }
}
