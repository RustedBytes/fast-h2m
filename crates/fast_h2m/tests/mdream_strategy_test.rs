use fast_h2m::{ConversionOptions, MarkdownStreamProcessor, TierStrategy, convert};

fn mdream_options() -> ConversionOptions {
    ConversionOptions {
        tier_strategy: TierStrategy::Mdream,
        ..ConversionOptions::default()
    }
}

#[test]
fn mdream_strategy_converts_common_blocks() {
    let result = convert("<h1>Hello</h1><p>World</p>", mdream_options()).expect("convert");
    let markdown = result.content.expect("content");

    assert!(markdown.contains("# Hello"));
    assert!(markdown.contains("World"));
}

#[test]
fn mdream_strategy_returns_lean_result_shape() {
    let result =
        convert("<table><tr><td>Cell</td></tr></table>", mdream_options()).expect("convert");

    assert!(
        result
            .content
            .as_deref()
            .unwrap_or_default()
            .contains("Cell")
    );
    assert!(result.document.is_none());
    assert!(result.tables.is_empty());
    assert!(result.warnings.is_empty());
    #[cfg(feature = "metadata")]
    {
        assert!(result.metadata.document.title.is_none());
        assert!(result.metadata.headers.is_empty());
        assert!(result.metadata.links.is_empty());
        assert!(result.metadata.images.is_empty());
        assert!(result.metadata.structured_data.is_empty());
    }
}

#[test]
fn mdream_stream_processor_converts_split_chunks() {
    let mut stream = MarkdownStreamProcessor::new(None);
    let mut markdown = String::new();

    markdown.push_str(&stream.process_chunk("<h1>Hello</h1>"));
    markdown.push_str(&stream.process_chunk("<p>World</p>"));
    markdown.push_str(&stream.finish());

    assert!(markdown.contains("# Hello"));
    assert!(markdown.contains("World"));
}

#[cfg(any(feature = "serde", feature = "metadata"))]
#[test]
fn mdream_strategy_deserializes_from_snake_case() {
    let options: ConversionOptions =
        serde_json::from_str(r#"{"tier_strategy":"mdream"}"#).expect("deserialize options");

    assert_eq!(options.tier_strategy, TierStrategy::Mdream);
}

#[cfg(any(feature = "serde", feature = "metadata"))]
#[test]
fn mdream_strategy_deserializes_from_camel_case_alias() {
    let options: ConversionOptions =
        serde_json::from_str(r#"{"tierStrategy":"mdream"}"#).expect("deserialize options");

    assert_eq!(options.tier_strategy, TierStrategy::Mdream);
}
