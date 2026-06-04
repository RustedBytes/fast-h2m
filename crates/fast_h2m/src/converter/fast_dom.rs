//! Lean DOM conversion path for throughput-oriented callers.
//!
//! This module intentionally implements a smaller HTML-to-Markdown surface than
//! the full Tier-2 converter. It uses `tl` for parsing, walks the DOM directly,
//! and avoids the full `DomContext`, collectors, visitor hooks, repair probes,
//! and final whole-output cleanup passes.

use std::borrow::Cow;

use crate::error::{ConversionError, Result};
use crate::options::{CodeBlockStyle, ConversionOptions, HeadingStyle, NewlineStyle};
use crate::text;

#[derive(Clone, Copy, Default)]
struct FastState {
    in_pre: bool,
    in_code: bool,
    inline: bool,
    list_depth: usize,
    ordered_list: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TagKind {
    Passthrough,
    Skip,
    Heading(usize),
    Paragraph,
    Break,
    Strong,
    Emphasis,
    Code,
    Pre,
    Link,
    Image,
    UnorderedList,
    OrderedList,
    ListItem,
    Blockquote,
    HorizontalRule,
    Table,
    TableSection,
    TableRow,
    TableCell,
}

pub fn convert(html: &str, options: &ConversionOptions) -> Result<String> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ConversionError::ParseError("Failed to parse HTML".to_string()))?;
    let parser = dom.parser();
    let mut output = String::with_capacity(html.len() / 2);
    let state = FastState::default();

    for child in dom.children() {
        walk_node(child, parser, &mut output, options, state);
    }

    trim_document_boundaries(&mut output);
    Ok(output)
}

fn walk_node(
    handle: &tl::NodeHandle,
    parser: &crate::tl_types::Parser,
    output: &mut String,
    options: &ConversionOptions,
    state: FastState,
) {
    let Some(node) = handle.get(parser) else {
        return;
    };

    match node {
        tl::Node::Raw(raw) => {
            let raw = raw.as_utf8_str();
            push_text(raw.as_ref(), output, options, state);
        }
        tl::Node::Comment(_) => {}
        tl::Node::Tag(tag) => {
            let name = tag.name().as_utf8_str();
            match classify_tag(name.as_ref()) {
                TagKind::Passthrough => {
                    walk_children(tag, parser, output, options, state);
                }
                TagKind::Skip => {}
                TagKind::Heading(level) => {
                    handle_heading(level, tag, parser, output, options, state);
                }
                TagKind::Paragraph => {
                    ensure_block_start(output, state);
                    let start = output.len();
                    walk_children(tag, parser, output, options, state);
                    trim_trailing_spaces(output);
                    if output.len() > start {
                        push_block_end(output, state);
                    }
                }
                TagKind::Break => match options.newline_style {
                    NewlineStyle::Backslash => output.push_str("\\\n"),
                    NewlineStyle::Spaces => output.push_str("  \n"),
                },
                TagKind::Strong => handle_wrapped(tag, parser, output, options, state, "**"),
                TagKind::Emphasis => handle_wrapped(tag, parser, output, options, state, "*"),
                TagKind::Code => {
                    if state.in_pre {
                        walk_children(
                            tag,
                            parser,
                            output,
                            options,
                            FastState {
                                in_code: true,
                                ..state
                            },
                        );
                    } else {
                        output.push('`');
                        let start = output.len();
                        walk_children(
                            tag,
                            parser,
                            output,
                            options,
                            FastState {
                                in_code: true,
                                inline: true,
                                ..state
                            },
                        );
                        trim_code_span(output, start);
                        output.push('`');
                    }
                }
                TagKind::Pre => handle_pre(tag, parser, output, options, state),
                TagKind::Link => handle_link(tag, parser, output, options, state),
                TagKind::Image => handle_img(tag, output, options, state),
                TagKind::UnorderedList => handle_list(tag, parser, output, options, state, false),
                TagKind::OrderedList => handle_list(tag, parser, output, options, state, true),
                TagKind::ListItem => handle_list_item(tag, parser, output, options, state),
                TagKind::Blockquote => handle_blockquote(tag, parser, output, options, state),
                TagKind::HorizontalRule => {
                    ensure_block_start(output, state);
                    output.push_str("---\n\n");
                }
                TagKind::Table => handle_table(tag, parser, output, options, state),
                TagKind::TableSection | TagKind::TableRow | TagKind::TableCell => {
                    walk_children(tag, parser, output, options, state);
                }
            }
        }
    }
}

fn walk_children(
    tag: &tl::HTMLTag,
    parser: &crate::tl_types::Parser,
    output: &mut String,
    options: &ConversionOptions,
    state: FastState,
) {
    for child in tag.children().top().iter() {
        walk_node(child, parser, output, options, state);
    }
}

fn classify_tag(name: &str) -> TagKind {
    let bytes = name.as_bytes();
    match bytes.len() {
        1 => {
            if eq_tag(bytes, b"a") {
                TagKind::Link
            } else if eq_tag(bytes, b"b") {
                TagKind::Strong
            } else if eq_tag(bytes, b"i") {
                TagKind::Emphasis
            } else if eq_tag(bytes, b"p") {
                TagKind::Paragraph
            } else {
                TagKind::Passthrough
            }
        }
        2 => {
            if (bytes[0] == b'h' || bytes[0] == b'H') && matches!(bytes[1], b'1'..=b'6') {
                TagKind::Heading((bytes[1] - b'0') as usize)
            } else if eq_tag(bytes, b"br") {
                TagKind::Break
            } else if eq_tag(bytes, b"em") {
                TagKind::Emphasis
            } else if eq_tag(bytes, b"hr") {
                TagKind::HorizontalRule
            } else if eq_tag(bytes, b"li") {
                TagKind::ListItem
            } else if eq_tag(bytes, b"ol") {
                TagKind::OrderedList
            } else if eq_tag(bytes, b"td") || eq_tag(bytes, b"th") {
                TagKind::TableCell
            } else if eq_tag(bytes, b"tr") {
                TagKind::TableRow
            } else if eq_tag(bytes, b"ul") {
                TagKind::UnorderedList
            } else {
                TagKind::Passthrough
            }
        }
        3 => {
            if eq_tag(bytes, b"img") {
                TagKind::Image
            } else if eq_tag(bytes, b"pre") {
                TagKind::Pre
            } else {
                TagKind::Passthrough
            }
        }
        4 => {
            if eq_tag(bytes, b"code") {
                TagKind::Code
            } else if eq_tag(bytes, b"head") {
                TagKind::Skip
            } else {
                TagKind::Passthrough
            }
        }
        5 => {
            if eq_tag(bytes, b"style") {
                TagKind::Skip
            } else if eq_tag(bytes, b"table") {
                TagKind::Table
            } else if eq_tag(bytes, b"tbody") || eq_tag(bytes, b"thead") || eq_tag(bytes, b"tfoot")
            {
                TagKind::TableSection
            } else {
                TagKind::Passthrough
            }
        }
        6 => {
            if eq_tag(bytes, b"script") {
                TagKind::Skip
            } else if eq_tag(bytes, b"strong") {
                TagKind::Strong
            } else {
                TagKind::Passthrough
            }
        }
        7 => {
            if eq_tag(bytes, b"caption") {
                TagKind::TableCell
            } else {
                TagKind::Passthrough
            }
        }
        8 => {
            if eq_tag(bytes, b"template") || eq_tag(bytes, b"noscript") {
                TagKind::Skip
            } else {
                TagKind::Passthrough
            }
        }
        10 => {
            if eq_tag(bytes, b"blockquote") {
                TagKind::Blockquote
            } else if eq_tag(bytes, b"figcaption") {
                TagKind::Passthrough
            } else {
                TagKind::Passthrough
            }
        }
        _ => TagKind::Passthrough,
    }
}

#[inline]
fn eq_tag(left: &[u8], right: &[u8]) -> bool {
    left.eq_ignore_ascii_case(right)
}

fn handle_heading(
    level: usize,
    tag: &tl::HTMLTag,
    parser: &crate::tl_types::Parser,
    output: &mut String,
    options: &ConversionOptions,
    state: FastState,
) {
    let mut content = String::new();
    walk_children(
        tag,
        parser,
        &mut content,
        options,
        FastState {
            inline: true,
            ..state
        },
    );
    let content = content.trim();
    if content.is_empty() {
        return;
    }

    ensure_block_start(output, state);
    match options.heading_style {
        HeadingStyle::Underlined if level <= 2 => {
            output.push_str(content);
            output.push('\n');
            output.push_str(&if level == 1 { "=" } else { "-" }.repeat(content.chars().count()));
            output.push_str("\n\n");
        }
        HeadingStyle::AtxClosed => {
            let marks = "#".repeat(level);
            output.push_str(&marks);
            output.push(' ');
            output.push_str(content);
            output.push(' ');
            output.push_str(&marks);
            output.push_str("\n\n");
        }
        HeadingStyle::Atx | HeadingStyle::Underlined => {
            output.push_str(&"#".repeat(level));
            output.push(' ');
            output.push_str(content);
            output.push_str("\n\n");
        }
    }
}

fn handle_wrapped(
    tag: &tl::HTMLTag,
    parser: &crate::tl_types::Parser,
    output: &mut String,
    options: &ConversionOptions,
    state: FastState,
    marker: &str,
) {
    output.push_str(marker);
    walk_children(
        tag,
        parser,
        output,
        options,
        FastState {
            inline: true,
            ..state
        },
    );
    output.push_str(marker);
}

fn handle_pre(
    tag: &tl::HTMLTag,
    parser: &crate::tl_types::Parser,
    output: &mut String,
    options: &ConversionOptions,
    state: FastState,
) {
    ensure_block_start(output, state);
    match options.code_block_style {
        CodeBlockStyle::Indented => {
            let mut content = String::new();
            walk_children(
                tag,
                parser,
                &mut content,
                options,
                FastState {
                    in_pre: true,
                    in_code: true,
                    ..state
                },
            );
            for line in content.trim_matches('\n').lines() {
                output.push_str("    ");
                output.push_str(line);
                output.push('\n');
            }
            output.push('\n');
        }
        CodeBlockStyle::Backticks | CodeBlockStyle::Tildes => {
            let fence = if options.code_block_style == CodeBlockStyle::Tildes {
                "~~~"
            } else {
                "```"
            };
            output.push_str(fence);
            output.push('\n');
            walk_children(
                tag,
                parser,
                output,
                options,
                FastState {
                    in_pre: true,
                    in_code: true,
                    ..state
                },
            );
            trim_document_end(output);
            output.push('\n');
            output.push_str(fence);
            output.push_str("\n\n");
        }
    }
}

fn handle_link(
    tag: &tl::HTMLTag,
    parser: &crate::tl_types::Parser,
    output: &mut String,
    options: &ConversionOptions,
    state: FastState,
) {
    let Some(href_value) = tag.attributes().get("href").flatten() else {
        walk_children(tag, parser, output, options, state);
        return;
    };
    let href_raw = href_value.as_utf8_str();
    let href = decode_attr_text(href_raw.as_ref());
    if href.is_empty() {
        walk_children(tag, parser, output, options, state);
        return;
    }

    let mut label = String::new();
    walk_children(
        tag,
        parser,
        &mut label,
        options,
        FastState {
            inline: true,
            ..state
        },
    );
    let label = label.trim();
    if label.is_empty() {
        output.push_str(href.as_ref());
        return;
    }

    output.push('[');
    output.push_str(&escape_link_label(label));
    output.push_str("](");
    output.push_str(&format_destination(href.as_ref()));
    if let Some(title_value) = tag.attributes().get("title").flatten() {
        let title_raw = title_value.as_utf8_str();
        let title = decode_attr_text(title_raw.as_ref());
        push_optional_title(output, title.as_ref());
    }
    output.push(')');
}

fn handle_img(
    tag: &tl::HTMLTag,
    output: &mut String,
    _options: &ConversionOptions,
    state: FastState,
) {
    if state.inline || state.in_code {
        if let Some(alt_value) = tag.attributes().get("alt").flatten() {
            let alt_raw = alt_value.as_utf8_str();
            let alt = decode_attr_text(alt_raw.as_ref());
            output.push_str(alt.as_ref());
        }
        return;
    }
    let Some(src_value) = tag.attributes().get("src").flatten() else {
        return;
    };
    let src_raw = src_value.as_utf8_str();
    let src = decode_attr_text(src_raw.as_ref());
    if src.is_empty() {
        return;
    }

    let alt_raw = tag
        .attributes()
        .get("alt")
        .flatten()
        .map(|value| value.as_utf8_str());
    let alt = alt_raw
        .as_ref()
        .map_or(Cow::Borrowed(""), |raw| decode_attr_text(raw.as_ref()));
    output.push_str("![");
    output.push_str(&escape_link_label(alt.as_ref()));
    output.push_str("](");
    output.push_str(&format_destination(src.as_ref()));
    if let Some(title_value) = tag.attributes().get("title").flatten() {
        let title_raw = title_value.as_utf8_str();
        let title = decode_attr_text(title_raw.as_ref());
        push_optional_title(output, title.as_ref());
    }
    output.push(')');
}

fn handle_list(
    tag: &tl::HTMLTag,
    parser: &crate::tl_types::Parser,
    output: &mut String,
    options: &ConversionOptions,
    state: FastState,
    ordered: bool,
) {
    ensure_block_start(output, state);
    walk_children(
        tag,
        parser,
        output,
        options,
        FastState {
            list_depth: state.list_depth + 1,
            ordered_list: ordered,
            ..state
        },
    );
    if !state.inline {
        push_block_end(output, state);
    }
}

fn handle_list_item(
    tag: &tl::HTMLTag,
    parser: &crate::tl_types::Parser,
    output: &mut String,
    options: &ConversionOptions,
    state: FastState,
) {
    trim_trailing_spaces(output);
    if !output.is_empty() && !output.ends_with('\n') {
        output.push('\n');
    }

    let indent = state.list_depth.saturating_sub(1) * options.list_indent_width;
    output.push_str(&" ".repeat(indent));
    if state.ordered_list {
        output.push_str("1. ");
    } else {
        let bullet = options.bullets.chars().next().unwrap_or('-');
        output.push(bullet);
        output.push(' ');
    }

    let mut content = String::new();
    walk_children(
        tag,
        parser,
        &mut content,
        options,
        FastState {
            inline: true,
            ..state
        },
    );
    output.push_str(content.trim());
    output.push('\n');
}

fn handle_blockquote(
    tag: &tl::HTMLTag,
    parser: &crate::tl_types::Parser,
    output: &mut String,
    options: &ConversionOptions,
    state: FastState,
) {
    ensure_block_start(output, state);
    let mut content = String::new();
    walk_children(tag, parser, &mut content, options, state);
    for line in content.trim().lines() {
        output.push_str("> ");
        output.push_str(line.trim());
        output.push('\n');
    }
    output.push('\n');
}

fn handle_table(
    tag: &tl::HTMLTag,
    parser: &crate::tl_types::Parser,
    output: &mut String,
    options: &ConversionOptions,
    state: FastState,
) {
    let rows = collect_table_rows(tag, parser, options, state);
    if rows.is_empty() {
        return;
    }
    ensure_block_start(output, state);
    let cols = rows.iter().map(Vec::len).max().unwrap_or(0);
    if cols == 0 {
        return;
    }

    let header = &rows[0];
    push_table_row(output, header, cols);
    output.push('|');
    for _ in 0..cols {
        output.push_str(" --- |");
    }
    output.push('\n');
    for row in rows.iter().skip(1) {
        push_table_row(output, row, cols);
    }
    output.push('\n');
}

fn collect_table_rows(
    tag: &tl::HTMLTag,
    parser: &crate::tl_types::Parser,
    options: &ConversionOptions,
    state: FastState,
) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    collect_rows_from_children(tag, parser, options, state, &mut rows);
    rows
}

fn collect_rows_from_children(
    tag: &tl::HTMLTag,
    parser: &crate::tl_types::Parser,
    options: &ConversionOptions,
    state: FastState,
    rows: &mut Vec<Vec<String>>,
) {
    for child in tag.children().top().iter() {
        let Some(tl::Node::Tag(child_tag)) = child.get(parser) else {
            continue;
        };
        let name = child_tag.name().as_utf8_str();
        match classify_tag(name.as_ref()) {
            TagKind::TableRow => rows.push(collect_cells(child_tag, parser, options, state)),
            TagKind::TableSection => {
                collect_rows_from_children(child_tag, parser, options, state, rows);
            }
            _ => {}
        }
    }
}

fn collect_cells(
    tag: &tl::HTMLTag,
    parser: &crate::tl_types::Parser,
    options: &ConversionOptions,
    state: FastState,
) -> Vec<String> {
    let mut cells = Vec::new();
    for child in tag.children().top().iter() {
        let Some(tl::Node::Tag(cell_tag)) = child.get(parser) else {
            continue;
        };
        let name = cell_tag.name().as_utf8_str();
        if classify_tag(name.as_ref()) == TagKind::TableCell {
            let mut content = String::new();
            walk_children(
                cell_tag,
                parser,
                &mut content,
                options,
                FastState {
                    inline: true,
                    ..state
                },
            );
            cells.push(content.trim().replace('|', r"\|"));
        }
    }
    cells
}

fn push_table_row(output: &mut String, row: &[String], cols: usize) {
    output.push('|');
    for index in 0..cols {
        output.push(' ');
        if let Some(cell) = row.get(index) {
            output.push_str(cell);
        }
        output.push_str(" |");
    }
    output.push('\n');
}

fn push_text(raw: &str, output: &mut String, options: &ConversionOptions, state: FastState) {
    if raw.is_empty() {
        return;
    }
    if state.in_pre || state.in_code {
        output.push_str(raw);
        return;
    }

    let decoded = if contains_byte(raw.as_bytes(), b'&') {
        text::decode_html_entities_cow(raw)
    } else {
        Cow::Borrowed(raw)
    };
    let normalized = normalize_text(decoded.as_ref());
    let escaped = text::escape(
        normalized.as_ref(),
        options.escape_misc,
        options.escape_asterisks,
        options.escape_underscores,
        options.escape_ascii,
    );
    push_collapsed(output, escaped.as_ref());
}

fn normalize_text(text: &str) -> Cow<'_, str> {
    if !may_need_text_normalization(text.as_bytes()) {
        return Cow::Borrowed(text);
    }

    let mut previous_was_space = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            if previous_was_space || ch != ' ' {
                return Cow::Owned(collapse_whitespace(text));
            }
            previous_was_space = true;
        } else {
            previous_was_space = false;
        }
    }
    Cow::Borrowed(text)
}

fn collapse_whitespace(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut previous_was_space = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            if !previous_was_space {
                output.push(' ');
                previous_was_space = true;
            }
        } else {
            output.push(ch);
            previous_was_space = false;
        }
    }
    output
}

fn push_collapsed(output: &mut String, text: &str) {
    if text.is_empty() {
        return;
    }
    let text = if output.ends_with([' ', '\n']) {
        text.trim_start()
    } else {
        text
    };
    output.push_str(text);
}

fn decode_attr_text(value: &str) -> Cow<'_, str> {
    if contains_byte(value.as_bytes(), b'&') {
        text::decode_html_entities_cow(value)
    } else {
        Cow::Borrowed(value)
    }
}

fn push_optional_title(output: &mut String, title: &str) {
    if title.is_empty() {
        return;
    }
    output.push_str(" \"");
    if title.as_bytes().contains(&b'"') {
        output.push_str(&title.replace('"', "\\\""));
    } else {
        output.push_str(title);
    }
    output.push('"');
}

fn escape_link_label(label: &str) -> String {
    if !label
        .as_bytes()
        .iter()
        .any(|b| matches!(b, b'[' | b']' | b'\\'))
    {
        return label.to_string();
    }
    let mut out = String::with_capacity(label.len() + 2);
    for ch in label.chars() {
        if matches!(ch, '[' | ']' | '\\') {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}

fn format_destination(url: &str) -> Cow<'_, str> {
    if url.as_bytes().iter().any(|b| b.is_ascii_whitespace()) {
        Cow::Owned(format!("<{}>", url.replace('\n', "%0A")))
    } else {
        Cow::Borrowed(url)
    }
}

fn ensure_block_start(output: &mut String, state: FastState) {
    if state.inline || output.is_empty() {
        return;
    }
    trim_trailing_spaces(output);
    if !output.ends_with("\n\n") {
        if output.ends_with('\n') {
            output.push('\n');
        } else {
            output.push_str("\n\n");
        }
    }
}

fn push_block_end(output: &mut String, state: FastState) {
    if state.inline {
        return;
    }
    trim_trailing_spaces(output);
    if !output.ends_with("\n\n") {
        if output.ends_with('\n') {
            output.push('\n');
        } else {
            output.push_str("\n\n");
        }
    }
}

fn trim_trailing_spaces(output: &mut String) {
    while output.ends_with([' ', '\t']) {
        output.pop();
    }
}

fn trim_document_end(output: &mut String) {
    let len = output.trim_end_matches(|ch: char| ch.is_whitespace()).len();
    output.truncate(len);
}

fn trim_document_boundaries(output: &mut String) {
    trim_document_end(output);
    let start = output
        .char_indices()
        .find_map(|(idx, ch)| (!ch.is_whitespace()).then_some(idx))
        .unwrap_or(output.len());
    if start > 0 {
        output.drain(..start);
    }
}

#[cfg(all(feature = "simd", nightly))]
#[inline]
fn contains_byte(bytes: &[u8], needle: u8) -> bool {
    crate::simd_scan::contains_byte(bytes, needle)
}

#[cfg(not(all(feature = "simd", nightly)))]
#[inline]
fn contains_byte(bytes: &[u8], needle: u8) -> bool {
    bytes.contains(&needle)
}

#[cfg(all(feature = "simd", nightly))]
#[inline]
fn may_need_text_normalization(bytes: &[u8]) -> bool {
    crate::simd_scan::contains_ascii_whitespace_or_non_ascii(bytes)
}

#[cfg(not(all(feature = "simd", nightly)))]
#[inline]
fn may_need_text_normalization(bytes: &[u8]) -> bool {
    bytes.iter().any(|byte| *byte <= b' ' || *byte >= 0x80)
}

fn trim_code_span(output: &mut String, start: usize) {
    let end = output.len();
    if start >= end {
        return;
    }
    let trimmed = output[start..end].trim();
    let owned = trimmed.to_string();
    output.truncate(start);
    output.push_str(&owned);
}
