use super::*;
use std::path::PathBuf;
use crate::config::GlobalConfig;
use crate::theme::TuiTheme;
use crossterm::event::KeyCode;

fn mock_app() -> App {
    let screensavers = vec![
        Screensaver {
            name: "Bubbles".to_string(),
            path: PathBuf::from("bubbles.scr"),
        },
        Screensaver {
            name: "Mystify".to_string(),
            path: PathBuf::from("mystify.scr"),
        },
        Screensaver {
            name: "Ribbons".to_string(),
            path: PathBuf::from("ribbons.scr"),
        },
    ];
    let global = GlobalConfig::default();
    let local = LocalConfig::default();
    let theme = TuiTheme::high_contrast(true);
    App::new(screensavers, global, local, theme)
}

#[test]
fn test_filtered_indices() {
    let mut app = mock_app();
    assert_eq!(app.filtered_indices(), vec![0, 1, 2]);
    app.local.hide_stock = true;
    assert_eq!(app.filtered_indices(), Vec::<usize>::new());
}

#[test]
fn test_handle_key_navigation_and_focus() {
    let mut app = mock_app();
    assert_eq!(app.focused, FocusedSection::GlobalPrefs);
    assert_eq!(app.global_field, GlobalField::Active);

    app.handle_key(KeyCode::Down, KeyModifiers::empty());
    assert_eq!(app.global_field, GlobalField::Timeout);

    app.handle_key(KeyCode::Down, KeyModifiers::empty());
    assert_eq!(app.global_field, GlobalField::PreventSleep);

    app.handle_key(KeyCode::Down, KeyModifiers::empty());
    assert_eq!(app.global_field, GlobalField::HideStock);

    app.handle_key(KeyCode::Tab, KeyModifiers::empty());
    assert_eq!(app.focused, FocusedSection::SaverList);
}

#[test]
fn test_handle_key_toggle_preferences() {
    let temp_dir = std::env::temp_dir().join("app-trance-tests-registry");
    let _ = std::fs::create_dir_all(&temp_dir);
    unsafe {
        std::env::set_var("APP_IGNITE_REGISTRY_PATH", temp_dir.to_str().unwrap());
    }

    let mut app = mock_app();
    assert_eq!(app.global.active, false);

    app.focused = FocusedSection::GlobalPrefs;
    app.global_field = GlobalField::Active;
    app.handle_key(KeyCode::Char(' '), KeyModifiers::empty());

    assert_eq!(app.global.active, true);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_handle_key_select_screensaver() {
    let temp_dir = std::env::temp_dir().join("app-trance-tests-registry-select");
    let _ = std::fs::create_dir_all(&temp_dir);
    unsafe {
        std::env::set_var("APP_IGNITE_REGISTRY_PATH", temp_dir.to_str().unwrap());
    }

    let mut app = mock_app();

    app.focused = FocusedSection::SaverList;

    assert_eq!(app.global.active_scr, "");

    app.handle_key(KeyCode::Enter, KeyModifiers::empty());
    assert_eq!(app.global.active_scr, "bubbles.scr");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_console_modes() {
    println!("TESTING RAW MODES:");
    match crossterm::terminal::enable_raw_mode() {
        Ok(_) => println!("  enable_raw_mode: OK"),
        Err(e) => println!("  enable_raw_mode: ERROR: {}", e),
    }
    match crossterm::terminal::disable_raw_mode() {
        Ok(_) => println!("  disable_raw_mode: OK"),
        Err(e) => println!("  disable_raw_mode: ERROR: {}", e),
    }
}
