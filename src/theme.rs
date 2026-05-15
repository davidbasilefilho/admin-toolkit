//! Semantic theme system for the TUI.
//!
//! Provides a centralized `Theme` struct with all styling constants
//! organized by screen role. All color values are defined here and
//! referenced by `ui.rs` for consistent dark high-contrast rendering.

use ratatui::style::Color;

/// Semantic theme struct with all TUI styling constants.
///
/// Fields are grouped by visual role:
/// - `fg_*`: Foreground text colors
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

    /// Outer frame border color (thick border around entire screen).
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
            fg_main: Color::White,
            fg_dim: Color::DarkGray,
            border_frame: Color::White,
            border_panel: Color::DarkGray,
            sep: Color::DarkGray,
            invert_bg: Color::White,
            invert_fg: Color::Black,
            accent: Color::Cyan,
            status_success: Color::Green,
            status_warning: Color::Yellow,
            status_error: Color::Red,
        }
    }
}

impl Theme {
    /// Dark high-contrast palette optimized for terminal rendering.
    ///
    /// This palette uses:
    /// - Dark background (terminal default) for maximum compatibility
    /// - High-contrast white foreground for readability
    /// - Dark gray borders for subtle panel definition
    /// - Cyan accent for interactive elements
    /// - Semantic status colors (green/yellow/red) for feedback
    ///
    /// All colors use 16-color ANSI palette for maximum terminal compatibility.
    pub fn dark_high_contrast() -> Self {
        Self {
            fg_main: Color::White,
            fg_dim: Color::DarkGray,
            border_frame: Color::White,
            border_panel: Color::Gray,
            sep: Color::Gray,
            invert_bg: Color::White,
            invert_fg: Color::Black,
            accent: Color::Cyan,
            status_success: Color::Green,
            status_warning: Color::Yellow,
            status_error: Color::Red,
        }
    }
}
