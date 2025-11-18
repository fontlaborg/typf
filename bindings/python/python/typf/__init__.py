"""
TYPF - High-performance text rendering pipeline

Python bindings for the TYPF text shaping and rendering library.
"""

# Import the compiled Rust extension
from typf import (
    Typf,
    FontInfo,
    render_simple,
    export_image,
    __version__,
)

__all__ = [
    "Typf",
    "FontInfo",
    "render_simple",
    "export_image",
    "__version__",
]
