//! Win32 platform integration re-exports.
//!
//! **Taxonomy Classification**: Platform (OS / Hardware Layer).

#![allow(unused_imports)]

pub use rcommon::clipboard::copy_text_to_clipboard;
pub use rcommon::event_log::log_system_event as log_windows_event;
pub use rcommon::notification::show_toast_notification;
pub use rcommon::sys_info::{query_os_version, GlyphMap};
pub use rcommon::window::{
    center_console_window, query_cursor_pos, get_window_rect, set_window_pos,
    BorderlessConsole, SingleInstanceGuard,
};
pub use crate::saver_win32::query_power_status;
pub use crate::saver_win32::PowerStatus;
pub use crate::saver_win32::RECT;
pub use crate::saver_win32::*;
