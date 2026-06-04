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
            let name = name.as_ref().to_ascii_lowercase();
            match name.as_str() {
                "html" | "body" | "main" | "article" | "section" | "div" | "span" => {
                    walk_children(tag, parser, output, options, state);
                }
                "head" | "script" | "style" | "template" | "noscript" => {}
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    handle_heading(&name, tag, parser, output, options, state);
                }
                "p" => {
                    ensure_block_start(output, state);
                    let start = output.len();
                    walk_children(tag, parser, output, options, state);
                    trim_trailing_spaces(output);
                    if output.len() > start {
                        push_block_end(output, state);
                    }
                }
                "br" => match options.newline_style {
                    NewlineStyle::Backslash => output.push_str("\\\n"),
                    NewlineStyle::Spaces => output.push_str("  \n"),
                },
                "strong" | "b" => handle_wrapped(tag, parser, output, options, state, "**"),
                "em" | "i" => handle_wrapped(tag, parser, output, options, state, "*"),
                "code" => {
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
                "pre" => handle_pre(tag, parser, output, options, state),
                "a" => handle_link(tag, parser, output, options, state),
                "img" => handle_img(tag, output, options, state),
                "ul" => handle_list(tag, parser, output, options, state, false),
                "ol" => handle_list(tag, parser, output, options, state, true),
                "li" => handle_list_item(tag, parser, output, options, state),
                "blockquote" => handle_blockquote(tag, parser, output, options, state),
                "hr" => {
                    ensure_block_start(output, state);
                    output.push_str("---\n\n");
                }
                "table" => handle_table(tag, parser, output, options, state),
                "thead" | "tbody" | "tfoot" | "tr" | "td" | "th" | "caption" => {
                    walk_children(tag, parser, output, options, state);
                }
                "sub" | "sup" | "mark" | "small" | "del" | "s" | "ins" | "u" | "abbr" | "dfn"
                | "kbd" | "samp" | "var" | "time" | "data" | "figure" | "figcaption" | "header"
                | "footer" | "nav" | "aside" => {
                    walk_children(tag, parser, output, options, state);
                }
                _ => walk_children(tag, parser, output, options, state),
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

fn handle_heading(
    name: &str,
    tag: &tl::HTMLTag,
    parser: &crate::tl_types::Parser,
    output: &mut String,
    options: &ConversionOptions,
    state: FastState,
) {
    let level = name
        .as_bytes()
        .get(1)
        .and_then(|b| b.checked_sub(b'0'))
        .filter(|level| (1..=6).contains(level))
        .unwrap_or(1) as usize;
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
    let href = attr(tag, "href");
    let Some(href) = href.filter(|href| !href.is_empty()) else {
        walk_children(tag, parser, output, options, state);
        return;
    };

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
    if let Some(title) = attr(tag, "title").filter(|title| !title.is_empty()) {
        output.push_str(" \"");
        output.push_str(&title.replace('"', "\\\""));
        output.push('"');
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
        if let Some(alt) = attr(tag, "alt") {
            output.push_str(alt.as_ref());
        }
        return;
    }
    let Some(src) = attr(tag, "src").filter(|src| !src.is_empty()) else {
        return;
    };
    let alt = attr(tag, "alt").unwrap_or(Cow::Borrowed(""));
    output.push_str("![");
    output.push_str(&escape_link_label(alt.as_ref()));
    output.push_str("](");
    output.push_str(&format_destination(src.as_ref()));
    if let Some(title) = attr(tag, "title").filter(|title| !title.is_empty()) {
        output.push_str(" \"");
        output.push_str(&title.replace('"', "\\\""));
        output.push('"');
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
        let name = child_tag.name().as_utf8_str().as_ref().to_ascii_lowercase();
        match name.as_str() {
            "tr" => rows.push(collect_cells(&child_tag, parser, options, state)),
            "thead" | "tbody" | "tfoot" => {
                collect_rows_from_children(&child_tag, parser, options, state, rows);
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
        let name = cell_tag.name().as_utf8_str().as_ref().to_ascii_lowercase();
        if matches!(name.as_str(), "td" | "th") {
            let mut content = String::new();
            walk_children(
                &cell_tag,
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

    let decoded = if raw.as_bytes().contains(&b'&') {
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

fn attr(tag: &tl::HTMLTag, name: &str) -> Option<Cow<'static, str>> {
    tag.attributes()
        .get(name)
        .flatten()
        .map(|value| Cow::Owned(text::decode_html_entities(value.as_utf8_str().as_ref())))
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
