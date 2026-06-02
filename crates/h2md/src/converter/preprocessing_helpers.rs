//! HTML preprocessing and validation helpers.
//!
//! This module contains helper functions for preprocessing HTML before conversion,
//! including validation and normalization checks.

use crate::converter::dom_context::DomContext;
use crate::converter::main_helpers::is_inline_element;
use crate::converter::utility::attributes::{attribute_matches_any, element_has_navigation_hint};
use crate::converter::utility::content::normalized_tag_name;
use crate::options::ConversionOptions;

/// Check if an inline ancestor element is allowed to contain block-level elements.
pub fn inline_ancestor_allows_block(tag_name: &str) -> bool {
    matches!(tag_name, "a" | "ins" | "del")
}

/// Detect block elements that were incorrectly nested under inline ancestors.
///
/// Excludes elements inside `<pre>` or `<code>` blocks, as they have special
/// whitespace preservation rules and should not be repaired.
///
/// Also detects table structural elements (`td`, `tr`, `th`) nested under `<p>` —
/// a structural impossibility in valid HTML that signals the `tl` parser absorbed
/// a table into a paragraph because of an unclosed `<p>` (common in Word/Outlook
/// HTML such as `<p class='MsoNormal'>` cells). Issue #336.
pub fn has_inline_block_misnest(dom_ctx: &DomContext, parser: &tl::Parser) -> bool {
    for handle in dom_ctx.node_map.iter().flatten() {
        if let Some(tl::Node::Tag(_tag)) = handle.get(parser) {
            let node_id = handle.get_inner();
            let Some(info) = dom_ctx.tag_info(node_id, parser) else {
                continue;
            };

            // Table elements under <p>: tl misparsed an unclosed <p> in <td>.
            if matches!(info.name.as_str(), "td" | "tr" | "th")
                && has_p_ancestor(dom_ctx, parser, node_id)
            {
                return true;
            }

            if !info.is_block {
                continue;
            }

            // Check if this block element or any ancestor is pre/code
            let mut check_parent = Some(node_id);
            let mut inside_preformatted = false;
            while let Some(check_id) = check_parent {
                if let Some(info) = dom_ctx.tag_info(check_id, parser) {
                    if matches!(info.name.as_str(), "pre" | "code") {
                        inside_preformatted = true;
                        break;
                    }
                }
                check_parent = dom_ctx.parent_of(check_id);
            }

            // Skip misnesting check for elements inside pre/code blocks
            if inside_preformatted {
                continue;
            }

            let mut current = dom_ctx.parent_of(node_id);
            while let Some(parent_id) = current {
                if let Some(parent_info) = dom_ctx.tag_info(parent_id, parser) {
                    if is_inline_element(parent_info.name.as_str())
                        && !inline_ancestor_allows_block(parent_info.name.as_str())
                    {
                        return true;
                    }
                } else if let Some(parent_handle) = dom_ctx.node_handle(parent_id) {
                    if let Some(tl::Node::Tag(parent_tag)) = parent_handle.get(parser) {
                        let parent_name = normalized_tag_name(parent_tag.name().as_utf8_str());
                        if is_inline_element(&parent_name)
                            && !inline_ancestor_allows_block(&parent_name)
                        {
                            return true;
                        }
                    }
                }
                current = dom_ctx.parent_of(parent_id);
            }
        }
    }

    false
}

/// Walk ancestors of `node_id` looking for a `<p>` element.
///
/// Stops ascending once it leaves the table hierarchy (`table`/`body`/`html`)
/// to avoid false positives where a `<p>` legitimately wraps a `<table>`.
fn has_p_ancestor(dom_ctx: &DomContext, parser: &tl::Parser, node_id: u32) -> bool {
    let mut current = dom_ctx.parent_of(node_id);
    while let Some(parent_id) = current {
        if let Some(parent_info) = dom_ctx.tag_info(parent_id, parser) {
            if parent_info.name == "p" {
                return true;
            }
            if matches!(parent_info.name.as_str(), "table" | "body" | "html") {
                return false;
            }
        }
        current = dom_ctx.parent_of(parent_id);
    }
    false
}

/// Determine if a node should be dropped during preprocessing.
///
/// Behavior depends on the [`PreprocessingPreset`]:
///
/// - **Minimal**: Only scripts/styles are stripped (handled elsewhere). This function
///   drops nothing — all structural elements are preserved.
/// - **Standard** (default): Drops `<nav>` unconditionally. Drops `<header>`, `<footer>`,
///   and `<aside>` only when they have navigation hints (class/role/aria attributes
///   indicating site chrome). Drops `<form>` when `remove_forms` is enabled.
/// - **Aggressive**: All of Standard, plus: drops `<footer>`, `<aside>`, `<noscript>`
///   unconditionally. Drops ANY element with navigation hints in class/id/role
///   (e.g. `<div class="sidebar">`). Drops elements with noise-related classes/roles.
pub fn should_drop_for_preprocessing(
    tag_name: &str,
    tag: &tl::HTMLTag,
    options: &ConversionOptions,
) -> bool {
    use crate::options::PreprocessingPreset;

    if !options.preprocessing.enabled {
        return false;
    }

    let preset = options.preprocessing.preset;

    // Minimal preset: drop nothing here (scripts/styles handled in earlier pipeline stage).
    if preset == PreprocessingPreset::Minimal {
        return false;
    }

    // Form removal — applies to both Standard and Aggressive when enabled.
    if options.preprocessing.remove_forms && tag_name == "form" {
        return true;
    }

    let is_aggressive = preset == PreprocessingPreset::Aggressive;

    // Aggressive: drop <noscript> — its content is fallback for no-JS browsers.
    if is_aggressive && tag_name == "noscript" {
        return true;
    }

    // Navigation removal — only when the flag is enabled.
    if !options.preprocessing.remove_navigation {
        return false;
    }

    let has_nav_hint = element_has_navigation_hint(tag);

    // <nav> is always navigation — drop in both Standard and Aggressive.
    if tag_name == "nav" {
        return true;
    }

    if tag_name == "header" {
        // Drop <header> only with navigation hints (e.g. class="site-header",
        // role="navigation"). A plain <header> often wraps article titles like
        // <header><h1>Title</h1></header> — dropping it loses content.
        return has_nav_hint;
    }

    if tag_name == "footer" || tag_name == "aside" {
        // Standard: drop only with navigation hints.
        // Aggressive: drop unconditionally.
        return is_aggressive || has_nav_hint;
    }

    // Aggressive: drop ANY element that has navigation hints in class/id/role.
    // This catches <div class="sidebar">, <div class="menu">, <section class="navigation">,
    // and similar non-semantic navigation containers.
    if is_aggressive && has_nav_hint {
        return true;
    }

    // Aggressive: drop elements with noise-related roles.
    if is_aggressive {
        if element_has_noise_hint(tag) {
            return true;
        }
    }

    false
}

/// Check if an element has noise-related hints (ads, cookie banners, social sharing).
fn element_has_noise_hint(tag: &tl::HTMLTag) -> bool {
    const NOISE_KEYWORDS: &[&str] = &[
        "cookie",
        "consent",
        "gdpr",
        "banner",
        "advertisement",
        "ad-container",
        "advert",
        "social-share",
        "share-buttons",
        "popup",
        "modal-overlay",
        "newsletter-signup",
    ];

    attribute_matches_any(tag, "class", NOISE_KEYWORDS)
        || attribute_matches_any(tag, "id", NOISE_KEYWORDS)
}
