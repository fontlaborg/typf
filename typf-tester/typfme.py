#!/usr/bin/env python3
"""TYPF v2.0 Backend Testing & Benchmarking Tool

A comprehensive tool to test, render, and benchmark all TYPF backend combinations
with multiple sample texts, font sizes, and output formats.

Usage:
    python typfme.py render           # Render samples with all backends
    python typfme.py bench            # Benchmark all backend combinations
    python typfme.py compare          # Side-by-side backend comparison

Made by FontLab https://www.fontlab.com/
"""

import json
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Optional, Tuple

import fire

# Try importing typf - fail gracefully with installation instructions
try:
    import typfpy as typf
except ImportError:
    print("Error: typfpy Python bindings not installed.")
    print("\nTo install:")
    print("  cd bindings/python")
    print("  maturin develop --release --features shaping-hb,export-png,export-svg")
    sys.exit(1)

# Try importing PIL for PNG verification
try:
    from PIL import Image
except ImportError:
    Image = None
    print("Note: PIL not available - PNG verification disabled")
    print("  Install: pip install pillow")


@dataclass
class BackendConfig:
    """Configuration for a shaping + rendering backend combination"""

    shaper: str
    renderer: str
    description: str
    available: bool = True
    error: Optional[str] = None


@dataclass
class BenchResult:
    """Benchmark result for a specific configuration"""

    config: BackendConfig
    text: str
    font_size: float
    iterations: int
    total_time: float
    avg_time_ms: float
    ops_per_sec: float
    success: bool
    error: Optional[str] = None
    output_size_bytes: Optional[int] = None  # Size of rendered output


class TypfTester:
    """TYPF v2.0 comprehensive testing and benchmarking tool"""

    def __init__(self):
        self.base_dir = Path(__file__).parent
        self.fonts_dir = self.base_dir / "fonts"
        self.output_dir = self.base_dir / "output"
        self.output_dir.mkdir(exist_ok=True)

        # Sample texts for testing (various scripts and complexities)
        self.sample_texts = {
            "latn": "AVAST Wallflower Efficiency",  # Kerning & ligatures
            "arab": "مرحبا بك في العالم",  # Arabic RTL text
            "mixd": "Hello, مرحبا, 你好!",  # Mixed scripts
        }

        # Longer texts for scaling tests
        self.long_texts = {
            "paragraph": (
                "The quick brown fox jumps over the lazy dog. "
                "This is a longer paragraph of text designed to test "
                "how the rendering engine scales with larger amounts of text. "
                "We want to see if performance degrades linearly or if there "
                "are any unexpected bottlenecks when rendering hundreds of characters. "
                "Typography is both an art and a science, requiring careful attention "
                "to detail and a deep understanding of how letterforms work together "
                "to create readable, beautiful text."
            ),  # ~500 chars
            "essay": (
                "Typography is the art and technique of arranging type to make written "
                "language legible, readable, and appealing when displayed. The arrangement "
                "of type involves selecting typefaces, point sizes, line lengths, line-spacing, "
                "and letter-spacing, and adjusting the space between pairs of letters. "
                "The term typography is also applied to the style, arrangement, and appearance "
                "of the letters, numbers, and symbols created by the process. Type design is "
                "a closely related craft, sometimes considered part of typography; most typographers "
                "do not design typefaces, and some type designers do not consider themselves typographers. "
                "Typography also may be used as an ornamental and decorative device, unrelated to the "
                "communication of information.\n\n"
                "In contemporary use, the practice and study of typography include a broad range, "
                "covering all aspects of letter design and application, both mechanical (typesetting, "
                "type design, and typefaces) and manual (handwriting and calligraphy). "
                "Typographical elements may appear in a wide variety of situations, and contexts including "
                "websites, mobile device interfaces, roadside signs, graffiti, product packaging, and books."
            ),  # ~1000+ chars
        }

        # Font sizes for benchmarking
        self.bench_sizes = [16.0, 32.0, 64.0, 128.0]

        # Available fonts
        self.fonts = {
            "kalnia": self.fonts_dir / "Kalnia[wdth,wght].ttf",
            "notoarabic": self.fonts_dir / "NotoNaskhArabic-Regular.ttf",
            "notosans": self.fonts_dir
            / "NotoSans-Regular.ttf",  # Broad Unicode coverage
        }

        # Verify fonts exist
        for name, path in self.fonts.items():
            if not path.exists():
                print(f"Warning: Font not found: {path}")

    def _detect_available_backends(self) -> List[BackendConfig]:
        """Detect which backend combinations are available"""
        backends = []

        # Define ALL possible shaping and rendering backends (TYPF v2.0 complete matrix!)
        # Shaping backends (4):
        shapers = [
            "none",  # Simple LTR advancement
            "harfbuzz",  # Full HarfBuzz OpenType shaping
            "coretext",  # macOS CoreText (native)
            "icu-hb",  # ICU normalization + HarfBuzz
        ]

        # Rendering backends (5):
        renderers = [
            "json",  # JSON output (for analysis/debugging)
            "orge",  # Pure Rust bitmap rasterizer
            "coregraphics",  # macOS CoreGraphics (native)
            "skia",  # tiny-skia rendering
            "zeno",  # Zeno rendering
        ]

        # Test all 4 × 5 = 20 combinations!
        for shaper in shapers:
            for renderer in renderers:
                desc = f"{shaper.upper()} + {renderer.upper()}"
                config = BackendConfig(shaper, renderer, desc)

                # Test if this combination works
                try:
                    engine = typf.Typf(shaper=shaper, renderer=renderer)
                    # Verify it actually works
                    actual_shaper = engine.get_shaper()
                    actual_renderer = engine.get_renderer()
                    # Update description with actual backend names
                    config.description = f"{actual_shaper} + {actual_renderer}"
                    config.available = True
                except Exception as e:
                    config.available = False
                    config.error = str(e)

                backends.append(config)

        return backends

    def _render_with_backend(
        self,
        config: BackendConfig,
        text: str,
        font_path: Path,
        size: float,
        output_format: str = "png",
    ) -> Tuple[bool, Optional[bytes], Optional[str]]:
        """Render text with specific backend and format

        Returns: (success, output_bytes, error_message)
        """
        try:
            engine = typf.Typf(shaper=config.shaper, renderer=config.renderer)

            # JSON renderer returns JSON string directly - save as .json
            if config.renderer == "json":
                if output_format != "json":
                    # For JSON renderer, we output JSON regardless of requested format
                    output_format = "json"

                result = engine.render_text(
                    text,
                    str(font_path),
                    size=size,
                    color=(0, 0, 0, 255),
                    background=(255, 255, 255, 255),
                    padding=20,
                )
                # Result is a JSON string for JSON renderer
                if isinstance(result, str):
                    return (True, result.encode("utf-8"), None)
                else:
                    return (False, None, "JSON renderer did not return string")

            # Handle SVG separately with proper vector export (non-JSON renderers)
            if output_format == "svg":
                svg_string = engine.render_to_svg(
                    text, str(font_path), size=size, color=(0, 0, 0, 255), padding=20
                )
                return (True, svg_string.encode("utf-8"), None)

            # Render to bitmap for other formats
            result = engine.render_text(
                text,
                str(font_path),
                size=size,
                color=(0, 0, 0, 255),
                background=(255, 255, 255, 255),
                padding=20,
            )

            # Export to requested format
            if output_format == "png":
                output_bytes = typf.export_image(result, format=output_format)
                return (True, output_bytes, None)
            elif output_format in ["ppm", "pgm", "pbm"]:
                output_bytes = typf.export_image(result, format=output_format)
                return (True, output_bytes, None)
            else:
                return (False, None, f"Unsupported format: {output_format}")

        except Exception as e:
            return (False, None, str(e))

    def render(self, backend: Optional[str] = None, format: str = "both"):
        """Render sample texts with all or specific backend combinations

        Args:
            backend: Specific backend to test (e.g., 'none', 'harfbuzz'), or None for all
            format: Output format - 'png', 'svg', or 'both' (default: 'both')

        Examples:
            python typfme.py render                    # All backends, both formats
            python typfme.py render --backend=none     # Only 'none' shaper
            python typfme.py render --format=svg       # Only SVG output
        """
        print("TYPF v2.0 Backend Rendering Test")
        print("=" * 80)
        print("Made by FontLab https://www.fontlab.com/\n")

        # Detect available backends
        backends = self._detect_available_backends()

        # Filter by requested backend
        if backend:
            backends = [b for b in backends if b.shaper == backend]
            if not backends:
                print(f"Error: Backend '{backend}' not found")
                return 1

        # Determine output formats
        formats = []
        if format in ["png", "both"]:
            formats.append("png")
        if format in ["svg", "both"]:
            formats.append("svg")

        if not formats:
            print(f"Error: Invalid format '{format}'. Use 'png', 'svg', or 'both'")
            return 1

        print(f"Testing {len(backends)} backend(s) with {len(formats)} format(s)\n")

        # Test each backend combination
        results = {}
        for config in backends:
            if not config.available:
                print(f"{config.description:30s} ✗ Not available: {config.error}")
                continue

            print(f"\n{config.description}")
            print("-" * 80)

            # Render each sample text
            for text_name, text in self.sample_texts.items():
                # Choose appropriate font based on script
                if text_name == "arab":
                    font_path = self.fonts["notoarabic"]
                    font_name = "NotoArabic"
                elif text_name == "mixd":
                    # Mixed scripts need broad Unicode coverage (includes CJK)
                    font_path = self.fonts["notosans"]
                    font_name = "NotoSans"
                else:
                    font_path = self.fonts["kalnia"]
                    font_name = "Kalnia"

                if not font_path.exists():
                    print(f"  {text_name:20s} ✗ Font not found: {font_path}")
                    continue

                # Render in each format (or JSON if renderer is json)
                render_formats = ["json"] if config.renderer == "json" else formats
                for fmt in render_formats:
                    success, output_bytes, error = self._render_with_backend(
                        config, text, font_path, 48.0, fmt
                    )

                    if success:
                        # Save output with appropriate extension
                        ext = "json" if config.renderer == "json" else fmt
                        filename = f"render-{config.shaper}-{config.renderer}-{text_name}.{ext}"
                        output_path = self.output_dir / filename
                        output_path.write_bytes(output_bytes)

                        # Verify size and dimensions
                        size_kb = len(output_bytes) / 1024
                        status = f"✓ {size_kb:.1f}KB"

                        if fmt == "png" and Image:
                            try:
                                img = Image.open(output_path)
                                status += f" ({img.width}x{img.height})"
                            except Exception:
                                pass
                        elif fmt == "json":
                            # Parse JSON to verify
                            try:
                                import json as json_mod

                                data = json_mod.loads(output_bytes.decode("utf-8"))
                                glyph_count = len(data.get("glyphs", []))
                                status += f" ({glyph_count} glyphs)"
                            except Exception:
                                pass

                        print(
                            f"  {text_name:20s} [{ext.upper()}] {status:30s} → {filename}"
                        )
                        results[filename] = output_path
                    else:
                        print(f"  {text_name:20s} [{fmt.upper()}] ✗ {error}")

        print(f"\n{'=' * 80}")
        print(f"Rendered {len(results)} images to {self.output_dir}")
        print("\nView outputs:")
        print(f"  ls {self.output_dir}")
        print(f"  open {self.output_dir}/*.png")

        return 0

    def bench(self, iterations: int = 100, detailed: bool = False):
        """Benchmark all backend combinations across multiple texts and sizes

        Args:
            iterations: Number of iterations per benchmark (default: 100)
            detailed: Include detailed per-text, per-size results (default: False)

        Examples:
            python typfme.py bench                      # Standard benchmark
            python typfme.py bench --iterations=1000    # More iterations
            python typfme.py bench --detailed=True      # Full detailed report
        """
        print("TYPF v2.0 Comprehensive Backend Benchmark")
        print("=" * 80)
        print("Made by FontLab https://www.fontlab.com/\n")

        # Detect available backends
        backends = self._detect_available_backends()
        available_backends = [b for b in backends if b.available]

        if not available_backends:
            print("Error: No backends available for benchmarking")
            return 1

        print(f"Benchmarking {len(available_backends)} backend(s)")
        print(f"Iterations: {iterations}")
        print(f"Sample texts: {len(self.sample_texts)}")
        print(f"Font sizes: {self.bench_sizes}")
        print()

        # Run benchmarks
        all_results: List[BenchResult] = []

        for config in available_backends:
            print(f"\n{config.description}")
            print("-" * 80)

            for text_name, text in self.sample_texts.items():
                # Choose appropriate font based on script
                if text_name == "arab":
                    font_path = self.fonts["notoarabic"]
                elif text_name == "mixd":
                    # Mixed scripts need broad Unicode coverage (includes CJK)
                    font_path = self.fonts["notosans"]
                else:
                    font_path = self.fonts["kalnia"]

                if not font_path.exists():
                    continue

                for size in self.bench_sizes:
                    try:
                        engine = typf.Typf(
                            shaper=config.shaper, renderer=config.renderer
                        )

                        # Warmup
                        for _ in range(5):
                            _ = engine.render_text(text, str(font_path), size=size)

                        # Benchmark
                        start = time.perf_counter()
                        output = None
                        for _ in range(iterations):
                            output = engine.render_text(text, str(font_path), size=size)
                        elapsed = time.perf_counter() - start

                        avg_ms = (elapsed / iterations) * 1000
                        ops_per_sec = iterations / elapsed

                        # Measure output size from last iteration
                        output_size = None
                        if output is not None:
                            if isinstance(output, str):
                                # JSON renderer returns string
                                output_size = len(output.encode("utf-8"))
                            elif isinstance(output, bytes):
                                output_size = len(output)
                            elif hasattr(output, "__len__"):
                                # Bitmap data - estimate size
                                try:
                                    output_bytes = typf.export_image(
                                        output, format="png"
                                    )
                                    output_size = len(output_bytes)
                                except:
                                    pass

                        result = BenchResult(
                            config=config,
                            text=text_name,
                            font_size=size,
                            iterations=iterations,
                            total_time=elapsed,
                            avg_time_ms=avg_ms,
                            ops_per_sec=ops_per_sec,
                            success=True,
                            output_size_bytes=output_size,
                        )
                        all_results.append(result)

                        if detailed:
                            print(
                                f"  {text_name:20s} @ {size:5.0f}px: "
                                f"{avg_ms:8.3f}ms/op ({ops_per_sec:8.1f} ops/sec)"
                            )

                    except Exception as e:
                        result = BenchResult(
                            config=config,
                            text=text_name,
                            font_size=size,
                            iterations=iterations,
                            total_time=0,
                            avg_time_ms=0,
                            ops_per_sec=0,
                            success=False,
                            error=str(e),
                        )
                        all_results.append(result)

                        if detailed:
                            print(f"  {text_name:20s} @ {size:5.0f}px: ✗ {str(e)[:40]}")

        # Generate summary report
        self._generate_benchmark_report(all_results, iterations)

        return 0

    def _generate_benchmark_report(self, results: List[BenchResult], iterations: int):
        """Generate comprehensive benchmark report"""
        print("\n" + "=" * 80)
        print("BENCHMARK SUMMARY")
        print("=" * 80)

        # Group results by backend
        by_backend: Dict[str, List[BenchResult]] = {}
        for result in results:
            key = result.config.description
            if key not in by_backend:
                by_backend[key] = []
            by_backend[key].append(result)

        # Summary table
        print(
            f"\n{'Backend':<30} {'Avg Time (ms)':<20} {'Ops/sec':<15} {'Success Rate':<15}"
        )
        print("-" * 80)

        for backend_desc, backend_results in sorted(by_backend.items()):
            successful = [r for r in backend_results if r.success]
            if successful:
                avg_time = sum(r.avg_time_ms for r in successful) / len(successful)
                avg_ops = sum(r.ops_per_sec for r in successful) / len(successful)
                success_rate = len(successful) / len(backend_results) * 100
                print(
                    f"{backend_desc:<30} {avg_time:>8.3f}            "
                    f"{avg_ops:>10.1f}      {success_rate:>6.1f}%"
                )
            else:
                print(f"{backend_desc:<30} {'N/A':<20} {'N/A':<15} {'0.0%':<15}")

        # Performance by text complexity
        print("\n" + "=" * 80)
        print("PERFORMANCE BY TEXT COMPLEXITY")
        print("=" * 80)

        by_text: Dict[str, List[BenchResult]] = {}
        for result in results:
            if result.success:
                if result.text not in by_text:
                    by_text[result.text] = []
                by_text[result.text].append(result)

        print(f"\n{'Text Type':<20} {'Avg Time (ms)':<20} {'Ops/sec':<15}")
        print("-" * 55)

        for text_name in sorted(by_text.keys()):
            text_results = by_text[text_name]
            avg_time = sum(r.avg_time_ms for r in text_results) / len(text_results)
            avg_ops = sum(r.ops_per_sec for r in text_results) / len(text_results)
            print(f"{text_name:<20} {avg_time:>8.3f}            {avg_ops:>10.1f}")

        # Load previous benchmark for regression detection
        # Use benchmark_baseline.json if it exists, otherwise compare to last report
        report_path = self.output_dir / "benchmark_report.json"
        baseline_path = self.output_dir / "benchmark_baseline.json"
        if not baseline_path.exists():
            baseline_path = report_path

        baseline_data = None
        regressions = []

        if baseline_path.exists():
            try:
                baseline_data = json.loads(baseline_path.read_text())
                # Build lookup dict: (shaper, renderer, text, size) -> avg_time_ms
                baseline_times = {}
                for b in baseline_data.get("backends", []):
                    key = (b["shaper"], b["renderer"], b["text"], b["font_size"])
                    baseline_times[key] = b["avg_time_ms"]

                # Check for regressions (>10% slowdown)
                for r in results:
                    if not r.success:
                        continue
                    key = (r.config.shaper, r.config.renderer, r.text, r.font_size)
                    if key in baseline_times:
                        baseline_time = baseline_times[key]
                        current_time = r.avg_time_ms
                        if baseline_time > 0:
                            slowdown_pct = (
                                (current_time - baseline_time) / baseline_time
                            ) * 100
                            if slowdown_pct > 10:  # >10% slower
                                regressions.append({
                                    "backend": r.config.description,
                                    "text": r.text,
                                    "size": r.font_size,
                                    "baseline_ms": baseline_time,
                                    "current_ms": current_time,
                                    "slowdown_pct": slowdown_pct,
                                })
            except (json.JSONDecodeError, KeyError):
                pass  # No valid baseline, skip regression detection

        # Save detailed JSON report
        report_data = {
            "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
            "iterations": iterations,
            "total_tests": len(results),
            "successful_tests": len([r for r in results if r.success]),
            "regressions": regressions,
            "backends": [
                {
                    "shaper": r.config.shaper,
                    "renderer": r.config.renderer,
                    "description": r.config.description,
                    "text": r.text,
                    "font_size": r.font_size,
                    "avg_time_ms": r.avg_time_ms,
                    "ops_per_sec": r.ops_per_sec,
                    "output_size_bytes": r.output_size_bytes,
                    "success": r.success,
                    "error": r.error,
                }
                for r in results
            ],
        }

        report_path.write_text(json.dumps(report_data, indent=2))

        # Print regression warnings
        if regressions:
            print("\n" + "!" * 80)
            print("⚠️  PERFORMANCE REGRESSIONS DETECTED (>10% slowdown)")
            print("!" * 80)
            for reg in regressions[:10]:  # Show top 10
                print(f"  {reg['backend']:40} {reg['text']:10} {reg['size']}px")
                print(
                    f"    Baseline: {reg['baseline_ms']:.3f}ms → Current: {reg['current_ms']:.3f}ms"
                )
                print(f"    Slowdown: {reg['slowdown_pct']:+.1f}%")
            if len(regressions) > 10:
                print(
                    f"  ... and {len(regressions) - 10} more (see benchmark_report.json)"
                )
            print("!" * 80)

        # Generate compact Markdown summary table
        md_path = self.output_dir / "benchmark_summary.md"
        md_lines = []
        md_lines.append("# TYPF Benchmark Summary\n")
        md_lines.append(f"**Date**: {time.strftime('%Y-%m-%d %H:%M:%S')}  ")
        md_lines.append(f"**Iterations**: {iterations}  ")
        md_lines.append(
            f"**Success Rate**: {len([r for r in results if r.success])}/{len(results)}\n"
        )

        # Add regression warnings to markdown
        if regressions:
            md_lines.append("## ⚠️ Performance Regressions Detected\n")
            md_lines.append(
                f"**{len(regressions)} backend(s)** are >10% slower than baseline:\n"
            )
            md_lines.append(
                "| Backend | Text | Size | Baseline | Current | Slowdown |\n"
            )
            md_lines.append("|---------|------|------|----------|---------|----------|")
            for reg in regressions[:10]:
                md_lines.append(
                    f"| {reg['backend']} | {reg['text']} | {reg['size']}px | "
                    f"{reg['baseline_ms']:.3f}ms | {reg['current_ms']:.3f}ms | "
                    f"{reg['slowdown_pct']:+.1f}% |"
                )
            if len(regressions) > 10:
                md_lines.append(
                    f"\n*...and {len(regressions) - 10} more (see benchmark_report.json)*\n"
                )
            md_lines.append("")

        # Detailed performance table (Ops/sec only)
        md_lines.append("## Detailed Performance (Ops/sec)\n")
        md_lines.append("| Backend | Text | Size | Ops/sec |\n")
        md_lines.append("|:---|:---|:---:|---:|")

        # Sort results for consistent output: Backend -> Text -> Size
        sorted_results = sorted(
            results,
            key=lambda r: (r.config.description, r.text, r.font_size)
        )

        for r in sorted_results:
            if r.success:
                ops_str = f"{r.ops_per_sec:,.1f}"
            else:
                ops_str = "FAILED"

            md_lines.append(
                f"| {r.config.description} | {r.text} | {r.font_size:.0f}px | {ops_str} |"
            )

        md_lines.append("\n---\n*Made by FontLab - https://www.fontlab.com/*\n")

        md_path.write_text("\n".join(md_lines))

        print("\n" + "=" * 80)
        print(f"Detailed report saved to: {report_path}")
        print(f"Markdown summary saved to: {md_path}")
        print("=" * 80)

    def bench_shaping(self, iterations: int = 1000, detailed: bool = False):
        """Benchmark shaping performance only (isolate from rendering)

        Args:
            iterations: Number of iterations per benchmark (default: 1000)
            detailed: Include detailed per-text, per-size results (default: False)

        Examples:
            python typfme.py bench-shaping                      # Standard shaping benchmark
            python typfme.py bench-shaping --iterations=10000   # More iterations
            python typfme.py bench-shaping --detailed=True      # Full detailed report
        """
        print("TYPF v2.0 Shaping-Only Performance Benchmark")
        print("=" * 80)
        print("Made by FontLab https://www.fontlab.com/\n")

        # Detect available backends
        backends = self._detect_available_backends()
        # Group by shaper only (ignore renderer since we're only benchmarking shaping)
        shapers_seen = set()
        shaping_backends = []
        for b in backends:
            if b.available and b.shaper not in shapers_seen:
                shapers_seen.add(b.shaper)
                shaping_backends.append(b)

        if not shaping_backends:
            print("Error: No shaping backends available")
            return 1

        print(f"Benchmarking {len(shaping_backends)} shaping backend(s)")
        print(f"Iterations: {iterations}")
        print(f"Sample texts: {len(self.sample_texts)}")
        print(f"Font sizes: {self.bench_sizes}")
        print()

        all_results = []

        for config in shaping_backends:
            print(f"\n{config.shaper.upper()} Shaper")
            print("-" * 80)

            for text_name, text in self.sample_texts.items():
                # Choose appropriate font based on script
                if text_name == "arab":
                    font_path = self.fonts["notoarabic"]
                elif text_name == "mixd":
                    # Mixed scripts need broad Unicode coverage (includes CJK)
                    font_path = self.fonts["notosans"]
                else:
                    font_path = self.fonts["kalnia"]

                if not font_path.exists():
                    continue

                for size in self.bench_sizes:
                    try:
                        engine = typf.Typf(
                            shaper=config.shaper, renderer=config.renderer
                        )

                        # Warmup
                        for _ in range(10):
                            _ = engine.shape_text(text, str(font_path), size=size)

                        # Benchmark
                        start = time.perf_counter()
                        for _ in range(iterations):
                            _ = engine.shape_text(text, str(font_path), size=size)
                        elapsed = time.perf_counter() - start

                        avg_us = (elapsed / iterations) * 1_000_000  # microseconds
                        ops_per_sec = iterations / elapsed

                        result = {
                            "shaper": config.shaper,
                            "text": text_name,
                            "font_size": size,
                            "iterations": iterations,
                            "avg_time_us": avg_us,
                            "ops_per_sec": ops_per_sec,
                            "success": True,
                        }
                        all_results.append(result)

                        if detailed:
                            print(
                                f"  {text_name:20s} @ {size:5.0f}px: "
                                f"{avg_us:8.1f}µs/op ({ops_per_sec:8.0f} ops/sec)"
                            )

                    except Exception as e:
                        result = {
                            "shaper": config.shaper,
                            "text": text_name,
                            "font_size": size,
                            "iterations": iterations,
                            "avg_time_us": 0,
                            "ops_per_sec": 0,
                            "success": False,
                            "error": str(e),
                        }
                        all_results.append(result)

                        if detailed:
                            print(f"  {text_name:20s} @ {size:5.0f}px: ✗ {str(e)[:40]}")

        # Generate summary report
        print("\n" + "=" * 80)
        print("SHAPING PERFORMANCE SUMMARY")
        print("=" * 80)

        # Group by shaper
        by_shaper = {}
        for result in all_results:
            if result["success"]:
                shaper = result["shaper"]
                if shaper not in by_shaper:
                    by_shaper[shaper] = []
                by_shaper[shaper].append(result)

        # Summary table
        print(f"\n{'Shaper':<15} {'Avg Time (µs)':<20} {'Ops/sec':<15}")
        print("-" * 50)

        for shaper in sorted(by_shaper.keys()):
            shaper_results = by_shaper[shaper]
            avg_time = sum(r["avg_time_us"] for r in shaper_results) / len(
                shaper_results
            )
            avg_ops = sum(r["ops_per_sec"] for r in shaper_results) / len(
                shaper_results
            )
            print(f"{shaper.upper():<15} {avg_time:>8.1f}            {avg_ops:>10.0f}")

        # Performance by text complexity
        print("\n" + "=" * 80)
        print("SHAPING PERFORMANCE BY TEXT TYPE")
        print("=" * 80)

        by_text = {}
        for result in all_results:
            if result["success"]:
                text_name = result["text"]
                if text_name not in by_text:
                    by_text[text_name] = []
                by_text[text_name].append(result)

        print(f"\n{'Text Type':<20} {'Avg Time (µs)':<20} {'Ops/sec':<15}")
        print("-" * 55)

        for text_name in sorted(by_text.keys()):
            text_results = by_text[text_name]
            avg_time = sum(r["avg_time_us"] for r in text_results) / len(text_results)
            avg_ops = sum(r["ops_per_sec"] for r in text_results) / len(text_results)
            print(f"{text_name:<20} {avg_time:>8.1f}            {avg_ops:>10.0f}")

        # Save detailed JSON report
        report_path = self.output_dir / "shaping_benchmark.json"
        report_data = {
            "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
            "iterations": iterations,
            "total_tests": len(all_results),
            "successful_tests": len([r for r in all_results if r["success"]]),
            "results": all_results,
        }

        report_path.write_text(json.dumps(report_data, indent=2))

        print("\n" + "=" * 80)
        print(f"Detailed shaping benchmark saved to: {report_path}")
        print("=" * 80)

        return 0

    def bench_rendering(self, iterations: int = 100, detailed: bool = False):
        """Benchmark rendering performance only (isolate from shaping)

        This computes rendering time by subtracting shaping time from total time.

        Args:
            iterations: Number of iterations per benchmark (default: 100)
            detailed: Include detailed per-text, per-size results (default: False)

        Examples:
            python typfme.py bench-rendering                     # Standard rendering benchmark
            python typfme.py bench-rendering --iterations=500    # More iterations
            python typfme.py bench-rendering --detailed=True     # Full detailed report
        """
        print("TYPF v2.0 Rendering-Only Performance Benchmark")
        print("=" * 80)
        print("Made by FontLab https://www.fontlab.com/\n")

        # Detect available backends
        backends = self._detect_available_backends()
        # Group by renderer only (ignore shaper differences)
        renderers_seen = set()
        rendering_backends = []
        for b in backends:
            if b.available and b.renderer not in renderers_seen:
                renderers_seen.add(b.renderer)
                rendering_backends.append(b)

        if not rendering_backends:
            print("Error: No rendering backends available")
            return 1

        print(f"Benchmarking {len(rendering_backends)} rendering backend(s)")
        print(f"Strategy: Total time - Shaping time = Rendering time")
        print(f"Iterations: {iterations}")
        print(f"Sample texts: {len(self.sample_texts)}")
        print(f"Font sizes: {self.bench_sizes}")
        print()

        all_results = []

        for config in rendering_backends:
            print(
                f"\n{config.renderer.upper()} Renderer (with {config.shaper.upper()} shaper)"
            )
            print("-" * 80)

            for text_name, text in self.sample_texts.items():
                # Choose appropriate font based on script
                if text_name == "arab":
                    font_path = self.fonts["notoarabic"]
                elif text_name == "mixd":
                    # Mixed scripts need broad Unicode coverage (includes CJK)
                    font_path = self.fonts["notosans"]
                else:
                    font_path = self.fonts["kalnia"]

                if not font_path.exists():
                    continue

                for size in self.bench_sizes:
                    try:
                        engine = typf.Typf(
                            shaper=config.shaper, renderer=config.renderer
                        )

                        # Measure shaping time
                        for _ in range(5):
                            _ = engine.shape_text(text, str(font_path), size=size)

                        start = time.perf_counter()
                        for _ in range(iterations):
                            _ = engine.shape_text(text, str(font_path), size=size)
                        shaping_time = time.perf_counter() - start

                        # Measure total time (shaping + rendering)
                        for _ in range(5):
                            _ = engine.render_text(text, str(font_path), size=size)

                        start = time.perf_counter()
                        for _ in range(iterations):
                            _ = engine.render_text(text, str(font_path), size=size)
                        total_time = time.perf_counter() - start

                        # Calculate rendering time
                        rendering_time = total_time - shaping_time
                        avg_us = (
                            rendering_time / iterations
                        ) * 1_000_000  # microseconds
                        ops_per_sec = (
                            iterations / rendering_time if rendering_time > 0 else 0
                        )

                        result = {
                            "renderer": config.renderer,
                            "shaper": config.shaper,
                            "text": text_name,
                            "font_size": size,
                            "iterations": iterations,
                            "shaping_time_us": (shaping_time / iterations) * 1_000_000,
                            "rendering_time_us": avg_us,
                            "total_time_us": (total_time / iterations) * 1_000_000,
                            "ops_per_sec": ops_per_sec,
                            "success": True,
                        }
                        all_results.append(result)

                        if detailed:
                            print(
                                f"  {text_name:20s} @ {size:5.0f}px: "
                                f"{avg_us:8.1f}µs/op ({ops_per_sec:8.0f} ops/sec)"
                            )

                    except Exception as e:
                        result = {
                            "renderer": config.renderer,
                            "shaper": config.shaper,
                            "text": text_name,
                            "font_size": size,
                            "iterations": iterations,
                            "rendering_time_us": 0,
                            "ops_per_sec": 0,
                            "success": False,
                            "error": str(e),
                        }
                        all_results.append(result)

                        if detailed:
                            print(f"  {text_name:20s} @ {size:5.0f}px: ✗ {str(e)[:40]}")

        # Generate summary report
        print("\n" + "=" * 80)
        print("RENDERING PERFORMANCE SUMMARY")
        print("=" * 80)

        # Group by renderer
        by_renderer = {}
        for result in all_results:
            if result["success"]:
                renderer = result["renderer"]
                if renderer not in by_renderer:
                    by_renderer[renderer] = []
                by_renderer[renderer].append(result)

        # Summary table
        print(
            f"\n{'Renderer':<15} {'Shape (µs)':<15} {'Render (µs)':<15} {'Total (µs)':<15} {'Ops/sec':<15}"
        )
        print("-" * 75)

        for renderer in sorted(by_renderer.keys()):
            renderer_results = by_renderer[renderer]
            avg_shape = sum(r["shaping_time_us"] for r in renderer_results) / len(
                renderer_results
            )
            avg_render = sum(r["rendering_time_us"] for r in renderer_results) / len(
                renderer_results
            )
            avg_total = avg_shape + avg_render
            avg_ops = sum(r["ops_per_sec"] for r in renderer_results) / len(
                renderer_results
            )
            print(
                f"{renderer.upper():<15} {avg_shape:>8.1f}        "
                f"{avg_render:>8.1f}        {avg_total:>8.1f}        {avg_ops:>10.0f}"
            )

        # Performance by font size
        print("\n" + "=" * 80)
        print("RENDERING PERFORMANCE BY FONT SIZE")
        print("=" * 80)

        by_size = {}
        for result in all_results:
            if result["success"]:
                size = result["font_size"]
                if size not in by_size:
                    by_size[size] = []
                by_size[size].append(result)

        print(f"\n{'Size (px)':<12} {'Render (µs)':<15} {'Ops/sec':<15}")
        print("-" * 42)

        for size in sorted(by_size.keys()):
            size_results = by_size[size]
            avg_render = sum(r["rendering_time_us"] for r in size_results) / len(
                size_results
            )
            avg_ops = sum(r["ops_per_sec"] for r in size_results) / len(size_results)
            print(f"{size:<12.0f} {avg_render:>8.1f}        {avg_ops:>10.0f}")

        # Save detailed JSON report
        report_path = self.output_dir / "rendering_benchmark.json"
        report_data = {
            "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
            "iterations": iterations,
            "total_tests": len(all_results),
            "successful_tests": len([r for r in all_results if r["success"]]),
            "results": all_results,
        }

        report_path.write_text(json.dumps(report_data, indent=2))

        print("\n" + "=" * 80)
        print(f"Detailed rendering benchmark saved to: {report_path}")
        print("=" * 80)

        return 0

    def bench_scaling(self, iterations: int = 50, detailed: bool = False):
        """Benchmark performance scaling with longer texts

        Tests how performance scales from short to long texts (10-1000+ characters).

        Args:
            iterations: Number of iterations per benchmark (default: 50)
            detailed: Include detailed results (default: False)

        Examples:
            python typfme.py bench-scaling                      # Standard scaling test
            python typfme.py bench-scaling --iterations=100     # More iterations
            python typfme.py bench-scaling --detailed=True      # Full detailed report
        """
        print("TYPF v2.0 Text Length Scaling Benchmark")
        print("=" * 80)
        print("Made by FontLab https://www.fontlab.com/\n")

        # Combine short and long texts
        all_texts = {**self.sample_texts, **self.long_texts}

        # Detect available backends
        backends = self._detect_available_backends()
        available_backends = [b for b in backends if b.available]

        if not available_backends:
            print("Error: No backends available")
            return 1

        print(f"Benchmarking text length scaling")
        print(f"Iterations: {iterations}")
        print(
            f"Text samples: {len(all_texts)} (from {min(len(t) for t in all_texts.values())} to {max(len(t) for t in all_texts.values())} chars)"
        )
        print(f"Font size: 48px")
        print()

        all_results = []
        test_size = 48.0

        for config in available_backends[:1]:  # Just use first backend
            print(f"\n{config.description}")
            print("-" * 80)

            for text_name, text in all_texts.items():
                font_path = self.fonts["kalnia"]

                if not font_path.exists():
                    continue

                try:
                    engine = typf.Typf(shaper=config.shaper, renderer=config.renderer)
                    char_count = len(text)

                    # Warmup
                    for _ in range(3):
                        _ = engine.render_text(text, str(font_path), size=test_size)

                    # Benchmark total time
                    start = time.perf_counter()
                    for _ in range(iterations):
                        _ = engine.render_text(text, str(font_path), size=test_size)
                    total_time = time.perf_counter() - start

                    # Benchmark shaping time
                    for _ in range(3):
                        _ = engine.shape_text(text, str(font_path), size=test_size)

                    start = time.perf_counter()
                    for _ in range(iterations):
                        _ = engine.shape_text(text, str(font_path), size=test_size)
                    shaping_time = time.perf_counter() - start

                    rendering_time = total_time - shaping_time

                    # Calculate per-character metrics
                    avg_total_ms = (total_time / iterations) * 1000
                    avg_shape_us = (shaping_time / iterations) * 1_000_000
                    avg_render_ms = (rendering_time / iterations) * 1000
                    us_per_char = (avg_total_ms * 1000) / char_count

                    result = {
                        "backend": config.description,
                        "text": text_name,
                        "char_count": char_count,
                        "iterations": iterations,
                        "total_time_ms": avg_total_ms,
                        "shaping_time_us": avg_shape_us,
                        "rendering_time_ms": avg_render_ms,
                        "us_per_char": us_per_char,
                        "ops_per_sec": iterations / total_time,
                        "success": True,
                    }
                    all_results.append(result)

                    if detailed:
                        print(
                            f"  {text_name:15s} ({char_count:4d} chars): "
                            f"{avg_total_ms:7.2f}ms total ({us_per_char:6.1f}µs/char)"
                        )
                    else:
                        print(
                            f"  {text_name:15s} ({char_count:4d} chars): {avg_total_ms:7.2f}ms"
                        )

                except Exception as e:
                    result = {
                        "backend": config.description,
                        "text": text_name,
                        "char_count": len(text),
                        "iterations": iterations,
                        "total_time_ms": 0,
                        "us_per_char": 0,
                        "ops_per_sec": 0,
                        "success": False,
                        "error": str(e),
                    }
                    all_results.append(result)
                    print(f"  {text_name:15s} ({len(text):4d} chars): ✗ {str(e)[:40]}")

        # Generate scaling analysis
        print("\n" + "=" * 80)
        print("PERFORMANCE SCALING ANALYSIS")
        print("=" * 80)

        successful = [r for r in all_results if r["success"]]
        if successful:
            # Sort by character count
            successful.sort(key=lambda x: x["char_count"])

            print(
                f"\n{'Text':<15} {'Chars':<8} {'Total (ms)':<12} {'µs/char':<12} {'Linear?':<10}"
            )
            print("-" * 57)

            baseline_us_per_char = successful[0]["us_per_char"]

            for result in successful:
                char_count = result["char_count"]
                total_ms = result["total_time_ms"]
                us_per_char = result["us_per_char"]

                # Check if scaling is roughly linear (within 20%)
                expected_time = baseline_us_per_char * char_count / 1000  # in ms
                actual_time = total_ms
                deviation = abs(actual_time - expected_time) / expected_time * 100

                linear_indicator = (
                    "✓ Yes" if deviation < 20 else f"✗ No ({deviation:.0f}% off)"
                )

                print(
                    f"{result['text']:<15} {char_count:<8} "
                    f"{total_ms:<12.2f} {us_per_char:<12.1f} {linear_indicator:<10}"
                )

            # Calculate scaling factor
            if len(successful) >= 2:
                shortest = successful[0]
                longest = successful[-1]

                char_ratio = longest["char_count"] / shortest["char_count"]
                time_ratio = longest["total_time_ms"] / shortest["total_time_ms"]

                print("\n" + "-" * 57)
                print(
                    f"Scaling factor: {char_ratio:.1f}x more characters → {time_ratio:.1f}x more time"
                )

                if time_ratio / char_ratio < 1.2:
                    print("✓ Performance scales linearly with text length")
                elif time_ratio / char_ratio < 1.5:
                    print("⚠ Performance scales slightly super-linearly")
                else:
                    print(
                        "✗ Performance has super-linear scaling (potential bottleneck)"
                    )

        # Save detailed JSON report
        report_path = self.output_dir / "scaling_benchmark.json"
        report_data = {
            "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
            "iterations": iterations,
            "font_size": test_size,
            "total_tests": len(all_results),
            "successful_tests": len([r for r in all_results if r["success"]]),
            "results": all_results,
        }

        report_path.write_text(json.dumps(report_data, indent=2))

        print("\n" + "=" * 80)
        print(f"Detailed scaling benchmark saved to: {report_path}")
        print("=" * 80)

        return 0

    def compare(self):
        """Generate side-by-side comparison of all backends

        Renders the same text with all available backends for visual comparison.
        """
        print("TYPF v2.0 Backend Comparison")
        print("=" * 80)
        print("Made by FontLab https://www.fontlab.com/\n")

        # Use a distinctive sample for comparison
        sample_text = "AVAST Typography"
        font_path = self.fonts["kalnia"]
        size = 64.0

        if not font_path.exists():
            print(f"Error: Font not found: {font_path}")
            return 1

        print(f'Sample text: "{sample_text}"')
        print(f"Font: {font_path.name}")
        print(f"Size: {size}px\n")

        backends = self._detect_available_backends()
        available = [b for b in backends if b.available]

        if not available:
            print("Error: No backends available")
            return 1

        print(f"Comparing {len(available)} backend(s):\n")

        results = {}
        for config in available:
            for fmt in ["png", "svg"]:
                success, output_bytes, error = self._render_with_backend(
                    config, sample_text, font_path, size, fmt
                )

                if success:
                    filename = f"compare-{config.shaper}-{config.renderer}.{fmt}"
                    output_path = self.output_dir / filename
                    output_path.write_bytes(output_bytes)

                    size_kb = len(output_bytes) / 1024
                    print(
                        f"  {config.description:30s} [{fmt.upper()}] ✓ {size_kb:6.1f}KB → {filename}"
                    )
                    results[filename] = output_path
                else:
                    print(f"  {config.description:30s} [{fmt.upper()}] ✗ {error}")

        print(f"\n{'=' * 80}")
        print(f"Generated {len(results)} comparison images")
        print("\nView comparisons:")
        print(f"  open {self.output_dir}/compare-*.png")

        return 0

    def info(self):
        """Display information about available backends and fonts"""
        print("TYPF v2.0 Testing Environment")
        print("=" * 80)
        print("Made by FontLab https://www.fontlab.com/\n")

        # Version and capabilities
        print(f"TYPF Version: {typf.__version__}")
        print(f"Python Version: {sys.version.split()[0]}")
        print(f"Base Directory: {self.base_dir}")
        print(f"Output Directory: {self.output_dir}\n")

        # Available backends with capabilities
        print("Available Backend Combinations:")
        print("-" * 80)
        backends = self._detect_available_backends()
        available_count = sum(1 for b in backends if b.available)
        print(f"Total: {available_count}/{len(backends)} combinations available\n")

        for config in backends:
            status = "✓ Available" if config.available else f"✗ {config.error}"
            print(f"  {config.description:30s} {status}")

        # Export formats
        print("\nSupported Export Formats:")
        print("-" * 80)
        formats = [
            ("PNG", "Portable Network Graphics (lossless bitmap)"),
            ("SVG", "Scalable Vector Graphics (true vector paths)"),
            ("PPM", "Portable Pixmap (P6 binary color)"),
            ("PGM", "Portable Graymap (P5 binary grayscale)"),
            ("PBM", "Portable Bitmap (P4 binary monochrome)"),
        ]
        for fmt, desc in formats:
            print(f"  {fmt:8s} - {desc}")

        # Available fonts
        print("\nAvailable Test Fonts:")
        print("-" * 80)
        for name, path in self.fonts.items():
            exists = "✓" if path.exists() else "✗"
            size = f"{path.stat().st_size / 1024:.1f}KB" if path.exists() else "N/A"
            print(f"  {exists} {name:15s} {size:>10s}  {path.name}")

        # Sample texts with character counts
        print("\nSample Texts:")
        print("-" * 80)
        for name, text in self.sample_texts.items():
            preview = text[:50] + "..." if len(text) > 50 else text
            char_count = len(text)
            print(f"  {name:20s} ({char_count:3d} chars)  {preview}")

        # Benchmark capabilities
        print("\nBenchmarking Capabilities:")
        print("-" * 80)
        commands = [
            ("render", "Render samples with all backends (PNG + SVG)"),
            ("bench", "Full benchmark with JSON + Markdown reports"),
            ("bench-shaping", "Shaping-only performance (isolate from rendering)"),
            ("bench-rendering", "Rendering-only performance (isolate from shaping)"),
            ("bench-scaling", "Text length scaling analysis"),
            ("compare", "Side-by-side backend comparison"),
        ]
        for cmd, desc in commands:
            print(f"  {cmd:18s} - {desc}")

        # Quick usage
        print("\nQuick Start:")
        print("-" * 80)
        print("  python typfme.py render           # Test all backends")
        print("  python typfme.py bench            # Run benchmarks")
        print("  python typfme.py info             # Show this information")

        print("\n" + "=" * 80)


if __name__ == "__main__":
    fire.Fire(TypfTester)
