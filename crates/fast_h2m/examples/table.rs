//! Example: Converting HTML tables to Markdown

fn convert(
    html: &str,
    opts: Option<fast_h2m::ConversionOptions>,
) -> fast_h2m::error::Result<String> {
    fast_h2m::convert(html, opts).map(|r| r.content.unwrap_or_default())
}

fn main() {
    let html = r"<table>
        <tr><th>Name</th><th>Age</th></tr>
        <tr><td>Alice</td><td>30</td></tr>
        <tr><td>Bob</td><td>25</td></tr>
    </table>";

    match convert(html, None) {
        Ok(markdown) => {
            println!("Markdown:\n{markdown}");
        }
        Err(e) => {
            eprintln!("Error: {e}");
        }
    }
}
