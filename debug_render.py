#!/usr/bin/env python3
"""Debug rendering to understand coordinate systems."""

import typf

def debug_backend(backend_name):
    print(f"\n=== {backend_name.upper()} ===")
    try:
        renderer = typf.TextRenderer(backend=backend_name)
        font = typf.Font("Arial", 48.0)
        text = "Ag"  # A has ascender, g has descender

        # First, shape the text to see the bounding box
        shaped = renderer.shape(text, font)
        print(f"Text: '{text}'")
        print(f"Glyphs: {len(shaped.glyphs)}")
        print(f"BBox: width={shaped.width:.2f}, height={shaped.height:.2f}")
        print(f"Advance: {shaped.advance:.2f}")

        for i, g in enumerate(shaped.glyphs):
            print(f"  Glyph {i}: id={g.id}, x={g.x:.2f}, y={g.y:.2f}, adv={g.advance:.2f}")

        # Now render
        result = renderer.render(text, font, format="png")
        if result:
            filename = f"debug-{backend_name}.png"
            with open(filename, "wb") as f:
                f.write(result)
            print(f"✓ Saved {filename}")
        else:
            print("✗ No output")

    except Exception as e:
        print(f"✗ Error: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    for backend in ["coretext", "orgehb"]:
        debug_backend(backend)
