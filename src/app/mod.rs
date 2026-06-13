//! Application state, focus, and key bindings.
//!
//! **Taxonomy Classification**: Interface (State Coordination).

use crate::config::{GlobalConfig, LocalConfig};
use crate::backend::preview::Screensaver;
use crate::theme::TuiTheme;

pub mod actions;
pub mod markdown;
pub mod keys;
pub mod helpers;
#[cfg(test)]
pub mod tests;

pub use crossterm::event::{KeyCode, KeyModifiers};

const README_CONTENT: &str = include_str!("../../README.md");
const SUPPORT_CONTENT: &str = include_str!("../../SUPPORT.md");
const LICENSE_CONTENT: &str = include_str!("../../LICENSE.md");
const COPYRIGHT_CONTENT: &str = include_str!("../../COPYRIGHT.md");
const PRIVACY_CONTENT: &str = include_str!("../../PRIVACY.md");
const SECURITY_CONTENT: &str = include_str!("../../SECURITY.md");
const CONTRIBUTING_CONTENT: &str = include_str!("../../CONTRIBUTING.md");

/// Focused section in the console dashboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedSection {
    /// Global preferences config pane.
    GlobalPrefs,
    /// Screensaver list selection.
    SaverList,
}

/// Dynamic global config fields in the console dashboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalField {
    /// Active screensaver enabled/disabled state.
    Active,
    /// Timeout length of the screensaver.
    Timeout,
    /// Prevent system sleep state.
    PreventSleep,
    /// Hide stock Windows screensavers.
    HideStock,
}

impl GlobalField {
    /// Helper to cycle focus across preferences.
    pub const ALL: &'static [GlobalField] = &[
        GlobalField::Active,
        GlobalField::Timeout,
        GlobalField::PreventSleep,
        GlobalField::HideStock,
    ];
}

/// Status message type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    /// Normal information status.
    Info,
    /// Error status.
    Error,
}

/// Status message displayed on the console status bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusMessage {
    /// Status message text.
    pub text: String,
    /// Semantic type of the status.
    pub kind: StatusKind,
}

/// Main application state struct.
pub struct App {
    /// Discovered/registered screensavers.
    pub screensavers: Vec<Screensaver>,
    /// Highlighted list index.
    pub highlighted: usize,
    /// Current focused panel.
    pub focused: FocusedSection,
    /// Current focused preference field.
    pub global_field: GlobalField,
    /// Global screensaver registry config.
    pub global: GlobalConfig,
    /// Local user config.
    pub local: LocalConfig,
    /// console theme colors.
    pub theme: TuiTheme,
    /// Status message state.
    pub status: Option<StatusMessage>,
    /// Quit flag signaling render loop exit.
    pub should_quit: bool,
    /// List display offset.
    pub list_offset: usize,
    /// Cached list items for rendering the screensavers list.
    pub list_items: Vec<ratatui::widgets::ListItem<'static>>,
    /// Help overlay visibility.
    pub show_help: bool,
    /// Selection column/row start bounds.
    pub selection_start: Option<(u16, u16)>,
    /// Selection column/row end bounds.
    pub selection_end: Option<(u16, u16)>,
    /// Selection copy-to-clipboard trigger.
    pub selection_pending_copy: bool,
    /// Opened markdown document name.
    pub show_markdown: Option<String>,
    /// Rendered lines of the markdown document.
    pub markdown_lines: Vec<ratatui::text::Line<'static>>,
    /// Scroll offset of the markdown document.
    pub markdown_scroll: usize,
    /// Loaded terminal character fallbacks (Adaptive Emoji/Glyph fallback)
    pub glyphs: crate::win32::GlyphMap,
    /// Whether the computer is currently running on battery power (Throttling)
    pub on_battery: bool,
    /// Last Instant the power/battery status was queried
    pub last_power_check: std::time::Instant,
    /// Shutdown button screen bounds.
    pub quit_btn_bounds: Option<(u16, u16, u16)>,
    /// Help button screen bounds.
    pub help_btn_bounds: Option<(u16, u16, u16)>,
    /// Custom console window dragging state.
    pub drag_active: bool,
    /// Cursor coordinates on drag start.
    pub drag_start_cursor: Option<(i32, i32)>,
    /// Console window coordinates on drag start.
    pub drag_start_window: Option<(i32, i32)>,
    pub username: String,
    pub hostname: String,
    pub os_version: String,
}

impl App {
    /// Create a new App state.
    pub fn new(
        screensavers: Vec<Screensaver>,
        global: GlobalConfig,
        local: LocalConfig,
        theme: TuiTheme,
    ) -> Self {
        let highlighted = local
            .last_selected
            .as_deref()
            .and_then(|name| {
                screensavers
                    .iter()
                    .position(|s| s.path.file_name().and_then(|f| f.to_str()) == Some(name))
            })
            .unwrap_or(0)
            .min(screensavers.len().saturating_sub(1));

        let mut app = App {
            screensavers,
            highlighted,
            focused: FocusedSection::GlobalPrefs,
            global_field: GlobalField::Active,
            global,
            local,
            theme,
            status: None,
            should_quit: false,
            list_offset: 0,
            list_items: Vec::new(),
            selection_start: None,
            selection_end: None,
            selection_pending_copy: false,
            show_help: false,
            show_markdown: None,
            markdown_lines: Vec::new(),
            markdown_scroll: 0,
            glyphs: crate::win32::GlyphMap::load(),
            on_battery: !crate::win32::query_power_status().ac_online,
            last_power_check: std::time::Instant::now(),
            quit_btn_bounds: None,
            help_btn_bounds: None,
            drag_active: false,
            drag_start_cursor: None,
            drag_start_window: None,
            username: crate::backend::identity::username(),
            hostname: crate::backend::identity::hostname(),
            os_version: crate::win32::query_os_version(),
        };
        app.update_list_items();
        app
    }
}
