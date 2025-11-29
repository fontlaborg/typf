#!/usr/bin/env python3
"""
Example of rendering text with a real font file
"""

import sys
import typf

if len(sys.argv) < 3:
    print("Usage: python render_with_font.py <font_path> <text>")
    print("Example: python render_with_font.py /System/Library/Fonts/Arial.ttf 'Hello World'")
    sys.exit(1)

font_path = sys.argv[1]
text = sys.argv[2]

# Create TYPF instance with HarfBuzz shaper
engine = typf.Typf(shaper="harfbuzz", renderer="opixa")

# Render text with custom colors
image_data = engine.render_text(
    text,
    font_path=font_path,
    size=64.0,
    color=(0, 0, 255, 255),  # Blue text
    background=(255, 255, 200, 255),  # Light yellow background
    padding=20
)

# Export to PNG
output_png = typf.export_image(image_data, format="png")
with open("output.png", "wb") as f:
    f.write(output_png)

print(f"✓ Rendered {image_data['width']}x{image_data['height']} image")
print(f"✓ Saved to output.png ({len(output_png)} bytes)")
