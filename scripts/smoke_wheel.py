#!/usr/bin/env python3
"""Exercise an installed wheel using real font outlines, exports and both imports."""
# this_file: scripts/smoke_wheel.py

import importlib.metadata
import os
import sys
from pathlib import Path

import typf
import typfpy


def main():
    candidates = (
        [Path(sys.argv[1])]
        if len(sys.argv) > 1
        else [
            Path("test-fonts/NotoSans-Regular.ttf"),
            Path("/System/Library/Fonts/Helvetica.ttc"),
            Path(os.environ.get("WINDIR", "C:/Windows")) / "Fonts/arial.ttf",
            Path("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"),
        ]
    )
    font = next((path for path in candidates if path.is_file()), None)
    if font is None:
        raise RuntimeError("Smoke test requires a real font file")
    assert typf.Typf is typfpy.Typf, "Both imports must expose the same API"
    assert typf.__version__ == importlib.metadata.version("typf")
    if sys.platform == "darwin":
        native_pipeline = typf.Typf(shaper="coretext", renderer="coregraphics")
        native_image = native_pipeline.render_text("CoreText Typf", str(font), size=36)
        assert any(native_image["data"][3::4]), (
            "CoreText/CoreGraphics produced no visible pixels"
        )
    else:
        for backend in (
            {"shaper": "coretext"},
            {"shaper": "none", "renderer": "coregraphics"},
        ):
            try:
                typf.Typf(**backend)
            except ValueError:
                pass
            else:
                raise AssertionError(f"macOS backend unexpectedly available: {backend}")
    for renderer in ("opixa", "skia", "zeno", "vello-cpu"):
        image = typf.Typf(shaper="harfbuzz", renderer=renderer).render_text(
            "Typf AV fi", str(font), size=36
        )
        assert image["width"] > 0 and image["height"] > 0, renderer
        assert len(image["data"]) == image["width"] * image["height"] * 4, renderer
        assert any(image["data"][3::4]), f"{renderer} produced no visible pixels"
        assert typf.export_image(image, format="png").startswith(b"\x89PNG\r\n\x1a\n")
        assert b"<svg" in typf.export_image(image, format="svg")
        print(f"{renderer}: {image['width']}x{image['height']}, PNG and SVG")
    if sys.platform in ("darwin", "win32"):
        assert typf.__linra_available__, "Native linra backend must remain available"
        image = typf.TypfLinra().render_text("Native Typf", str(font), size=36)
        assert image["width"] > 0 and image["height"] > 0
        assert any(image["data"][3::4]), "Native linra produced no visible pixels"
        assert typf.export_image(image, format="png").startswith(b"\x89PNG\r\n\x1a\n")
        print(f"native linra: {image['width']}x{image['height']}, PNG")
    else:
        assert not typf.__linra_available__, (
            "Unsupported platform advertised native linra"
        )
    print(f"typf and typfpy {typf.__version__}: smoke passed")


if __name__ == "__main__":
    main()
