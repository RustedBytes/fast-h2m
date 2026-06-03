//! Integration tests for issue 339 regressions.

//! Regression tests for issue #339: content after `<!-- ... --->` is silently dropped.
//!
//! The `astral-tl` parser mishandles HTML comments whose closing sequence contains more
//! than two dashes (e.g. `--->` instead of `-->`).  When it encounters such a comment
//! the remaining document is consumed and never emitted, so every node after the comment
//! disappears from the output.
//!
//! The fix is applied in `preprocess_html`: any `<!--` … `---+>` pattern is normalised
//! to a well-formed `<!-- … -->` before the input is handed to `tl`.

fn convert(html: &str) -> String {
    fast_h2m::convert(html, None)
        .expect("conversion should not fail")
        .content
        .unwrap_or_default()
}

// ── Basic reproduction ────────────────────────────────────────────────────────

/// Content after `<!-- /// --->` must NOT be dropped (the exact input from the report).
#[test]
fn test_content_after_triple_dash_comment_not_dropped() {
    let html = "<h1>One</h1>\n<!-- /// --->\n<p>Two</p>";
    let result = convert(html);
    assert!(
        result.contains("One"),
        "Heading 'One' must be present. Got:\n{result}"
    );
    assert!(
        result.contains("Two"),
        "Paragraph 'Two' must NOT be dropped after `<!-- /// --->`. Got:\n{result}"
    );
}

/// Normal `<!-- comment -->` (double-dash close) must still work correctly.
#[test]
fn test_normal_comment_does_not_affect_output() {
    let html = "<h1>One</h1>\n<!-- comment -->\n<p>Two</p>";
    let result = convert(html);
    assert!(result.contains("One"), "Heading must be present");
    assert!(
        result.contains("Two"),
        "Paragraph must be present after normal comment"
    );
}

/// Comment with exactly three dashes: `--->`.
#[test]
fn test_three_dash_comment_close() {
    let html = "<p>Before</p><!-- note ---><p>After</p>";
    let result = convert(html);
    assert!(
        result.contains("Before"),
        "Content before comment must be present. Got:\n{result}"
    );
    assert!(
        result.contains("After"),
        "Content after `<!-- note --->` must not be dropped. Got:\n{result}"
    );
}

/// Comment with four dashes: `---->`
#[test]
fn test_four_dash_comment_close() {
    let html = "<p>Before</p><!-- note ----><p>After</p>";
    let result = convert(html);
    assert!(
        result.contains("Before"),
        "Content before comment must be present. Got:\n{result}"
    );
    assert!(
        result.contains("After"),
        "Content after `<!-- note ---->` must not be dropped. Got:\n{result}"
    );
}

/// Multiple bogus comments in sequence — none should eat the document tail.
#[test]
fn test_multiple_bogus_comments_preserve_all_content() {
    let html = "<p>A</p><!-- x ---><p>B</p><!-- y ---><p>C</p>";
    let result = convert(html);
    assert!(result.contains('A'), "A must be present. Got:\n{result}");
    assert!(result.contains('B'), "B must be present. Got:\n{result}");
    assert!(result.contains('C'), "C must be present. Got:\n{result}");
}

/// Content-less bogus comment `<!--->`  should not drop what follows.
#[test]
fn test_empty_bogus_comment_preserves_following_content() {
    let html = "<p>Before</p><!---><p>After</p>";
    let result = convert(html);
    assert!(
        result.contains("After"),
        "Content after `<!--->` must not be dropped. Got:\n{result}"
    );
}
