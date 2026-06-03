# fast-h2m

Python bindings for `fast_h2m`, a high-performance HTML to Markdown converter.

```python
import fast_h2m

markdown = fast_h2m.convert_to_markdown("<h1>Hello</h1><p>World</p>")
result = fast_h2m.convert("<h1>Hello</h1>", {"include_document_structure": True})
```

The package targets Python 3.8+ and exposes the Rust converter through PyO3.
