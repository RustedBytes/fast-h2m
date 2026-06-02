//! Text wrapping functionality for Markdown output.
//!
//! This module provides text wrapping capabilities similar to Python's `textwrap.fill()`,
//! specifically designed to work with Markdown content while preserving formatting.

pub use sync::wrap_markdown;

mod sync;
mod utils;
