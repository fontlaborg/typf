#!/usr/bin/env python3
"""
Performance Comparison Tool for Typf Renderers

Analyzes benchmark data and creates visual comparisons of renderer performance.
"""

import json
from collections import defaultdict
from pathlib import Path


def load_benchmark_data():
    """Load benchmark data from JSON report."""
    report_path = Path("output/benchmark_report.json")
    if not report_path.exists():
        print("❌ Benchmark report not found. Run ./build.sh first.")
        return None

    with open(report_path) as f:
        return json.load(f)


def group_by_renderer(data):
    """Group benchmark results by renderer."""
    renderer_stats = defaultdict(lambda: {"times": [], "ops": []})

    for backend in data["backends"]:
        renderer = backend["renderer"]
        renderer_stats[renderer]["times"].append(backend["avg_time_ms"])
        renderer_stats[renderer]["ops"].append(backend["ops_per_sec"])

    return renderer_stats


def create_comparison_table(stats):
    """Create ASCII table comparing renderer performance."""
    print("\n" + "=" * 80)
    print("RENDERER PERFORMANCE COMPARISON")
    print("=" * 80)
    print(f"{'Renderer':<15} {'Avg Time (ms)':<18} {'Avg Ops/sec':<18} {'Samples':<10}")
    print("-" * 80)

    # Calculate averages and sort by speed (ops/sec descending)
    renderer_avgs = []
    for renderer, data in stats.items():
        avg_time = sum(data["times"]) / len(data["times"])
        avg_ops = sum(data["ops"]) / len(data["ops"])
        samples = len(data["times"])
        renderer_avgs.append((renderer, avg_time, avg_ops, samples))

    renderer_avgs.sort(key=lambda x: x[2], reverse=True)  # Sort by ops/sec

    for renderer, avg_time, avg_ops, samples in renderer_avgs:
        print(f"{renderer:<15} {avg_time:>15.3f}   {avg_ops:>15.1f}   {samples:>8}")

    print("=" * 80)

    # Show speed ratios
    fastest = renderer_avgs[0]
    slowest = renderer_avgs[-1]

    print(f"\n📊 Performance Insights:")
    print(f"  • Fastest: {fastest[0]} at {fastest[2]:.1f} ops/sec")
    print(f"  • Slowest: {slowest[0]} at {slowest[2]:.1f} ops/sec")
    print(f"  • Speed ratio: {fastest[2] / slowest[2]:.1f}x faster")

    # Bitmap renderers only (exclude JSON)
    bitmap_renderers = [r for r in renderer_avgs if r[0] not in ["json", "svg"]]
    if bitmap_renderers:
        print(f"\n🎨 Bitmap Renderers:")
        for renderer, avg_time, avg_ops, samples in bitmap_renderers:
            print(f"  • {renderer:15s}: {avg_time:6.2f}ms  ({avg_ops:8.1f} ops/sec)")


def create_visual_chart(stats):
    """Create simple ASCII bar chart."""
    print("\n" + "=" * 80)
    print("RELATIVE PERFORMANCE CHART (Ops/sec)")
    print("=" * 80)

    # Calculate averages
    renderer_avgs = []
    for renderer, data in stats.items():
        avg_ops = sum(data["ops"]) / len(data["ops"])
        renderer_avgs.append((renderer, avg_ops))

    renderer_avgs.sort(key=lambda x: x[1], reverse=True)

    # Find max for scaling
    max_ops = renderer_avgs[0][1]

    for renderer, ops in renderer_avgs:
        bar_length = int((ops / max_ops) * 50)
        bar = "█" * bar_length
        print(f"{renderer:15s} {bar} {ops:8.1f}")

    print("=" * 80)


def main():
    """Main entry point."""
    data = load_benchmark_data()
    if not data:
        return 1

    stats = group_by_renderer(data)
    create_comparison_table(stats)
    create_visual_chart(stats)

    print(f"\n✅ Analysis complete!")
    print(f"   Total backends tested: {len(data['backends'])}")
    print(f"   Unique renderers: {len(stats)}")
    print(f"\nCommunity project by FontLab - https://www.fontlab.org/\n")

    return 0


if __name__ == "__main__":
    import sys

    sys.exit(main())
