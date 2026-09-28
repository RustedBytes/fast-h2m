# fast-h2m

Python bindings for `fast_h2m`, a high-performance HTML to Markdown converter.

```python
import fast_h2m

markdown = fast_h2m.convert_to_markdown("<h1>Hello</h1><p>World</p>")
result = fast_h2m.convert("<h1>Hello</h1>", {"include_document_structure": True})
```

For throughput-oriented conversion of common HTML, opt into the lean DOM path:

```python
markdown = fast_h2m.convert_to_markdown(
    html,
    {"tier_strategy": "fast_dom"},
)
```

`fast_dom` skips the richer metadata, structure, visitor, selector, and repair
machinery used by the full converter.

On supported Linux and Windows wheels, choose the parser backend at runtime:

```python
markdown = fast_h2m.convert_to_markdown(
    html,
    {"tier_strategy": "tier2", "parser_backend": "asm_tl"},
)
```

`rustedbytes_tl` remains the default. Selecting `asm_tl` in a build that does
not include or support it raises a configuration error.

For mdream-backed lean conversion:

```python
markdown = fast_h2m.convert_to_markdown(
    html,
    {"tier_strategy": "mdream"},
)
```

For chunked streaming conversion:

```python
stream = fast_h2m.MarkdownStreamProcessor()
markdown = ""
markdown += stream.process_chunk("<h1>Hello</h1>")
markdown += stream.process_chunk("<p>World</p>")
markdown += stream.finish()
```

The package targets Python 3.9+ and exposes the Rust converter through PyO3.
CPython wheels use the stable `abi3` ABI, while version-specific wheels are
published for PyPy 3.11 and 3.12.

When building locally with PyPy, disable the CPython-only `abi3` feature:

```sh
maturin build --release --no-default-features --features extension-module,asm-tl
```
