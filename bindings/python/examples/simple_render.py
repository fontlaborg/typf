#!/usr/bin/env python3
"""Simple example of using Typf Python bindings"""

from typfpy import render_simple, export_image

# Simple rendering with stub font
image_data = render_simple("Hello, Typf!", size=48.0)
output = export_image(image_data, format="ppm")

with open("output.ppm", "wb") as f:
    f.write(output)

print(f"Rendered {image_data['width']}x{image_data['height']} image to output.ppm")
