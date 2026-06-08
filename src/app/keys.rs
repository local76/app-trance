//! Keyboard input and focus control handling.
//!
//! **Taxonomy Classification**: Interface (TUI / Keyboard & Focus).

use crate::app::{App, FocusedSection, GlobalField, StatusMessage, StatusKind};
use crate::config::GlobalConfig;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};

impl App {
    /// Adjust the highlight in the saver list, clamping to bounds.
    pub fn move_highlight(&mut self, delta: i32) {
        let indices = self.filtered_indices();
        if indices.is_empty() {
            return;
        }
        let current_pos = indices
            .iter()
            .position(|&i| i == self.highlighted)
            .unwrap_or(0);
        let len = indices.len() as i32;
        let next = (current_pos as i32 + delta).rem_euclid(len);
        self.highlighted = indices[next as usize];
    }

    /// Cycle the focused section.
    pub fn cycle_focus(&mut self) {
        self.focused = match self.focused {
            FocusedSection::GlobalPrefs => FocusedSection::SaverList,
            FocusedSection::SaverList => FocusedSection::GlobalPrefs,
        };
        self.status = Some(StatusMessage {
            text: format!("Focused Section: {}", match self.focused {
                FocusedSection::GlobalPrefs => "Global Preferences",
                FocusedSection::SaverList => "Screensaver List",
            }),
            kind: StatusKind::Info,
        });
    }

    /// Move focus / highlight depending on direction.
    pub fn move_focus(&mut self, delta: i32) {
        match self.focused {
            FocusedSection::GlobalPrefs => {
                let idx = GlobalField::ALL
                    .iter()
                    .position(|f| *f == self.global_field)
                    .unwrap_or(0) as i32;
                let len = GlobalField::ALL.len() as i32;
                let next = (idx + delta).rem_euclid(len);
                self.global_field = GlobalField::ALL[next as usize];
            }
            FocusedSection::SaverList => self.move_highlight(delta),
        }
    }

    /// Handle a single key event. Returns `true` if the app should quit.
    pub fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        // Clear any error status on any user keypress. Info status remains subject to the timer.
        if let Some(ref msg) = self.status {
            if msg.kind == StatusKind::Error {
                self.status = None;
            }
        }

        if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
            return true;
        }

        if self.show_help {
            match code {
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc | KeyCode::Char('h') | KeyCode::Char('H') => {
                    self.show_help = false;
                    self.status = Some(StatusMessage {
                        text: "Help overlay closed.".to_string(),
                        kind: StatusKind::Info,
                    });
                }
                KeyCode::F(1) => {
                    self.show_help = false;
                    self.open_embedded_markdown("README.md", super::README_CONTENT);
                }
                KeyCode::F(2) => {
                    self.show_help = false;
                    self.open_embedded_markdown("SUPPORT.md", super::SUPPORT_CONTENT);
                }
                KeyCode::F(3) => {
                    self.show_help = false;
                    self.open_embedded_markdown("LICENSE.md", super::LICENSE_CONTENT);
                }
                KeyCode::F(4) => {
                    self.show_help = false;
                    self.open_embedded_markdown("COPYRIGHT.md", super::COPYRIGHT_CONTENT);
                }
                KeyCode::F(5) => {
                    self.show_help = false;
                    self.open_embedded_markdown("PRIVACY.md", super::PRIVACY_CONTENT);
                }
                KeyCode::F(6) => {
                    self.show_help = false;
                    self.open_embedded_markdown("SECURITY.md", super::SECURITY_CONTENT);
                }
                KeyCode::F(7) => {
                    self.show_help = false;
                    self.open_embedded_markdown("CONTRIBUTING.md", super::CONTRIBUTING_CONTENT);
                }
                _ => {}
            }
            return false;
        }

        if self.show_markdown.is_some() {
            match code {
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                    self.show_markdown = None;
                }
                KeyCode::F(1) => {
                    self.open_embedded_markdown("README.md", super::README_CONTENT);
                }
                KeyCode::F(2) => {
                    self.open_embedded_markdown("SUPPORT.md", super::SUPPORT_CONTENT);
                }
                KeyCode::F(3) => {
                    self.open_embedded_markdown("LICENSE.md", super::LICENSE_CONTENT);
                }
                KeyCode::F(4) => {
                    self.open_embedded_markdown("COPYRIGHT.md", super::COPYRIGHT_CONTENT);
                }
                KeyCode::F(5) => {
                    self.open_embedded_markdown("PRIVACY.md", super::PRIVACY_CONTENT);
                }
                KeyCode::F(6) => {
                    self.open_embedded_markdown("SECURITY.md", super::SECURITY_CONTENT);
                }
                KeyCode::F(7) => {
                    self.open_embedded_markdown("CONTRIBUTING.md", super::CONTRIBUTING_CONTENT);
                }
                KeyCode::Up => {
                    self.markdown_scroll = self.markdown_scroll.saturating_sub(1);
                }
                KeyCode::Down => {
                    if self.markdown_scroll + 10 < self.markdown_lines.len() {
                        self.markdown_scroll += 1;
                    }
                }
                KeyCode::PageUp => {
                    self.markdown_scroll = self.markdown_scroll.saturating_sub(15);
                }
                KeyCode::PageDown => {
                    if self.markdown_scroll + 15 < self.markdown_lines.len() {
                        self.markdown_scroll += 15;
                    } else {
                        self.markdown_scroll = self.markdown_lines.len().saturating_sub(10);
                    }
                }
                _ => {}
            }
            return false;
        }

        match code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('r') | KeyCode::Char('R') => self.refresh_screensavers(),
            KeyCode::Tab => self.cycle_focus(),
            KeyCode::BackTab => self.cycle_focus(),
            KeyCode::Up => self.move_focus(-1),
            KeyCode::Down => self.move_focus(1),
            KeyCode::Left => self.on_left(),
            KeyCode::Right => self.on_right(),
            KeyCode::Char(' ') => {
                self.on_activate();
            }
            KeyCode::Enter => self.on_activate(),
            KeyCode::Char('p') | KeyCode::Char('P') | KeyCode::Char('t') | KeyCode::Char('T') => {
                self.preview_highlighted()
            }
            KeyCode::Char('c') | KeyCode::Char('C') => self.configure_highlighted(),
            KeyCode::Char('d') | KeyCode::Char('D') => {
                if self.focused == FocusedSection::SaverList {
                    self.delete_highlighted();
                }
            }
            KeyCode::F(1) => {
                self.open_embedded_markdown("README.md", super::README_CONTENT);
            }
            KeyCode::F(2) => {
                self.open_embedded_markdown("SUPPORT.md", super::SUPPORT_CONTENT);
            }
            KeyCode::F(3) => {
                self.open_embedded_markdown("LICENSE.md", super::LICENSE_CONTENT);
            }
            KeyCode::F(4) => {
                self.open_embedded_markdown("COPYRIGHT.md", super::COPYRIGHT_CONTENT);
            }
            KeyCode::F(5) => {
                self.open_embedded_markdown("PRIVACY.md", super::PRIVACY_CONTENT);
            }
            KeyCode::F(6) => {
                self.open_embedded_markdown("SECURITY.md", super::SECURITY_CONTENT);
            }
            KeyCode::F(7) => {
                self.open_embedded_markdown("CONTRIBUTING.md", super::CONTRIBUTING_CONTENT);
            }
            KeyCode::Char('h') | KeyCode::Char('H') => {
                self.show_help = true;
                self.status = Some(StatusMessage {
                    text: "Help overlay active. Press ESC/q to close.".to_string(),
                    kind: StatusKind::Info,
                });
            }
            _ => {}
        }
        self.should_quit
    }

    fn on_left(&mut self) {
        if self.focused == FocusedSection::GlobalPrefs {
            match self.global_field {
                GlobalField::Timeout => self.adjust_timeout(-1),
                GlobalField::CycleTime => self.adjust_cycle_time(-1),
                _ => {}
            }
        }
    }

    fn on_right(&mut self) {
        if self.focused == FocusedSection::GlobalPrefs {
            match self.global_field {
                GlobalField::Timeout => self.adjust_timeout(1),
                GlobalField::CycleTime => self.adjust_cycle_time(1),
                _ => {}
            }
        }
    }

    fn on_activate(&mut self) {
        match self.focused {
            FocusedSection::GlobalPrefs => match self.global_field {
                GlobalField::Active => self.toggle_active(),
                GlobalField::PreventSleep => self.toggle_prevent_sleep(),
                GlobalField::HideStock => self.toggle_hide_stock(),
                GlobalField::Timeout | GlobalField::CycleTime => {}
            },
            FocusedSection::SaverList => self.toggle_and_apply_highlighted(),
        }
    }

    /// Check if the registry matches the global config, reload if out of sync.
    pub fn check_registry_sync(&mut self) -> bool {
        let current_reg = GlobalConfig::load();
        if current_reg.active_scr != self.global.active_scr
            || current_reg.active != self.global.active
            || current_reg.timeout != self.global.timeout
        {
            self.global = current_reg;
            self.status = Some(StatusMessage {
                text: "External registry change detected! Config reloaded.".to_string(),
                kind: StatusKind::Info,
            });
            self.update_list_items();
            true
        } else {
            false
        }
    }
}
