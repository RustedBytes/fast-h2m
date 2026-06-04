# fast_h2m

[![Crates.io Version](https://img.shields.io/crates/v/fast-h2m)](https://crates.io/crates/fast-h2m)
[![PyPI - Version](https://img.shields.io/pypi/v/fast-h2m)](https://pypi.org/project/fast-h2m/)
[![CI](https://github.com/RustedBytes/fast-h2m/actions/workflows/ci.yml/badge.svg)](https://github.com/RustedBytes/fast-h2m/actions/workflows/ci.yml)
[![PyPI Downloads](https://static.pepy.tech/personalized-badge/fast-h2m?period=total&units=INTERNATIONAL_SYSTEM&left_color=BLACK&right_color=GREEN&left_text=downloads)](https://pepy.tech/projects/fast-h2m)


High-performance HTML to Markdown converter written in Rust.

## Install

```toml
[dependencies]
fast_h2m = "0.4"
```

## Basic Usage

`convert()` returns a structured `ConversionResult` with the converted text, metadata, tables, and more:

```rust
use fast_h2m::convert;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let html = r#"
        <html lang="en">
          <head><title>Welcome</title></head>
          <body>
            <h1>Welcome</h1>
            <p>This is <strong>fast</strong> conversion!</p>
            <ul>
                <li>Built with Rust</li>
                <li>CommonMark compliant</li>
            </ul>
          </body>
        </html>
    "#;

    let result = convert(html, None)?;
    println!("{}", result.content.unwrap_or_default());

    println!("Title: {:?}", result.metadata.document.title);
    println!("Headers: {:?}", result.metadata.headers);

    for table in &result.tables {
        println!("Table with {} rows", table.grid.rows);
    }

    Ok(())
}
```

## Error Handling

Conversion returns a `Result<ConversionResult, ConversionError>`. Inputs that look like binary data are rejected with
`ConversionError::InvalidInput` to prevent runaway allocations. Table `colspan`/`rowspan` values are also clamped
internally to keep output sizes bounded.

## Configuration

### Builder Pattern

```rust
use fast_h2m::{convert, ConversionOptions, HeadingStyle};

let options = ConversionOptions::builder()
    .heading_style(HeadingStyle::Atx)
    .list_indent_width(2)
    .bullets("-")
    .autolinks(true)
    .wrap(true)
    .wrap_width(80)
    .build();

let result = convert(html, Some(options))?;
println!("{}", result.content.unwrap_or_default());
```

### Struct Literal

```rust
use fast_h2m::{
    convert, ConversionOptions, HeadingStyle, ListIndentType,
};

let options = ConversionOptions {
    heading_style: HeadingStyle::Atx,
    list_indent_width: 2,
    list_indent_type: ListIndentType::Spaces,
    bullets: "-".to_string(),
    strong_em_symbol: '*',
    escape_asterisks: false,
    escape_underscores: false,
    newline_style: fast_h2m::NewlineStyle::Backslash,
    code_block_style: fast_h2m::CodeBlockStyle::Backticks,
    ..Default::default()
};

let result = convert(html, Some(options))?;
println!("{}", result.content.unwrap_or_default());
```

### Fast DOM Mode

For throughput-oriented conversion of common HTML, use the lean DOM strategy:

```rust
use fast_h2m::{convert, ConversionOptions, TierStrategy};

let html = r#"
<article>
    <h1>Hello</h1>
    <p>This path keeps common Markdown conversion fast.</p>
    <ul><li>Headings</li><li>Links</li><li>Tables</li></ul>
</article>
"#;

let options = ConversionOptions {
    tier_strategy: TierStrategy::FastDom,
    ..Default::default()
};

let result = convert(html, Some(options))?;
println!("{}", result.content.unwrap_or_default());
```

`FastDom` skips the richer metadata, structure, visitor, selector, and repair
machinery used by the full Tier-2 converter. Prefer it when raw Markdown
throughput matters more than the full structured `ConversionResult`.

### mdream Mode

For mdream-backed lean conversion, use the `Mdream` strategy:

```rust
use fast_h2m::{convert, ConversionOptions, TierStrategy};

let options = ConversionOptions {
    tier_strategy: TierStrategy::Mdream,
    ..Default::default()
};

let result = convert("<h1>Hello</h1><p>World</p>", Some(options))?;
println!("{}", result.content.unwrap_or_default());
```

For chunked streaming conversion:

```rust
use fast_h2m::MarkdownStreamProcessor;

let mut stream = MarkdownStreamProcessor::new(None);
let mut markdown = String::new();
markdown.push_str(&stream.process_chunk("<h1>Hello</h1>"));
markdown.push_str(&stream.process_chunk("<p>World</p>"));
markdown.push_str(&stream.finish());
```

`Mdream` is a lean mode and does not populate rich side channels such as
metadata, document structure, tables, visitor output, or inline images.

### Preserving HTML Tags

The `preserve_tags` option allows you to keep specific HTML tags in their original form instead of converting them to Markdown:

```rust
use fast_h2m::{convert, ConversionOptions};

let html = r#"
<p>Before table</p>
<table class="data">
    <tr><th>Name</th><th>Value</th></tr>
    <tr><td>Item 1</td><td>100</td></tr>
</table>
<p>After table</p>
"#;

let options = ConversionOptions {
    preserve_tags: vec!["table".to_string()],
    ..Default::default()
};

let result = convert(html, Some(options))?;
// result.content => "Before table\n\n<table class=\"data\">...</table>\n\nAfter table\n"
```

## Web Scraping with Preprocessing

```rust
use fast_h2m::{convert, ConversionOptions};

let mut options = ConversionOptions::default();
options.preprocessing.enabled = true;
options.preprocessing.preset = fast_h2m::PreprocessingPreset::Aggressive;
options.preprocessing.remove_navigation = true;
options.preprocessing.remove_forms = true;

let result = convert(scraped_html, Some(options))?;
println!("{}", result.content.unwrap_or_default());
```

## Metadata Extraction

Metadata is automatically included in the result when the default `metadata` feature is enabled.
Disable metadata extraction when you do not need it:

```rust
use fast_h2m::{convert, ConversionOptions};

let options = ConversionOptions::builder()
    .extract_metadata(false)
    .build();

let result = convert(html, Some(options))?;
println!("{}", result.content.unwrap_or_default());
```

With metadata extraction enabled, read the collected fields from `result.metadata`:

```rust
use fast_h2m::convert;

let result = convert(html, None)?;
println!("Title: {:?}", result.metadata.document.title);
for header in &result.metadata.headers {
    println!("H{}: {}", header.level, header.text);
}
for link in &result.metadata.links {
    println!("Link: {} -> {}", link.text, link.href);
}
```

## Image Extraction

Inline image extraction requires the `inline-images` Cargo feature:

```toml
[dependencies]
fast_h2m = { version = "0.1", features = ["inline-images"] }
```

```rust
use fast_h2m::{convert, ConversionOptions};

let options = ConversionOptions::builder()
    .extract_images(true)
    .max_image_size(5 * 1024 * 1024) // 5 MB max
    .infer_dimensions(true)
    .build();

let result = convert(html, Some(options))?;
println!("{}", result.content.unwrap_or_default());
for img in &result.images {
    println!(
        "Image: {:?} ({} bytes, format: {})",
        img.filename,
        img.data.len(),
        img.format
    );
}
```

## Table Extraction

Structured table data is always included in `ConversionResult.tables`:

```rust
use fast_h2m::convert;

let html = r#"
<table>
    <tr><th>Name</th><th>Age</th></tr>
    <tr><td>Alice</td><td>30</td></tr>
    <tr><td>Bob</td><td>25</td></tr>
</table>
"#;

let result = convert(html, None)?;

println!("{}", result.content.unwrap_or_default());
for table in &result.tables {
    println!("Table with {} rows and {} columns:", table.grid.rows, table.grid.cols);
    for cell in &table.grid.cells {
        let prefix = if cell.is_header { "Header" } else { "Cell" };
        println!(
            "  {} ({}, {}): {}",
            prefix,
            cell.row,
            cell.col,
            cell.content
        );
    }
}
```

## Custom Visitors

Custom visitors require the `visitor` Cargo feature:

```toml
[dependencies]
fast_h2m = { version = "0.1", features = ["visitor"] }
```

```rust
use fast_h2m::{convert, ConversionOptions};
use fast_h2m::visitor::{HtmlVisitor, NodeContext, VisitResult};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct NoImagesVisitor;

impl HtmlVisitor for NoImagesVisitor {
    fn visit_image(
        &mut self,
        _ctx: &NodeContext,
        _src: &str,
        _alt: &str,
        _title: Option<&str>,
    ) -> VisitResult {
        VisitResult::Skip
    }
}

let options = ConversionOptions::builder()
    .visitor(Some(Arc::new(Mutex::new(NoImagesVisitor))))
    .build();

let result = convert(html, Some(options))?;
println!("{}", result.content.unwrap_or_default());
```

## Development

```sh
cargo test
cargo clippy --workspace --all-targets
```

## Acknowledgements

This project builds on ideas and ecosystem work from
[kreuzberg-dev/html-to-markdown](https://github.com/kreuzberg-dev/html-to-markdown)
and [harlan-zw/mdream](https://github.com/harlan-zw/mdream).

## License

MIT
