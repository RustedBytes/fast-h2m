//! Integration tests for issue 391 xhtml self closing.

//! Regression test for issue #391: XHTML-style self-closing tags like `<td/>`
//! (no space before `/`) silently truncated tables and dropped the rest of the
//! document. The bundled astral-tl parser treats `/` as an identifier character,
//! so `<td/>` was parsed as a literal tag named `"td/"` and subsequent siblings
//! became its children.
//!
//! The fix preprocesses input HTML to rewrite `<tag/>` → `<tag />` so the parser
//! reads the trailing slash as a self-closing marker. This test pins the
//! before/after behavior for `<td/>` (the originally reported case from an
//! EPUB-derived HTML page), `<br/>` (must still render as a line break), and a
//! plain non-table element.

use fast_h2m::convert;

#[test]
fn empty_td_self_closing_does_not_truncate_table() {
    let html = "<table>\
        <tr><td>x</td><td>y</td><td/><td>z</td><td>w</td></tr>\
        <tr><td>aa</td><td>bb</td><td>cc</td><td>dd</td><td>ee</td></tr>\
        </table>\
        <p>after</p>";
    let result = convert(html, None).expect("conversion should succeed");
    let content = result.content.unwrap_or_default();
    assert!(
        content.contains("| x"),
        "first row should be a markdown table row, got:\n{content}"
    );
    assert!(
        content.contains("| aa"),
        "second row must still appear after a self-closing cell, got:\n{content}"
    );
    assert!(
        content.contains("after"),
        "content after the table must not be dropped, got:\n{content}"
    );
}

#[test]
fn br_self_closing_still_renders_as_line_break() {
    let html = "<p>a<br/>b</p>";
    let result = convert(html, None).expect("conversion should succeed");
    let content = result.content.unwrap_or_default();
    // Markdown line break is two spaces + newline.
    assert!(
        content.contains("a  \nb"),
        "<br/> should render as a markdown line break, got:\n{content:?}"
    );
}

#[test]
fn nested_self_closing_does_not_affect_following_paragraphs() {
    let html = "<div><span/>before<p>after</p></div>";
    let result = convert(html, None).expect("conversion should succeed");
    let content = result.content.unwrap_or_default();
    assert!(
        content.contains("before") && content.contains("after"),
        "both 'before' and 'after' must render, got:\n{content}"
    );
}

#[test]
fn explicit_space_before_self_closing_unchanged() {
    let html = "<table><tr><td>x</td><td /><td>y</td></tr></table>";
    let result = convert(html, None).expect("conversion should succeed");
    let content = result.content.unwrap_or_default();
    assert!(
        content.contains("| x") && content.contains("| y"),
        "explicit-space self-close should already work, got:\n{content}"
    );
}
