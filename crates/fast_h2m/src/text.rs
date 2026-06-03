//! Text processing utilities for Markdown conversion.

use std::borrow::Cow;

/// Escape Markdown special characters in text.
///
/// # Arguments
///
/// * `text` - Text to escape
/// * `escape_misc` - Escape miscellaneous characters (`\` `&` `<` `` ` `` `[` `>` `~` `#` `=` `+` `|` `-`)
/// * `escape_asterisks` - Escape asterisks (`*`)
/// * `escape_underscores` - Escape underscores (`_`)
/// * `escape_ascii` - Escape all ASCII punctuation (for `CommonMark` spec compliance)
///
/// # Returns
///
/// Escaped text
#[inline]
pub fn escape(
    text: &str,
    escape_misc: bool,
    escape_asterisks: bool,
    escape_underscores: bool,
    escape_ascii: bool,
) -> Cow<'_, str> {
    if text.is_empty() {
        return Cow::Borrowed("");
    }

    if !escape_misc && !escape_asterisks && !escape_underscores && !escape_ascii {
        return Cow::Borrowed(text);
    }

    if escape_ascii && !contains_ascii_punctuation(text.as_bytes()) {
        return Cow::Borrowed(text);
    }

    if !escape_ascii && escape_misc && !escape_asterisks && !escape_underscores {
        let needs_misc = contains_misc_markdown(text.as_bytes());
        let needs_numbered = contains_numbered_marker_punctuation(text.as_bytes());
        if !needs_misc && !needs_numbered {
            return Cow::Borrowed(text);
        }
    }

    let mut result: Cow<'_, str> = Cow::Borrowed(text);

    if escape_ascii {
        return escape_chars(text, is_ascii_punctuation).map_or(Cow::Borrowed(text), Cow::Owned);
    }

    if escape_misc {
        if let Some(escaped) = escape_chars(result.as_ref(), is_misc_markdown_char) {
            result = Cow::Owned(escaped);
        }
        if let Some(escaped) = escape_numbered_list_markers(result.as_ref()) {
            result = Cow::Owned(escaped);
        }
    }

    if escape_asterisks && result.contains('*') {
        result = Cow::Owned(result.replace('*', r"\*"));
    }

    if escape_underscores && result.contains('_') {
        result = Cow::Owned(result.replace('_', r"\_"));
    }

    result
}

#[cfg(all(feature = "simd", nightly))]
#[inline]
fn contains_ascii_punctuation(bytes: &[u8]) -> bool {
    crate::simd_scan::contains_ascii_punctuation(bytes)
}

#[cfg(not(all(feature = "simd", nightly)))]
#[inline]
fn contains_ascii_punctuation(bytes: &[u8]) -> bool {
    bytes.iter().any(|byte| byte.is_ascii_punctuation())
}

#[cfg(all(feature = "simd", nightly))]
#[inline]
fn contains_misc_markdown(bytes: &[u8]) -> bool {
    crate::simd_scan::contains_misc_markdown(bytes)
}

#[cfg(not(all(feature = "simd", nightly)))]
#[inline]
fn contains_misc_markdown(bytes: &[u8]) -> bool {
    bytes.iter().any(|b| {
        matches!(
            b,
            b'\\'
                | b'&'
                | b'<'
                | b'`'
                | b'['
                | b']'
                | b'>'
                | b'~'
                | b'#'
                | b'='
                | b'+'
                | b'|'
                | b'-'
        )
    })
}

#[cfg(all(feature = "simd", nightly))]
#[inline]
fn contains_numbered_marker_punctuation(bytes: &[u8]) -> bool {
    crate::simd_scan::find_any2(bytes, b'.', b')').is_some()
}

#[cfg(not(all(feature = "simd", nightly)))]
#[inline]
fn contains_numbered_marker_punctuation(bytes: &[u8]) -> bool {
    bytes.iter().any(|b| matches!(b, b'.' | b')'))
}

fn escape_chars(text: &str, should_escape: fn(char) -> bool) -> Option<String> {
    if !text.chars().any(should_escape) {
        return None;
    }

    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        if should_escape(ch) {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    Some(escaped)
}

fn escape_numbered_list_markers(text: &str) -> Option<String> {
    let mut prev_was_digit = false;
    if !text.chars().any(|ch| {
        let should_escape = prev_was_digit && matches!(ch, '.' | ')');
        prev_was_digit = ch.is_ascii_digit();
        should_escape
    }) {
        return None;
    }

    let mut escaped = String::with_capacity(text.len());
    let mut prev_was_digit = false;
    for ch in text.chars() {
        if prev_was_digit && matches!(ch, '.' | ')') {
            escaped.push('\\');
        }
        escaped.push(ch);
        prev_was_digit = ch.is_ascii_digit();
    }
    Some(escaped)
}

const fn is_misc_markdown_char(ch: char) -> bool {
    matches!(
        ch,
        '\\' | '&' | '<' | '`' | '[' | ']' | '>' | '~' | '#' | '=' | '+' | '|' | '-'
    )
}

const fn is_ascii_punctuation(ch: char) -> bool {
    ch.is_ascii_punctuation()
}

/// Extract boundary whitespace from text (chomp).
///
/// Returns (prefix, suffix, `trimmed_text`) tuple.
/// Prefix/suffix are " " if original text had leading/trailing whitespace.
/// However, suffix is "" if the trailing whitespace is only newlines (not spaces/tabs).
/// This prevents trailing newlines from becoming trailing spaces in the output.
/// The trimmed text has all leading/trailing whitespace removed.
#[must_use]
#[inline]
pub fn chomp(text: &str) -> (&str, &str, &str) {
    if text.is_empty() {
        return ("", "", "");
    }

    let prefix = if text.starts_with(|c: char| c.is_whitespace()) {
        " "
    } else {
        ""
    };

    let suffix = if text.ends_with("\n\n") || text.ends_with("\r\n\r\n") {
        "\n\n"
    } else if text.ends_with([' ', '\t']) {
        " "
    } else {
        ""
    };

    let trimmed = if suffix == "\n\n" {
        text.trim_end_matches("\n\n")
            .trim_end_matches("\r\n\r\n")
            .trim()
    } else {
        text.trim()
    };

    (prefix, suffix, trimmed)
}

/// Normalize whitespace by collapsing consecutive spaces and tabs.
///
/// Multiple spaces and tabs are replaced with a single space.
/// Newlines are preserved.
/// Unicode spaces are normalized to ASCII spaces.
///
/// # Arguments
///
/// * `text` - The text to normalize
///
/// # Returns
///
/// Normalized text with collapsed spaces/tabs but preserved newlines
#[must_use]
pub fn normalize_whitespace(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut prev_was_space = false;

    for ch in text.chars() {
        let is_space = ch == ' ' || ch == '\t' || is_unicode_space(ch);

        if is_space {
            if !prev_was_space {
                result.push(' ');
                prev_was_space = true;
            }
        } else {
            result.push(ch);
            prev_was_space = false;
        }
    }

    result
}

/// Normalize whitespace in text, returning borrowed or owned result as needed.
///
/// This function optimizes memory by returning a borrowed reference when no normalization
/// is needed, and only allocating a new string when whitespace changes are necessary.
///
/// Multiple consecutive spaces, tabs, and Unicode space characters are replaced with
/// a single ASCII space. Newlines are preserved as-is.
///
/// # Arguments
///
/// * `text` - The text to normalize
///
/// # Returns
///
/// `Cow::Borrowed` if text is already normalized, or `Cow::Owned` with normalized text
#[must_use]
#[inline]
pub fn normalize_whitespace_cow(text: &str) -> Cow<'_, str> {
    let mut prev_was_space = false;

    for ch in text.chars() {
        let is_space = ch == ' ' || ch == '\t' || is_unicode_space(ch);
        if is_space {
            if prev_was_space || ch != ' ' {
                return Cow::Owned(normalize_whitespace(text));
            }
            prev_was_space = true;
        } else {
            prev_was_space = false;
        }
    }

    Cow::Borrowed(text)
}

/// Decode common HTML entities.
///
/// Decodes the most common HTML entities to their character equivalents:
/// - `&quot;` → `"`
/// - `&apos;` → `'`
/// - `&lt;` → `<`
/// - `&gt;` → `>`
/// - `&amp;` → `&` (must be last to avoid double-decoding)
///
/// # Arguments
///
/// * `text` - Text containing HTML entities
///
/// # Returns
///
/// Text with entities decoded
#[must_use]
pub fn decode_html_entities(text: &str) -> String {
    html_escape::decode_html_entities(text).into_owned()
}

/// Decode HTML entities in text, returning borrowed or owned result as needed.
///
/// This function optimizes memory by returning a borrowed reference when no HTML
/// entities are present, and only allocating a new string when entity decoding
/// is necessary.
///
/// Decodes common HTML entities like:
/// - `&quot;` → `"`
/// - `&apos;` → `'`
/// - `&lt;` → `<`
/// - `&gt;` → `>`
/// - `&amp;` → `&` (decoded last to avoid double-decoding)
///
/// # Arguments
///
/// * `text` - Text potentially containing HTML entities
///
/// # Returns
///
/// `Cow::Borrowed` if no entities found, or `Cow::Owned` with entities decoded
#[must_use]
#[inline]
pub fn decode_html_entities_cow(text: &str) -> Cow<'_, str> {
    if !text.contains('&') {
        return Cow::Borrowed(text);
    }

    html_escape::decode_html_entities(text)
}

/// Check if a character is a unicode space character.
///
/// Includes: non-breaking space, various width spaces, etc.
#[inline]
const fn is_unicode_space(ch: char) -> bool {
    matches!(
        ch,
        '\u{00A0}'
            | '\u{1680}'
            | '\u{2000}'
            | '\u{2001}'
            | '\u{2002}'
            | '\u{2003}'
            | '\u{2004}'
            | '\u{2005}'
            | '\u{2006}'
            | '\u{2007}'
            | '\u{2008}'
            | '\u{2009}'
            | '\u{200A}'
            | '\u{202F}'
            | '\u{205F}'
            | '\u{3000}'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_misc() {
        assert_eq!(
            escape("foo & bar", true, false, false, false),
            r"foo \& bar"
        );
        assert_eq!(
            escape("foo [bar]", true, false, false, false),
            r"foo \[bar\]"
        );
        assert_eq!(escape("1. Item", true, false, false, false), r"1\. Item");
        assert_eq!(escape("1) Item", true, false, false, false), r"1\) Item");
    }

    #[test]
    fn test_escape_asterisks() {
        assert_eq!(
            escape("foo * bar", false, true, false, false),
            r"foo \* bar"
        );
        assert_eq!(
            escape("**bold**", false, true, false, false),
            r"\*\*bold\*\*"
        );
    }

    #[test]
    fn test_escape_underscores() {
        assert_eq!(escape("foo_bar", false, false, true, false), r"foo\_bar");
        assert_eq!(
            escape("__bold__", false, false, true, false),
            r"\_\_bold\_\_"
        );
    }

    #[test]
    fn test_escape_ascii() {
        assert_eq!(
            escape(r##"!"#$%&"##, false, false, false, true),
            r#"\!\"\#\$\%\&"#
        );
        assert_eq!(escape("*+,-./", false, false, false, true), r"\*\+\,\-\.\/");
        assert_eq!(escape("<=>?@", false, false, false, true), r"\<\=\>\?\@");
        assert_eq!(
            escape(r"[\]^_`", false, false, false, true),
            r"\[\\\]\^\_\`"
        );
        assert_eq!(escape("{|}~", false, false, false, true), r"\{\|\}\~");
    }

    #[test]
    fn test_chomp() {
        assert_eq!(chomp("  text  "), (" ", " ", "text"));
        assert_eq!(chomp("text"), ("", "", "text"));
        assert_eq!(chomp(" text"), (" ", "", "text"));
        assert_eq!(chomp("text "), ("", " ", "text"));
        assert_eq!(chomp(""), ("", "", ""));
    }
}
