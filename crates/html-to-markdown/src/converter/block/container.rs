//! Handler for structural container elements.
//!
//! This module provides handlers for structural containers that process their
//! children without special formatting or whitespace truncation:
//! - body, html: Structural document containers
//! - time, data: Inline semantic containers
//! - thead, tbody, tfoot, tr, th, td: Table structure (handled elsewhere)
//! - source: Media source element
//! - wbr: Word break opportunity (no-op)

use crate::options::ConversionOptions;
use tl::{NodeHandle, Parser};

// Type aliases for Context and DomContext to avoid circular imports
type Context = crate::converter::Context;
type DomContext = crate::converter::DomContext;

/// Handle structural container elements that recursively process children.
///
/// This is used for elements like `body` and `html` that should process their
/// children directly without any whitespace truncation or special formatting.
///
/// # Arguments
/// * `node_handle` - Handle to the HTML node
/// * `parser` - The HTML parser
/// * `output` - Accumulation buffer for Markdown output
/// * `options` - Conversion options
/// * `ctx` - Current conversion context
/// * `depth` - Current recursion depth
/// * `dom_ctx` - DOM context for tracking relationships
pub fn handle_structural_container(
    node_handle: &NodeHandle,
    parser: &Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
    dom_ctx: &DomContext,
) {
    let Some(node) = node_handle.get(parser) else {
        return;
    };

    let tl::Node::Tag(tag) = node else {
        return;
    };

    let children = tag.children();
    for child_handle in children.top().iter() {
        crate::converter::main::walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
    }
}

/// Handle pass-through container elements that process children inline.
///
/// This is used for semantic elements like `time` and `data` that wrap content
/// but should not add any additional formatting or block breaks.
///
/// # Arguments
/// * `node_handle` - Handle to the HTML node
/// * `parser` - The HTML parser
/// * `output` - Accumulation buffer for Markdown output
/// * `options` - Conversion options
/// * `ctx` - Current conversion context
/// * `depth` - Current recursion depth
/// * `dom_ctx` - DOM context for tracking relationships
pub fn handle_passthrough(
    node_handle: &NodeHandle,
    parser: &Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
    dom_ctx: &DomContext,
) {
    let Some(node) = node_handle.get(parser) else {
        return;
    };

    let tl::Node::Tag(tag) = node else {
        return;
    };

    let children = tag.children();
    for child_handle in children.top().iter() {
        crate::converter::main::walk_node(child_handle, parser, output, options, ctx, depth + 1, dom_ctx);
    }
}

/// Handle no-op container elements that should be ignored.
///
/// This is used for elements like `wbr` (word break opportunity) and `source`
/// (media source specification) that should not produce any output.
///
/// # Arguments
/// * `_node_handle` - Handle to the HTML node (unused)
/// * `_parser` - The HTML parser (unused)
/// * `_output` - Accumulation buffer for Markdown output (unused)
/// * `_options` - Conversion options (unused)
/// * `_ctx` - Current conversion context (unused)
/// * `_depth` - Current recursion depth (unused)
/// * `_dom_ctx` - DOM context (unused)
#[inline]
pub fn handle_noop(
    _node_handle: &NodeHandle,
    _parser: &Parser,
    _output: &mut String,
    _options: &ConversionOptions,
    _ctx: &Context,
    _depth: usize,
    _dom_ctx: &DomContext,
) {
    // Intentionally empty: these elements produce no Markdown output
}
