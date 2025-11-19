#!/usr/bin/env python3
"""Example: Handling Long Text with TYPF Python Bindings

This example demonstrates strategies for rendering long text that exceeds
the bitmap width limit (~10,000 pixels):

1. Detecting when text is too long for bitmap rendering
2. Using SVG export as an alternative (no width limits)
3. Implementing simple line wrapping for multi-line rendering
4. Measuring text width to make informed decisions

Usage:
    python long_text_handling.py

Requirements:
    pip install typf
"""

import typf
from pathlib import Path


# Sample long text (from typography essay - ~1000 chars)
LONG_TEXT = """Typography is the art and technique of arranging type to make written \
language legible, readable, and appealing when displayed. The arrangement \
of type involves selecting typefaces, point sizes, line lengths, line-spacing, \
and letter-spacing, and adjusting the space between pairs of letters. \
The term typography is also applied to the style, arrangement, and appearance \
of the letters, numbers, and symbols created by the process. Type design is \
a closely related craft, sometimes considered part of typography; most typographers \
do not design typefaces, and some type designers do not consider themselves typographers. \
Typography also may be used as an ornamental and decorative device, unrelated to the \
communication of information. In contemporary use, the practice and study of typography \
include a broad range, covering all aspects of letter design and application, both \
mechanical (typesetting, type design, and typefaces) and manual (handwriting and calligraphy)."""


def wrap_text(text: str, max_chars: int) -> list[str]:
    """Simple word-based line wrapping"""
    lines = []
    current_line = ""

    for word in text.split():
        if not current_line:
            current_line = word
        elif len(current_line) + len(word) + 1 <= max_chars:
            current_line += " " + word
        else:
            lines.append(current_line)
            current_line = word

    if current_line:
        lines.append(current_line)

    return lines


def calculate_adaptive_font_size(char_count: int, target_width: int) -> float:
    """Calculate adaptive font size to fit text in target width"""
    # Assuming ~0.55 character width ratio
    char_width_ratio = 0.55
    max_font_size = 72.0  # Don't go above this
    min_font_size = 8.0   # Don't go below this

    calculated_size = target_width / (char_count * char_width_ratio)
    return max(min_font_size, min(max_font_size, calculated_size))


def main():
    print("TYPF Python Long Text Handling Examples")
    print("=" * 80)
    print(f"\nText length: {len(LONG_TEXT)} characters\n")

    # Get a font path (adjust to your system)
    font_path = "/System/Library/Fonts/Helvetica.ttc"  # macOS
    # On Linux: font_path = "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"
    # On Windows: font_path = "C:\\Windows\\Fonts\\arial.ttf"

    if not Path(font_path).exists():
        print(f"⚠️  Font not found: {font_path}")
        print("Please adjust font_path in the script for your system")
        return

    # Strategy 1: Check if text fits within bitmap limits
    print("Strategy 1: Check Width Before Rendering")
    print("-" * 80)

    font_size = 48.0
    max_bitmap_width = 10_000

    # Estimate: typical character width is ~0.5-0.6 of font size
    estimated_char_width = font_size * 0.55
    estimated_width = int(len(LONG_TEXT) * estimated_char_width)

    print(f"Font size: {font_size}px")
    print(f"Estimated width: {estimated_width}px")
    print(f"Bitmap limit: {max_bitmap_width}px")

    if estimated_width > max_bitmap_width:
        print("⚠️  Text too wide for bitmap rendering!")
        print("   Recommendation: Use SVG export or line wrapping\n")

    # Strategy 2: Use SVG export (no width limits)
    print("Strategy 2: SVG Export (No Width Limits)")
    print("-" * 80)

    try:
        engine = typf.Typf(shaper="harfbuzz", renderer="orge")

        # SVG export has no practical width limits
        svg_output = engine.render_to_svg(
            LONG_TEXT,
            font_path,
            size=font_size,
            padding=20
        )

        output_file = "long_text_output.svg"
        with open(output_file, "w") as f:
            f.write(svg_output)

        print(f"✓ Successfully rendered {len(LONG_TEXT)} characters to SVG")
        print(f"  Output: {output_file}")
        print(f"  File size: {len(svg_output) / 1024:.1f}KB")
        print("  (SVG has no width limits - can handle any length)\n")

    except Exception as e:
        print(f"⚠️  SVG export failed: {e}")
        print("  (Make sure TYPF was built with export-svg feature)\n")

    # Strategy 3: Simple line wrapping
    print("Strategy 3: Line Wrapping for Multi-line Rendering")
    print("-" * 80)

    max_chars_per_line = int(max_bitmap_width / estimated_char_width)
    print(f"Max characters per line: ~{max_chars_per_line}")

    # Simple word-based line wrapping
    lines = wrap_text(LONG_TEXT, max_chars_per_line)
    print(f"Wrapped into {len(lines)} lines:")

    for i, line in enumerate(lines[:3]):
        preview = line[:50] + "..." if len(line) > 50 else line
        print(f"  Line {i + 1}: {len(line)} chars - \"{preview}\"")

    if len(lines) > 3:
        print(f"  ... {len(lines) - 3} more lines")
    print()

    # Render each line separately (would work within bitmap limits)
    print("  Each line can be rendered separately:")
    try:
        for i, line in enumerate(lines[:2]):  # Just show first 2 as example
            result = engine.shape_text(line, font_path, size=font_size)
            print(f"    Line {i + 1}: {result['glyph_count']} glyphs, "
                  f"{result['width']:.0f}px wide ✓")
    except Exception as e:
        print(f"    Error: {e}")
    print()

    # Strategy 4: Adaptive font sizing
    print("Strategy 4: Adaptive Font Sizing")
    print("-" * 80)

    target_width = 800  # Target width in pixels
    adaptive_size = calculate_adaptive_font_size(len(LONG_TEXT), target_width)

    print(f"For target width of {target_width}px:")
    print(f"Recommended font size: {adaptive_size:.1f}px")
    print(f"This would fit {int(target_width / (adaptive_size * 0.55))} characters")

    try:
        result = engine.shape_text(LONG_TEXT, font_path, size=adaptive_size)
        print(f"Actual width at {adaptive_size:.1f}px: {result['width']:.0f}px ✓\n")
    except Exception as e:
        print(f"Error: {e}\n")

    # Strategy 5: Chunked rendering
    print("Strategy 5: Chunked Rendering")
    print("-" * 80)

    chunk_size = 200  # Characters per chunk
    chunks = [LONG_TEXT[i:i+chunk_size] for i in range(0, len(LONG_TEXT), chunk_size)]

    print(f"Split into {len(chunks)} chunks of ~{chunk_size} characters:")
    print("Each chunk can be rendered separately and composited")
    print(f"Chunk 1: \"{chunks[0][:50]}...\"")
    print()

    print("=" * 80)
    print("Summary:")
    print("- For text < 200 chars at 48px: Bitmap rendering works fine")
    print("- For text > 200 chars: Use SVG export or line wrapping")
    print("- For very long documents: Use adaptive sizing or chunking")
    print("- SVG export is recommended for production use with long texts")


if __name__ == "__main__":
    main()
