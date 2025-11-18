#!/usr/bin/env python3
"""Diagnostic script to compare OrgeHB vs SkiaHB rendering outputs."""

import typf
from PIL import Image
import numpy as np


def analyze_bitmap(data, width, height, name):
    """Analyze a bitmap and print statistics."""
    arr = np.array(data, dtype=np.uint8).reshape(height, width, 4)
    alpha = arr[:, :, 3]

    # Count non-zero pixels
    non_zero = np.count_nonzero(alpha)
    total = alpha.size
    percentage = (non_zero / total) * 100

    # Get alpha value statistics
    alpha_values = alpha[alpha > 0]

    print(f"\n{name} Statistics:")
    print(f"  Dimensions: {width}x{height} ({total:,} pixels)")
    print(f"  Non-zero pixels: {non_zero:,} ({percentage:.2f}%)")
    if len(alpha_values) > 0:
        print(f"  Alpha range: {alpha_values.min()}-{alpha_values.max()}")
        print(f"  Alpha mean: {alpha_values.mean():.1f}")

    return {
        'width': width,
        'height': height,
        'non_zero': non_zero,
        'percentage': percentage,
        'alpha_mean': alpha_values.mean() if len(alpha_values) > 0 else 0
    }


def compare_backends():
    """Compare OrgeHB and SkiaHB rendering outputs."""
    text = "The quick brown fox"
    font_size = 48.0

    # Test with OrgeHB
    print("=" * 60)
    print("Testing OrgeHB backend")
    print("=" * 60)

    renderer_orgehb = typf.TextRenderer(backend="orgehb")
    font = typf.Font("Georgia", font_size)

    # Shape and render
    shaped_orgehb = typf.shape_text(text, font, backend="orgehb")
    print(f"\nShaped result:")
    print(f"  Text: {shaped_orgehb.text}")
    print(f"  Glyphs: {len(shaped_orgehb.glyphs)}")
    print(f"  Advance: {shaped_orgehb.advance:.2f}")
    print(f"  BBox: x={shaped_orgehb.bbox.x:.2f}, y={shaped_orgehb.bbox.y:.2f}, "
          f"w={shaped_orgehb.bbox.width:.2f}, h={shaped_orgehb.bbox.height:.2f}")

    if shaped_orgehb.glyphs:
        print(f"\nFirst 3 glyphs:")
        for i, g in enumerate(shaped_orgehb.glyphs[:3]):
            print(f"    [{i}] id={g.id}, x={g.x:.2f}, y={g.y:.2f}, advance={g.advance:.2f}")

    result_orgehb = renderer_orgehb.render(text, font, format="raw")
    bitmap_orgehb = typf.Bitmap(result_orgehb, shaped_orgehb.bbox.width, shaped_orgehb.bbox.height)

    orgehb_stats = analyze_bitmap(
        bitmap_orgehb.data,
        bitmap_orgehb.width,
        bitmap_orgehb.height,
        "OrgeHB"
    )

    # Save image
    img_orgehb = Image.frombytes('RGBA', (bitmap_orgehb.width, bitmap_orgehb.height), bytes(bitmap_orgehb.data))
    img_orgehb.save('debug-orgehb.png')
    print(f"  Saved: debug-orgehb.png")

    # Test with SkiaHB
    print("\n" + "=" * 60)
    print("Testing SkiaHB backend")
    print("=" * 60)

    renderer_skiahb = typf.TextRenderer(backend="skiahb")
    shaped_skiahb = typf.shape_text(text, font, backend="skiahb")

    print(f"\nShaped result:")
    print(f"  Text: {shaped_skiahb.text}")
    print(f"  Glyphs: {len(shaped_skiahb.glyphs)}")
    print(f"  Advance: {shaped_skiahb.advance:.2f}")
    print(f"  BBox: x={shaped_skiahb.bbox.x:.2f}, y={shaped_skiahb.bbox.y:.2f}, "
          f"w={shaped_skiahb.bbox.width:.2f}, h={shaped_skiahb.bbox.height:.2f}")

    if shaped_skiahb.glyphs:
        print(f"\nFirst 3 glyphs:")
        for i, g in enumerate(shaped_skiahb.glyphs[:3]):
            print(f"    [{i}] id={g.id}, x={g.x:.2f}, y={g.y:.2f}, advance={g.advance:.2f}")

    result_skiahb = renderer_skiahb.render(text, font, format="raw")
    bitmap_skiahb = typf.Bitmap(result_skiahb, shaped_skiahb.bbox.width, shaped_skiahb.bbox.height)

    skiahb_stats = analyze_bitmap(
        bitmap_skiahb.data,
        bitmap_skiahb.width,
        bitmap_skiahb.height,
        "SkiaHB"
    )

    # Save image
    img_skiahb = Image.frombytes('RGBA', (bitmap_skiahb.width, bitmap_skiahb.height), bytes(bitmap_skiahb.data))
    img_skiahb.save('debug-skiahb.png')
    print(f"  Saved: debug-skiahb.png")

    # Comparison
    print("\n" + "=" * 60)
    print("COMPARISON")
    print("=" * 60)

    print(f"\nDimensions:")
    print(f"  OrgeHB: {orgehb_stats['width']}x{orgehb_stats['height']}")
    print(f"  SkiaHB: {skiahb_stats['width']}x{skiahb_stats['height']}")

    print(f"\nVisible pixels:")
    print(f"  OrgeHB: {orgehb_stats['percentage']:.2f}%")
    print(f"  SkiaHB: {skiahb_stats['percentage']:.2f}%")
    if skiahb_stats['percentage'] > 0:
        print(f"  Ratio: {orgehb_stats['percentage'] / skiahb_stats['percentage']:.2f}x")

    print(f"\nBBox comparison:")
    print(f"  OrgeHB bbox: {shaped_orgehb.bbox.width:.2f}x{shaped_orgehb.bbox.height:.2f}")
    print(f"  SkiaHB bbox: {shaped_skiahb.bbox.width:.2f}x{shaped_skiahb.bbox.height:.2f}")

    print(f"\nAdvance comparison:")
    print(f"  OrgeHB: {shaped_orgehb.advance:.2f}")
    print(f"  SkiaHB: {shaped_skiahb.advance:.2f}")
    if shaped_skiahb.advance > 0:
        print(f"  Ratio: {shaped_orgehb.advance / shaped_skiahb.advance:.4f}x")


if __name__ == "__main__":
    compare_backends()
