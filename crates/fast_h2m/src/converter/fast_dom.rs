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

#[inline]
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

#[inline]
fn classify_tag(name: &str) -> TagKind {
    let bytes = name.as_bytes();
    match bytes.len() {
        1 => match ascii_lower(bytes[0]) {
            b'a' => TagKind::Link,
            b'b' => TagKind::Strong,
            b'i' => TagKind::Emphasis,
            b'p' => TagKind::Paragraph,
            _ => TagKind::Passthrough,
        },
        2 => match pack2_lower(bytes) {
            TAG_H1..=TAG_H6 if matches!(bytes[1], b'1'..=b'6') => {
                TagKind::Heading((bytes[1] - b'0') as usize)
            }
            TAG_BR => TagKind::Break,
            TAG_EM => TagKind::Emphasis,
            TAG_HR => TagKind::HorizontalRule,
            TAG_LI => TagKind::ListItem,
            TAG_OL => TagKind::OrderedList,
            TAG_TD | TAG_TH => TagKind::TableCell,
            TAG_TR => TagKind::TableRow,
            TAG_UL => TagKind::UnorderedList,
            _ => TagKind::Passthrough,
        },
        3 => match pack3_lower(bytes) {
            TAG_IMG => TagKind::Image,
            TAG_PRE => TagKind::Pre,
            _ => TagKind::Passthrough,
        },
        4 => match pack4_lower(bytes) {
            TAG_CODE => TagKind::Code,
            TAG_HEAD => TagKind::Skip,
            _ => TagKind::Passthrough,
        },
        5 => match pack5_lower(bytes) {
            TAG_STYLE => TagKind::Skip,
            TAG_TABLE => TagKind::Table,
            TAG_TBODY | TAG_THEAD | TAG_TFOOT => TagKind::TableSection,
            _ => TagKind::Passthrough,
        },
        6 => match pack6_lower(bytes) {
            TAG_SCRIPT => TagKind::Skip,
            TAG_STRONG => TagKind::Strong,
            _ => TagKind::Passthrough,
        },
        7 => match pack7_lower(bytes) {
            TAG_CAPTION => TagKind::TableCell,
            _ => TagKind::Passthrough,
        },
        8 => match pack8_lower(bytes) {
            TAG_TEMPLATE | TAG_NOSCRIPT => TagKind::Skip,
            _ => TagKind::Passthrough,
        },
        10 => {
            if eq_tag(bytes, b"blockquote") {
                TagKind::Blockquote
            } else {
                TagKind::Passthrough
            }
        }
        _ => TagKind::Passthrough,
    }
}

const TAG_BR: u16 = pack2_const(b"br");
const TAG_EM: u16 = pack2_const(b"em");
const TAG_H1: u16 = pack2_const(b"h1");
const TAG_H6: u16 = pack2_const(b"h6");
const TAG_HR: u16 = pack2_const(b"hr");
const TAG_LI: u16 = pack2_const(b"li");
const TAG_OL: u16 = pack2_const(b"ol");
const TAG_TD: u16 = pack2_const(b"td");
const TAG_TH: u16 = pack2_const(b"th");
const TAG_TR: u16 = pack2_const(b"tr");
const TAG_UL: u16 = pack2_const(b"ul");
const TAG_IMG: u32 = pack3_const(b"img");
const TAG_PRE: u32 = pack3_const(b"pre");
const TAG_CODE: u32 = pack4_const(b"code");
const TAG_HEAD: u32 = pack4_const(b"head");
const TAG_STYLE: u64 = pack5_const(b"style");
const TAG_TABLE: u64 = pack5_const(b"table");
const TAG_TBODY: u64 = pack5_const(b"tbody");
const TAG_THEAD: u64 = pack5_const(b"thead");
const TAG_TFOOT: u64 = pack5_const(b"tfoot");
const TAG_SCRIPT: u64 = pack6_const(b"script");
const TAG_STRONG: u64 = pack6_const(b"strong");
const TAG_CAPTION: u64 = pack7_const(b"caption");
const TAG_TEMPLATE: u64 = pack8_const(b"template");
const TAG_NOSCRIPT: u64 = pack8_const(b"noscript");

#[inline]
fn ascii_lower(byte: u8) -> u8 {
    byte | (u8::from(byte.is_ascii_uppercase()) * 0x20)
}

#[inline]
fn pack2_lower(bytes: &[u8]) -> u16 {
    u16::from(ascii_lower(bytes[0])) | (u16::from(ascii_lower(bytes[1])) << 8)
}

#[inline]
fn pack3_lower(bytes: &[u8]) -> u32 {
    u32::from(ascii_lower(bytes[0]))
        | (u32::from(ascii_lower(bytes[1])) << 8)
        | (u32::from(ascii_lower(bytes[2])) << 16)
}

#[inline]
fn pack4_lower(bytes: &[u8]) -> u32 {
    u32::from(ascii_lower(bytes[0]))
        | (u32::from(ascii_lower(bytes[1])) << 8)
        | (u32::from(ascii_lower(bytes[2])) << 16)
        | (u32::from(ascii_lower(bytes[3])) << 24)
}

#[inline]
fn pack5_lower(bytes: &[u8]) -> u64 {
    u64::from(pack4_lower(bytes)) | (u64::from(ascii_lower(bytes[4])) << 32)
}

#[inline]
fn pack6_lower(bytes: &[u8]) -> u64 {
    pack5_lower(bytes) | (u64::from(ascii_lower(bytes[5])) << 40)
}

#[inline]
fn pack7_lower(bytes: &[u8]) -> u64 {
    pack6_lower(bytes) | (u64::from(ascii_lower(bytes[6])) << 48)
}

#[inline]
fn pack8_lower(bytes: &[u8]) -> u64 {
    pack7_lower(bytes) | (u64::from(ascii_lower(bytes[7])) << 56)
}

const fn pack2_const(bytes: &[u8; 2]) -> u16 {
    bytes[0] as u16 | ((bytes[1] as u16) << 8)
}

const fn pack3_const(bytes: &[u8; 3]) -> u32 {
    bytes[0] as u32 | ((bytes[1] as u32) << 8) | ((bytes[2] as u32) << 16)
}

const fn pack4_const(bytes: &[u8; 4]) -> u32 {
    bytes[0] as u32
        | ((bytes[1] as u32) << 8)
        | ((bytes[2] as u32) << 16)
        | ((bytes[3] as u32) << 24)
}

const fn pack5_const(bytes: &[u8; 5]) -> u64 {
    pack4_const(&[bytes[0], bytes[1], bytes[2], bytes[3]]) as u64 | ((bytes[4] as u64) << 32)
}

const fn pack6_const(bytes: &[u8; 6]) -> u64 {
    pack5_const(&[bytes[0], bytes[1], bytes[2], bytes[3], bytes[4]]) | ((bytes[5] as u64) << 40)
}

const fn pack7_const(bytes: &[u8; 7]) -> u64 {
    pack6_const(&[bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]])
        | ((bytes[6] as u64) << 48)
}

const fn pack8_const(bytes: &[u8; 8]) -> u64 {
    pack7_const(&[
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6],
    ]) | ((bytes[7] as u64) << 56)
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
    let bytes = text.as_bytes();
    if !may_need_text_normalization(bytes) {
        return Cow::Borrowed(text);
    }

    let mut previous_was_space = false;
    for byte in bytes {
        if !byte.is_ascii() {
            return normalize_unicode_text(text);
        }
        if is_ascii_markdown_whitespace(*byte) {
            if previous_was_space || *byte != b' ' {
                return Cow::Owned(collapse_ascii_whitespace(bytes));
            }
            previous_was_space = true;
        } else {
            previous_was_space = false;
        }
    }
    Cow::Borrowed(text)
}

fn normalize_unicode_text(text: &str) -> Cow<'_, str> {
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

fn collapse_ascii_whitespace(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len());
    let mut previous_was_space = false;
    for byte in bytes {
        if is_ascii_markdown_whitespace(*byte) {
            if !previous_was_space {
                output.push(' ');
                previous_was_space = true;
            }
        } else {
            output.push(char::from(*byte));
            previous_was_space = false;
        }
    }
    output
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

#[inline]
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

#[inline]
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
    if contains_byte(title.as_bytes(), b'"') {
        output.push_str(&title.replace('"', "\\\""));
    } else {
        output.push_str(title);
    }
    output.push('"');
}

fn escape_link_label(label: &str) -> String {
    if !contains_link_label_escape_byte(label.as_bytes()) {
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

#[inline]
fn format_destination(url: &str) -> Cow<'_, str> {
    if contains_ascii_whitespace(url.as_bytes()) {
        Cow::Owned(format!("<{}>", url.replace('\n', "%0A")))
    } else {
        Cow::Borrowed(url)
    }
}

#[inline]
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

#[inline]
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

#[inline]
fn trim_trailing_spaces(output: &mut String) {
    let len = output
        .as_bytes()
        .iter()
        .rposition(|byte| !matches!(byte, b' ' | b'\t'))
        .map_or(0, |index| index + 1);
    output.truncate(len);
}

#[inline]
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

#[inline]
fn is_ascii_markdown_whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | 0x0B | 0x0C | b'\r')
}

#[inline]
fn contains_ascii_whitespace(bytes: &[u8]) -> bool {
    bytes.iter().any(|byte| byte.is_ascii_whitespace())
}

#[inline]
fn contains_link_label_escape_byte(bytes: &[u8]) -> bool {
    bytes.iter().any(|byte| matches!(byte, b'[' | b']' | b'\\'))
}

#[inline]
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
