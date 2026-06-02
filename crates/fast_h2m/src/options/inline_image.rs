//! Inline image configuration.
//!
//! This module provides configuration for controlling how images are rendered
//! within specific HTML elements.

/// Inline image configuration that specifies contexts where images remain as markdown links.
///
/// This is a wrapper type that provides semantic clarity for the vector of element
/// names where inline images should be preserved.
#[derive(Debug, Clone)]
pub struct InlineImageConfig {
    /// HTML elements where images should remain as markdown links (not converted to alt text)
    pub keep_inline_images_in: Vec<String>,
}

impl InlineImageConfig {
    /// Create a new inline image configuration with an empty list.
    #[must_use]
    pub fn new() -> Self {
        Self {
            keep_inline_images_in: Vec::new(),
        }
    }

    /// Create a new inline image configuration from a list of element names.
    ///
    /// # Arguments
    ///
    /// * `elements` - A vector of HTML element names where inline images should be kept
    #[must_use]
    pub fn from_elements(elements: Vec<String>) -> Self {
        Self {
            keep_inline_images_in: elements,
        }
    }

    /// Add an element name to the list of elements where images are kept inline.
    ///
    /// # Arguments
    ///
    /// * `element` - The HTML element name to add (e.g., "p", "div")
    pub fn add_element(&mut self, element: String) {
        self.keep_inline_images_in.push(element);
    }

    /// Check if a given element should keep images inline.
    ///
    /// # Arguments
    ///
    /// * `element` - The HTML element name to check
    ///
    /// # Returns
    ///
    /// `true` if the element is in the configured list, `false` otherwise
    #[must_use]
    pub fn should_keep_images(&self, element: &str) -> bool {
        self.keep_inline_images_in.iter().any(|e| e == element)
    }
}

impl Default for InlineImageConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_image_config_new() {
        let config = InlineImageConfig::new();
        assert_eq!(config.keep_inline_images_in.len(), 0);
    }

    #[test]
    fn test_inline_image_config_from_elements() {
        let elements = vec!["p".to_string(), "div".to_string()];
        let config = InlineImageConfig::from_elements(elements);
        assert_eq!(config.keep_inline_images_in.len(), 2);
        assert!(config.should_keep_images("p"));
        assert!(config.should_keep_images("div"));
        assert!(!config.should_keep_images("span"));
    }

    #[test]
    fn test_inline_image_config_add_element() {
        let mut config = InlineImageConfig::new();
        config.add_element("p".to_string());
        config.add_element("div".to_string());

        assert_eq!(config.keep_inline_images_in.len(), 2);
        assert!(config.should_keep_images("p"));
        assert!(config.should_keep_images("div"));
    }

    #[test]
    fn test_inline_image_config_should_keep_images() {
        let config = InlineImageConfig::from_elements(vec!["figure".to_string()]);
        assert!(config.should_keep_images("figure"));
        assert!(!config.should_keep_images("p"));
    }

    #[test]
    fn test_inline_image_config_default() {
        let config = InlineImageConfig::default();
        assert_eq!(config.keep_inline_images_in.len(), 0);
        assert!(!config.should_keep_images("p"));
    }
}
