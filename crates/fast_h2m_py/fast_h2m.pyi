from __future__ import annotations

from typing import Any, Dict, List, Literal, Optional, TypedDict, Union

__version__: str

TierStrategy = Literal["auto", "tier2", "fast_dom"]

class ConversionOptions(TypedDict, total=False):
    tier_strategy: TierStrategy
    tierStrategy: TierStrategy
    heading_style: str
    list_indent_type: str
    list_indent_width: int
    bullets: str
    strong_em_symbol: str
    escape_asterisks: bool
    escape_underscores: bool
    escape_misc: bool
    escape_ascii: bool
    code_language: str
    autolinks: bool
    default_title: bool
    br_in_tables: bool
    compact_tables: bool
    highlight_style: str
    extract_metadata: bool
    whitespace_mode: str
    strip_newlines: bool
    wrap: bool
    wrap_width: int
    convert_as_inline: bool
    sub_symbol: str
    sup_symbol: str
    newline_style: str
    code_block_style: str
    keep_inline_images_in: List[str]
    preprocessing: Dict[str, Any]
    encoding: str
    debug: bool
    strip_tags: List[str]
    preserve_tags: List[str]
    skip_images: bool
    url_escape_style: str
    link_style: str
    output_format: str
    include_document_structure: bool
    extract_images: bool
    max_image_size: int
    capture_svg: bool
    infer_dimensions: bool
    max_depth: Optional[int]
    exclude_selectors: List[str]

def convert(
    html: str, options: Optional[Union[ConversionOptions, Dict[str, Any]]] = None
) -> Dict[str, Any]: ...
def convert_to_markdown(
    html: str, options: Optional[Union[ConversionOptions, Dict[str, Any]]] = None
) -> str: ...
