use super::*;
use ratatui::layout::Rect;
use ratatui::style::Color;
use crate::ui::theme::ThemeColors;

fn dummy_theme() -> ThemeColors {
    ThemeColors {
        border: Color::Reset,
        border_active: Color::Reset,
        text_main: Color::Reset,
        text_dim: Color::Reset,
        accent: Color::Yellow,
        username: Color::Reset,
        help_btn: Color::Reset,
        quit_btn: Color::Reset,
        warning: Color::Reset,
        success: Color::Reset,
        selection_bg: Color::Reset,
        selection_fg: Color::Reset,
    }
}

#[test]
fn test_button_rect_new() {
    let br = ButtonRect::new(5, 10, 20);
    assert_eq!(br.y, 5);
    assert_eq!(br.x_start, 10);
    assert_eq!(br.x_end, 20);
}

#[test]
fn test_button_rect_contains_inside() {
    let br = ButtonRect::new(5, 10, 20);
    assert!(br.contains(5, 10));
    assert!(br.contains(5, 15));
    assert!(br.contains(5, 19));
}

#[test]
fn test_button_rect_contains_outside_x() {
    let br = ButtonRect::new(5, 10, 20);
    assert!(!br.contains(5, 9));
    assert!(!br.contains(5, 20));
    assert!(!br.contains(5, 25));
}

#[test]
fn test_button_rect_contains_outside_y() {
    let br = ButtonRect::new(5, 10, 20);
    assert!(!br.contains(4, 15));
    assert!(!br.contains(6, 15));
}

#[test]
fn test_centered_rect_basic() {
    let area = Rect::new(0, 0, 100, 100);
    let centered = centered_rect(50, 50, area);
    // 50% width -> 50. x position starts at (100-50)/2 = 25.
    // 50% height -> 50. y position starts at (100-50)/2 = 25.
    assert_eq!(centered.x, 25);
    assert_eq!(centered.y, 25);
    assert_eq!(centered.width, 50);
    assert_eq!(centered.height, 50);
}

#[test]
fn test_format_help_row_empty_desc() {
    let theme = dummy_theme();
    let rows = format_help_row("Ctrl-Q", "", 20, &theme);
    assert_eq!(rows.len(), 1);
    let line = &rows[0];
    assert!(line.spans.len() >= 2);
    assert!(line.spans[0].content.contains("Ctrl-Q"));
}

#[test]
fn test_format_help_row_single_line() {
    let theme = dummy_theme();
    let rows = format_help_row("Tab", "Cycle focus", 20, &theme);
    assert_eq!(rows.len(), 1);
    assert!(rows[0].spans.len() >= 3);
    assert_eq!(rows[0].spans[2].content, "Cycle focus");
}

#[test]
fn test_format_help_row_multi_line() {
    let theme = dummy_theme();
    let rows = format_help_row("Enter", "Select screensaver action from the list", 10, &theme);
    // "Select screensaver action from the list" wrapped at 10 chars
    // "Select" (6), "screensaver" (11 > 10, split to "scre", "ensa", "ver" or similar depending on wrap logic)
    // Whatever the wrapping logic produces, it should result in multiple lines.
    assert!(rows.len() > 1);
}
