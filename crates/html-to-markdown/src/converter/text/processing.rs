//! Text processing utilities for HTML to Markdown conversion.
//!
//! This module provides functions for processing text content, including
//! code block dedentation and special character handling.

/// Remove common leading whitespace from all lines in a code block.
///
/// This is useful when HTML authors indent `<pre>` content for readability,
/// so we can strip the shared indentation without touching meaningful spacing.
///
/// # Examples
///
/// ```text
/// "    line1\n    line2" → "line1\nline2"
/// "  indent1\n    indent2" → "indent1\n  indent2" (removes 2 chars, minimum indent)
/// "  \n  code" → "\ncode"
/// ```
pub fn dedent_code_block(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return String::new();
    }

    let min_indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.chars().take_while(|c| c.is_whitespace()).count())
        .min()
        .unwrap_or(0);

    lines.iter().fold(String::new(), |mut acc, line| {
        if !acc.is_empty() {
            acc.push('\n');
        }
        let processed = if line.trim().is_empty() {
            *line
        } else {
            let mut remaining = min_indent;
            let mut cut = 0;
            for (idx, ch) in line.char_indices() {
                if remaining == 0 {
                    break;
                }
                if ch.is_whitespace() {
                    remaining -= 1;
                    cut = idx + ch.len_utf8();
                } else {
                    break;
                }
            }
            &line[cut..]
        };
        acc.push_str(processed);
        acc
    })
}
