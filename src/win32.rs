//! Win32 platform integration re-exports.
//!
//! **Taxonomy Classification**: Platform (OS / Hardware Layer).

#![allow(unused_imports)]

pub use crate::clipboard::copy_text_to_clipboard;
pub use crate::backend::event_log::log_system_event as log_windows_event;
pub use crate::backend::notification::show_toast_notification;
pub use crate::backend::sys_info::{query_os_version, GlyphMap};
pub use crate::backend::window::{
    query_cursor_pos, get_window_rect, set_window_pos,
};
pub use crate::bootstrap_guards::{BorderlessConsole, SingleInstanceGuard};
pub use crate::win32_relaunch::{relaunch_in_conhost, should_relaunch_in_conhost};
pub use crate::backend::saver_win32::query_power_status;
pub use crate::backend::saver_win32::PowerStatus;
pub use crate::backend::saver_win32::RECT;
pub use crate::backend::saver_win32::*;

#[cfg(target_os = "windows")]
#[allow(dead_code)]
pub fn spawn_linux_screensaver(_path: &std::path::Path, _arg: &str) -> std::io::Result<std::process::Child> {
    Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "spawn_linux_screensaver is not supported on Windows"))
}
