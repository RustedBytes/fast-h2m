#![allow(missing_docs)]

fn convert(
    html: &str,
    opts: Option<html_to_markdown_rs::ConversionOptions>,
) -> html_to_markdown_rs::error::Result<String> {
    html_to_markdown_rs::convert(html, opts).map(|r| r.content.unwrap_or_default())
}

use std::fs;
use std::path::PathBuf;

use html_to_markdown_rs::ConversionOptions;

fn fixture_path(name: &str) -> PathBuf {
    [env!("CARGO_MANIFEST_DIR"), "../../test_documents/html/issues", name]
        .iter()
        .collect()
}

fn options_with_wrap() -> ConversionOptions {
    ConversionOptions {
        wrap: true,
        wrap_width: 80,
        extract_metadata: false,
        autolinks: false,
        ..Default::default()
    }
}

fn normalize_newlines(input: &str) -> String {
    input.replace("\r\n", "\n").replace('\r', "\n")
}

#[test]
fn split_closing_tag_does_not_merge_nested_list() {
    // Regression: tl parser mishandles </tag\n> (closing bracket on next line).
    // The <a> element absorbs the nested <ul> because the closing </a\n> isn't
    // recognised, so all nested items end up inside the link text.
    // Note: the `>` closing the opening <a> and the `>` closing </a are on
    // separate lines (JSX-style formatting). tl must still parse this correctly.
    let html = r##"<ul>
  <li>
    <a href="#beyond"
      >Beyond triads</a
    >
    <ul>
      <li><a href="#c">Child</a></li>
    </ul>
  </li>
  <li>
    <a href="#sibling"
      >Sibling item</a
    >
    <ul>
      <li><a href="#d">Deep child</a></li>
    </ul>
  </li>
</ul>"##;
    let opts = ConversionOptions {
        wrap: false,
        extract_metadata: false,
        autolinks: false,
        ..Default::default()
    };
    let result = convert(html, Some(opts)).expect("conversion should succeed");
    // The nested list items must appear on their own lines, not merged into the
    // parent link text.
    assert!(
        result.contains("Child"),
        "nested item 'Child' should be present: got {result:?}"
    );
    assert!(
        result.contains("Sibling item"),
        "sibling list item should be present: got {result:?}"
    );
    assert!(
        result.contains("Deep child"),
        "deep nested item should be present: got {result:?}"
    );
    // None of the nested items should appear inside the link text of the parent item.
    assert!(
        !result.contains("triads * [") && !result.contains("triads - [Child"),
        "nested items must not be merged into parent link text: got {result:?}"
    );
}

#[test]
fn wrap_preserves_link_only_list_items() {
    let html = fs::read_to_string(fixture_path("gh-143-links-wordwrap.html")).unwrap();
    let expected = fs::read_to_string(fixture_path("gh-143-links-wordwrap.md")).unwrap();

    let result = convert(&html, Some(options_with_wrap())).expect("conversion should succeed");

    assert_eq!(
        normalize_newlines(&result).trim(),
        normalize_newlines(&expected).trim(),
        "word wrapping should not merge nested link-only list items"
    );
}
