#!/usr/bin/env python3
"""Test to understand bbox values."""

import typf

# Create renderer and font
renderer = typf.TextRenderer(backend="coretext")
font = typf.Font("Arial", 48.0)

# Test with a simple character
text = "A"

# Try rendering
result = renderer.render(text, font, format="png")
if result:
    with open("test-A.png", "wb") as f:
        f.write(result)
    print(f"Rendered '{text}' to test-A.png")
