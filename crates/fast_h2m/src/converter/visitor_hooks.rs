//! Visitor callback hooks for custom HTML traversal during conversion.
//!
//! This module contains the visitor pattern implementation hooks that are called
//! before and after element processing during the HTML to Markdown conversion tree walk.
//! These hooks enable custom processing, analysis, or modification of elements during conversion.

use std::borrow::Cow;

#[cfg(feature = "visitor")]
use crate::converter::utility::content::collect_tag_attributes;
use crate::converter::utility::content::is_block_level_element;
use crate::visitor::{NodeContext, NodeType, VisitResult};

/// Handles visitor callback for element start (before processing).
///
/// This function is called when entering an element during tree traversal,
/// before the element's content is processed. The visitor can:
/// - Continue with normal processing (Continue)
/// - Skip the element entirely (Skip)
/// - Provide custom output to replace the element (Custom)
/// - Signal an error (Error)
///
/// # Arguments
///
/// * `visitor_handle` - Reference to the visitor for callbacks
/// * `tag_name` - The normalized tag name being processed
/// * `node_handle` - Handle to the DOM node
/// * `tag` - Reference to the tag object
/// * `parser` - Reference to the tl parser
/// * `output` - Mutable reference to output string
/// * `ctx` - Reference to the conversion context
/// * `depth` - Current tree depth
/// * `dom_ctx` - Reference to DOM context for tree navigation
///
/// # Returns
///
/// `VisitAction` enum indicating what should happen next:
/// - `VisitAction::Continue` - Process element normally
/// - `VisitAction::Skip` - Skip element, don't process or call visit_element_end
/// - `VisitAction::Custom(output)` - Use custom output, skip normal processing
/// - `VisitAction::Error` - Stop processing with error
pub fn handle_visitor_element_start(
    visitor_handle: &crate::visitor::VisitorHandle,
    tag_name: &str,
    node_handle: &tl::NodeHandle,
    tag: &tl::HTMLTag,
    parser: &crate::tl_types::Parser<'_>,
    output: &mut String,
    _ctx: &crate::converter::Context,
    depth: usize,
    dom_ctx: &crate::converter::DomContext,
) -> VisitAction {
    let attributes = collect_tag_attributes(tag);

    let node_id = node_handle.get_inner();
    let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
    let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);

    let node_ctx = NodeContext {
        node_type: NodeType::Element,
        tag_name: Cow::Borrowed(tag_name),
        attributes,
        depth,
        index_in_parent,
        parent_tag,
        is_inline: !is_block_level_element(tag_name),
    };

    let visitor_start_result = {
        let mut visitor = visitor_handle.lock().expect("visitor mutex poisoned");
        visitor.visit_element_start(&node_ctx)
    };

    match visitor_start_result {
        crate::visitor::VisitResult::Continue => VisitAction::Continue,
        crate::visitor::VisitResult::Skip => VisitAction::Skip,
        crate::visitor::VisitResult::Custom(custom_output) => {
            output.push_str(&custom_output);

            // For custom output, still call visit_element_end (except for tables)
            if !matches!(tag_name, "table") {
                let element_content = &custom_output;
                let mut visitor = visitor_handle.lock().expect("visitor mutex poisoned");
                let _ = visitor.visit_element_end(&node_ctx, element_content);
            }

            VisitAction::Custom
        }
        crate::visitor::VisitResult::Error(_msg) => VisitAction::Error,
        _ => VisitAction::Continue,
    }
}

/// Handles visitor callback for element end (after processing).
///
/// This function is called when exiting an element after its content has been processed.
/// The visitor can:
/// - Accept the output normally (Continue)
/// - Replace the output with custom content (Custom)
/// - Remove the output entirely (Skip)
/// - Signal an error (Error)
///
/// # Arguments
///
/// * `visitor_handle` - Reference to the visitor for callbacks
/// * `tag_name` - The normalized tag name that was processed
/// * `node_handle` - Handle to the DOM node
/// * `tag` - Reference to the tag object
/// * `parser` - Reference to the tl parser
/// * `output` - Mutable reference to output string
/// * `element_output_start` - Byte position where this element's output started
/// * `ctx` - Reference to the conversion context
/// * `depth` - Current tree depth
/// * `dom_ctx` - Reference to DOM context for tree navigation
pub fn handle_visitor_element_end(
    visitor_handle: &crate::visitor::VisitorHandle,
    tag_name: &str,
    node_handle: &tl::NodeHandle,
    tag: &tl::HTMLTag,
    parser: &crate::tl_types::Parser<'_>,
    output: &mut String,
    element_output_start: usize,
    ctx: &crate::converter::Context,
    depth: usize,
    dom_ctx: &crate::converter::DomContext,
) {
    // Skip visitor callback for table elements
    if matches!(tag_name, "table") {
        return;
    }

    let attributes = collect_tag_attributes(tag);

    let node_id = node_handle.get_inner();
    let parent_tag = dom_ctx.parent_tag_name(node_id, parser);
    let index_in_parent = dom_ctx.get_sibling_index(node_id).unwrap_or(0);

    let node_ctx = NodeContext {
        node_type: NodeType::Element,
        tag_name: Cow::Borrowed(tag_name),
        attributes,
        depth,
        index_in_parent,
        parent_tag,
        is_inline: !is_block_level_element(tag_name),
    };

    // The saved `element_output_start` can become stale in two ways:
    // 1. A child visitor returning Custom/Skip truncates `output`, making the
    //    saved position point past the end of the now-shorter string.
    // 2. Element handlers (e.g. div) trim trailing whitespace that was present
    //    when the position was captured, then append new multi-byte content.
    //    The old position can land inside a multi-byte character.
    // Clamp to output length, then retreat to a valid char boundary.
    let safe_start = element_output_start.min(output.len());
    let safe_start = crate::converter::utility::content::floor_char_boundary(output, safe_start);
    let element_content = &output[safe_start..];

    let mut visitor = visitor_handle.lock().expect("visitor mutex poisoned");
    match visitor.visit_element_end(&node_ctx, element_content) {
        VisitResult::Continue => {}
        VisitResult::Custom(custom) => {
            output.truncate(safe_start);
            output.push_str(&custom);
        }
        VisitResult::Skip => {
            output.truncate(safe_start);
        }
        VisitResult::Error(err) => {
            if ctx.visitor_error.borrow().is_none() {
                *ctx.visitor_error.borrow_mut() = Some(err);
            }
        }
        VisitResult::PreserveHtml => {}
    }
}

/// Result of visitor element start callback indicating what should happen next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisitAction {
    /// Continue with normal element processing
    Continue,
    /// Skip the element entirely (don't process children or call visit_element_end)
    Skip,
    /// Custom output was provided, skip normal processing
    Custom,
    /// Error occurred during visitor callback
    Error,
}
