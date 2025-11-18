#!/usr/bin/env python3
"""Test native module directly."""

from typf import typf as native

# Test CoreText directly
renderer = native.TextRenderer("coretext")
font = native.Font("Arial", 48.0)

result = renderer.render("Hello", font, format="png")
print(f"Result type: {type(result)}")
print(f"Result length: {len(result)} bytes")

with open("test-native-coretext.png", "wb") as f:
    f.write(result)
print("Saved test-native-coretext.png")

# Test OrgeHB directly
renderer2 = native.TextRenderer("orgehb")
result2 = renderer2.render("Hello", font, format="png")
with open("test-native-orgehb.png", "wb") as f:
    f.write(result2)
print("Saved test-native-orgehb.png")
