"""
Simple font rendering backends for comparison with TYPF.

This package provides reference implementations using:
- CoreText (macOS native, via PyObjC)
- HarfBuzz + FreeType (cross-platform)

These are kept separate from the main TYPF package for comparison purposes.

Made by FontLab https://www.fontlab.com/
"""

from __future__ import annotations

import platform
from pathlib import Path
from typing import Any

import numpy as np

from .base import BaseRenderer, RendererInitError, RendererUnavailableError
from .constants import DEFAULT_FONT_SIZE, RENDER_HEIGHT, RENDER_WIDTH

# Try to import HarfBuzz renderer (cross-platform)
try:
    from .harfbuzzpy import HarfBuzzRenderer
    HARFBUZZ_AVAILABLE = True
except ImportError:
    HarfBuzzRenderer = None  # type: ignore
    HARFBUZZ_AVAILABLE = False

# Try to import CoreText renderer (macOS only)
try:
    from .coretextpy import CoreTextRenderer
    CORETEXT_AVAILABLE = True
except ImportError:
    CoreTextRenderer = None  # type: ignore
    CORETEXT_AVAILABLE = False


def list_available() -> list[str]:
    """List available simple rendering backends."""
    available = []
    if CORETEXT_AVAILABLE and CoreTextRenderer.is_available():  # type: ignore
        available.append("simple-coretext")
    if HARFBUZZ_AVAILABLE and HarfBuzzRenderer.is_available():  # type: ignore
        available.append("simple-harfbuzz")
    return available


def create_renderer(
    backend: str,
    font_path: Path | str,
    font_size: int = 64,
    width: int = 2000,
    height: int = 200,
) -> BaseRenderer:
    """
    Create a simple renderer instance.

    Args:
        backend: Backend name ("simple-coretext" or "simple-harfbuzz")
        font_path: Path to font file
        font_size: Font size in points
        width: Canvas width in pixels
        height: Canvas height in pixels

    Returns:
        Renderer instance

    Raises:
        RendererInitError: If backend is not available or initialization fails
    """
    font_path = Path(font_path)

    if backend == "simple-coretext":
        if not CORETEXT_AVAILABLE or not CoreTextRenderer.is_available():  # type: ignore
            raise RendererUnavailableError("CoreText renderer not available")
        return CoreTextRenderer(  # type: ignore
            font_path,
            width=width,
            height=height,
            font_size=font_size,
        )
    elif backend == "simple-harfbuzz":
        if not HARFBUZZ_AVAILABLE or not HarfBuzzRenderer.is_available():  # type: ignore
            raise RendererUnavailableError("HarfBuzz renderer not available")
        return HarfBuzzRenderer(  # type: ignore
            font_path,
            width=width,
            height=height,
            font_size=font_size,
        )
    else:
        raise RendererInitError(f"Unknown backend: {backend}")


def render_text_simple(
    text: str,
    font_path: Path | str,
    backend: str = "auto",
    font_size: int = 64,
    width: int = 2000,
    height: int = 200,
) -> np.ndarray:
    """
    Simple API to render text with automatic backend selection.

    Args:
        text: Text to render
        font_path: Path to font file
        backend: Backend name or "auto" for automatic selection
        font_size: Font size in points
        width: Canvas width in pixels
        height: Canvas height in pixels

    Returns:
        Grayscale numpy array (0=black, 255=white)
    """
    if backend == "auto":
        # Auto-select: prefer CoreText on macOS, HarfBuzz elsewhere
        available = list_available()
        if not available:
            raise RendererUnavailableError("No simple renderers available")

        if platform.system() == "Darwin" and "simple-coretext" in available:
            backend = "simple-coretext"
        elif "simple-harfbuzz" in available:
            backend = "simple-harfbuzz"
        else:
            backend = available[0]

    renderer = create_renderer(backend, font_path, font_size, width, height)
    return renderer.render_text(text)


__all__ = [
    "BaseRenderer",
    "RendererInitError",
    "RendererUnavailableError",
    "list_available",
    "create_renderer",
    "render_text_simple",
    "DEFAULT_FONT_SIZE",
    "RENDER_WIDTH",
    "RENDER_HEIGHT",
]

if CORETEXT_AVAILABLE:
    __all__.append("CoreTextRenderer")
if HARFBUZZ_AVAILABLE:
    __all__.append("HarfBuzzRenderer")
