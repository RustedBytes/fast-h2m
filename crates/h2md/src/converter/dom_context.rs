//! DOM context providing efficient access to parent/child relationships and text content.
//!
//! This module defines the `DomContext` structure which is built once during conversion
//! and provides O(1) access to node relationships via precomputed maps. It also includes
//! an LRU cache for text content extraction to avoid redundant string allocations.

use lru::LruCache;
use std::cell::{OnceCell, RefCell};

use crate::converter::main_helpers::is_inline_element;
use crate::converter::utility::content::{is_block_level_name, normalized_tag_name};
use crate::text;

pub enum TagName {
    Static(&'static str),
    Owned(String),
}

impl TagName {
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

fn common_html_tag(raw: &str) -> Option<&'static str> {
    match raw.len() {
        1 if raw.eq_ignore_ascii_case("a") => Some("a"),
        1 if raw.eq_ignore_ascii_case("b") => Some("b"),
        1 if raw.eq_ignore_ascii_case("i") => Some("i"),
        1 if raw.eq_ignore_ascii_case("p") => Some("p"),
        1 if raw.eq_ignore_ascii_case("s") => Some("s"),
        1 if raw.eq_ignore_ascii_case("u") => Some("u"),
        2 if raw.eq_ignore_ascii_case("br") => Some("br"),
        2 if raw.eq_ignore_ascii_case("dd") => Some("dd"),
        2 if raw.eq_ignore_ascii_case("dl") => Some("dl"),
        2 if raw.eq_ignore_ascii_case("dt") => Some("dt"),
        2 if raw.eq_ignore_ascii_case("em") => Some("em"),
        2 if raw.eq_ignore_ascii_case("h1") => Some("h1"),
        2 if raw.eq_ignore_ascii_case("h2") => Some("h2"),
        2 if raw.eq_ignore_ascii_case("h3") => Some("h3"),
        2 if raw.eq_ignore_ascii_case("h4") => Some("h4"),
        2 if raw.eq_ignore_ascii_case("h5") => Some("h5"),
        2 if raw.eq_ignore_ascii_case("h6") => Some("h6"),
        2 if raw.eq_ignore_ascii_case("hr") => Some("hr"),
        2 if raw.eq_ignore_ascii_case("li") => Some("li"),
        2 if raw.eq_ignore_ascii_case("ol") => Some("ol"),
        2 if raw.eq_ignore_ascii_case("rb") => Some("rb"),
        2 if raw.eq_ignore_ascii_case("rp") => Some("rp"),
        2 if raw.eq_ignore_ascii_case("rt") => Some("rt"),
        2 if raw.eq_ignore_ascii_case("td") => Some("td"),
        2 if raw.eq_ignore_ascii_case("th") => Some("th"),
        2 if raw.eq_ignore_ascii_case("tr") => Some("tr"),
        2 if raw.eq_ignore_ascii_case("ul") => Some("ul"),
        3 if raw.eq_ignore_ascii_case("div") => Some("div"),
        3 if raw.eq_ignore_ascii_case("dfn") => Some("dfn"),
        3 if raw.eq_ignore_ascii_case("img") => Some("img"),
        3 if raw.eq_ignore_ascii_case("ins") => Some("ins"),
        3 if raw.eq_ignore_ascii_case("kbd") => Some("kbd"),
        3 if raw.eq_ignore_ascii_case("nav") => Some("nav"),
        3 if raw.eq_ignore_ascii_case("pre") => Some("pre"),
        3 if raw.eq_ignore_ascii_case("rtc") => Some("rtc"),
        3 if raw.eq_ignore_ascii_case("sub") => Some("sub"),
        3 if raw.eq_ignore_ascii_case("sup") => Some("sup"),
        3 if raw.eq_ignore_ascii_case("var") => Some("var"),
        3 if raw.eq_ignore_ascii_case("wbr") => Some("wbr"),
        4 if raw.eq_ignore_ascii_case("abbr") => Some("abbr"),
        4 if raw.eq_ignore_ascii_case("body") => Some("body"),
        4 if raw.eq_ignore_ascii_case("code") => Some("code"),
        4 if raw.eq_ignore_ascii_case("data") => Some("data"),
        4 if raw.eq_ignore_ascii_case("form") => Some("form"),
        4 if raw.eq_ignore_ascii_case("head") => Some("head"),
        4 if raw.eq_ignore_ascii_case("html") => Some("html"),
        4 if raw.eq_ignore_ascii_case("link") => Some("link"),
        4 if raw.eq_ignore_ascii_case("main") => Some("main"),
        4 if raw.eq_ignore_ascii_case("mark") => Some("mark"),
        4 if raw.eq_ignore_ascii_case("ruby") => Some("ruby"),
        4 if raw.eq_ignore_ascii_case("samp") => Some("samp"),
        4 if raw.eq_ignore_ascii_case("span") => Some("span"),
        4 if raw.eq_ignore_ascii_case("time") => Some("time"),
        5 if raw.eq_ignore_ascii_case("aside") => Some("aside"),
        5 if raw.eq_ignore_ascii_case("small") => Some("small"),
        5 if raw.eq_ignore_ascii_case("style") => Some("style"),
        5 if raw.eq_ignore_ascii_case("table") => Some("table"),
        5 if raw.eq_ignore_ascii_case("tbody") => Some("tbody"),
        5 if raw.eq_ignore_ascii_case("tfoot") => Some("tfoot"),
        5 if raw.eq_ignore_ascii_case("thead") => Some("thead"),
        6 if raw.eq_ignore_ascii_case("figure") => Some("figure"),
        6 if raw.eq_ignore_ascii_case("footer") => Some("footer"),
        6 if raw.eq_ignore_ascii_case("header") => Some("header"),
        6 if raw.eq_ignore_ascii_case("iframe") => Some("iframe"),
        6 if raw.eq_ignore_ascii_case("script") => Some("script"),
        6 if raw.eq_ignore_ascii_case("source") => Some("source"),
        6 if raw.eq_ignore_ascii_case("strong") => Some("strong"),
        7 if raw.eq_ignore_ascii_case("address") => Some("address"),
        7 if raw.eq_ignore_ascii_case("article") => Some("article"),
        7 if raw.eq_ignore_ascii_case("caption") => Some("caption"),
        7 if raw.eq_ignore_ascii_case("details") => Some("details"),
        7 if raw.eq_ignore_ascii_case("section") => Some("section"),
        7 if raw.eq_ignore_ascii_case("summary") => Some("summary"),
        10 if raw.eq_ignore_ascii_case("blockquote") => Some("blockquote"),
        10 if raw.eq_ignore_ascii_case("figcaption") => Some("figcaption"),
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

/// DOM context that provides efficient access to parent/child relationships and text content.
///
/// This context is built once during conversion and provides O(1) access to node relationships
/// via precomputed maps. It also includes an LRU cache for text content extraction.
pub struct DomContext {
    pub(crate) parent_map: Vec<Option<u32>>,
    pub(crate) children_map: Vec<Option<Vec<tl::NodeHandle>>>,
    pub(crate) sibling_index_map: Vec<Option<usize>>,
    pub(crate) root_children: Vec<tl::NodeHandle>,
    pub(crate) node_map: Vec<Option<tl::NodeHandle>>,
    pub(crate) tag_info_map: Vec<OnceCell<Option<TagInfo>>>,
    pub(crate) prev_inline_like_map: Vec<OnceCell<bool>>,
    pub(crate) next_inline_like_map: Vec<OnceCell<bool>>,
    pub(crate) next_tag_map: Vec<OnceCell<Option<u32>>>,
    pub(crate) next_whitespace_map: Vec<OnceCell<bool>>,
    pub(crate) text_cache: RefCell<LruCache<u32, String>>,
}

impl DomContext {
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

    pub(crate) fn parent_of(&self, id: u32) -> Option<u32> {
        self.parent_map.get(id as usize).copied().flatten()
    }

    pub(crate) fn node_handle(&self, id: u32) -> Option<&tl::NodeHandle> {
        self.node_map
            .get(id as usize)
            .and_then(|node| node.as_ref())
    }

    pub(crate) fn children_of(&self, id: u32) -> Option<&Vec<tl::NodeHandle>> {
        self.children_map
            .get(id as usize)
            .and_then(|children| children.as_ref())
    }

    pub(crate) fn sibling_index(&self, id: u32) -> Option<usize> {
        self.sibling_index_map.get(id as usize).copied().flatten()
    }

    pub(crate) fn tag_info(&self, id: u32, parser: &tl::Parser) -> Option<&TagInfo> {
        self.tag_info_map.get(id as usize).and_then(|cell| {
            cell.get_or_init(|| self.build_tag_info(id, parser))
                .as_ref()
        })
    }

    pub(crate) fn tag_name_for<'a>(
        &'a self,
        node_handle: tl::NodeHandle,
        parser: &'a tl::Parser,
    ) -> Option<std::borrow::Cow<'a, str>> {
        if let Some(info) = self.tag_info(node_handle.get_inner(), parser) {
            return Some(std::borrow::Cow::Borrowed(info.name.as_str()));
        }
        if let Some(tl::Node::Tag(tag)) = node_handle.get(parser) {
            return Some(normalized_tag_name(tag.name().as_utf8_str()));
        }
        None
    }

    pub(crate) fn next_tag_name<'a>(
        &'a self,
        node_handle: tl::NodeHandle,
        parser: &'a tl::Parser,
    ) -> Option<&'a str> {
        let next_id = self.next_tag_id(node_handle.get_inner(), parser)?;
        self.tag_info(next_id, parser)
            .map(|info| info.name.as_str())
    }

    pub(crate) fn previous_inline_like(
        &self,
        node_handle: tl::NodeHandle,
        parser: &tl::Parser,
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
        parser: &tl::Parser,
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
        parser: &tl::Parser,
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

    pub(crate) fn next_tag_id(&self, id: u32, parser: &tl::Parser) -> Option<u32> {
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
                        if let Some(tl::Node::Raw(raw)) = sibling.get(parser) {
                            if !raw.as_utf8_str().trim().is_empty() {
                                return None;
                            }
                        }
                    }
                    None
                })
                .as_ref()
            })
            .copied()
    }

    pub(crate) fn build_tag_info(&self, id: u32, parser: &tl::Parser) -> Option<TagInfo> {
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

    pub(crate) fn text_content(&self, node_handle: tl::NodeHandle, parser: &tl::Parser) -> String {
        let id = node_handle.get_inner();
        let cached = {
            let mut cache = self.text_cache.borrow_mut();
            cache.get(&id).cloned()
        };
        if let Some(value) = cached {
            return value;
        }

        let value = self.text_content_uncached(node_handle, parser);
        self.text_cache.borrow_mut().put(id, value.clone());
        value
    }

    pub(crate) fn text_content_uncached(
        &self,
        node_handle: tl::NodeHandle,
        parser: &tl::Parser,
    ) -> String {
        let mut text = String::with_capacity(64);
        if let Some(node) = node_handle.get(parser) {
            match node {
                tl::Node::Raw(bytes) => {
                    let raw = bytes.as_utf8_str();
                    let decoded = text::decode_html_entities_cow(raw.as_ref());
                    text.push_str(decoded.as_ref());
                }
                tl::Node::Tag(tag) => {
                    let children = tag.children();
                    for child_handle in children.top().iter() {
                        text.push_str(&self.text_content(*child_handle, parser));
                    }
                }
                tl::Node::Comment(_) => {}
            }
        }
        text
    }

    /// Get the parent tag name for a given node ID.
    ///
    /// Returns the tag name of the parent element if it exists and is a tag,
    /// otherwise returns None.
    #[cfg_attr(not(feature = "visitor"), allow(dead_code))]
    pub(crate) fn parent_tag_name(&self, node_id: u32, parser: &tl::Parser) -> Option<String> {
        let parent_id = self.parent_of(node_id)?;
        let parent_handle = self.node_handle(parent_id)?;

        if let Some(info) = self.tag_info(parent_id, parser) {
            return Some(info.name.as_str().to_string());
        }

        if let Some(tl::Node::Tag(tag)) = parent_handle.get(parser) {
            let name = normalized_tag_name(tag.name().as_utf8_str());
            return Some(name.into_owned());
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
