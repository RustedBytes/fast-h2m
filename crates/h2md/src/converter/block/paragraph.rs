//! Handler for paragraph elements (p, div).
//!
//! Converts HTML paragraph tags to Markdown paragraphs with proper spacing
//! and support for:
//! - Continuation handling in tables and lists
//! - Proper blank line spacing
//! - Empty element filtering
//! - Visitor callbacks for custom paragraph processing

use crate::options::ConversionOptions;
use crate::tl_types::Parser;
use tl::NodeHandle;

// Type aliases for Context and DomContext to avoid circular imports
type Context = crate::converter::Context;
type DomContext = crate::converter::DomContext;

/// Handle paragraph elements (p, div).
///
/// Processes children with proper context, manages spacing,
/// and handles special cases for table cells and list items.
pub fn handle(
    node_handle: &NodeHandle,
    parser: &Parser,
    output: &mut String,
    options: &ConversionOptions,
    ctx: &Context,
    depth: usize,
    dom_ctx: &DomContext,
) {
    use crate::converter::walk_node;

    let content_start_pos = output.len();

    let is_table_continuation = ctx.in_table_cell
        && !output.is_empty()
        && !output.ends_with('|')
        && !output.ends_with("<br>");

    let is_list_continuation = ctx.in_list_item
        && !output.is_empty()
        && !output.ends_with("* ")
        && !output.ends_with("- ")
        && !output.ends_with(". ");

    let after_code_block = output.ends_with("```\n");
    let needs_leading_sep = !ctx.in_table_cell
        && !ctx.in_list_item
        && !ctx.convert_as_inline
        && ctx.blockquote_depth == 0
        && !output.is_empty()
        && !output.ends_with("\n\n")
        && !after_code_block;

    if is_table_continuation {
        crate::converter::trim_trailing_whitespace(output);
        output.push_str("<br>");
    } else if is_list_continuation {
        add_list_continuation_indent(output, ctx.list_depth, true, options);
    } else if needs_leading_sep {
        crate::converter::trim_trailing_whitespace(output);
        output.push_str("\n\n");
    }

    let p_ctx = Context {
        in_paragraph: true,
        block_content_start: output.len(),
        ..ctx.clone()
    };

    if let Some(node) = node_handle.get(parser) {
        if let tl::Node::Tag(tag) = node {
            let children = tag.children();
            let child_handles: Vec<_> = children.top().iter().collect();

            for (i, child_handle) in child_handles.iter().enumerate() {
                if let Some(node) = child_handle.get(parser) {
                    if let tl::Node::Raw(bytes) = node {
                        let text = bytes.as_utf8_str();
                        if text.trim().is_empty() && i > 0 && i < child_handles.len() - 1 {
                            let prev = &child_handles[i - 1];
                            let next = &child_handles[i + 1];
                            if is_empty_inline_element(prev, parser, dom_ctx)
                                && is_empty_inline_element(next, parser, dom_ctx)
                            {
                                continue;
                            }
                        }
                    }
                }

                walk_node(
                    child_handle,
                    parser,
                    output,
                    options,
                    &p_ctx,
                    depth + 1,
                    dom_ctx,
                );
            }
        }
    }

    let has_content = output.len() > content_start_pos;

    if has_content && !ctx.convert_as_inline && !ctx.in_table_cell {
        output.push_str("\n\n");
    }

    // Notify the structure collector if present and we produced non-empty top-level paragraph content.
    if has_content && !ctx.in_table_cell && !ctx.in_list_item && !ctx.convert_as_inline {
        if let Some(ref sc) = ctx.structure_collector {
            // An inline element's whitespace-normalisation pop can remove a byte from the
            // separator that was appended after `content_start_pos` was captured, leaving
            // `content_start_pos` pointing at the interior of a multibyte character.
            // Clamp to the nearest valid char boundary to avoid a slice panic (#380).
            let safe_start =
                crate::converter::utility::content::floor_char_boundary(output, content_start_pos);
            let text = output[safe_start..].trim().to_string();
            if !text.is_empty() {
                sc.borrow_mut().push_paragraph(&text);
            }
        }
    }
}

/// Add continuation indentation for list items.
fn add_list_continuation_indent(
    output: &mut String,
    list_depth: usize,
    needs_space: bool,
    _options: &ConversionOptions,
) {
    if needs_space && !output.ends_with(' ') && !output.ends_with('\n') {
        output.push(' ');
    }
    for _ in 0..(4 * list_depth) {
        output.push(' ');
    }
}

/// Check if an element is empty (has no text content).
fn is_empty_inline_element(
    node_handle: &NodeHandle,
    parser: &Parser,
    _dom_ctx: &DomContext,
) -> bool {
    if let Some(node) = node_handle.get(parser) {
        match node {
            tl::Node::Tag(tag) => {
                let tag_name = tag.name().as_utf8_str();
                // Elements that are always empty or only contain attributes
                matches!(
                    tag_name.as_ref(),
                    "br" | "hr" | "img" | "input" | "meta" | "link"
                )
            }
            _ => false,
        }
    } else {
        false
    }
}
