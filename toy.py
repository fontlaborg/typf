#!/usr/bin/env python3
# this_file: toy.py
"""A simple CLI to benchmark and render with typf.

Usage:
    python toy.py bench          # Run benchmarks
    python toy.py render         # Render samples with all backends (TYPF + simple)
    python toy.py compare        # Compare TYPF vs simple backends

Made by FontLab https://www.fontlab.com/
"""

import fire
import subprocess
import sys
from pathlib import Path


def find_merriweather_font():
    """Find Merriweather font file path."""
    # Try local testdata first
    local_font = Path(__file__).parent / "testdata" / "fonts" / "Merriweather[opsz,wdth,wght].ttf"
    if local_font.exists():
        return local_font

    # Try common locations
    try:
        result = subprocess.run(
            ["mdfind", "kMDItemDisplayName == 'Merriweather*.ttf'"],
            capture_output=True,
            text=True,
            timeout=5,
        )
        fonts = [p for p in result.stdout.strip().split('\n') if p and '.ttf' in p.lower()]
        if fonts:
            return Path(fonts[0])
    except:
        pass

    # Fallback to Arial
    return Path("/System/Library/Fonts/Supplemental/Arial.ttf")


class Toy:
    """A simple CLI to benchmark and render with typf."""

    def compare(self):
        """Compare TYPF backends with simple reference implementations."""
        print("Comparing TYPF backends with simple reference renderers...\n")
        print("Made by FontLab https://www.fontlab.com/\n")

        try:
            import typf
            sys.path.insert(0, str(Path(__file__).parent))
            import simple_font_rendering_py as simple
            from PIL import Image
        except ImportError as e:
            print(f"Error: Missing dependencies: {e}")
            print("Install: pip install pillow pyobjc-framework-CoreText uharfbuzz freetype-py")
            return 1

        # Sample text and settings
        sample_text = "The quick brown fox jumps over the lazy dog."
        font_family = "Merriweather"
        font_size = 64.0

        # Find Merriweather font file
        font_path = find_merriweather_font()
        print(f"Using font: {font_path.name}")
        print(f"Font path: {font_path}\n")

        # Get available backends
        typf_backends = typf.TextRenderer.list_available_backends()
        simple_backends = simple.list_available()

        print(f"TYPF backends: {', '.join(typf_backends)}")
        print(f"Simple backends: {', '.join(simple_backends)}\n")

        # Render with all backends
        print("Rendering...")
        results = {}

        # TYPF backends
        for backend_name in typf_backends:
            try:
                print(f"  typf-{backend_name}... ", end="", flush=True)
                renderer = typf.TextRenderer(backend=backend_name)
                font = typf.Font(font_family, font_size)
                result = renderer.render(sample_text, font, format="png")

                if result:
                    filename = f"compare-typf-{backend_name}.png"
                    with open(filename, "wb") as f:
                        f.write(result)
                    results[f"typf-{backend_name}"] = filename
                    print("✓")
                else:
                    print("✗ No output")
            except Exception as e:
                print(f"✗ {str(e)[:50]}")

        # Simple backends (using same Merriweather font)
        for backend_name in simple_backends:
            try:
                print(f"  {backend_name}... ", end="", flush=True)
                renderer = simple.create_renderer(
                    backend_name,
                    font_path,
                    font_size=int(font_size),
                    width=2000,
                    height=200,
                )
                result = renderer.render_text(sample_text)

                # Convert numpy array to PNG
                img = Image.fromarray(result)
                filename = f"compare-{backend_name}.png"
                img.save(filename)
                results[backend_name] = filename
                print("✓")
            except Exception as e:
                print(f"✗ {str(e)[:50]}")

        print(f"\nRendered {len(results)} images:")
        for name, path in results.items():
            print(f"  {name:30s} → {path}")

        print("\nOpen images to compare:")
        print("  open compare-*.png")

        return 0

    def bench(self):
        """Benchmark all available rendering backends with comparison table (monochrome + grayscale)."""
        print("Benchmarking TYPF rendering backends...\n")
        print("Made by FontLab https://www.fontlab.com/\n")

        try:
            import typf
            import time
        except ImportError:
            print("Error: typf Python bindings not installed.")
            return 1

        # Sample text and settings
        sample_text = "The quick brown fox jumps over the lazy dog."
        font_family = "Merriweather"
        font_size = 64.0
        iterations = 100  # Number of iterations for timing

        # Get available backends
        available_backends = typf.TextRenderer.list_available_backends()

        # Benchmark each backend (default grayscale antialiasing)
        # Note: Monochrome mode would be benchmarked separately if/when antialias parameter is exposed
        results = []
        for backend_name in available_backends:
            try:
                renderer = typf.TextRenderer(backend=backend_name)
                font = typf.Font(font_family, font_size)

                # Warmup
                for _ in range(5):
                    renderer.render(sample_text, font, format="png")

                # Benchmark
                start = time.perf_counter()
                for _ in range(iterations):
                    renderer.render(sample_text, font, format="png")
                elapsed = time.perf_counter() - start

                avg_ms = (elapsed / iterations) * 1000
                results.append({
                    'backend': backend_name,
                    'avg_ms': avg_ms,
                    'ops_per_sec': iterations / elapsed
                })
            except Exception as e:
                results.append({
                    'backend': backend_name,
                    'avg_ms': None,
                    'ops_per_sec': None,
                    'error': str(e)
                })

        # Print comparison table
        print(f"\nBenchmark Results ({iterations} iterations, grayscale antialiasing):")
        print("=" * 80)
        print(f"{'Backend':<15} {'Avg Time (ms)':<20} {'Ops/sec':<15} {'Status':<20}")
        print("-" * 80)

        for result in sorted(results, key=lambda x: x['avg_ms'] if x['avg_ms'] else float('inf')):
            backend = result['backend']
            if result['avg_ms'] is not None:
                avg_ms = f"{result['avg_ms']:.3f}"
                ops_sec = f"{result['ops_per_sec']:.1f}"
                status = "✓ OK"
            else:
                avg_ms = "N/A"
                ops_sec = "N/A"
                status = f"✗ {result.get('error', 'Unknown error')[:15]}"

            print(f"{backend:<15} {avg_ms:<20} {ops_sec:<15} {status:<20}")

        print("=" * 80)
        print()

        # Print relative performance
        valid_results = [r for r in results if r['avg_ms'] is not None]
        if len(valid_results) > 1:
            fastest = min(valid_results, key=lambda x: x['avg_ms'])
            print(f"Relative Performance (vs {fastest['backend']}):")
            print("-" * 60)
            for result in sorted(valid_results, key=lambda x: x['avg_ms']):
                ratio = result['avg_ms'] / fastest['avg_ms']
                bar = "█" * int(ratio * 20)
                print(f"{result['backend']:<15} {ratio:>5.2f}x  {bar}")
            print()

        return 0

    def render(self, backend=None):
        """Render sample images with specified or all available backends (TYPF + simple).

        Args:
            backend: Specific backend name to use (e.g., 'coretext', 'orgehb', 'skiahb', 'orge').
                    If not specified, renders with all available backends.

        Examples:
            python toy.py render                      # All backends
            python toy.py render --backend=coretext   # Only CoreText
            python toy.py render --backend=skiahb     # Only SkiaHB
        """
        if backend:
            print(f"Rendering sample text with backend: {backend}\n")
        else:
            print("Rendering sample text with all available backends...\n")
        print("Made by FontLab https://www.fontlab.com/\n")

        try:
            import typf
            sys.path.insert(0, str(Path(__file__).parent))
            import simple_font_rendering_py as simple
            from PIL import Image
        except ImportError as e:
            print(f"Error: Missing dependencies: {e}")
            if "simple_font_rendering_py" in str(e):
                print("Note: simple_font_rendering_py requires:")
                print("  pip install pillow pyobjc-framework-CoreText uharfbuzz freetype-py")
            else:
                print("Error: typf Python bindings not installed.")
            # Continue with TYPF backends only
            simple = None

        # Sample text and settings - using distinctive Merriweather variable font
        sample_text = "The quick brown fox jumps over the lazy dog."
        font_family = "Merriweather"
        font_size = 64.0  # Larger size for better visibility

        # Find font file for simple backends
        font_path = find_merriweather_font()
        print(f"Using font: {font_path.name}")
        print(f"Font path: {font_path}\n")

        # Get available backends
        all_typf_backends = typf.TextRenderer.list_available_backends()
        simple_backends = simple.list_available() if simple else []

        # Filter backends if specific backend requested
        if backend:
            if backend in all_typf_backends:
                typf_backends = [backend]
            else:
                print(f"Error: Backend '{backend}' not available.")
                print(f"Available backends: {', '.join(all_typf_backends)}")
                return 1
        else:
            typf_backends = all_typf_backends

        all_backends = [f"typf-{b}" for b in typf_backends] + (simple_backends if not backend else [])
        print(f"Rendering with: {', '.join(all_backends)}\n")

        # Render with TYPF backends
        for backend_name in typf_backends:
            try:
                print(f"{backend_name:15s} ", end="", flush=True)
                renderer = typf.TextRenderer(backend=backend_name)
                font = typf.Font(font_family, font_size)

                # Render to PNG
                result = renderer.render(sample_text, font, format="png")

                if result:
                    filename = f"render-{backend_name}.png"
                    if hasattr(result, 'save'):
                        # It's a Bitmap object with save method
                        result.save(filename)
                        print(f"✓ Saved {filename}")
                    elif isinstance(result, bytes):
                        # It's raw bytes
                        with open(filename, "wb") as f:
                            f.write(result)
                        print(f"✓ Saved {filename}")
                    else:
                        print(f"✗ Unexpected result type: {type(result)}")
                else:
                    print("✗ No output")

            except Exception as e:
                print(f"✗ {str(e)}")

        # Render with simple backends
        if simple:
            for backend_name in simple_backends:
                try:
                    print(f"{backend_name:15s} ", end="", flush=True)
                    renderer = simple.create_renderer(
                        backend_name,
                        font_path,
                        font_size=int(font_size),
                        width=2000,
                        height=200,
                    )
                    result = renderer.render_text(sample_text)

                    # Convert numpy array to PNG
                    img = Image.fromarray(result)
                    filename = f"render-{backend_name}.png"
                    img.save(filename)
                    print(f"✓ Saved {filename}")

                except Exception as e:
                    print(f"✗ {str(e)[:60]}")

        return 0


if __name__ == "__main__":
    fire.Fire(Toy)
