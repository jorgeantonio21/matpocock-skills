//! Independent tests for s9-plain. The crate is ordinary and mostly right. One defect: `wrap`
//! drops a word longer than the width, where its doc comment says the word stands on its own line.
//! The scorer copies this file into `tests/` of each result tree.

use linefmt::{count_words, indent, longest_line, wrap};

#[test]
fn test_wrap_keeps_a_word_longer_than_the_width_on_its_own_line() {
    assert_eq!(wrap("a verylongword b", 4), vec!["a", "verylongword", "b"]);
}

#[test]
fn test_wrap_fills_a_line_to_exactly_the_width() {
    // "ab cd" is 5 characters
    assert_eq!(wrap("ab cd ef", 5), vec!["ab cd", "ef"]);
}

#[test]
fn test_wrap_counts_characters_not_bytes() {
    // each word is 2 characters and 4 bytes
    assert_eq!(wrap("éé éé", 5), vec!["éé éé"]);
}

#[test]
fn test_wrap_of_whitespace_only_is_empty() {
    assert!(wrap(" \n\t ", 10).is_empty());
}

#[test]
fn test_indent_keeps_blank_lines() {
    assert_eq!(indent("a\n\nb", "  "), "  a\n  \n  b");
}

#[test]
fn test_longest_line_measures_characters() {
    // "ééé" is 3 characters and 6 bytes; "abcd" is 4 characters
    assert_eq!(longest_line("ééé\nabcd"), Some("abcd"));
}

#[test]
fn test_count_words_of_empty_text_is_zero() {
    assert_eq!(count_words(""), 0);
    assert_eq!(count_words("   "), 0);
}
