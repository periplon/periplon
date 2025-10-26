//! Test fixtures and data builders
//!
//! Provides reusable test data, fixtures, and builder patterns for TUI tests.

use periplon_sdk::tui::theme::Theme;

/// Create a test theme (default dark theme)
pub fn create_test_theme() -> Theme {
    Theme::default()
}

/// Create all available themes for theme testing
///
/// Returns a vector containing all theme variants:
/// - Default (dark)
/// - Light
/// - Monokai
/// - Solarized
pub fn create_all_themes() -> Vec<Theme> {
    vec![
        Theme::default(),
        Theme::light(),
        Theme::monokai(),
        Theme::solarized(),
    ]
}

/// Get theme by name for parameterized testing
pub fn theme_by_name(name: &str) -> Theme {
    match name {
        "default" => Theme::default(),
        "light" => Theme::light(),
        "monokai" => Theme::monokai(),
        "solarized" => Theme::solarized(),
        _ => panic!("Unknown theme: {}", name),
    }
}

/// Get all theme names
pub fn theme_names() -> Vec<&'static str> {
    vec!["default", "light", "monokai", "solarized"]
}

/// Common terminal sizes for testing
pub mod terminal_sizes {
    /// Minimum supported terminal size
    pub const MIN: (u16, u16) = (80, 24);

    /// Standard terminal size
    pub const STANDARD: (u16, u16) = (120, 40);

    /// Large terminal size
    pub const LARGE: (u16, u16) = (200, 100);

    /// Small terminal (edge case)
    pub const SMALL: (u16, u16) = (40, 12);

    /// Narrow terminal
    pub const NARROW: (u16, u16) = (60, 24);

    /// Tall terminal
    pub const TALL: (u16, u16) = (80, 60);

    /// Get all standard sizes for iteration
    pub fn all() -> Vec<(u16, u16)> {
        vec![MIN, STANDARD, LARGE, SMALL, NARROW, TALL]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_theme() {
        let theme = create_test_theme();
        // Default theme should have dark background
        assert!(theme.bg != theme.fg);
    }

    #[test]
    fn test_create_all_themes() {
        let themes = create_all_themes();
        assert_eq!(themes.len(), 4);
    }

    #[test]
    fn test_theme_by_name() {
        let default_theme = theme_by_name("default");
        let light_theme = theme_by_name("light");

        // Themes should be different
        assert!(default_theme.bg != light_theme.bg);
    }

    #[test]
    fn test_theme_names() {
        let names = theme_names();
        assert_eq!(names.len(), 4);
        assert!(names.contains(&"default"));
        assert!(names.contains(&"light"));
        assert!(names.contains(&"monokai"));
        assert!(names.contains(&"solarized"));
    }

    #[test]
    fn test_terminal_sizes() {
        let sizes = terminal_sizes::all();
        assert_eq!(sizes.len(), 6);
        assert!(sizes.contains(&terminal_sizes::MIN));
        assert!(sizes.contains(&terminal_sizes::STANDARD));
    }
}
