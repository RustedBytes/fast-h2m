//! Integration tests for serde round-trip of public enum types.

use fast_h2m::options::ConversionOptions;

#[test]
fn test_deserialize_highlight_bold_lowercase() {
    let json = r#"{"highlight_style":"bold"}"#;
    let result = serde_json::from_str::<ConversionOptions>(json);
    assert!(result.is_ok(), "Failed to deserialize: {:?}", result.err());
    if let Ok(opts) = result {
        assert_eq!(opts.highlight_style, fast_h2m::HighlightStyle::Bold);
    }
}
