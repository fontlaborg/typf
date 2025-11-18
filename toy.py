#!/usr/bin/env python3
# this_file: toy.py
"""A simple CLI to benchmark and render with typf.

Usage:
    python toy.py bench          # Run Rust benchmarks
    python toy.py render         # Render samples with all backends

Made by FontLab https://www.fontlab.com/
"""

import fire
import subprocess
import sys
from pathlib import Path


class Toy:
    """A simple CLI to benchmark and render with typf."""

    def bench(self):
        """Run the project's Rust benchmarks."""
        print("Running benchmarks...\n")
        result = subprocess.run(
            [
                "cargo",
                "bench",
                "--workspace",
                "--bench",
                "speed",
            ],
            cwd=Path(__file__).parent,
        )
        return result.returncode

    def render(self):
        """Render sample images with all available backends."""
        print("Rendering sample text with all available backends...\n")

        try:
            import typf
        except ImportError:
            print(
                "Error: typf Python bindings not installed.\n"
                "Please build them first:\n"
                "  cd python\n"
                "  maturin develop --release --features 'python,icu,mac,orge'"
            )
            return 1

        # Sample text and settings
        sample_text = "The quick brown fox jumps over the lazy dog."
        font_size = 48.0

        # Get available backends from the library
        available_backends = typf.TextRenderer.list_available_backends()
        print(f"Available backends: {', '.join(available_backends)}\n")

        # Try each available backend
        for backend_name in available_backends:
            try:
                print(f"{backend_name:15s} ", end="", flush=True)
                renderer = typf.TextRenderer(backend=backend_name)
                font = typf.Font("Arial", font_size)

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

        return 0


if __name__ == "__main__":
    fire.Fire(Toy)
