#![allow(missing_docs)]

fn convert(
    html: &str,
    opts: Option<fast_h2m::ConversionOptions>,
) -> fast_h2m::error::Result<String> {
    fast_h2m::convert(html, opts).map(|r| r.content.unwrap_or_default())
}

use std::fs;
use std::path::PathBuf;

use fast_h2m::ConversionOptions;

fn fixture_path(name: &str) -> PathBuf {
    [
        env!("CARGO_MANIFEST_DIR"),
        "../../fixtures/test_documents/html/issues",
        name,
    ]
    .iter()
    .collect()
}

#[test]
fn test_spa_first_half() {
    let html = fs::read_to_string(fixture_path("gh-121-minimal-failing.html")).expect("read html");

    let opts = ConversionOptions {
        extract_metadata: false,
        autolinks: false,
        ..Default::default()
    };

    let result = convert(&html, Some(opts)).unwrap();
    eprintln!("Result length: {}", result.len());
    assert!(!result.is_empty());
}
