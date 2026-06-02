//! Content extraction and result handling.
//!
//! This module provides the `VisitorDispatch` enum and helper methods for
//! processing the results of visitor callbacks.

/// Result of dispatching a visitor callback.
///
/// This enum represents the outcome of a visitor callback dispatch,
/// providing a more ergonomic interface for control flow than the
/// raw `VisitResult` type.
#[allow(dead_code)]
#[derive(Debug)]
pub enum VisitorDispatch {
    /// Continue with default conversion behavior
    Continue,

    /// Replace default output with custom markdown
    Custom(String),

    /// Skip this element entirely (don't output anything)
    Skip,

    /// Preserve original HTML (don't convert to markdown)
    PreserveHtml,
}

impl VisitorDispatch {
    /// Check if this dispatch result indicates continuation.
    #[allow(dead_code)]
    #[inline]
    #[must_use]
    pub const fn is_continue(&self) -> bool {
        matches!(self, Self::Continue)
    }

    /// Check if this dispatch result contains custom output.
    #[allow(dead_code)]
    #[inline]
    #[must_use]
    pub const fn is_custom(&self) -> bool {
        matches!(self, Self::Custom(_))
    }

    /// Check if this dispatch result indicates skipping.
    #[allow(dead_code)]
    #[inline]
    #[must_use]
    pub const fn is_skip(&self) -> bool {
        matches!(self, Self::Skip)
    }

    /// Check if this dispatch result indicates HTML preservation.
    #[allow(dead_code)]
    #[inline]
    #[must_use]
    pub const fn is_preserve_html(&self) -> bool {
        matches!(self, Self::PreserveHtml)
    }

    /// Extract custom output if present.
    #[allow(dead_code)]
    #[inline]
    #[must_use]
    pub fn into_custom(self) -> Option<String> {
        match self {
            Self::Custom(output) => Some(output),
            _ => None,
        }
    }

    /// Extract custom output reference if present.
    #[allow(dead_code)]
    #[inline]
    #[must_use]
    pub fn as_custom(&self) -> Option<&str> {
        match self {
            Self::Custom(output) => Some(output),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visitor_dispatch_predicates() {
        let continue_dispatch = VisitorDispatch::Continue;
        assert!(continue_dispatch.is_continue());
        assert!(!continue_dispatch.is_custom());
        assert!(!continue_dispatch.is_skip());

        let custom_dispatch = VisitorDispatch::Custom("output".to_string());
        assert!(!custom_dispatch.is_continue());
        assert!(custom_dispatch.is_custom());
        assert!(!custom_dispatch.is_skip());

        let skip_dispatch = VisitorDispatch::Skip;
        assert!(!skip_dispatch.is_continue());
        assert!(!skip_dispatch.is_custom());
        assert!(skip_dispatch.is_skip());

        let preserve_dispatch = VisitorDispatch::PreserveHtml;
        assert!(!preserve_dispatch.is_continue());
        assert!(preserve_dispatch.is_preserve_html());
    }

    #[test]
    fn test_visitor_dispatch_into_custom() {
        let custom = VisitorDispatch::Custom("test".to_string());
        assert_eq!(custom.into_custom(), Some("test".to_string()));

        let continue_dispatch = VisitorDispatch::Continue;
        assert_eq!(continue_dispatch.into_custom(), None);
    }

    #[test]
    fn test_visitor_dispatch_as_custom() {
        let custom = VisitorDispatch::Custom("test".to_string());
        assert_eq!(custom.as_custom(), Some("test"));

        let skip = VisitorDispatch::Skip;
        assert_eq!(skip.as_custom(), None);
    }
}
