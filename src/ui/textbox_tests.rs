use super::*;
use crossterm::event::KeyCode;

#[test]
fn test_textbox_new_default() {
    let box1 = TextBox::new();
    assert_eq!(box1.text, "");
    assert_eq!(box1.cursor_pos, 0);
    assert!(!box1.active);

    let box2 = TextBox::default();
    assert_eq!(box2.text, "");
    assert_eq!(box2.cursor_pos, 0);
    assert!(!box2.active);
}

#[test]
fn test_textbox_inactive_key_handling() {
    let mut tb = TextBox::new();
    tb.active = false;
    tb.handle_key(KeyCode::Char('a'));
    assert_eq!(tb.text, "");
    assert_eq!(tb.cursor_pos, 0);
}

#[test]
fn test_textbox_active_key_handling() {
    let mut tb = TextBox::new();
    tb.active = true;
    tb.handle_key(KeyCode::Char('a'));
    assert_eq!(tb.text, "a");
    assert_eq!(tb.cursor_pos, 1);
}

#[test]
fn test_textbox_handle_char() {
    let mut tb = TextBox::new();
    tb.active = true;
    tb.handle_key(KeyCode::Char('x'));
    tb.handle_key(KeyCode::Char('y'));
    tb.handle_key(KeyCode::Char('z'));
    assert_eq!(tb.text, "xyz");
    assert_eq!(tb.cursor_pos, 3);
}

#[test]
fn test_textbox_handle_backspace_middle() {
    let mut tb = TextBox::new();
    tb.active = true;
    tb.text = "abc".to_string();
    tb.cursor_pos = 2; // after 'b'
    tb.handle_key(KeyCode::Backspace);
    assert_eq!(tb.text, "ac");
    assert_eq!(tb.cursor_pos, 1);
}

#[test]
fn test_textbox_handle_backspace_beginning() {
    let mut tb = TextBox::new();
    tb.active = true;
    tb.text = "abc".to_string();
    tb.cursor_pos = 0;
    tb.handle_key(KeyCode::Backspace);
    assert_eq!(tb.text, "abc");
    assert_eq!(tb.cursor_pos, 0);
}

#[test]
fn test_textbox_handle_backspace_end() {
    let mut tb = TextBox::new();
    tb.active = true;
    tb.text = "abc".to_string();
    tb.cursor_pos = 3;
    tb.handle_key(KeyCode::Backspace);
    assert_eq!(tb.text, "ab");
    assert_eq!(tb.cursor_pos, 2);
}

#[test]
fn test_textbox_handle_delete_middle() {
    let mut tb = TextBox::new();
    tb.active = true;
    tb.text = "abc".to_string();
    tb.cursor_pos = 1; // at 'b'
    tb.handle_key(KeyCode::Delete);
    assert_eq!(tb.text, "ac");
    assert_eq!(tb.cursor_pos, 1);
}

#[test]
fn test_textbox_handle_delete_beginning() {
    let mut tb = TextBox::new();
    tb.active = true;
    tb.text = "abc".to_string();
    tb.cursor_pos = 0; // at 'a'
    tb.handle_key(KeyCode::Delete);
    assert_eq!(tb.text, "bc");
    assert_eq!(tb.cursor_pos, 0);
}

#[test]
fn test_textbox_handle_delete_end() {
    let mut tb = TextBox::new();
    tb.active = true;
    tb.text = "abc".to_string();
    tb.cursor_pos = 3; // out of bounds / end
    tb.handle_key(KeyCode::Delete);
    assert_eq!(tb.text, "abc");
    assert_eq!(tb.cursor_pos, 3);
}

#[test]
fn test_textbox_handle_left_right() {
    let mut tb = TextBox::new();
    tb.active = true;
    tb.text = "abc".to_string();
    tb.cursor_pos = 3;

    tb.handle_key(KeyCode::Left);
    assert_eq!(tb.cursor_pos, 2);

    tb.handle_key(KeyCode::Right);
    assert_eq!(tb.cursor_pos, 3);

    // Test clamping left
    tb.cursor_pos = 0;
    tb.handle_key(KeyCode::Left);
    assert_eq!(tb.cursor_pos, 0);

    // Test clamping right
    tb.cursor_pos = 3;
    tb.handle_key(KeyCode::Right);
    assert_eq!(tb.cursor_pos, 3);
}

#[test]
fn test_textbox_handle_home() {
    let mut tb = TextBox::new();
    tb.active = true;
    tb.text = "abcde".to_string();
    tb.cursor_pos = 4;
    tb.handle_key(KeyCode::Home);
    assert_eq!(tb.cursor_pos, 0);
}

#[test]
fn test_textbox_handle_end() {
    let mut tb = TextBox::new();
    tb.active = true;
    tb.text = "abcde".to_string();
    tb.cursor_pos = 1;
    tb.handle_key(KeyCode::End);
    assert_eq!(tb.cursor_pos, 5);
}

#[test]
fn test_textbox_clear() {
    let mut tb = TextBox::new();
    tb.text = "hello".to_string();
    tb.cursor_pos = 3;
    tb.clear();
    assert_eq!(tb.text, "");
    assert_eq!(tb.cursor_pos, 0);
}
