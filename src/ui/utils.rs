//! Helper styling and formatting utilities for UI modules.
//!
//! **Taxonomy Classification**: Interface (TUI / UI Utilities).

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use crate::theme::TuiTheme;

pub fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

pub fn format_help_row(
    key: &str,
    description: &str,
    max_desc_width: usize,
    theme: &TuiTheme,
) -> Vec<Line<'static>> {
    let wrapped = wrap_text(description, max_desc_width);
    let mut lines = Vec::new();

    let key_col_width = 18;
    let key_str = format!("  {:<15} ", key);

    if wrapped.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(
                key_str,
                Style::default()
                    .fg(theme.accent_primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(": ", Style::default().fg(theme.text_main)),
        ]));
    } else {
        for (i, chunk) in wrapped.into_iter().enumerate() {
            if i == 0 {
                lines.push(Line::from(vec![
                    Span::styled(
                        key_str.clone(),
                        Style::default()
                            .fg(theme.accent_primary)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(": ", Style::default().fg(theme.text_main)),
                    Span::styled(chunk, Style::default().fg(theme.text_main)),
                ]));
            } else {
                let padding = " ".repeat(key_col_width + 2);
                lines.push(Line::from(vec![
                    Span::styled(padding, Style::default().fg(theme.text_main)),
                    Span::styled(chunk, Style::default().fg(theme.text_main)),
                ]));
            }
        }
    }
    lines
}

pub fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    if max_width == 0 {
        return vec![text.to_string()];
    }
    for paragraph in text.split('\n') {
        let mut current_line = String::new();
        for word in paragraph.split_whitespace() {
            if current_line.is_empty() {
                current_line.push_str(word);
            } else if current_line.len() + 1 + word.len() <= max_width {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }
    }
    lines
}
