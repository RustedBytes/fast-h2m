#![allow(missing_docs)]

//! Regression test for issue #399: spurious blank lines after the YAML
//! frontmatter and after lists produced markdown that violates markdownlint
//! MD012 (no multiple consecutive blank lines). Block-level emission must
//! collapse runs of more than one blank line so the output contains at most a
//! single empty line between blocks.

use html_to_markdown_rs::{ConversionOptions, convert};

#[test]
fn no_double_blank_line_after_frontmatter_or_list() {
    let html =
        "<head>\n  <title>Foobar</title>\n</head>\n<body>\n  <p>Baz</p><ul><li>qux</li></ul><p>Thud</p>\n</body>";
    let options = ConversionOptions {
        extract_metadata: true,
        ..Default::default()
    };
    let result = convert(html, Some(options)).expect("conversion should succeed");
    let content = result.content.unwrap_or_default();
    assert_eq!(content, "---\ntitle: Foobar\n---\n\nBaz\n\n- qux\n\nThud\n");
}

#[test]
fn no_triple_newline_between_blocks_without_frontmatter() {
    let html = "<p>Baz</p><ul><li>qux</li></ul><p>Thud</p>";
    let options = ConversionOptions::default();
    let result = convert(html, Some(options)).expect("conversion should succeed");
    let content = result.content.unwrap_or_default();
    assert_eq!(content, "Baz\n\n- qux\n\nThud\n");
}
