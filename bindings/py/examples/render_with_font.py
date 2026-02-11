#!/usr/bin/env python3
"""Render text with a real font file using Typf"""

import sys
from typfpy import Typf, export_image

if len(sys.argv) < 3:
    print("Usage: python render_with_font.py <font_path> <text>")
    print("Example: python render_with_font.py /System/Library/Fonts/Arial.ttf 'Hello World'")
    sys.exit(1)

font_path = sys.argv[1]
text = sys.argv[2]

# Create rendering pipeline with HarfBuzz shaping and Opixa rendering
typf = Typf(shaper="harfbuzz", renderer="opixa")

# Render text with the specified font
image_data = typf.render_text(
    text,
    font_path=font_path,
    size=48.0,
    color=(0, 0, 0, 255),
    padding=10,
)

# Export as PNG
output = export_image(image_data, format="png")

with open("output.png", "wb") as f:
    f.write(output)

print(f"Rendered '{text}' ({image_data['width']}x{image_data['height']}) to output.png")
