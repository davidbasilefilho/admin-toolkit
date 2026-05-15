//! Semantic theme system for the TUI.
//!
//! Provides a centralized `Theme` struct with all styling constants
//! organized by screen role. All color values are defined here and
//! referenced by `ui.rs` for consistent dark cohesive rendering.

use ratatui::style::Color;

/// Semantic theme struct with all TUI styling constants.
///
/// Fields are grouped by visual role:
/// - `fg_*`: Foreground text colors
/// - `bg`: Background fill for panels
/// - `border_*`: Border and frame colors
/// - `invert_*`: Focus/highlight colors
/// - `accent`: Interactive element color
/// - `status_*`: Semantic status colors (success, warning, error)
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    /// Primary foreground color for main text content.
    pub fg_main: Color,

    /// Dim foreground color for secondary text (labels, subtitles, footers).
    pub fg_dim: Color,

    /// Background fill for interior of panels.
    pub bg: Color,

    /// Outer frame border color (thick border around entire screen).
    #[allow(dead_code)]
    pub border_frame: Color,

    /// Panel border color (thick borders around content panels).
    pub border_panel: Color,

    /// Separator line color (horizontal dividers between sections).
    pub sep: Color,

    /// Inverted background color for focused/highlighted rows.
    pub invert_bg: Color,

    /// Inverted foreground color for focused/highlighted rows.
    pub invert_fg: Color,

    /// Accent color for column headers, target values, interactive elements.
    pub accent: Color,

    /// Success color for enabled checkmarks, elevated status indicators.
    pub status_success: Color,

    /// Warning color for status messages, caution indicators.
    pub status_warning: Color,

    /// Error color for blocked state, disabled indicators, errors.
    pub status_error: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            fg_main: Color::Rgb(0xC0, 0xCA, 0xF5),
            fg_dim: Color::Rgb(0x56, 0x5F, 0x89),
            bg: Color::Rgb(0x1E, 0x22, 0x33),
            border_frame: Color::Rgb(0x7A, 0xA2, 0xF7),
            border_panel: Color::Rgb(0x33, 0x3A, 0x50),
            sep: Color::Rgb(0x33, 0x3A, 0x50),
            invert_bg: Color::Rgb(0x7A, 0xA2, 0xF7),
            invert_fg: Color::Rgb(0x1E, 0x22, 0x33),
            accent: Color::Rgb(0x7D, 0xCF, 0xFF),
            status_success: Color::Rgb(0x9E, 0xCE, 0x6A),
            status_warning: Color::Rgb(0xE0, 0xAF, 0x68),
            status_error: Color::Rgb(0xF7, 0x76, 0x8E),
        }
    }
}

impl Theme {
    /// Dark cohesive palette optimized for terminal rendering.
    ///
    /// This palette uses:
    /// - Navy-slate background for deep, comfortable contrast
    /// - Periwinkle blue accent for interactive/structural elements
    /// - Cyan bright accent for emphasis
    /// - Semantic status colors with consistent saturation
    ///
    /// All colors use TrueColor (24-bit RGB) for rich terminal rendering.
    pub fn cohesive_dark() -> Self {
        Self::default()
    }
}
