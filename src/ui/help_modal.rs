use ratatui::Frame;
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use crate::app::App;

pub fn render_help_modal(app: &App, frame: &mut Frame) {
    if app.show_help {
        let theme = app.theme;
        let area = crate::ui::layout_helpers::centered_rect(65, 75, frame.area());
        let popup_block = Block::default()
            .title(" Keyboard Shortcuts & App Commands ")
            .title_style(
                Style::default()
                    .fg(theme.accent_primary)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent_primary));

        let key_col_width = 18;
        let border_padding = 2;
        let total_inner_width = area.width.saturating_sub(border_padding);
        let max_desc_width = (total_inner_width as usize)
            .saturating_sub(key_col_width)
            .saturating_sub(2); // for ": "

        let mut help_text = Vec::new();
        help_text.push(Line::from(""));

        help_text.extend(crate::ui::text_format::format_help_row(
            "Tab / Shift-Tab",
            "Cycle active panel focus",
            max_desc_width,
            &theme,
        ));
        help_text.extend(crate::ui::text_format::format_help_row(
            "Up / Down",
            "Navigate lists and preference fields",
            max_desc_width,
            &theme,
        ));
        help_text.extend(crate::ui::text_format::format_help_row(
            "Left / Right",
            "Adjust settings and toggle flags",
            max_desc_width,
            &theme,
        ));
        help_text.extend(crate::ui::text_format::format_help_row(
            "Space / Enter",
            "Toggle screensaver selection / Apply settings",
            max_desc_width,
            &theme,
        ));
        help_text.extend(crate::ui::text_format::format_help_row(
            "p / t",
            "Preview highlighted screensaver",
            max_desc_width,
            &theme,
        ));
        help_text.extend(crate::ui::text_format::format_help_row(
            "c / C",
            "Configure highlighted screensaver",
            max_desc_width,
            &theme,
        ));
        help_text.extend(crate::ui::text_format::format_help_row(
            "d / D",
            "Delete downloaded screensaver from list",
            max_desc_width,
            &theme,
        ));
        help_text.extend(crate::ui::text_format::format_help_row(
            "r / R",
            "Refresh screensavers list",
            max_desc_width,
            &theme,
        ));
        help_text.extend(crate::ui::text_format::format_help_row(
            "Esc / q",
            "Close dialogs / Help Overlay, or Quit application",
            max_desc_width,
            &theme,
        ));
        help_text.extend(crate::ui::text_format::format_help_row(
            "h / H",
            "Toggle this help shortcut overlay modal",
            max_desc_width,
            &theme,
        ));

        help_text.push(Line::from(""));
        help_text.extend(crate::ui::text_format::format_help_row(
            "F1",
            "View README.md document",
            max_desc_width,
            &theme,
        ));
        help_text.extend(crate::ui::text_format::format_help_row(
            "F2",
            "View SUPPORT.md document",
            max_desc_width,
            &theme,
        ));
        help_text.extend(crate::ui::text_format::format_help_row(
            "F3",
            "View LICENSE.md document",
            max_desc_width,
            &theme,
        ));
        help_text.extend(crate::ui::text_format::format_help_row(
            "F4",
            "View COPYRIGHT.md document",
            max_desc_width,
            &theme,
        ));
        help_text.extend(crate::ui::text_format::format_help_row(
            "F5",
            "View PRIVACY.md document",
            max_desc_width,
            &theme,
        ));
        help_text.extend(crate::ui::text_format::format_help_row(
            "F6",
            "View SECURITY.md document",
            max_desc_width,
            &theme,
        ));
        help_text.extend(crate::ui::text_format::format_help_row(
            "F7",
            "View CONTRIBUTING.md document",
            max_desc_width,
            &theme,
        ));

        help_text.push(Line::from(""));
        help_text.extend(crate::ui::text_format::format_help_row(
            "CLI Subcommands",
            "trance.exe [tui | doctor]",
            max_desc_width,
            &theme,
        ));

        frame.render_widget(Clear, area);
        let paragraph = Paragraph::new(help_text).block(popup_block);
        frame.render_widget(paragraph, area);
    }
}
