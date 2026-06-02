#![allow(missing_docs)]

//! Regression tests for issue #347: inconsistent URL escaping between `<a href>` and `<img src>`.
//!
//! URLs containing spaces, parentheses, or other CommonMark-unsafe characters are correctly
//! wrapped in angle brackets when emitted from `<a href>`, but were previously emitted raw
//! from `<img src>`, breaking `CommonMark` round-trip.
//!
//! The fix ensures `format_image_markdown` applies the same escaping logic as
//! `append_markdown_link`.

fn convert(html: &str) -> String {
    fast_h2m::convert(html, None)
        .expect("conversion should not fail")
        .content
        .unwrap_or_default()
}

// ── Link baseline — must keep working ────────────────────────────────────────

/// `<a href>` with spaces in URL must be wrapped in angle brackets.
#[test]
fn test_link_href_with_spaces_uses_angle_brackets() {
    let html = r#"<a href="/path with spaces">link</a>"#;
    let result = convert(html);
    assert!(
        result.contains("](</path with spaces>)"),
        "link href with spaces must be angle-bracket wrapped. Got:\n{result}"
    );
}

/// `<a href>` with parentheses in URL must be wrapped in angle brackets when unbalanced.
#[test]
fn test_link_href_with_parens_uses_angle_brackets() {
    let html = r#"<a href="/path (1).jpg">link</a>"#;
    let result = convert(html);
    // Unbalanced parens → angle-bracket wrap OR escape; spaces → angle-bracket wrap.
    // The actual href "/path (1).jpg" has a space, so it must get angle-bracket wrapped.
    assert!(
        result.contains("](</path (1).jpg>)"),
        "link href with space+parens must be angle-bracket wrapped. Got:\n{result}"
    );
}

// ── Image bug — was broken, must now be fixed ─────────────────────────────────

/// `<img src>` with spaces in URL must be wrapped in angle brackets (same as `<a href>`).
#[test]
fn test_img_src_with_spaces_uses_angle_brackets() {
    let html = r#"<img src="/img with spaces.png" alt="alt">"#;
    let result = convert(html);
    assert!(
        result.contains("(</img with spaces.png>)"),
        "img src with spaces must be angle-bracket wrapped. Got:\n{result}"
    );
}

/// `<img src>` with space and parentheses in URL must be wrapped in angle brackets.
#[test]
fn test_img_src_with_parens_uses_angle_brackets() {
    let html = r#"<img src="/img (1).png" alt="alt">"#;
    let result = convert(html);
    assert!(
        result.contains("(</img (1).png>)"),
        "img src with space+parens must be angle-bracket wrapped. Got:\n{result}"
    );
}

/// Newline in `<img src>` must be wrapped in angle brackets.
#[test]
fn test_img_src_with_newline_uses_angle_brackets() {
    // Newlines in URLs are pathological but must not break the output.
    let html = "<img src=\"/img\npath.png\" alt=\"alt\">";
    let result = convert(html);
    assert!(
        result.contains("(<"),
        "img src with newline must be angle-bracket wrapped. Got:\n{result}"
    );
}

/// Unbalanced open paren in `<img src>` — no space, so escaping via backslash is expected.
#[test]
fn test_img_src_unbalanced_open_paren_is_escaped() {
    let html = r#"<img src="/img(path.png" alt="alt">"#;
    let result = convert(html);
    // No space → escape unbalanced parens with backslash (same rule as links).
    assert!(
        result.contains(r"\(") || result.contains('<'),
        "img src with unbalanced paren must be escaped or angle-bracket wrapped. Got:\n{result}"
    );
    assert!(
        !result.contains("![alt](/img(path.png)"),
        "img src with unbalanced paren must NOT be emitted raw. Got:\n{result}"
    );
}

// ── Cross-element consistency ─────────────────────────────────────────────────

/// When both `<a href>` and `<img src>` use the same URL with spaces, both must be
/// wrapped identically.
#[test]
fn test_link_and_image_use_identical_url_escaping() {
    let html = r#"
<a href="/path (1).jpg">link</a>
<img src="/path (1).jpg" alt="img">
"#;
    let result = convert(html);

    // The link must wrap the URL.
    assert!(
        result.contains("](</path (1).jpg>)"),
        "link href with space+parens must be angle-bracket wrapped. Got:\n{result}"
    );
    // The image must also wrap the URL — it must emit `![img](</path (1).jpg>)`.
    assert!(
        result.contains("![img](</path (1).jpg>)"),
        "img src with space+parens must be angle-bracket wrapped (same as link). Got:\n{result}"
    );
}

// ── Normal (no-escaping-needed) images must keep working ──────────────────────

/// Plain `<img src>` with no unsafe characters must continue to emit the URL verbatim.
#[test]
fn test_img_src_plain_url_unchanged() {
    let html = r#"<img src="image.jpg" alt="Alt text">"#;
    let result = convert(html);
    assert!(
        result.contains("![Alt text](image.jpg)"),
        "plain img src must not be modified. Got:\n{result}"
    );
}

/// `<img src>` with a fully-qualified URL and no unsafe chars must be unchanged.
#[test]
fn test_img_src_https_url_unchanged() {
    let html = r#"<img src="https://example.com/image.png" alt="Example">"#;
    let result = convert(html);
    assert!(
        result.contains("![Example](https://example.com/image.png)"),
        "https img src must not be modified. Got:\n{result}"
    );
}

/// Balanced parentheses in `<img src>` must not be escaped.
#[test]
fn test_img_src_balanced_parens_unchanged() {
    let html = r#"<img src="/img(balanced).png" alt="alt">"#;
    let result = convert(html);
    assert!(
        result.contains("![alt](/img(balanced).png)"),
        "img src with balanced parens must not be escaped. Got:\n{result}"
    );
}
