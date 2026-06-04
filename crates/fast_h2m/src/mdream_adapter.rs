use crate::ConversionOptions;

/// Convert HTML using mdream's lean converter.
pub(crate) fn convert(html: &str, _options: &ConversionOptions) -> String {
    mdream::html_to_markdown(html, mdream::types::HTMLToMarkdownOptions::default())
}

/// Streaming HTML-to-Markdown converter backed by mdream.
pub struct MarkdownStreamProcessor {
    inner: mdream::MarkdownStreamProcessor,
}

impl MarkdownStreamProcessor {
    /// Create a streaming converter.
    #[must_use]
    pub fn new(options: impl Into<Option<ConversionOptions>>) -> Self {
        let _ = options.into();
        Self {
            inner: mdream::MarkdownStreamProcessor::new(
                mdream::types::HTMLToMarkdownOptions::default(),
            ),
        }
    }

    /// Process one HTML chunk and return newly available Markdown.
    pub fn process_chunk(&mut self, chunk: &str) -> String {
        self.inner.process_chunk(chunk)
    }

    /// Flush buffered HTML and return remaining Markdown.
    pub fn finish(&mut self) -> String {
        self.inner.finish()
    }
}
