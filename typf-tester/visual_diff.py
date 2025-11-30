#!/usr/bin/env python3
"""
Visual Diff Tool for Typf Renderers

Creates side-by-side comparisons of PNG outputs from different renderers
to help identify visual differences and quality issues.

Also computes pixel-level difference metrics (MSE, PSNR, SSIM) and generates
diff heatmaps for quantitative quality analysis.
"""

import sys
import json
import math
from pathlib import Path
from typing import List, Tuple, Optional, Dict, Any
import numpy as np

try:
    from PIL import Image, ImageDraw, ImageFont
except ImportError:
    print("❌ Pillow not installed. Install with: pip install Pillow")
    sys.exit(1)


class VisualDiff:
    """Tool for creating visual comparisons of renderer outputs."""

    def __init__(self):
        """Initialize with output directory."""
        self.base_dir = Path(__file__).parent
        self.output_dir = self.base_dir / "output"

        if not self.output_dir.exists():
            print(f"❌ Output directory not found: {self.output_dir}")
            sys.exit(1)

    def find_renders(self, shaper: str, text: str) -> List[Tuple[str, Path]]:
        """Find all PNG renders for a specific shaper and text."""
        pattern = f"render-{shaper}-*-{text}.png"
        files = list(self.output_dir.glob(pattern))

        # Extract renderer name from filename
        renders = []
        for f in files:
            # Format: render-{shaper}-{renderer}-{text}.png
            parts = f.stem.split('-')
            if len(parts) >= 3:
                renderer = parts[2]
                renders.append((renderer, f))

        return sorted(renders)

    def compute_mse(self, img1: Image.Image, img2: Image.Image) -> float:
        """Compute Mean Squared Error between two images."""
        # Convert to numpy arrays
        arr1 = np.array(img1.convert('L'), dtype=np.float64)
        arr2 = np.array(img2.convert('L'), dtype=np.float64)

        # Ensure same dimensions
        if arr1.shape != arr2.shape:
            return float('inf')

        # Compute MSE
        mse = np.mean((arr1 - arr2) ** 2)
        return float(mse)

    def compute_psnr(self, img1: Image.Image, img2: Image.Image) -> float:
        """Compute Peak Signal-to-Noise Ratio between two images."""
        mse = self.compute_mse(img1, img2)

        if mse == 0:
            return float('inf')  # Perfect match

        if math.isinf(mse):
            return 0.0  # Dimension mismatch

        # PSNR = 20 * log10(MAX_I) - 10 * log10(MSE)
        # For 8-bit images, MAX_I = 255
        psnr = 20 * math.log10(255.0) - 10 * math.log10(mse)
        return float(psnr)

    def create_diff_heatmap(
        self,
        img1: Image.Image,
        img2: Image.Image,
        label1: str,
        label2: str
    ) -> Tuple[Image.Image, Dict[str, float]]:
        """Create a difference heatmap and compute metrics."""
        # Convert to grayscale numpy arrays
        arr1 = np.array(img1.convert('L'), dtype=np.float64)
        arr2 = np.array(img2.convert('L'), dtype=np.float64)

        # Ensure same dimensions
        if arr1.shape != arr2.shape:
            # Create blank heatmap indicating dimension mismatch
            blank = Image.new('RGB', (max(img1.width, img2.width), max(img1.height, img2.height)), 'white')
            draw = ImageDraw.Draw(blank)
            draw.text((10, 10), f"Dimension mismatch: {arr1.shape} vs {arr2.shape}", fill='red')
            return blank, {'mse': float('inf'), 'psnr': 0.0, 'max_diff': 0.0}

        # Compute absolute difference
        diff = np.abs(arr1 - arr2)

        # Compute metrics
        mse = np.mean(diff ** 2)
        max_diff = np.max(diff)

        psnr = float('inf') if mse == 0 else 20 * math.log10(255.0) - 10 * math.log10(mse)

        # Create heatmap (scale to 0-255)
        if max_diff > 0:
            heatmap_data = (diff / max_diff * 255).astype(np.uint8)
        else:
            heatmap_data = np.zeros_like(diff, dtype=np.uint8)

        # Apply color map (black = no diff, red = max diff)
        heatmap_colored = np.zeros((*heatmap_data.shape, 3), dtype=np.uint8)
        heatmap_colored[:, :, 0] = heatmap_data  # Red channel

        heatmap_img = Image.fromarray(heatmap_colored, 'RGB')

        # Add label
        draw = ImageDraw.Draw(heatmap_img)
        draw.text((5, 5), f"{label1} vs {label2}", fill='white')

        metrics = {
            'mse': float(mse),
            'psnr': float(psnr),
            'max_diff': float(max_diff)
        }

        return heatmap_img, metrics

    def analyze_all_pairs(
        self,
        shaper: str,
        text: str
    ) -> Optional[Dict[str, Any]]:
        """Analyze all renderer pairs for a shaper/text combination."""
        renders = self.find_renders(shaper, text)

        if len(renders) < 2:
            return None

        print(f"\n🔬 Analyzing pixel differences for {shaper} + {text}")

        # Load all images
        images = []
        labels = []
        for renderer, path in renders:
            try:
                img = Image.open(path).convert('RGB')
                images.append(img)
                labels.append(renderer)
            except Exception as e:
                print(f"⚠ Failed to load {path}: {e}")
                continue

        if len(images) < 2:
            return None

        # Compute pairwise comparisons
        comparisons = []
        heatmaps = []

        for i in range(len(images)):
            for j in range(i + 1, len(images)):
                label1 = labels[i]
                label2 = labels[j]

                # Compute metrics
                mse = self.compute_mse(images[i], images[j])
                psnr = self.compute_psnr(images[i], images[j])

                # Create heatmap
                heatmap, metrics = self.create_diff_heatmap(
                    images[i], images[j], label1, label2
                )

                comparison = {
                    'renderer1': label1,
                    'renderer2': label2,
                    'mse': metrics['mse'],
                    'psnr': metrics['psnr'],
                    'max_diff': metrics['max_diff']
                }

                comparisons.append(comparison)
                heatmaps.append((f"{label1}-vs-{label2}", heatmap))

                # Print metrics
                if math.isinf(psnr):
                    psnr_str = "∞ (identical)"
                else:
                    psnr_str = f"{psnr:.2f} dB"

                print(f"   {label1} vs {label2}: MSE={mse:.2f}, PSNR={psnr_str}, MaxDiff={metrics['max_diff']:.1f}")

        # Save heatmaps
        for pair_label, heatmap in heatmaps:
            heatmap_path = self.output_dir / f"heatmap-{shaper}-{pair_label}-{text}.png"
            heatmap.save(heatmap_path)
            print(f"   ✅ Saved heatmap: {heatmap_path.name}")

        return {
            'shaper': shaper,
            'text': text,
            'comparisons': comparisons
        }

    def create_comparison(
        self,
        shaper: str,
        text: str,
        output_path: Optional[Path] = None
    ) -> Optional[Path]:
        """Create side-by-side comparison of all renderers for given shaper/text."""
        renders = self.find_renders(shaper, text)

        if len(renders) < 2:
            print(f"⚠ Not enough renders found for {shaper}/{text} (need at least 2)")
            return None

        print(f"\n📊 Creating comparison for {shaper} + {text}")
        print(f"   Found {len(renders)} renderers: {[r[0] for r in renders]}")

        # Load all images
        images = []
        labels = []
        for renderer, path in renders:
            try:
                img = Image.open(path).convert('RGB')
                images.append(img)
                labels.append(renderer)
            except Exception as e:
                print(f"⚠ Failed to load {path}: {e}")
                continue

        if not images:
            print("❌ No images could be loaded")
            return None

        # Calculate layout
        img_width = max(img.width for img in images)
        img_height = max(img.height for img in images)
        label_height = 30  # Space for labels
        padding = 20

        # Create comparison grid (2 columns)
        cols = 2
        rows = (len(images) + cols - 1) // cols

        canvas_width = (img_width + padding) * cols + padding
        canvas_height = (img_height + label_height + padding) * rows + padding

        # Create canvas with white background
        canvas = Image.new('RGB', (canvas_width, canvas_height), 'white')
        draw = ImageDraw.Draw(canvas)

        # Place images in grid
        for i, (img, label) in enumerate(zip(images, labels)):
            row = i // cols
            col = i % cols

            x = padding + col * (img_width + padding)
            y = padding + row * (img_height + label_height + padding)

            # Paste image
            canvas.paste(img, (x, y + label_height))

            # Draw label
            label_text = f"{label} ({img.width}×{img.height})"
            # Use default font (pillow built-in)
            draw.text((x, y), label_text, fill='black')

            # Draw border around image
            draw.rectangle(
                [x, y + label_height, x + img.width, y + label_height + img.height],
                outline='gray',
                width=1
            )

        # Add title
        title = f"Comparison: {shaper} shaper, {text} text"
        draw.text((padding, 5), title, fill='black')

        # Save comparison
        if output_path is None:
            output_path = self.output_dir / f"diff-{shaper}-{text}.png"

        canvas.save(output_path)
        print(f"✅ Saved comparison to: {output_path}")
        return output_path

    def create_all_comparisons(self):
        """Create comparisons for all shaper/text combinations."""
        # Find all unique shaper/text combinations
        all_files = list(self.output_dir.glob("render-*-*-*.png"))
        combinations = set()

        for f in all_files:
            parts = f.stem.split('-')
            if len(parts) >= 4:
                shaper = parts[1]
                text = parts[3]
                combinations.add((shaper, text))

        print(f"Found {len(combinations)} shaper/text combinations")
        print("=" * 80)

        created = []
        for shaper, text in sorted(combinations):
            output = self.create_comparison(shaper, text)
            if output:
                created.append(output)

        print("\n" + "=" * 80)
        print(f"✅ Created {len(created)} comparison images")
        return created

    def analyze_all_combinations(self) -> Dict[str, Any]:
        """Analyze all shaper/text combinations and create report."""
        # Find all unique shaper/text combinations
        all_files = list(self.output_dir.glob("render-*-*-*.png"))
        combinations = set()

        for f in all_files:
            parts = f.stem.split('-')
            if len(parts) >= 4:
                shaper = parts[1]
                text = parts[3]
                combinations.add((shaper, text))

        print(f"\n🔬 Analyzing {len(combinations)} shaper/text combinations")
        print("=" * 80)

        all_results = []
        for shaper, text in sorted(combinations):
            result = self.analyze_all_pairs(shaper, text)
            if result:
                all_results.append(result)

        # Save JSON report
        report_path = self.output_dir / "pixel_diff_analysis.json"
        with open(report_path, 'w') as f:
            json.dump(all_results, f, indent=2)

        print("\n" + "=" * 80)
        print(f"✅ Analyzed {len(all_results)} combinations")
        print(f"✅ Saved analysis report to: {report_path}")

        return {
            'total_combinations': len(all_results),
            'results': all_results
        }


def main():
    """Main entry point."""
    import argparse

    parser = argparse.ArgumentParser(
        description="Visual diff tool for Typf renders with pixel-level analysis"
    )
    parser.add_argument(
        "--shaper",
        help="Specific shaper to compare (e.g., harfbuzz)",
        default=None
    )
    parser.add_argument(
        "--text",
        help="Specific text to compare (e.g., latn, arab, mixd)",
        default=None
    )
    parser.add_argument(
        "--all",
        action="store_true",
        help="Create comparisons for all shaper/text combinations"
    )
    parser.add_argument(
        "--analyze",
        action="store_true",
        help="Compute pixel-level metrics (MSE, PSNR) and create diff heatmaps"
    )

    args = parser.parse_args()

    diff = VisualDiff()

    if args.analyze:
        # Pixel-level analysis mode
        if args.shaper and args.text:
            # Analyze specific combination
            result = diff.analyze_all_pairs(args.shaper, args.text)
            if result:
                print(f"\n✅ Analysis complete for {args.shaper} + {args.text}")
            else:
                print(f"❌ No renders found for {args.shaper} + {args.text}")
        else:
            # Analyze all combinations
            diff.analyze_all_combinations()
    elif args.all:
        # Visual comparison mode
        diff.create_all_comparisons()
    elif args.shaper and args.text:
        # Single comparison
        diff.create_comparison(args.shaper, args.text)
    elif args.shaper or args.text:
        print("❌ Must specify both --shaper and --text, or use --all")
        return 1
    else:
        # Default: create all comparisons
        diff.create_all_comparisons()

    return 0


if __name__ == "__main__":
    sys.exit(main())
