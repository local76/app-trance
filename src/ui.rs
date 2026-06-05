//! Ratatui-based rendering. Pure function of `App` -> `Frame`.
//!
//! # Model-Render Split
//! rSaver uses a strict Model-Render architectural split:
//!
//! * **Model (`app.rs`)**: Owns the state (selected saver, timer configuration, focus, etc.)
//!   and implements the business logic, key handlers, and state modifications.
//! * **Render (`ui.rs`)**: Takes a mutable reference to the `App` state and draws the layout,
//!   widgets, list view, borders, help texts, and active indicators to the screen.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};

use crate::app::{App, FocusedSection, GlobalField};

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    let theme = app.theme;
    let (min_w, min_h) = crate::theme::recommended_min_size(96);

    if area.width < min_w || area.height < min_h {
        render_too_small(theme, frame, area);
        return;
    }

    // Split entire area vertically into bordered boxes
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // 0: Header box
            Constraint::Length(7), // 1: Global Prefs & Help (side-by-side)
            Constraint::Min(10),   // 2: Screensaver Preferences list
            Constraint::Length(3), // 3: Status / Progress footer box
        ])
        .split(area);

    // 0. Render Header Info Box
    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(Span::styled(
            " Rust Screensaver Manager ",
            Style::default().fg(theme.accent_primary).add_modifier(Modifier::BOLD),
        ));
    
    let username = std::env::var("USERNAME").unwrap_or_else(|_| std::env::var("USER").unwrap_or_else(|_| "user".to_string()));
    let hostname = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "localhost".to_string());
    let os_str = crate::win32::query_os_version();

    let header_line = Line::from(vec![
        Span::styled(" rSaver ", Style::default().fg(theme.accent_primary).add_modifier(Modifier::BOLD)),
        Span::styled(" │ ", Style::default().fg(theme.border)),
        Span::styled(format!("{}@{}", username, hostname), Style::default().fg(Color::Rgb(255, 215, 0)).add_modifier(Modifier::BOLD)),
        Span::styled(" │ ", Style::default().fg(theme.border)),
        Span::styled(os_str, Style::default().fg(theme.text_main)),
    ]);
    let header_inner = header_block.inner(chunks[0]);
    frame.render_widget(header_block, chunks[0]);
    frame.render_widget(Paragraph::new(header_line), header_inner);

    // 1. Render Side-by-Side Global Prefs & Help
    let top_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(chunks[1]);

    render_prefs(app, frame, top_split[0]);
    render_help(theme, frame, top_split[1]);

    // 2. Render Screensaver Preferences List Table
    render_list(app, frame, chunks[2]);

    // 3. Render Footer Status Box
    let footer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(Span::styled(
            " Status ",
            Style::default().fg(theme.accent_primary).add_modifier(Modifier::BOLD),
        ));

    #[cfg(feature = "downloader")]
    let mut is_downloading = false;
    #[cfg(feature = "downloader")]
    let mut download_name = String::new();
    #[cfg(feature = "downloader")]
    {
        if let Some(ref state_mutex) = app.download_state {
            is_downloading = true;
            if let Ok(state) = state_mutex.lock() {
                download_name = state.name.clone();
            }
        }
    }

    #[cfg(feature = "downloader")]
    let footer_p = if is_downloading {
        let progress = app.visual_progress;
        let track_width = 30;
        let pacman_pos = ((progress * track_width as f64).round() as usize).min(track_width);
        
        let mut track = String::new();
        let is_mouth_open = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() / 150)
            .unwrap_or(0) % 2) == 0;
            
        let pacman_char = if progress >= 1.0 {
            "o"
        } else if is_mouth_open {
            "ᗧ"
        } else {
            "o"
        };

        if progress < 1.0 {
            for _ in 0..pacman_pos {
                track.push(' ');
            }
            track.push_str(pacman_char);
            for i in (pacman_pos + 1)..track_width {
                if i == track_width - 1 {
                    track.push('ᗣ');
                } else {
                    track.push('·');
                }
            }
        } else {
            for _ in 0..track_width.saturating_sub(1) {
                track.push(' ');
            }
            track.push('o');
        }

        Paragraph::new(Line::from(vec![
            Span::styled(format!("Downloading ({}): ", download_name), Style::default().fg(theme.text_main).add_modifier(Modifier::BOLD)),
            Span::styled(" [", Style::default().fg(theme.border)),
            Span::styled(track, Style::default().fg(theme.accent_primary)),
            Span::styled("]", Style::default().fg(theme.border)),
            Span::styled(format!(" {:>3.0}%", progress * 100.0), Style::default().fg(theme.accent_secondary)),
        ]))
    } else if let Some(ref status) = app.status {
        let color = match status.kind {
            crate::app::StatusKind::Info => theme.accent_primary,
            crate::app::StatusKind::Error => theme.missing,
        };
        Paragraph::new(Line::from(vec![
            Span::styled(&status.text, Style::default().fg(color).add_modifier(Modifier::BOLD)),
        ]))
    } else {
        Paragraph::new(Line::from(vec![
            Span::styled("Ready. Press Tab to cycle focus.", Style::default().fg(theme.text_dim)),
        ]))
    };

    #[cfg(not(feature = "downloader"))]
    let footer_p = if let Some(ref status) = app.status {
        let color = match status.kind {
            crate::app::StatusKind::Info => theme.accent_primary,
            crate::app::StatusKind::Error => theme.missing,
        };
        Paragraph::new(Line::from(vec![
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
}

fn render_too_small(theme: crate::theme::TuiTheme, frame: &mut Frame, area: Rect) {
    let block = Block::default().borders(Borders::ALL);
    let (min_w, min_h) = crate::theme::recommended_min_size(96);
    let text = vec![
        Line::from(Span::styled(
            "Terminal too small",
            Style::default()
                .fg(theme.accent_secondary)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(format!(
            "Need at least {min_w}x{min_h}, current {}x{}.",
            area.width, area.height
        )),
    ];
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(text).block(block).wrap(Wrap { trim: false }),
        area,
    );
}

fn render_prefs(app: &mut App, frame: &mut Frame, area: Rect) {
    let theme = app.theme;
    let active = app.focused == FocusedSection::GlobalPrefs;
    let border_color = if active { theme.border_active } else { theme.border };

    let prefs_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            " Global Screensaver Preferences ",
            Style::default().fg(if active { theme.accent_primary } else { theme.header }).add_modifier(Modifier::BOLD),
        ));

    let active_status = if app.global.active { "ACTIVE" } else { "DISABLED" };
    let active_color = if app.global.active { theme.accent_secondary } else { theme.text_dim };
    
    let sleep_status = if app.local.prevent_sleep { "ACTIVE (SYSTEM AWAKE)" } else { "DISABLED (NORMAL)" };
    let sleep_color = if app.local.prevent_sleep { theme.accent_secondary } else { theme.text_dim };
    
    let hide_stock_status = if app.local.hide_stock { "YES" } else { "NO" };
    let hide_stock_color = if app.local.hide_stock { theme.accent_secondary } else { theme.text_dim };
    
    let timeout_value = format!("{} minutes", app.global.timeout / 60);
    let cycle_time_value = format!("{} seconds", app.local.random_cycle_secs);

    let mut lines = Vec::new();

    let mut add_field = |field: GlobalField, label: &str, value: String, value_color: Color| {
        let focused = active && app.global_field == field;
        let arrow_span = Span::styled(if focused { " ▶ " } else { "   " }, Style::default().fg(theme.accent_primary));
        let label_style = if focused {
            Style::default().fg(theme.accent_secondary).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text_main)
        };
        lines.push(Line::from(vec![
            arrow_span,
            Span::styled(label.to_string(), label_style),
            Span::styled(" ", Style::default()),
            Span::styled(value, Style::default().fg(value_color)),
        ]));
    };

    add_field(GlobalField::Active,       "Active:        ", active_status.to_string(), active_color);
    add_field(GlobalField::Timeout,      "Timeout:       ", timeout_value, theme.accent_primary);
    add_field(GlobalField::PreventSleep, "Prevent sleep: ", sleep_status.to_string(), sleep_color);
    add_field(GlobalField::CycleTime,    "Cycle time:    ", cycle_time_value, theme.accent_primary);
    add_field(GlobalField::HideStock,    "Hide stock:    ", hide_stock_status.to_string(), hide_stock_color);

    let prefs_inner = prefs_block.inner(area);
    frame.render_widget(prefs_block, area);
    frame.render_widget(Paragraph::new(lines), prefs_inner);
}

fn render_help(theme: crate::theme::TuiTheme, frame: &mut Frame, area: Rect) {
    let help_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(Span::styled(
            " Help & Keyboard Shortcuts ",
            Style::default().fg(theme.header).add_modifier(Modifier::BOLD),
        ));

    let col1 = [
        ("Tab", "Focus"),
        ("↑/↓", "Move"),
        ("←/→", "Adjust"),
        ("Space/Enter", "Toggle/Apply"),
        ("? / H", "Help Info"),
    ];

    let col2 = [
        ("F5 / R", "Rescan"),
        ("P", "Preview"),
        ("C", "Config"),
        ("D", "Delete"),
        ("q/Esc", "Quit"),
    ];

    let mut lines = Vec::new();

    for i in 0..5 {
        let (k1, d1) = col1[i];
        let (k2, d2) = col2[i];
        lines.push(Line::from(vec![
            Span::styled(format!("  {:<12}", k1), Style::default().fg(theme.accent_primary)),
            Span::raw(format!("{:<15}", d1)),
            Span::styled(format!("  {:<8}", k2), Style::default().fg(theme.accent_primary)),
            Span::raw(d2),
        ]));
    }

    let help_inner = help_block.inner(area);
    frame.render_widget(help_block, area);
    frame.render_widget(Paragraph::new(lines), help_inner);
}

fn render_list(app: &mut App, frame: &mut Frame, area: Rect) {
    let theme = app.theme;
    let active = app.focused == FocusedSection::SaverList;
    let border_color = if active { theme.border_active } else { theme.border };

    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            " Screensaver Preferences ",
            Style::default().fg(if active { theme.accent_primary } else { theme.header }).add_modifier(Modifier::BOLD),
        ));

    let list_inner = list_block.inner(area);
    frame.render_widget(list_block, area);

    let list_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Table Header
            Constraint::Min(1),    // List Items
        ])
        .split(list_inner);

    // Table Header Alignment to match the List items
    let header_line = Line::from(vec![
        Span::raw("   "),
        Span::styled("STATUS        ", if active { theme.accent_primary } else { theme.header }),
        Span::styled("LOCATION      ", Style::default().fg(theme.accent_primary).add_modifier(Modifier::BOLD)),
        Span::styled("FRIENDLY NAME             ", Style::default().fg(theme.accent_primary).add_modifier(Modifier::BOLD)),
        Span::styled("FILE NAME           ", Style::default().fg(theme.accent_primary).add_modifier(Modifier::BOLD)),
        Span::styled("TYPE", Style::default().fg(theme.accent_primary).add_modifier(Modifier::BOLD)),
    ]);
    frame.render_widget(Paragraph::new(header_line), list_chunks[0]);

    let indices = app.filtered_indices();

    if indices.is_empty() {
        let text = vec![
            Line::from("  No .scr files found."),
            Line::from(Span::styled(
                "  Drop one into %APPDATA%\\rSaver\\screensavers",
                Style::default().fg(theme.text_dim),
            )),
        ];
        frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: false }), list_chunks[1]);
        return;
    }

    let total_items = indices.len();
    let visible_height = list_chunks[1].height as usize;
    let selected_pos = indices
        .iter()
        .position(|&i| i == app.highlighted)
        .unwrap_or(0);

    // Adjust list_offset to keep selected_pos in view
    if selected_pos < app.list_offset {
        app.list_offset = selected_pos;
    } else if selected_pos >= app.list_offset + visible_height {
        app.list_offset = selected_pos - visible_height + 1;
    }
    if app.list_offset + visible_height > total_items {
        app.list_offset = total_items.saturating_sub(visible_height);
    }

    let start = app.list_offset;
    let end = (start + visible_height).min(total_items);
    let visible_indices = &indices[start..end];

    let items: Vec<ListItem> = visible_indices
        .iter()
        .map(|&i| app.list_items[i].clone())
        .collect();

    let mut state = ListState::default().with_selected(Some(selected_pos.saturating_sub(start)));
    let list = List::new(items)
        .highlight_style(
            Style::default()
                .fg(theme.text_main)
                .bg(theme.bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(if active { " ▶ " } else { " ▷ " });
    frame.render_stateful_widget(list, list_chunks[1], &mut state);
}

pub fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}
