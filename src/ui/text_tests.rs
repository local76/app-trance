use super::*;

#[test]
fn test_wrap_text_empty() {
    assert_eq!(wrap_text("", 10), Vec::<String>::new());
}

#[test]
fn test_wrap_text_zero_max_width() {
    assert_eq!(wrap_text("hello world", 0), vec!["hello world".to_string()]);
}

#[test]
fn test_wrap_text_under_max_width() {
    assert_eq!(wrap_text("hello", 10), vec!["hello".to_string()]);
}

#[test]
fn test_wrap_text_simple_wrap() {
    let input = "hello world standard text";
    let wrapped = wrap_text(input, 12);
    // "hello world" is 11 chars. "standard" would make it 20 chars (> 12).
    // So "hello world" is the first line, then "standard text" is 13 chars (> 12).
    // Wait, "standard" is 8, "text" is 4. "standard" + " " + "text" = 13.
    // So "standard" fits, and "text" wraps.
    assert_eq!(wrapped, vec![
        "hello world".to_string(),
        "standard".to_string(),
        "text".to_string()
    ]);
}

#[test]
fn test_wrap_text_exact_width() {
    let input = "abc def ghi";
    // Each word is 3 chars, + spaces. Total 11 chars.
    // Width 7: "abc def" is 7. "ghi" is 3.
    assert_eq!(wrap_text(input, 7), vec!["abc def".to_string(), "ghi".to_string()]);
}

#[test]
fn test_wrap_text_long_word_splitting() {
    // Word longer than max_width
    assert_eq!(wrap_text("abcdefgh", 3), vec!["abc".to_string(), "def".to_string(), "gh".to_string()]);
}

#[test]
fn test_wrap_text_long_word_mixed() {
    let input = "hi abcdefgh bye";
    // width 3: "hi", "abc", "def", "gh", "bye"
    // Let's verify what happens:
    // "hi" (len 2) is added.
    // "abcdefgh" is a new word. "hi" + 1 + 8 > 3. So "hi" is pushed.
    // Current word "abcdefgh" len 8 > 3. It gets split: "abc", "def", "gh".
    // Next word "bye" len 3. "bye" fits.
    assert_eq!(wrap_text(input, 3), vec![
        "hi".to_string(),
        "abc".to_string(),
        "def".to_string(),
        "gh".to_string(),
        "bye".to_string()
    ]);
}

#[test]
fn test_wrap_text_newlines_preserved() {
    let input = "hello\n\nworld";
    assert_eq!(wrap_text(input, 10), vec![
        "hello".to_string(),
        "".to_string(),
        "world".to_string()
    ]);
}

#[test]
fn test_char_width_standard() {
    assert_eq!(char_width('a'), 1);
    assert_eq!(char_width('1'), 1);
}

#[test]
fn test_char_width_emoji() {
    // Emojis starting from 0x1F000
    assert_eq!(char_width('🧠'), 2); // U+1F9E0
    assert_eq!(char_width('🎮'), 2); // U+1F3AE
}

#[test]
fn test_char_width_variation_selector() {
    assert_eq!(char_width('\u{FE0F}'), 0);
}

#[test]
fn test_visible_len_no_escapes() {
    assert_eq!(visible_len("hello"), 5);
}

#[test]
fn test_visible_len_basic_escapes() {
    // "\x1b[31mhello\x1b[0m" has red color escape sequences.
    // Escaped sequences should not be counted.
    assert_eq!(visible_len("\x1b[31mhello\x1b[0m"), 5);
}

#[test]
fn test_visible_len_mixed_unicode_escapes() {
    // "\x1b[1m🧠\x1b[0m" should have visual len 2 because of emoji.
    assert_eq!(visible_len("\x1b[1m🧠\x1b[0m"), 2);
}
