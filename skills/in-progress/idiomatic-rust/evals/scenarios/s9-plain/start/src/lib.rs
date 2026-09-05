//! Line-oriented text helpers for a terminal report.
//!
//! 1. `wrap` breaks text into lines of at most `width` characters, at spaces.
//! 2. `indent` puts a prefix in front of every line.
//! 3. `longest_line` and `count_words` are the two measurements the report prints.

/// Breaks `text` into lines of at most `width` characters, at spaces. Runs of whitespace
/// collapse to one space. A word longer than `width` stands on its own line.
#[must_use]
pub fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if word.chars().count() > width {
            continue;
        }
        if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > width {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

/// Puts `prefix` in front of every line of `text`. An empty `text` stays empty.
#[must_use]
pub fn indent(text: &str, prefix: &str) -> String {
    text.lines()
        .map(|line| format!("{prefix}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The first line of `text` with the most characters, or `None` when `text` is empty.
#[must_use]
pub fn longest_line(text: &str) -> Option<&str> {
    let mut longest: Option<&str> = None;
    for line in text.lines() {
        // A strict comparison keeps the first of two lines with the same length.
        if longest.is_none_or(|current| line.chars().count() > current.chars().count()) {
            longest = Some(line);
        }
    }
    longest
}

/// How many whitespace-separated words `text` holds.
#[must_use]
pub fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_breaks_at_spaces_within_the_width() {
        // "the quick" is 9 characters; "brown" would make it 15
        assert_eq!(
            wrap("the quick brown fox", 10),
            vec!["the quick", "brown fox"]
        );
    }

    #[test]
    fn test_wrap_collapses_whitespace() {
        assert_eq!(wrap("a   b\n\tc", 10), vec!["a b c"]);
    }

    #[test]
    fn test_wrap_of_empty_text_is_empty() {
        assert!(wrap("", 10).is_empty());
    }

    #[test]
    fn test_indent_prefixes_every_line() {
        assert_eq!(indent("a\nb", "> "), "> a\n> b");
        assert_eq!(indent("", "> "), "");
    }

    #[test]
    fn test_longest_line_keeps_the_first_of_equals() {
        assert_eq!(longest_line("ab\ncd\ne"), Some("ab"));
        assert_eq!(longest_line(""), None);
    }

    #[test]
    fn test_count_words_ignores_extra_whitespace() {
        assert_eq!(count_words("  one two\tthree\n"), 3);
    }
}
