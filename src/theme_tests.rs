use super::*;
use crate::win32::{Palette, PowerStatus, Rgb, SystemMetrics};

fn mock_metrics(dark_mode: bool, high_contrast: bool) -> SystemMetrics {
    SystemMetrics {
        screen_w: 1920,
        screen_h: 1080,
        dpi: 96,
        window_dpi: 96,
        dark_mode,
        high_contrast,
        accent: Rgb(0, 120, 215),
        power: PowerStatus {
            ac_online: true,
            battery_percent: 100,
        },
    }
}

#[test]
fn test_recommended_min_size() {
    assert_eq!(recommended_min_size(96), (60, 25));
    assert_eq!(recommended_min_size(144), (90, 38)); // 1.5x scale
    assert_eq!(recommended_min_size(192), (120, 50)); // 2x scale
}

#[test]
fn test_theme_detect_high_contrast_dark() {
    let metrics = mock_metrics(true, true);
    let palette = Palette::default();
    let theme = TuiTheme::detect_impl(metrics, palette, false);
    assert!(theme.high_contrast);
    assert!(!theme.no_color);
    assert_eq!(theme.bg, Color::Black);
    assert_eq!(theme.border, Color::White);
    assert_eq!(theme.border_active, Color::Yellow);
}

#[test]
fn test_theme_detect_high_contrast_light() {
    let metrics = mock_metrics(false, true);
    let palette = Palette::default();
    let theme = TuiTheme::detect_impl(metrics, palette, false);
    assert!(theme.high_contrast);
    assert!(!theme.no_color);
    assert_eq!(theme.bg, Color::White);
    assert_eq!(theme.border, Color::Black);
    assert_eq!(theme.border_active, Color::Yellow);
}

#[test]
fn test_theme_detect_no_color_dark() {
    let metrics = mock_metrics(true, false);
    let palette = Palette::default();
    let theme = TuiTheme::detect_impl(metrics, palette, true); // no_color = true
    assert!(!theme.high_contrast);
    assert!(theme.no_color);
    assert_eq!(theme.border, Color::DarkGray);
    assert_eq!(theme.border_active, Color::White);
}

#[test]
fn test_theme_detect_normal_dark() {
    let metrics = mock_metrics(true, false);
    let palette = Palette::default();
    let theme = TuiTheme::detect_impl(metrics, palette, false);
    assert!(!theme.high_contrast);
    assert!(!theme.no_color);
    assert_eq!(theme.bg, Color::Reset);
    assert_eq!(theme.border, Color::Rgb(118, 118, 118)); // dark grey (palette index 8)
    assert_eq!(theme.border_active, Color::Rgb(0, 120, 215)); // accent color
}

#[test]
fn test_tui_theme_detect_overrides() {
    let t_light = TuiTheme::detect(Some("light"));
    assert!(!t_light.dark_mode);

    let t_dark = TuiTheme::detect(Some("dark"));
    assert!(t_dark.dark_mode);

    let t_hc = TuiTheme::detect(Some("high-contrast"));
    assert!(t_hc.high_contrast);

    let t_nc = TuiTheme::detect(Some("no-color"));
    assert!(t_nc.no_color);

    // Invalid/ignored override should not panic
    let _ = TuiTheme::detect(Some("invalid-override-xyz"));
}

#[test]
fn test_theme_no_color_light() {
    let theme = TuiTheme::no_color(false); // light mode
    assert!(!theme.high_contrast);
    assert!(theme.no_color);
    assert_eq!(theme.border, Color::Gray);
    assert_eq!(theme.border_active, Color::Black);
}

#[test]
fn test_from_metrics_and_palette_zero_accent() {
    let mut metrics = mock_metrics(true, false);
    metrics.accent = Rgb(0, 0, 0); // No saturation/zero
    let palette = Palette::default();
    let expected_color = Color::Rgb(palette.colors[14].0, palette.colors[14].1, palette.colors[14].2);
    let theme = TuiTheme::from_metrics_and_palette(metrics, palette);
    assert_eq!(theme.border_active, expected_color);
}


