# fast_h2m

High-performance HTML to Markdown converter written in Rust.

## Install

```toml
[dependencies]
fast_h2m = "0.1"
```

## Usage

```rust
use fast_h2m::convert;

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
