//! Output format renderers for HTML to Markdown/Djot conversion.
//!
//! This module provides format-specific rendering through the `FormatRenderer` trait,
//! allowing clean separation between Markdown and Djot output syntax.

mod djot;
mod markdown;

/// Trait for format-specific rendering of inline elements.
///
/// Implementations provide the syntax for emphasis, strong, strikethrough, etc.
/// in their respective output formats.
pub trait FormatRenderer: Send + Sync {
    /// Render emphasis (em, i elements)
    fn emphasis(&self, content: &str) -> String;

    /// Render strong emphasis (strong, b elements)
    fn strong(&self, content: &str, symbol: char) -> String;

    /// Render strikethrough (del, s elements)
    fn strikethrough(&self, content: &str) -> String;

    /// Render highlight (mark element)
    fn highlight(&self, content: &str) -> String;

    /// Render inserted text (ins element)
    fn inserted(&self, content: &str) -> String;

    /// Render subscript (sub element)
    fn subscript(&self, content: &str, custom_symbol: &str) -> String;

    /// Render superscript (sup element)
    fn superscript(&self, content: &str, custom_symbol: &str) -> String;

    /// Render span with attributes (for Djot: [text]{.class})
    fn span_with_attributes(&self, content: &str, classes: &[&str], id: Option<&str>) -> String;

    /// Render div with attributes (for Djot: ::: class)
    fn div_with_attributes(&self, content: &str, classes: &[&str]) -> String;

    /// Check if this is Djot format
    fn is_djot(&self) -> bool;
}
