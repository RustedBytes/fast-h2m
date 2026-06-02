//! Example: Basic HTML to Markdown conversion

fn convert(
    html: &str,
    opts: Option<h2md::ConversionOptions>,
) -> h2md::error::Result<String> {
    h2md::convert(html, opts).map(|r| r.content.unwrap_or_default())
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
