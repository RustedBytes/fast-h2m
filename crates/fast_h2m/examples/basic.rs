//! Example: Basic HTML to Markdown conversion

fn convert(
    html: &str,
    opts: Option<fast_h2m::ConversionOptions>,
) -> fast_h2m::error::Result<String> {
    fast_h2m::convert(html, opts).map(|r| r.content.unwrap_or_default())
}

fn main() {
    let html = "<h1>Hello World</h1><p>This is a <strong>test</strong>.</p>";

    match convert(html, None) {
        Ok(markdown) => {
            println!("HTML:");
            println!("{html}");
            println!("\nMarkdown:");
            println!("{markdown}");
        }
        Err(e) => {
            eprintln!("Error: {e}");
        }
    }
}
