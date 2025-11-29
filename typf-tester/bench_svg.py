#!/usr/bin/env python3
"""
SVG vs PNG Benchmark Tool for TYPF

Compares performance and file sizes between SVG vector output
and PNG bitmap output across all renderers.
"""

import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Optional

# Import from bindings/python if available (development)
sys.path.insert(0, str(Path(__file__).parent.parent / "bindings" / "python"))

try:
    import typf
except ImportError:
    print("❌ TYPF not available. Build with: cd bindings/python && maturin develop")
    sys.exit(1)


@dataclass
class BenchmarkResult:
    """Results from a single benchmark run."""

    renderer: str
    format: str
    text: str
    iterations: int
    total_time_sec: float
    avg_time_ms: float
    ops_per_sec: float
    output_size_bytes: int


class SVGBenchmark:
    """SVG vs PNG performance benchmark."""

    def __init__(self):
        """Initialize benchmark configuration."""
        self.base_dir = Path(__file__).parent
        self.output_dir = self.base_dir / "output"
        self.output_dir.mkdir(exist_ok=True)

        # Test configurations
        self.sample_text = "AVAST Wallflower Efficiency"  # Same as typfme.py
        self.font_path = self.base_dir.parent / "test-fonts" / "Kalnia[wdth,wght].ttf"
        self.font_size = 48.0
        self.iterations = 500  # More iterations for statistical significance

        # Renderers that support both PNG and SVG
        self.renderers = ["coregraphics", "opixa", "skia", "zeno"]

    def benchmark_format(self, renderer: str, format: str) -> Optional[BenchmarkResult]:
        """Benchmark a specific renderer and format combination."""
        try:
            engine = typf.Typf(shaper="harfbuzz", renderer=renderer)

            # Warmup (5 iterations)
            for _ in range(5):
                if format == "svg":
                    _ = engine.render_to_svg(
                        self.sample_text,
                        str(self.font_path),
                        size=self.font_size,
                        color=(0, 0, 0, 255),
                        padding=20,
                    )
                else:  # png
                    result = engine.render_text(
                        self.sample_text,
                        str(self.font_path),
                        size=self.font_size,
                        color=(0, 0, 0, 255),
                        background=(255, 255, 255, 255),
                        padding=20,
                    )
                    _ = typf.export_image(result, format="png")

            # Actual benchmark
            start = time.perf_counter()
            output = None

            for _ in range(self.iterations):
                if format == "svg":
                    output = engine.render_to_svg(
                        self.sample_text,
                        str(self.font_path),
                        size=self.font_size,
                        color=(0, 0, 0, 255),
                        padding=20,
                    )
                else:  # png
                    result = engine.render_text(
                        self.sample_text,
                        str(self.font_path),
                        size=self.font_size,
                        color=(0, 0, 0, 255),
                        background=(255, 255, 255, 255),
                        padding=20,
                    )
                    output = typf.export_image(result, format="png")

            elapsed = time.perf_counter() - start

            # Calculate metrics
            avg_ms = (elapsed / self.iterations) * 1000
            ops_per_sec = self.iterations / elapsed

            # Get output size from last iteration
            if format == "svg":
                output_size = (
                    len(output.encode("utf-8"))
                    if isinstance(output, str)
                    else len(output)
                )
            else:
                output_size = len(output) if output else 0

            return BenchmarkResult(
                renderer=renderer,
                format=format,
                text=self.sample_text,
                iterations=self.iterations,
                total_time_sec=elapsed,
                avg_time_ms=avg_ms,
                ops_per_sec=ops_per_sec,
                output_size_bytes=output_size,
            )

        except Exception as e:
            print(f"⚠ Error benchmarking {renderer} {format}: {e}")
            return None

    def run(self):
        """Run comprehensive SVG vs PNG benchmark."""
        print("=" * 100)
        print("TYPF SVG vs PNG BENCHMARK")
        print("=" * 100)
        print(f"Text: {self.sample_text}")
        print(f"Font: {self.font_path.name}")
        print(f"Size: {self.font_size}px")
        print(f"Iterations: {self.iterations}\n")

        results: List[BenchmarkResult] = []

        # Benchmark each renderer with both formats
        for renderer in self.renderers:
            print(f"\n{renderer.upper()}")
            print("-" * 100)

            for format in ["png", "svg"]:
                result = self.benchmark_format(renderer, format)
                if result:
                    results.append(result)
                    size_kb = result.output_size_bytes / 1024
                    print(
                        f"  {format.upper():<5} {result.avg_time_ms:>8.3f}ms/op  "
                        f"{result.ops_per_sec:>10.1f} ops/sec  {size_kb:>8.2f} KB"
                    )

        if not results:
            print("❌ No benchmark results collected")
            return 1

        # Generate comparison tables
        self._generate_comparison_table(results)
        self._generate_efficiency_analysis(results)
        self._generate_summary(results)

        return 0

    def _generate_comparison_table(self, results: List[BenchmarkResult]):
        """Generate performance comparison table."""
        print("\n" + "=" * 100)
        print("PERFORMANCE COMPARISON: SVG vs PNG")
        print("=" * 100)
        print(
            f"{'Renderer':<15} {'PNG (ms)':<12} {'SVG (ms)':<12} {'Speedup':<10} {'PNG Size':<12} {'SVG Size':<12} {'Ratio'}"
        )
        print("-" * 100)

        # Group by renderer
        by_renderer: Dict[str, Dict[str, BenchmarkResult]] = {}
        for r in results:
            if r.renderer not in by_renderer:
                by_renderer[r.renderer] = {}
            by_renderer[r.renderer][r.format] = r

        for renderer in sorted(by_renderer.keys()):
            formats = by_renderer[renderer]
            if "png" in formats and "svg" in formats:
                png = formats["png"]
                svg = formats["svg"]

                speedup = png.avg_time_ms / svg.avg_time_ms
                size_ratio = svg.output_size_bytes / png.output_size_bytes

                png_kb = png.output_size_bytes / 1024
                svg_kb = svg.output_size_bytes / 1024

                # Determine which is faster
                if speedup > 1:
                    speedup_str = f"SVG {speedup:.2f}x"
                else:
                    speedup_str = f"PNG {1 / speedup:.2f}x"

                print(
                    f"{renderer:<15} {png.avg_time_ms:>9.3f}   {svg.avg_time_ms:>9.3f}   "
                    f"{speedup_str:<10} {png_kb:>9.2f} KB  {svg_kb:>9.2f} KB  {size_ratio:>5.2f}x"
                )

    def _generate_efficiency_analysis(self, results: List[BenchmarkResult]):
        """Analyze efficiency trade-offs."""
        print("\n" + "=" * 100)
        print("EFFICIENCY ANALYSIS")
        print("=" * 100)

        # Find best in each category
        png_results = [r for r in results if r.format == "png"]
        svg_results = [r for r in results if r.format == "svg"]

        if png_results:
            fastest_png = min(png_results, key=lambda r: r.avg_time_ms)
            smallest_png = min(png_results, key=lambda r: r.output_size_bytes)

            print(f"\n📊 PNG Results:")
            print(
                f"   Fastest: {fastest_png.renderer} @ {fastest_png.avg_time_ms:.3f}ms"
            )
            print(
                f"   Smallest: {smallest_png.renderer} @ {smallest_png.output_size_bytes / 1024:.2f} KB"
            )

        if svg_results:
            fastest_svg = min(svg_results, key=lambda r: r.avg_time_ms)
            smallest_svg = min(svg_results, key=lambda r: r.output_size_bytes)

            print(f"\n🎨 SVG Results:")
            print(
                f"   Fastest: {fastest_svg.renderer} @ {fastest_svg.avg_time_ms:.3f}ms"
            )
            print(
                f"   Smallest: {smallest_svg.renderer} @ {smallest_svg.output_size_bytes / 1024:.2f} KB"
            )

        # Overall winner
        all_fastest = min(results, key=lambda r: r.avg_time_ms)
        all_smallest = min(results, key=lambda r: r.output_size_bytes)

        print(f"\n🏆 Overall Winners:")
        print(
            f"   Fastest: {all_fastest.renderer} {all_fastest.format.upper()} @ {all_fastest.avg_time_ms:.3f}ms"
        )
        print(
            f"   Smallest: {all_smallest.renderer} {all_smallest.format.upper()} @ {all_smallest.output_size_bytes / 1024:.2f} KB"
        )

    def _generate_summary(self, results: List[BenchmarkResult]):
        """Generate executive summary."""
        print("\n" + "=" * 100)
        print("SUMMARY")
        print("=" * 100)

        png_count = len([r for r in results if r.format == "png"])
        svg_count = len([r for r in results if r.format == "svg"])

        png_avg_time = sum(r.avg_time_ms for r in results if r.format == "png") / max(
            png_count, 1
        )
        svg_avg_time = sum(r.avg_time_ms for r in results if r.format == "svg") / max(
            svg_count, 1
        )

        png_avg_size = sum(
            r.output_size_bytes for r in results if r.format == "png"
        ) / max(png_count, 1)
        svg_avg_size = sum(
            r.output_size_bytes for r in results if r.format == "svg"
        ) / max(svg_count, 1)

        print(f"\n📈 Average Performance:")
        print(f"   PNG: {png_avg_time:.3f}ms/op")
        print(f"   SVG: {svg_avg_time:.3f}ms/op")

        if svg_avg_time > 0:
            if png_avg_time < svg_avg_time:
                print(
                    f"   → PNG is {svg_avg_time / png_avg_time:.2f}x faster on average"
                )
            else:
                print(
                    f"   → SVG is {png_avg_time / svg_avg_time:.2f}x faster on average"
                )

        print(f"\n💾 Average File Size:")
        print(f"   PNG: {png_avg_size / 1024:.2f} KB")
        print(f"   SVG: {svg_avg_size / 1024:.2f} KB")
        print(
            f"   → Size ratio: SVG is {svg_avg_size / png_avg_size:.2f}x the size of PNG"
        )

        print("\n" + "=" * 100)
        print(f"✅ Benchmarked {len(results)} configurations successfully")
        print("=" * 100)


def main():
    """Main entry point."""
    benchmark = SVGBenchmark()
    return benchmark.run()


if __name__ == "__main__":
    sys.exit(main())
