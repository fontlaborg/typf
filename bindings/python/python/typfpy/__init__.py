"""
TYPF - Professional text rendering, now in Python

Where Rust performance meets Python elegance. This package brings
TYPF's world-class text shaping and rendering to your Python projects
with zero compromise on speed or quality.

## Quick Start

```python
from typfpy import Typf

# Create your rendering pipeline
typf = Typf(shaper="harfbuzz", renderer="opixa")

# Render beautiful text
result = typf.render_text("Hello, World!", "font.ttf", size=48)
```
"""

# Import the compiled Rust extension that brings the power
from .typf import (
    FontInfo,       # Font inspection and metadata
    Typf,          # Main rendering pipeline
    __version__,   # Version information
    export_image,  # Convert results to files
    render_simple,  # Quick rendering without fonts
)

# What we expose to the Python world
__all__ = [
    "Typf",
    "FontInfo",
    "render_simple",
    "export_image",
    "__version__",
]
