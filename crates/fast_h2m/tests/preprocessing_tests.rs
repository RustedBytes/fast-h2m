//! Integration tests for preprocessing tests.

fn convert(
    html: &str,
    opts: Option<fast_h2m::ConversionOptions>,
) -> fast_h2m::error::Result<String> {
    fast_h2m::convert(html, opts).map(|r| r.content.unwrap_or_default())
}

use fast_h2m::ConversionOptions;

#[test]
fn footer_without_navigation_hint_is_preserved() {
    let html = r#"<!DOCTYPE html>
<html lang="en">
  <body>
    <main>
      <h1>Simple Webpage</h1>
      <p>This is a simple webpage without external images.</p>
    </main>
    <footer>
      <p>Test page for processors validation</p>
    </footer>
  </body>
</html>"#;

    let markdown = convert(html, None).unwrap();
    assert!(
        markdown.contains("Test page for processors validation"),
        "footer content should be retained in markdown:\n{markdown}"
    );
}

#[test]
fn footer_with_navigation_hint_is_removed() {
    let html = r#"<!DOCTYPE html>
<html lang="en">
  <body>
    <main>
      <h1>Simple Webpage</h1>
    </main>
    <footer class="site-footer">
      <p>Test page for processors validation</p>
      <nav><a href="/about">About</a></nav>
    </footer>
  </body>
</html>"#;

    let options = ConversionOptions {
        preprocessing: fast_h2m::PreprocessingOptions {
            enabled: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let markdown = convert(html, Some(options)).unwrap();
    assert!(
        !markdown.contains("processors validation"),
        "navigational footers should still be stripped entirely:\n{markdown}"
    );
}
