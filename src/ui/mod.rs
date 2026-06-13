//! Ratatui-based rendering. Pure function of `App` -> `Frame`.
//!
//! **Taxonomy Classification**: Interface (Main Rendering Layout).

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;

pub mod colors;
pub mod layout_guard;
pub mod layout_helpers;
pub mod markdown;
pub mod scrollbar;
pub mod text;
pub mod text_format;
pub mod textbox;
pub mod theme;
pub mod title_banner;
pub mod widgets;
pub mod overlays;
pub mod help_modal;

pub use text_format::truncate;

/// Render the entire application interface to the Ratatui Frame.
pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    let theme = app.theme;
    let min_w = 100;
    let min_h = 35;

    if area.width < min_w || area.height < min_h {
        overlays::render_too_small(theme, frame, area);
        return;
    }

    // Split entire area vertically into bordered boxes
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // 0: Header box
            Constraint::Length(7), // 1: Global Screensaver Preferences (full width)
            Constraint::Min(10),   // 2: Screensaver Preferences list
            Constraint::Length(3), // 3: Status / Progress footer box
        ])
        .split(area);

    // 0. Render Header Info Box
    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(Span::styled(
            " trance - Screensaver Manager ",
            Style::default().fg(theme.accent_primary).add_modifier(Modifier::BOLD),
        ));
    
    let username = &app.username;
    let hostname = &app.hostname;
    let os_str_val = app.os_version.clone();

    let ver_str = format!("trance v{}", env!("CARGO_PKG_VERSION"));
    let user_host_str = format!("{}@{}", username, hostname);

    let button_y = chunks[0].y + 1;
    let inner_width = chunks[0].width.saturating_sub(2) as usize;
    
    let left_len = ver_str.len() + 3 + user_host_str.len() + 3 + os_str_val.len();
    let right_len = 6 + 3 + 6; // " help " + " │ " + " quit "

    let header_line = if inner_width > left_len + right_len {
        let padding_len = inner_width - (left_len + right_len);
        let padding_str = " ".repeat(padding_len);
        
        let help_offset = 1 + left_len + padding_len;
        let help_start_x = chunks[0].x + help_offset as u16;
        let help_end_x = help_start_x + 6;
        app.help_btn_bounds = Some((button_y, help_start_x, help_end_x));

        let quit_offset = help_offset + 6 + 3;
        let quit_start_x = chunks[0].x + quit_offset as u16;
        let quit_end_x = quit_start_x + 6;
        app.quit_btn_bounds = Some((button_y, quit_start_x, quit_end_x));

        Line::from(vec![
            Span::styled(ver_str, Style::default().fg(theme.accent_primary).add_modifier(Modifier::BOLD)),
            Span::styled(" │ ", Style::default().fg(theme.border)),
            Span::styled(user_host_str, Style::default().fg(Color::Rgb(255, 215, 0)).add_modifier(Modifier::BOLD)),
            Span::styled(" │ ", Style::default().fg(theme.border)),
            Span::styled(os_str_val, Style::default().fg(theme.accent_primary).add_modifier(Modifier::BOLD)),
            Span::styled(padding_str, Style::default()),
            Span::styled(" ", Style::default().bg(Color::Rgb(250, 210, 50)).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled("h", Style::default().bg(Color::Rgb(250, 210, 50)).fg(Color::Black).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
            Span::styled("elp ", Style::default().bg(Color::Rgb(250, 210, 50)).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(" │ ", Style::default().fg(theme.border)),
            Span::styled(" ", Style::default().bg(Color::Rgb(255, 85, 85)).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("q", Style::default().bg(Color::Rgb(255, 85, 85)).fg(Color::White).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
            Span::styled("uit ", Style::default().bg(Color::Rgb(255, 85, 85)).fg(Color::White).add_modifier(Modifier::BOLD)),
        ])
    } else {
        let help_offset = 1 + ver_str.len() + 3 + user_host_str.len() + 3 + os_str_val.len() + 3;
        let help_start_x = chunks[0].x + help_offset as u16;
        let help_end_x = help_start_x + 6;
        app.help_btn_bounds = Some((button_y, help_start_x, help_end_x));

        let quit_offset = help_offset + 6 + 3;
        let quit_start_x = chunks[0].x + quit_offset as u16;
        let quit_end_x = quit_start_x + 6;
        app.quit_btn_bounds = Some((button_y, quit_start_x, quit_end_x));

        Line::from(vec![
            Span::styled(ver_str, Style::default().fg(theme.accent_primary).add_modifier(Modifier::BOLD)),
            Span::styled(" │ ", Style::default().fg(theme.border)),
            Span::styled(user_host_str, Style::default().fg(Color::Rgb(255, 215, 0)).add_modifier(Modifier::BOLD)),
            Span::styled(" │ ", Style::default().fg(theme.border)),
            Span::styled(os_str_val, Style::default().fg(theme.accent_primary).add_modifier(Modifier::BOLD)),
            Span::styled(" │ ", Style::default().fg(theme.border)),
            Span::styled(" ", Style::default().bg(Color::Rgb(250, 210, 50)).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled("h", Style::default().bg(Color::Rgb(250, 210, 50)).fg(Color::Black).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
            Span::styled("elp ", Style::default().bg(Color::Rgb(250, 210, 50)).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(" │ ", Style::default().fg(theme.border)),
            Span::styled(" ", Style::default().bg(Color::Rgb(255, 85, 85)).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("q", Style::default().bg(Color::Rgb(255, 85, 85)).fg(Color::White).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
            Span::styled("uit ", Style::default().bg(Color::Rgb(255, 85, 85)).fg(Color::White).add_modifier(Modifier::BOLD)),
        ])
    };
    let header_inner = header_block.inner(chunks[0]);
    frame.render_widget(header_block, chunks[0]);
    frame.render_widget(Paragraph::new(header_line), header_inner);

    // 1. Render Global Screensaver Preferences (full width)
    widgets::render_prefs(app, frame, chunks[1]);

    // 2. Render Screensaver Preferences List Table
    widgets::render_list(app, frame, chunks[2]);

    // 3. Render Footer Status Box
    let footer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(Span::styled(
            " Status ",
            Style::default().fg(theme.accent_primary).add_modifier(Modifier::BOLD),
        ));

    let footer_p = if let Some(ref status) = app.status {
        let (color, icon) = match status.kind {
            crate::app::StatusKind::Info => (theme.accent_primary, app.glyphs.info),
            crate::app::StatusKind::Error => (theme.missing, app.glyphs.status_err),
        };
        Paragraph::new(Line::from(vec![
            Span::styled(format!("{} ", icon), Style::default().fg(color)),
            Span::styled(&status.text, Style::default().fg(color).add_modifier(Modifier::BOLD)),
        ]))
    } else {
        Paragraph::new(Line::from(vec![
            Span::styled("Ready. Press Tab to cycle focus.", Style::default().fg(theme.text_dim)),
        ]))
    };

    let footer_inner = footer_block.inner(chunks[3]);
    frame.render_widget(footer_block, chunks[3]);
    frame.render_widget(footer_p, footer_inner);

    // Handle Mouse Selection Highlights & Clipboard Copy
    overlays::handle_selection_highlights(app, frame);

    // 5. Scrollable Markdown Document Viewer Modal
    overlays::render_markdown_modal(app, frame);

    // 6. Help Shortcuts Overlay Modal
    help_modal::render_help_modal(app, frame);
}


