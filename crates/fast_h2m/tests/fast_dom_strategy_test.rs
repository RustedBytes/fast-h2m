use fast_h2m::{ConversionOptions, TierStrategy, convert};

fn fast_dom_options() -> ConversionOptions {
    ConversionOptions {
        tier_strategy: TierStrategy::FastDom,
        ..ConversionOptions::default()
    }
}

#[test]
fn fast_dom_converts_common_blocks() {
    let html = r#"
<article>
  <h1>Hello</h1>
  <p>A <strong>bold</strong> and <em>small</em> paragraph with <a href="https://example.com">a link</a>.</p>
  <ul><li>One</li><li>Two</li></ul>
  <blockquote><p>Quoted text</p></blockquote>
</article>
"#;

    let result = convert(html, fast_dom_options()).expect("convert");
    let markdown = result.content.expect("content");

    assert!(markdown.contains("# Hello"));
    assert!(markdown.contains("A **bold** and *small* paragraph"));
    assert!(markdown.contains("[a link](https://example.com)"));
    assert!(markdown.contains("- One"));
    assert!(markdown.contains("- Two"));
    assert!(markdown.contains("> Quoted text"));
}

#[test]
fn fast_dom_converts_simple_table() {
    let html = r#"
<table>
  <tr><th>Name</th><th>Language</th></tr>
  <tr><td>fast_h2m</td><td>Rust</td></tr>
</table>
"#;

    let result = convert(html, fast_dom_options()).expect("convert");
    let markdown = result.content.expect("content");

    assert!(markdown.contains("| Name | Language |"));
    assert!(markdown.contains("| --- | --- |"));
    assert!(markdown.contains("| fast_h2m | Rust |"));
    assert!(result.tables.is_empty());
    assert!(result.document.is_none());
}

#[test]
fn fast_dom_skips_script_style_and_head() {
    let html = r#"
<head><title>Hidden</title></head>
<style>.x { color: red }</style>
<script>alert("ignored")</script>
<p>Visible</p>
"#;

    let result = convert(html, fast_dom_options()).expect("convert");
    let markdown = result.content.expect("content");

    assert_eq!(markdown, "Visible");
}
