#!/usr/bin/env python3
"""
Rapid iteration tool for typf-render-opixa.
"""

import sys
import time
from pathlib import Path

# Ensure we can import typf
# Assuming we are running from typf-tester/opixa/
# We need to add the bindings directory to sys.path if not installed in site-packages
# But typically it's installed in the venv.
try:
    import typfpy as typf
except ImportError:
    print("Error: typfpy Python bindings not installed.")
    sys.exit(1)


def main():
    base_dir = Path(__file__).parent.parent  # typf-tester
    font_path = base_dir.parent / "test-fonts" / "Kalnia[wdth,wght].ttf"
    output_dir = Path(__file__).parent

    if not font_path.exists():
        print(f"Error: Font not found at {font_path}")
        sys.exit(1)

    print(f"Using font: {font_path}")

    # Initialize engine with none shaper and opixa renderer
    try:
        engine = typf.Typf(shaper="none", renderer="opixa")
        print(
            f"Initialized engine: Shaper={engine.get_shaper()}, Renderer={engine.get_renderer()}"
        )
    except Exception as e:
        print(f"Failed to initialize engine: {e}")
        sys.exit(1)

    text = "Hello World"
    size = 64.0

    print(f"Rendering text: '{text}' at size {size}")

    start_time = time.perf_counter()
    try:
        result = engine.render_text(
            text,
            str(font_path),
            size=size,
            color=(0, 0, 0, 255),
            background=(255, 255, 255, 255),
            padding=20,
        )
    except Exception as e:
        print(f"Rendering failed: {e}")
        sys.exit(1)

    elapsed = (time.perf_counter() - start_time) * 1000
    print(f"Rendered in {elapsed:.3f} ms")

    # Export to PNG
    output_filename = "output.png"
    output_path = output_dir / output_filename

    try:
        png_bytes = typf.export_image(result, format="png")
        output_path.write_bytes(png_bytes)
        print(f"Saved output to {output_path}")
    except Exception as e:
        print(f"Failed to export image: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
