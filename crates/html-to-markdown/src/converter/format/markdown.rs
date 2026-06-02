//! Markdown format renderer.

use super::FormatRenderer;

/// Renderer for standard Markdown output.
#[derive(Debug, Clone, Copy, Default)]
pub struct MarkdownRenderer;

impl FormatRenderer for MarkdownRenderer {
    fn emphasis(&self, content: &str) -> String {
        format!("*{content}*")
    }

    fn strong(&self, content: &str, symbol: char) -> String {
        format!("{symbol}{symbol}{content}{symbol}{symbol}")
    }

    fn strikethrough(&self, content: &str) -> String {
        format!("~~{content}~~")
    }

    fn highlight(&self, content: &str) -> String {
        format!("=={content}==")
    }

    fn inserted(&self, content: &str) -> String {
        format!("++{content}++")
    }

    fn subscript(&self, content: &str, custom_symbol: &str) -> String {
        if custom_symbol.is_empty() {
            format!("~{content}~")
        } else {
            format!("{custom_symbol}{content}{custom_symbol}")
        }
    }

    fn superscript(&self, content: &str, custom_symbol: &str) -> String {
        if custom_symbol.is_empty() {
            format!("^{content}^")
        } else {
            format!("{custom_symbol}{content}{custom_symbol}")
        }
    }

    fn span_with_attributes(&self, content: &str, _classes: &[&str], _id: Option<&str>) -> String {
        // Markdown doesn't support span attributes, just return content
        content.to_string()
    }

    fn div_with_attributes(&self, content: &str, _classes: &[&str]) -> String {
        // Markdown doesn't support div attributes, just return content
        content.to_string()
    }

    fn is_djot(&self) -> bool {
        false
    }
}
