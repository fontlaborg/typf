#!/usr/bin/env python3
"""
Renderer Quality Comparison Tool for Typf

Analyzes PNG outputs to compare rendering quality across different backends.
Provides metrics for:
- Pixel coverage (how much ink is used)
- Anti-aliasing quality (number of intermediate gray levels)
- File size efficiency
- Visual consistency across shapers
"""

import sys
from pathlib import Path
from collections import defaultdict
from typing import Dict, List, Tuple

try:
    from PIL import Image
except ImportError:
    print("❌ Pillow not installed. Install with: pip install Pillow")
    sys.exit(1)


def analyze_png_quality(png_path: Path) -> Dict:
    """Analyze quality metrics of a PNG file."""
    img = Image.open(png_path).convert('L')  # Convert to grayscale
    pixels = list(img.getdata())
    width, height = img.size

    # Count pixel distribution
    black_pixels = sum(1 for p in pixels if p == 0)
    white_pixels = sum(1 for p in pixels if p == 255)
    gray_pixels = sum(1 for p in pixels if 0 < p < 255)

    # Calculate coverage (percentage of non-white pixels)
    coverage = ((black_pixels + gray_pixels) / len(pixels)) * 100

    # Anti-aliasing quality (number of unique gray levels)
    unique_grays = len(set(p for p in pixels if 0 < p < 255))

    # Smoothness score (higher is better anti-aliasing)
    smoothness = (gray_pixels / (black_pixels + gray_pixels + 1)) * 100

    return {
        'width': width,
        'height': height,
        'total_pixels': len(pixels),
        'black_pixels': black_pixels,
        'white_pixels': white_pixels,
        'gray_pixels': gray_pixels,
        'coverage_pct': coverage,
        'unique_grays': unique_grays,
        'smoothness_pct': smoothness,
        'file_size_kb': png_path.stat().st_size / 1024
    }


def parse_filename(filename: str) -> Tuple[str, str]:
    """Extract shaper and renderer from filename."""
    # Format: render-{shaper}-{renderer}-latn.png
    parts = filename.replace('.png', '').split('-')
    if len(parts) >= 4:
        shaper = parts[1]
        renderer = parts[2]
        return shaper, renderer
    return 'unknown', 'unknown'


def group_by_renderer(results: Dict) -> Dict[str, List]:
    """Group analysis results by renderer."""
    by_renderer = defaultdict(list)
    for filename, stats in results.items():
        _, renderer = parse_filename(filename)
        by_renderer[renderer].append((filename, stats))
    return by_renderer


def create_quality_table(results: Dict):
    """Create ASCII table comparing renderer quality."""
    print("\n" + "="*100)
    print("RENDERER QUALITY COMPARISON")
    print("="*100)
    print(f"{'Renderer':<15} {'Coverage':<10} {'AA Grays':<10} {'Smoothness':<12} {'File Size':<10} {'Resolution'}")
    print("-"*100)

    by_renderer = group_by_renderer(results)

    for renderer in sorted(by_renderer.keys()):
        samples = by_renderer[renderer]
        # Average metrics across shapers
        avg_coverage = sum(s[1]['coverage_pct'] for s in samples) / len(samples)
        avg_grays = sum(s[1]['unique_grays'] for s in samples) / len(samples)
        avg_smoothness = sum(s[1]['smoothness_pct'] for s in samples) / len(samples)
        avg_size = sum(s[1]['file_size_kb'] for s in samples) / len(samples)
        resolution = f"{samples[0][1]['width']}x{samples[0][1]['height']}"

        print(f"{renderer:<15} {avg_coverage:>7.2f}%  {avg_grays:>6.1f}    {avg_smoothness:>9.2f}%  {avg_size:>7.2f} KB  {resolution}")


def create_quality_insights(results: Dict):
    """Generate insights about quality differences."""
    print("\n" + "="*100)
    print("QUALITY INSIGHTS")
    print("="*100)

    by_renderer = group_by_renderer(results)

    # Find best in each category
    renderer_stats = {}
    for renderer, samples in by_renderer.items():
        renderer_stats[renderer] = {
            'coverage': sum(s[1]['coverage_pct'] for s in samples) / len(samples),
            'grays': sum(s[1]['unique_grays'] for s in samples) / len(samples),
            'smoothness': sum(s[1]['smoothness_pct'] for s in samples) / len(samples),
            'size': sum(s[1]['file_size_kb'] for s in samples) / len(samples)
        }

    best_aa = max(renderer_stats.items(), key=lambda x: x[1]['grays'])
    best_smooth = max(renderer_stats.items(), key=lambda x: x[1]['smoothness'])
    smallest = min(renderer_stats.items(), key=lambda x: x[1]['size'])

    print(f"\n🏆 Best Anti-Aliasing: {best_aa[0]}")
    print(f"   → {best_aa[1]['grays']:.1f} unique gray levels")
    print(f"   → {best_aa[1]['smoothness']:.2f}% smoothness score")

    print(f"\n🎨 Smoothest Rendering: {best_smooth[0]}")
    print(f"   → {best_smooth[1]['smoothness']:.2f}% smoothness score")
    print(f"   → {best_smooth[1]['grays']:.1f} unique gray levels")

    print(f"\n💾 Most Efficient Compression: {smallest[0]}")
    print(f"   → {smallest[1]['size']:.2f} KB average file size")

    # Check consistency across shapers
    print("\n📊 Cross-Shaper Consistency:")
    for renderer, samples in sorted(by_renderer.items()):
        coverages = [s[1]['coverage_pct'] for s in samples]
        variance = max(coverages) - min(coverages)
        status = "✓ Consistent" if variance < 0.1 else "⚠ Variance detected"
        print(f"   {renderer:<15} {status:20} (Δ {variance:.3f}% coverage)")


def create_visual_chart(results: Dict):
    """Create visual bar chart of quality metrics."""
    print("\n" + "="*100)
    print("ANTI-ALIASING QUALITY (Gray Levels)")
    print("="*100)

    by_renderer = group_by_renderer(results)

    for renderer in sorted(by_renderer.keys()):
        samples = by_renderer[renderer]
        avg_grays = sum(s[1]['unique_grays'] for s in samples) / len(samples)

        # Scale to 50 characters max
        bar_length = int((avg_grays / 256) * 50)
        bar = '█' * bar_length

        print(f"{renderer:<15} {bar} {avg_grays:.1f} levels")


def main():
    """Main entry point."""
    output_dir = Path("output")
    if not output_dir.exists():
        print("❌ Output directory not found. Run ./build.sh first.")
        return 1

    # Find all PNG files (excluding JSON renderer)
    png_files = list(output_dir.glob("render-*-latn.png"))
    png_files = [f for f in png_files if '-json-' not in f.name]

    if not png_files:
        print("❌ No PNG files found in output directory.")
        return 1

    print(f"📊 Analyzing {len(png_files)} PNG outputs...\n")

    # Analyze all files
    results = {}
    for png_file in sorted(png_files):
        try:
            stats = analyze_png_quality(png_file)
            results[png_file.name] = stats
        except Exception as e:
            print(f"⚠ Failed to analyze {png_file.name}: {e}")

    if not results:
        print("❌ No results to analyze.")
        return 1

    # Generate reports
    create_quality_table(results)
    create_visual_chart(results)
    create_quality_insights(results)

    print("\n" + "="*100)
    print(f"✅ Analyzed {len(results)} PNG files successfully")
    print("="*100)

    return 0


if __name__ == "__main__":
    sys.exit(main())
