from __future__ import annotations

from typing import Any, Dict, Optional

__version__: str

def convert(html: str, options: Optional[Dict[str, Any]] = None) -> Dict[str, Any]: ...
def convert_to_markdown(html: str, options: Optional[Dict[str, Any]] = None) -> str: ...
