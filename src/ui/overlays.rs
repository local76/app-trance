use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use crate::app::App;
use crate::theme::TuiTheme;

pub fn render_too_small(theme: TuiTheme, frame: &mut Frame, area: Rect) {
    let block = Block::default().borders(Borders::ALL);
    let min_w = 100;
    let min_h = 35;
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

pub fn handle_selection_highlights(app: &mut App, frame: &mut Frame) {
    let theme = app.theme;
    if let (Some(start), Some(end)) = (app.selection_start, app.selection_end) {
        let buf = frame.buffer_mut();
        let width = buf.area.width;
        let height = buf.area.height;

        let (col1, row1) = start;
        let (col2, row2) = end;

        let is_selected = |x: u16, y: u16| -> bool {
            let (c1, r1) = (col1, row1);
            let (c2, r2) = (col2, row2);
            if r1 == r2 {
                y == r1 && x >= c1.min(c2) && x <= c1.max(c2)
            } else if r1 < r2 {
                (y == r1 && x >= c1) || (y > r1 && y < r2) || (y == r2 && x <= c2)
            } else {
                (y == r2 && x >= c2) || (y > r2 && y < r1) || (y == r1 && x <= c1)
            }
        };

        // 1. Draw Highlight
        for y in 0..height {
            for x in 0..width {
                if is_selected(x, y) {
                    let cell = &mut buf[(x, y)];
                    cell.set_bg(Color::Rgb(0, 120, 215));
                    cell.set_fg(Color::White);
                }
            }
        }

        // 2. Perform Copy on Release
        if app.selection_pending_copy {
            let mut selected_text = String::new();
            let mut current_row: Option<u16> = None;
            let mut current_line = String::new();

            for y in 0..height {
                for x in 0..width {
                    if is_selected(x, y) {
                        let cell = &buf[(x, y)];
                        if current_row != Some(y) {
                            if current_row.is_some() {
                                selected_text.push_str(current_line.trim_end());
                                selected_text.push('\n');
                                current_line.clear();
                            }
                            current_row = Some(y);
                        }
                        current_line.push_str(cell.symbol());
                    }
                }
            }
            if !current_line.is_empty() {
                selected_text.push_str(current_line.trim_end());
            }

            if !selected_text.is_empty() {
                let _ = crate::win32::copy_text_to_clipboard(&selected_text);
                let truncated = if selected_text.len() > 30 {
                    format!("{}...", &selected_text[..27].replace('\n', " "))
                } else {
                    selected_text.replace('\n', " ")
                };
                app.status = Some(crate::app::StatusMessage {
                    text: format!("{} Copied selection to clipboard: {}", app.glyphs.clipboard, truncated),
                    kind: crate::app::StatusKind::Info,
                });
            }

            app.selection_start = None;
            app.selection_end = None;
            app.selection_pending_copy = false;
        }
    }
}

pub fn render_markdown_modal(app: &App, frame: &mut Frame) {
    if let Some(ref filename) = app.show_markdown {
        let theme = app.theme;
        let area = crate::ui::layout_helpers::centered_rect(85, 80, frame.area());
        let popup_block = Block::default()
            .title(format!(
                " Document Viewer: {} (Press Esc/q to Close) ",
                filename
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent_primary));

        let paragraph = Paragraph::new(app.markdown_lines.clone())
            .block(popup_block)
            .wrap(ratatui::widgets::Wrap { trim: true })
            .alignment(ratatui::layout::Alignment::Left)
            .scroll((app.markdown_scroll as u16, 0));

        frame.render_widget(Clear, area);
        frame.render_widget(paragraph, area);
    }
}


