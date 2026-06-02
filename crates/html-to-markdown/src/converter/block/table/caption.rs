//! Caption element handler for table captions.
//!
//! Handles HTML `<caption>` elements within tables, converting them to
//! Markdown with escaped hyphens to prevent interpretation as table separators.

/// Handles caption elements within tables.
///
/// Extracts text content from the caption and formats it as italicized text
/// with escaped hyphens to prevent Markdown table separator interpretation.
///
/// # Arguments
/// * `node_handle` - Handle to the caption element
/// * `parser` - HTML parser instance
/// * `output` - Output string to append caption text to
/// * `options` - Conversion options
/// * `ctx` - Conversion context
/// * `depth` - Current recursion depth
/// * `dom_ctx` - DOM context for tag name resolution
pub fn handle_caption(
    node_handle: &tl::NodeHandle,
    parser: &tl::Parser,
    output: &mut String,
    options: &crate::options::ConversionOptions,
    ctx: &super::super::super::Context,
    depth: usize,
    dom_ctx: &super::super::super::DomContext,
) {
    if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
        let mut text = String::new();
        let children = tag.children();
        {
            for child_handle in children.top().iter() {
                super::super::super::walk_node(child_handle, parser, &mut text, options, ctx, depth + 1, dom_ctx);
            }
        }
        let text = text.trim();
        if !text.is_empty() {
            let escaped_text = text.replace('-', r"\-");
            output.push('*');
            output.push_str(&escaped_text);
            output.push_str("*\n\n");
        }
    }
}
