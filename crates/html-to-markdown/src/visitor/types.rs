//! Type definitions for the visitor pattern.
//!
//! This module contains the core data types used in the visitor pattern:
//! - `NodeType`: Categorization of HTML elements
//! - `NodeContext`: Metadata about the current node being visited
//! - `VisitResult`: Control flow signals from visitor methods

use std::collections::BTreeMap;

/// Node type enumeration covering all HTML element types.
///
/// This enum categorizes all HTML elements that the converter recognizes,
/// providing a coarse-grained classification for visitor dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NodeType {
    /// Text node (most frequent - 100+ per document)
    Text,

    /// Generic element node
    Element,

    /// Heading elements (h1-h6)
    Heading,
    /// Paragraph element
    Paragraph,
    /// Generic div container
    Div,
    /// Blockquote element
    Blockquote,
    /// Preformatted text block
    Pre,
    /// Horizontal rule
    Hr,

    /// Ordered or unordered list (ul, ol)
    List,
    /// List item (li)
    ListItem,
    /// Definition list (dl)
    DefinitionList,
    /// Definition term (dt)
    DefinitionTerm,
    /// Definition description (dd)
    DefinitionDescription,

    /// Table element
    Table,
    /// Table row (tr)
    TableRow,
    /// Table cell (td, th)
    TableCell,
    /// Table header cell (th)
    TableHeader,
    /// Table body (tbody)
    TableBody,
    /// Table head (thead)
    TableHead,
    /// Table foot (tfoot)
    TableFoot,

    /// Anchor link (a)
    Link,
    /// Image (img)
    Image,
    /// Strong/bold (strong, b)
    Strong,
    /// Emphasis/italic (em, i)
    Em,
    /// Inline code (code)
    Code,
    /// Strikethrough (s, del, strike)
    Strikethrough,
    /// Underline (u, ins)
    Underline,
    /// Subscript (sub)
    Subscript,
    /// Superscript (sup)
    Superscript,
    /// Mark/highlight (mark)
    Mark,
    /// Small text (small)
    Small,
    /// Line break (br)
    Br,
    /// Span element
    Span,

    /// Article element
    Article,
    /// Section element
    Section,
    /// Navigation element
    Nav,
    /// Aside element
    Aside,
    /// Header element
    Header,
    /// Footer element
    Footer,
    /// Main element
    Main,
    /// Figure element
    Figure,
    /// Figure caption
    Figcaption,
    /// Time element
    Time,
    /// Details element
    Details,
    /// Summary element
    Summary,

    /// Form element
    Form,
    /// Input element
    Input,
    /// Select element
    Select,
    /// Option element
    Option,
    /// Button element
    Button,
    /// Textarea element
    Textarea,
    /// Label element
    Label,
    /// Fieldset element
    Fieldset,
    /// Legend element
    Legend,

    /// Audio element
    Audio,
    /// Video element
    Video,
    /// Picture element
    Picture,
    /// Source element
    Source,
    /// Iframe element
    Iframe,
    /// SVG element
    Svg,
    /// Canvas element
    Canvas,

    /// Ruby annotation
    Ruby,
    /// Ruby text
    Rt,
    /// Ruby parenthesis
    Rp,
    /// Abbreviation
    Abbr,
    /// Keyboard input
    Kbd,
    /// Sample output
    Samp,
    /// Variable
    Var,
    /// Citation
    Cite,
    /// Quote
    Q,
    /// Deleted text
    Del,
    /// Inserted text
    Ins,
    /// Data element
    Data,
    /// Meter element
    Meter,
    /// Progress element
    Progress,
    /// Output element
    Output,
    /// Template element
    Template,
    /// Slot element
    Slot,

    /// HTML root element
    Html,
    /// Head element
    Head,
    /// Body element
    Body,
    /// Title element
    Title,
    /// Meta element
    Meta,
    /// Link element (not anchor)
    LinkTag,
    /// Style element
    Style,
    /// Script element
    Script,
    /// Base element
    Base,

    /// Custom element (web components) or unknown tag
    Custom,
}

/// Context information passed to all visitor methods.
///
/// Provides comprehensive metadata about the current node being visited,
/// including its type, attributes, position in the DOM tree, and parent context.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeContext {
    /// Coarse-grained node type classification
    pub node_type: NodeType,

    /// Raw HTML tag name (e.g., "div", "h1", "custom-element")
    pub tag_name: String,

    /// All HTML attributes as key-value pairs
    pub attributes: BTreeMap<String, String>,

    /// Depth in the DOM tree (0 = root)
    pub depth: usize,

    /// Index among siblings (0-based)
    pub index_in_parent: usize,

    /// Parent element's tag name (None if root)
    pub parent_tag: Option<String>,

    /// Whether this element is treated as inline vs block
    pub is_inline: bool,
}

/// Result of a visitor callback.
///
/// Allows visitors to control the conversion flow by either proceeding
/// with default behavior, providing custom output, skipping elements,
/// preserving HTML, or signaling errors.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum VisitResult {
    #[default]
    /// Continue with default conversion behavior
    Continue,

    /// Replace default output with custom markdown
    ///
    /// The visitor takes full responsibility for the markdown output
    /// of this node and its children.
    Custom(String),

    /// Skip this element entirely (don't output anything)
    ///
    /// The element and all its children are ignored in the output.
    Skip,

    /// Preserve original HTML (don't convert to markdown)
    ///
    /// The element's raw HTML is included verbatim in the output.
    PreserveHtml,

    /// Stop conversion with an error
    ///
    /// The conversion process halts and returns this error message.
    Error(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_type_equality() {
        assert_eq!(NodeType::Text, NodeType::Text);
        assert_ne!(NodeType::Text, NodeType::Element);
        assert_eq!(NodeType::Heading, NodeType::Heading);
    }

    #[test]
    fn test_node_context_creation() {
        let ctx = NodeContext {
            node_type: NodeType::Heading,
            tag_name: "h1".to_string(),
            attributes: BTreeMap::new(),
            depth: 1,
            index_in_parent: 0,
            parent_tag: Some("body".to_string()),
            is_inline: false,
        };

        assert_eq!(ctx.node_type, NodeType::Heading);
        assert_eq!(ctx.tag_name, "h1");
        assert_eq!(ctx.depth, 1);
        assert!(!ctx.is_inline);
    }

    #[test]
    fn test_visit_result_variants() {
        let continue_result = VisitResult::Continue;
        matches!(continue_result, VisitResult::Continue);

        let custom_result = VisitResult::Custom("# Custom Output".to_string());
        if let VisitResult::Custom(output) = custom_result {
            assert_eq!(output, "# Custom Output");
        }

        let skip_result = VisitResult::Skip;
        matches!(skip_result, VisitResult::Skip);

        let preserve_result = VisitResult::PreserveHtml;
        matches!(preserve_result, VisitResult::PreserveHtml);

        let error_result = VisitResult::Error("Test error".to_string());
        if let VisitResult::Error(msg) = error_result {
            assert_eq!(msg, "Test error");
        }
    }
}
