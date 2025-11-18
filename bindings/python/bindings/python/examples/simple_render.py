#!/usr/bin/env python3
"""
Simple example of using TYPF Python bindings
"""

import typf

# Simple rendering with stub font
image_data = typf.render_simple("Hello, TYPF!", size=48.0)
output = typf.export_image(image_data, format="ppm")

with open("output.ppm", "wb") as f:
    f.write(output)

print(f"Rendered {image_data['width']}x{image_data['height']} image to output.ppm")
