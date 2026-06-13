//! Theme coloring utility and factory for ratatui-based TUIs.

use ratatui::style::Color;

/// Theme color definitions for styling console panels and text.
#[derive(Debug, Clone, Copy)]
pub struct ThemeColors {
    pub border: Color,
    pub border_active: Color,
    pub text_main: Color,
    pub text_dim: Color,
    pub accent: Color,
    pub username: Color,
    pub help_btn: Color,
    pub quit_btn: Color,
    pub warning: Color,
    pub success: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
}
