#![allow(missing_docs)]

fn convert(
    html: &str,
    opts: Option<h2md::ConversionOptions>,
) -> h2md::error::Result<String> {
    h2md::convert(html, opts).map(|r| r.content.unwrap_or_default())
}

use h2md::ConversionOptions;

#[test]
fn long_multibyte_link_label_does_not_panic() {
    let mut html = String::from("<a href=\"https://example.com/article\">");
    html.push_str(&"a".repeat(511));
    html.push('👍');
    html.push_str("</a>");

    let markdown = convert(&html, Some(ConversionOptions::default())).unwrap();
    let expected_label = format!("{}👍", "a".repeat(511));

    assert!(
        markdown.contains(&format!("[{expected_label}]")),
        "expected full label to appear in markdown output; got: {markdown}"
    );
}
