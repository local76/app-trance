use crate::app::{App, StatusMessage, StatusKind};
use crate::backend::preview::{self, Screensaver};

impl App {
    /// Indices into `self.screensavers` that match the current filter.
    pub fn filtered_indices(&self) -> Vec<usize> {
        let indices: Vec<usize> = (0..self.screensavers.len()).collect();
        if self.local.hide_stock {
            indices
                .into_iter()
                .filter(|&i| !preview::is_stock_screensaver(&self.screensavers[i].path))
                .collect()
        } else {
            indices
        }
    }

    /// Map a position in the filtered list to the real index, clamping.
    pub fn resolve_highlight(&mut self) {
        let indices = self.filtered_indices();
        if indices.is_empty() {
            self.highlighted = 0;
            return;
        }
        if let Some(pos) = indices.iter().position(|&i| i == self.highlighted) {
            self.highlighted = indices[pos];
        } else {
            self.highlighted = indices[0];
        }
    }

    /// Update the cached ListItem widgets in `self.list_items`.
    pub fn update_list_items(&mut self) {
        let theme = self.theme;
        let active_scr_path = self.global.active_scr.clone();
        let is_global_active = self.global.active;
        self.list_items = self
            .screensavers
            .iter()
            .map(|s| {
                let s_path_str = s.path.to_string_lossy().into_owned();
                let is_checked = is_global_active && active_scr_path == s_path_str;
                let is_stock = preview::is_stock_screensaver(&s.path);

                let active_str = if is_checked { "yes" } else { "no" };
                let active_color = if is_checked { theme.applied } else { theme.text_dim };

                let name = crate::ui::truncate(&s.name, 28);
                let name_str = format!("{:<30}  ", name);
                let name_color = if is_checked {
                    theme.text_main
                } else {
                    theme.text_dim
                };

                let type_str = if is_stock {
                    "Stock"
                } else {
                    "Custom"
                };
                let type_color = if is_stock {
                    theme.text_dim
                } else {
                    theme.accent_secondary
                };

                let spans = vec![
                    ratatui::text::Span::styled(
                        format!("{:<8}  ", active_str),
                        ratatui::style::Style::default().fg(active_color),
                    ),
                    ratatui::text::Span::styled(
                        name_str,
                        ratatui::style::Style::default().fg(name_color),
                    ),
                    ratatui::text::Span::styled(
                        type_str.to_string(),
                        ratatui::style::Style::default().fg(type_color),
                    ),
                ];
                ratatui::widgets::ListItem::new(ratatui::text::Line::from(spans))
            })
            .collect();
    }

    /// Return the currently highlighted screensaver object.
    pub fn current_screensaver(&self) -> Option<&Screensaver> {
        self.screensavers.get(self.highlighted)
    }

    /// Load and open an embedded markdown document in the viewer modal.
    pub fn open_embedded_markdown(&mut self, title: &str, content: &str) {
        self.markdown_lines = super::markdown::parse_markdown_to_lines(content, &self.theme);
        self.show_markdown = Some(title.to_string());
        self.markdown_scroll = 0;
        self.status = Some(StatusMessage {
            text: format!("Opened document: {}", title),
            kind: StatusKind::Info,
        });
    }

    /// Checks system power status periodically and adjusts throttling state.
    pub fn sync_power_status_if_needed(&mut self) {
        if self.last_power_check.elapsed() > std::time::Duration::from_millis(5000) {
            self.last_power_check = std::time::Instant::now();
            let power = crate::win32::query_power_status();
            let current_on_battery = !power.ac_online;
            if current_on_battery != self.on_battery {
                self.on_battery = current_on_battery;
                let state = if current_on_battery {
                    "Battery (Power-Saving Throttling Enabled)"
                } else {
                    "AC Power (Full Speed)"
                };
                info!("Power source changed. Status: {}", state);
                self.status = Some(StatusMessage {
                    text: format!("Power Source Changed: {}", state),
                    kind: StatusKind::Info,
                });
            }
        }
    }
}
