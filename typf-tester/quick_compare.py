#!/usr/bin/env python3
"""Quick visual comparison tool for Typf renderer outputs.

Opens all PNG outputs for a given shaper-text combination side-by-side for visual inspection.
"""

import subprocess
import sys
from pathlib import Path


def quick_compare(shaper: str = "harfbuzz", text: str = "latn"):
    """Open all renderer outputs for quick visual comparison.

    Args:
        shaper: Shaping backend (harfbuzz, coretext, icu-hb, none)
        text: Text type (latn, arab, mixd)
    """
    output_dir = Path(__file__).parent / "output"

    # Find all PNG files for this shaper-text combination
    pattern = f"render-{shaper}-*-{text}.png"
    files = sorted(output_dir.glob(pattern))

    if not files:
        print(f"❌ No files found matching pattern: {pattern}")
        print(f"   Searched in: {output_dir}")
        return 1

    print(f"📊 Opening {len(files)} renderer outputs for {shaper} + {text}:")
    for f in files:
        renderer = f.stem.split('-')[2]  # Extract renderer name
        print(f"   • {renderer:15} → {f.name}")

    # Open all files
    subprocess.run(["open"] + [str(f) for f in files])
    return 0


def main():
    """Command-line interface."""
    import fire
    fire.Fire(quick_compare)


if __name__ == "__main__":
    sys.exit(main() or 0)
