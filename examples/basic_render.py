#!/usr/bin/env python3
# this_file: examples/basic_render.py
"""Basic rendering example for typf."""

import sys

try:
    import typf
except ImportError:
    print("Error: typf not installed. Install with: pip install typf")
    sys.exit(1)


def main():
    """Demonstrate basic text rendering."""
    # Create renderer with auto backend selection
    renderer = typf.TextRenderer()

    print(f"Using backend: {renderer.backend}")
    print(f"typf version: {typf.__version__}")

    # Render simple text
    font = typf.Font("Arial", 48.0)

    texts = [
        "Hello World!",
        "Привет мир",  # Russian
        "Γειά σου κόσμε",  # Greek
        "مرحبا بالعالم",  # Arabic
        "你好世界",  # Chinese
    ]

    print("\nRendering test texts...")
    for text in texts:
        try:
            result = renderer.render(text, font, format="raw")
            if result:
                print(f"  ✓ {text[:20]}... -> {len(result)} bytes")
            else:
                print(f"  ✗ {text[:20]}... -> No output")
        except Exception as e:
            print(f"  ✗ {text[:20]}... -> Error: {e}")

    print("\n✓ Basic rendering test complete")


if __name__ == "__main__":
    main()
