//! Djot format renderer.

use super::FormatRenderer;

/// Renderer for Djot lightweight markup output.
#[derive(Debug, Clone, Copy, Default)]
pub struct DjotRenderer;

impl FormatRenderer for DjotRenderer {
    fn emphasis(&self, content: &str) -> String {
        format!("_{content}_")
    }

    fn strong(&self, content: &str, _symbol: char) -> String {
        // Djot always uses single asterisk for strong
        format!("*{content}*")
    }

    fn strikethrough(&self, content: &str) -> String {
        format!("{{-{content}-}}")
    }

    fn highlight(&self, content: &str) -> String {
        format!("{{={content}=}}")
    }

    fn inserted(&self, content: &str) -> String {
        format!("{{+{content}+}}")
    }

    fn subscript(&self, content: &str, _custom_symbol: &str) -> String {
        // Djot has native subscript support
        format!("~{content}~")
    }

    fn superscript(&self, content: &str, _custom_symbol: &str) -> String {
        // Djot has native superscript support
        format!("^{content}^")
    }

    fn span_with_attributes(&self, content: &str, classes: &[&str], id: Option<&str>) -> String {
        if classes.is_empty() && id.is_none() {
            return content.to_string();
        }

        let mut attrs = classes.iter().map(|c| format!(".{c}")).collect::<Vec<_>>();
        if let Some(id_val) = id {
            attrs.push(format!("#{id_val}"));
        }
        format!("[{content}]{{{}}}", attrs.join(" "))
    }

    fn div_with_attributes(&self, content: &str, classes: &[&str]) -> String {
        if classes.is_empty() {
            return content.to_string();
        }
        let class_str = classes.join(" ");
        format!("::: {class_str}\n{content}\n:::")
    }

    fn is_djot(&self) -> bool {
        true
    }
}
