//! DOM context providing efficient access to parent/child relationships and text content.
//!
//! This module defines the `DomContext` structure which is built once during conversion
//! and provides O(1) access to node relationships via precomputed maps. It also includes
//! a bounded cache for text content extraction to avoid redundant string allocations.

use std::cell::{OnceCell, RefCell};
use std::num::NonZeroUsize;

use crate::converter::main_helpers::is_inline_element;
use crate::converter::utility::content::{is_block_level_name, normalized_tag_name};
use crate::text;

#[derive(Clone, Copy)]
pub struct ChildRange {
    pub start: u32,
    pub len: u32,
}

pub enum TagName {
    Static(&'static str),
    Owned(String),
}

impl TagName {
    #[inline]
    fn from_raw(raw: std::borrow::Cow<'_, str>) -> Self {
        if let Some(name) = common_html_tag(raw.as_ref()) {
            return Self::Static(name);
        }

        match normalized_tag_name(raw) {
            std::borrow::Cow::Borrowed(name) => Self::Owned(name.to_string()),
            std::borrow::Cow::Owned(name) => {
                common_html_tag(&name).map_or_else(|| Self::Owned(name), Self::Static)
            }
        }
    }

    #[inline]
    pub(crate) fn as_str(&self) -> &str {
        match self {
            Self::Static(name) => name,
            Self::Owned(name) => name.as_str(),
        }
    }
}

impl PartialEq<&str> for TagName {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl AsRef<str> for TagName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[inline]
fn common_html_tag(raw: &str) -> Option<&'static str> {
    if raw.as_bytes().iter().any(u8::is_ascii_uppercase) {
        let mut buf = [0u8; 10];
        let bytes = raw.as_bytes();
        if bytes.len() > buf.len() || !bytes.is_ascii() {
            return None;
        }
        for (slot, byte) in buf.iter_mut().zip(bytes.iter().copied()) {
            *slot = byte.to_ascii_lowercase();
        }
        return std::str::from_utf8(&buf[..bytes.len()])
            .ok()
            .and_then(common_lowercase_html_tag);
    }

    common_lowercase_html_tag(raw)
}

#[inline]
fn common_lowercase_html_tag(raw: &str) -> Option<&'static str> {
    match raw {
        "a" => Some("a"),
        "b" => Some("b"),
        "i" => Some("i"),
        "p" => Some("p"),
        "s" => Some("s"),
        "u" => Some("u"),
        "br" => Some("br"),
        "dd" => Some("dd"),
        "dl" => Some("dl"),
        "dt" => Some("dt"),
        "em" => Some("em"),
        "h1" => Some("h1"),
        "h2" => Some("h2"),
        "h3" => Some("h3"),
        "h4" => Some("h4"),
        "h5" => Some("h5"),
        "h6" => Some("h6"),
        "hr" => Some("hr"),
        "li" => Some("li"),
        "ol" => Some("ol"),
        "rb" => Some("rb"),
        "rp" => Some("rp"),
        "rt" => Some("rt"),
        "td" => Some("td"),
        "th" => Some("th"),
        "tr" => Some("tr"),
        "ul" => Some("ul"),
        "div" => Some("div"),
        "dfn" => Some("dfn"),
        "img" => Some("img"),
        "ins" => Some("ins"),
        "kbd" => Some("kbd"),
        "nav" => Some("nav"),
        "pre" => Some("pre"),
        "rtc" => Some("rtc"),
        "sub" => Some("sub"),
        "sup" => Some("sup"),
        "var" => Some("var"),
        "wbr" => Some("wbr"),
        "abbr" => Some("abbr"),
        "body" => Some("body"),
        "code" => Some("code"),
        "data" => Some("data"),
        "form" => Some("form"),
        "head" => Some("head"),
        "html" => Some("html"),
        "link" => Some("link"),
        "main" => Some("main"),
        "mark" => Some("mark"),
        "ruby" => Some("ruby"),
        "samp" => Some("samp"),
        "span" => Some("span"),
        "time" => Some("time"),
        "aside" => Some("aside"),
        "small" => Some("small"),
        "style" => Some("style"),
        "table" => Some("table"),
        "tbody" => Some("tbody"),
        "tfoot" => Some("tfoot"),
        "thead" => Some("thead"),
        "figure" => Some("figure"),
        "footer" => Some("footer"),
        "header" => Some("header"),
        "iframe" => Some("iframe"),
        "script" => Some("script"),
        "source" => Some("source"),
        "strong" => Some("strong"),
        "address" => Some("address"),
        "article" => Some("article"),
        "caption" => Some("caption"),
        "details" => Some("details"),
        "section" => Some("section"),
        "summary" => Some("summary"),
        "blockquote" => Some("blockquote"),
        "figcaption" => Some("figcaption"),
        _ => None,
    }
}

/// Cached information about an HTML tag element.
///
/// This struct stores pre-computed information about tag elements to avoid
/// repeated parsing during tree traversal.
pub struct TagInfo {
    /// The normalized (lowercase) tag name.
    pub(crate) name: TagName,
    /// Whether this element behaves like an inline element (including script/style).
    pub(crate) is_inline_like: bool,
    /// Whether this element is a block-level element.
    pub(crate) is_block: bool,
}

pub(crate) struct TextCache {
    entries: Vec<(u32, String)>,
    capacity: usize,
}

impl TextCache {
    pub(crate) fn new(capacity: NonZeroUsize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity.get()),
            capacity: capacity.get(),
        }
    }

    pub(crate) fn get(&mut self, id: u32) -> Option<String> {
        let index = self.entries.iter().position(|(key, _)| *key == id)?;
        let (_, value) = self.entries.remove(index);
        let cloned = value.clone();
        self.entries.push((id, value));
        Some(cloned)
    }

    pub(crate) fn put(&mut self, id: u32, value: String) {
        if let Some(index) = self.entries.iter().position(|(key, _)| *key == id) {
            self.entries.remove(index);
        } else if self.entries.len() == self.capacity {
            self.entries.remove(0);
        }

        self.entries.push((id, value));
    }
}

/// DOM context that provides efficient access to parent/child relationships and text content.
///
/// This context is built once during conversion and provides O(1) access to node relationships
/// via precomputed maps. It also includes a bounded cache for text content extraction.
pub struct DomContext {
    pub(crate) parent_map: Vec<Option<u32>>,
    pub(crate) children_map: Vec<Option<ChildRange>>,
    pub(crate) child_handles: Vec<tl::NodeHandle>,
    pub(crate) sibling_index_map: Vec<Option<usize>>,
    pub(crate) root_children: Vec<tl::NodeHandle>,
    pub(crate) node_map: Vec<Option<tl::NodeHandle>>,
    pub(crate) tag_info_map: Vec<OnceCell<Option<TagInfo>>>,
    pub(crate) prev_inline_like_map: Vec<OnceCell<bool>>,
    pub(crate) next_inline_like_map: Vec<OnceCell<bool>>,
    pub(crate) next_tag_map: Vec<OnceCell<Option<u32>>>,
    pub(crate) next_whitespace_map: Vec<OnceCell<bool>>,
    pub(crate) text_cache: RefCell<TextCache>,
}

impl DomContext {
    #[inline]
    pub(crate) fn ensure_capacity(&mut self, id: u32) {
        let idx = id as usize;
        if self.parent_map.len() <= idx {
            let new_len = idx + 1;
            self.parent_map.resize(new_len, None);
            self.children_map.resize_with(new_len, || None);
            self.sibling_index_map.resize_with(new_len, || None);
            self.node_map.resize(new_len, None);
            self.tag_info_map.resize_with(new_len, OnceCell::new);
            self.prev_inline_like_map
                .resize_with(new_len, OnceCell::new);
            self.next_inline_like_map
                .resize_with(new_len, OnceCell::new);
            self.next_tag_map.resize_with(new_len, OnceCell::new);
            self.next_whitespace_map.resize_with(new_len, OnceCell::new);
        }
    }

    #[inline]
    pub(crate) fn parent_of(&self, id: u32) -> Option<u32> {
        self.parent_map.get(id as usize).copied().flatten()
    }

    #[inline]
    pub(crate) fn node_handle(&self, id: u32) -> Option<&tl::NodeHandle> {
        self.node_map
            .get(id as usize)
            .and_then(|node| node.as_ref())
    }

    #[inline]
    pub(crate) fn children_of(&self, id: u32) -> Option<&[tl::NodeHandle]> {
        let range = self.children_map.get(id as usize).and_then(|r| *r)?;
        let start = range.start as usize;
        let len = range.len as usize;
        self.child_handles.get(start..start + len)
    }

    #[inline]
    pub(crate) fn sibling_index(&self, id: u32) -> Option<usize> {
        self.sibling_index_map.get(id as usize).copied().flatten()
    }

    #[inline]
    pub(crate) fn tag_info(&self, id: u32, parser: &crate::tl_types::Parser) -> Option<&TagInfo> {
        self.tag_info_map.get(id as usize).and_then(|cell| {
            cell.get_or_init(|| self.build_tag_info(id, parser))
                .as_ref()
        })
    }

    #[inline]
    pub(crate) fn tag_name_for<'a>(
        &'a self,
        node_handle: tl::NodeHandle,
        parser: &'a crate::tl_types::Parser,
    ) -> Option<std::borrow::Cow<'a, str>> {
        if let Some(info) = self.tag_info(node_handle.get_inner(), parser) {
            return Some(std::borrow::Cow::Borrowed(info.name.as_str()));
        }
        if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
            return Some(normalized_tag_name(tag.name().as_utf8_str()));
        }
        None
    }

    #[inline]
    pub(crate) fn next_tag_name<'a>(
        &'a self,
        node_handle: tl::NodeHandle,
        parser: &'a crate::tl_types::Parser,
    ) -> Option<&'a str> {
        let next_id = self.next_tag_id(node_handle.get_inner(), parser)?;
        self.tag_info(next_id, parser)
            .map(|info| info.name.as_str())
    }

    pub(crate) fn previous_inline_like(
        &self,
        node_handle: tl::NodeHandle,
        parser: &crate::tl_types::Parser,
    ) -> bool {
        let id = node_handle.get_inner();
        self.prev_inline_like_map
            .get(id as usize)
            .is_some_and(|cell| {
                *cell.get_or_init(|| {
                    let parent = self.parent_of(id);
                    let siblings = if let Some(parent_id) = parent {
                        if let Some(children) = self.children_of(parent_id) {
                            children
                        } else {
                            return false;
                        }
                    } else {
                        &self.root_children
                    };

                    let Some(position) = self
                        .sibling_index(id)
                        .or_else(|| siblings.iter().position(|handle| handle.get_inner() == id))
                    else {
                        return false;
                    };

                    for sibling in siblings.iter().take(position).rev() {
                        if let Some(info) = self.tag_info(sibling.get_inner(), parser) {
                            return info.is_inline_like;
                        }
                        if let Some(tl::Node::Raw(raw)) = sibling.get(parser) {
                            if raw.as_utf8_str().trim().is_empty() {
                                continue;
                            }
                            return false;
                        }
                    }

                    false
                })
            })
    }

    pub(crate) fn next_inline_like(
        &self,
        node_handle: tl::NodeHandle,
        parser: &crate::tl_types::Parser,
    ) -> bool {
        let id = node_handle.get_inner();
        self.next_inline_like_map
            .get(id as usize)
            .is_some_and(|cell| {
                *cell.get_or_init(|| {
                    let parent = self.parent_of(id);
                    let siblings = if let Some(parent_id) = parent {
                        if let Some(children) = self.children_of(parent_id) {
                            children
                        } else {
                            return false;
                        }
                    } else {
                        &self.root_children
                    };

                    let Some(position) = self
                        .sibling_index(id)
                        .or_else(|| siblings.iter().position(|handle| handle.get_inner() == id))
                    else {
                        return false;
                    };

                    for sibling in siblings.iter().skip(position + 1) {
                        if let Some(info) = self.tag_info(sibling.get_inner(), parser) {
                            return info.is_inline_like;
                        }
                        if let Some(tl::Node::Raw(raw)) = sibling.get(parser) {
                            if raw.as_utf8_str().trim().is_empty() {
                                continue;
                            }
                            return false;
                        }
                    }

                    false
                })
            })
    }

    pub(crate) fn next_whitespace_text(
        &self,
        node_handle: tl::NodeHandle,
        parser: &crate::tl_types::Parser,
    ) -> bool {
        let id = node_handle.get_inner();
        self.next_whitespace_map
            .get(id as usize)
            .is_some_and(|cell| {
                *cell.get_or_init(|| {
                    let parent = self.parent_of(id);
                    let siblings = if let Some(parent_id) = parent {
                        if let Some(children) = self.children_of(parent_id) {
                            children
                        } else {
                            return false;
                        }
                    } else {
                        &self.root_children
                    };

                    let Some(position) = self
                        .sibling_index(id)
                        .or_else(|| siblings.iter().position(|handle| handle.get_inner() == id))
                    else {
                        return false;
                    };

                    for sibling in siblings.iter().skip(position + 1) {
                        if let Some(node) = sibling.get(parser) {
                            match node {
                                tl::Node::Raw(raw) => return raw.as_utf8_str().trim().is_empty(),
                                tl::Node::Tag(_) => return false,
                                tl::Node::Comment(_) => {}
                            }
                        }
                    }

                    false
                })
            })
    }

    pub(crate) fn next_tag_id(&self, id: u32, parser: &crate::tl_types::Parser) -> Option<u32> {
        self.next_tag_map
            .get(id as usize)
            .and_then(|cell| {
                cell.get_or_init(|| {
                    let parent = self.parent_of(id);
                    let siblings = if let Some(parent_id) = parent {
                        self.children_of(parent_id)?
                    } else {
                        &self.root_children
                    };

                    let position = self
                        .sibling_index(id)
                        .or_else(|| siblings.iter().position(|handle| handle.get_inner() == id))?;

                    for sibling in siblings.iter().skip(position + 1) {
                        if self.tag_info(sibling.get_inner(), parser).is_some() {
                            let sibling_id = sibling.get_inner();
                            return Some(sibling_id);
                        }
                        if let Some(tl::Node::Raw(raw)) = sibling.get(parser)
                            && !raw.as_utf8_str().trim().is_empty()
                        {
                            return None;
                        }
                    }
                    None
                })
                .as_ref()
            })
            .copied()
    }

    pub(crate) fn build_tag_info(
        &self,
        id: u32,
        parser: &crate::tl_types::Parser,
    ) -> Option<TagInfo> {
        let node_handle = self.node_handle(id)?;
        match node_handle.get(parser) {
            Some(tl::Node::Tag(tag)) => {
                let name = TagName::from_raw(tag.name().as_utf8_str());
                let is_inline = is_inline_element(name.as_str());
                let is_inline_like = is_inline || matches!(name.as_str(), "script" | "style");
                let is_block = is_block_level_name(name.as_str(), is_inline);
                Some(TagInfo {
                    name,
                    is_inline_like,
                    is_block,
                })
            }
            _ => None,
        }
    }

    pub(crate) fn text_content(
        &self,
        node_handle: tl::NodeHandle,
        parser: &crate::tl_types::Parser,
    ) -> String {
        let id = node_handle.get_inner();
        let cached = {
            let mut cache = self.text_cache.borrow_mut();
            cache.get(id)
        };
        if let Some(value) = cached {
            return value;
        }

        let value = self.text_content_uncached(node_handle, parser);
        self.text_cache.borrow_mut().put(id, value.clone());
        value
    }

    pub(crate) fn append_text_content(
        &self,
        node_handle: tl::NodeHandle,
        parser: &crate::tl_types::Parser,
        output: &mut String,
    ) {
        if let Some(value) = {
            let mut cache = self.text_cache.borrow_mut();
            cache.get(node_handle.get_inner())
        } {
            output.push_str(&value);
            return;
        }

        let start = output.len();
        self.append_text_content_uncached(node_handle, parser, output);
        if output.len() > start {
            self.text_cache
                .borrow_mut()
                .put(node_handle.get_inner(), output[start..].to_string());
        }
    }

    pub(crate) fn text_content_uncached(
        &self,
        node_handle: tl::NodeHandle,
        parser: &crate::tl_types::Parser,
    ) -> String {
        let mut text = String::with_capacity(64);
        self.append_text_content_uncached(node_handle, parser, &mut text);
        text
    }

    pub(crate) fn append_text_content_uncached(
        &self,
        node_handle: tl::NodeHandle,
        parser: &crate::tl_types::Parser,
        output: &mut String,
    ) {
        if let Some(node) = node_handle.get(parser) {
            match node {
                tl::Node::Raw(bytes) => {
                    let raw = bytes.as_utf8_str();
                    let decoded = text::decode_html_entities_cow(raw.as_ref());
                    output.push_str(decoded.as_ref());
                }
                tl::Node::Tag(tag) => {
                    let children = tag.children();
                    for child_handle in children.top().iter() {
                        self.append_text_content(*child_handle, parser, output);
                    }
                }
                tl::Node::Comment(_) => {}
            }
        }
    }

    /// Get the parent tag name for a given node ID.
    ///
    /// Returns the tag name of the parent element if it exists and is a tag,
    /// otherwise returns None.
    #[cfg_attr(not(feature = "visitor"), allow(dead_code))]
    pub(crate) fn parent_tag_name<'a>(
        &'a self,
        node_id: u32,
        parser: &'a crate::tl_types::Parser<'a>,
    ) -> Option<std::borrow::Cow<'a, str>> {
        let parent_id = self.parent_of(node_id)?;
        let parent_handle = self.node_handle(parent_id)?;

        if let Some(info) = self.tag_info(parent_id, parser) {
            return Some(std::borrow::Cow::Borrowed(info.name.as_str()));
        }

        if let Some(tl::Node::Tag(tag)) = parent_handle.get(parser) {
            let name = normalized_tag_name(tag.name().as_utf8_str());
            return Some(name);
        }

        None
    }

    /// Get the index of a node among its siblings.
    ///
    /// Returns the 0-based index if the node has siblings,
    /// otherwise returns None.
    #[cfg_attr(not(feature = "visitor"), allow(dead_code))]
    pub(crate) fn get_sibling_index(&self, node_id: u32) -> Option<usize> {
        self.sibling_index(node_id)
    }
}
