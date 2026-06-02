# h2md

High-performance HTML to Markdown converter written in Rust.

## Install

```toml
[dependencies]
h2md = "0.1"
```

## Usage

```rust
use h2md::convert;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = convert("<h1>Hello</h1><p>From HTML.</p>", None)?;
    println!("{}", result.content.unwrap_or_default());
    Ok(())
}
```

## Development

```sh
cargo test
cargo clippy --workspace --all-targets
```

## License

MIT
