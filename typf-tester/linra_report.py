#!/usr/bin/env python3
"""
Linra Analysis Report Generator for Typf

Combines performance benchmarks, pixel-level quality analysis, and visual
comparisons into a single comprehensive report.
"""

import sys
import json
from pathlib import Path
from typing import Dict, Any, List
from datetime import datetime


class LinraReport:
    """Generate comprehensive analysis reports combining all data sources."""

    def __init__(self):
        """Initialize with output directory."""
        self.base_dir = Path(__file__).parent
        self.output_dir = self.base_dir / "output"

        if not self.output_dir.exists():
            print(f"❌ Output directory not found: {self.output_dir}")
            sys.exit(1)

    def load_benchmark_data(self) -> Dict[str, Any]:
        """Load performance benchmark data."""
        benchmark_file = self.output_dir / "benchmark_report.json"
        if not benchmark_file.exists():
            print(f"⚠ Benchmark report not found: {benchmark_file}")
            return {}

        with open(benchmark_file) as f:
            return json.load(f)

    def load_pixel_diff_data(self) -> Dict[str, Any]:
        """Load pixel-level difference analysis data."""
        pixel_file = self.output_dir / "pixel_diff_analysis.json"
        if not pixel_file.exists():
            print(f"⚠ Pixel diff analysis not found: {pixel_file}")
            return {}

        with open(pixel_file) as f:
            data = json.load(f)
            # Convert list to dict keyed by shaper+text
            result = {}
            for item in data:
                key = f"{item['shaper']}+{item['text']}"
                result[key] = item
            return result

    def analyze_quality_metrics(self) -> Dict[str, Any]:
        """Analyze image quality from PNG files."""
        try:
            from PIL import Image
            import numpy as np
        except ImportError:
            print("⚠ PIL/numpy not available, skipping quality analysis")
            return {}

        png_files = list(self.output_dir.glob("render-*-*-*.png"))
        quality_data = {}

        for png_file in png_files:
            parts = png_file.stem.split('-')
            if len(parts) >= 4:
                shaper = parts[1]
                renderer = parts[2]
                text = parts[3]

                try:
                    img = Image.open(png_file)
                    arr = np.array(img.convert('L'))

                    # Calculate metrics
                    coverage = float(np.sum(arr < 255) / arr.size * 100)
                    unique_grays = len(np.unique(arr))
                    file_size = png_file.stat().st_size / 1024  # KB

                    key = f"{shaper}+{renderer}+{text}"
                    quality_data[key] = {
                        'shaper': shaper,
                        'renderer': renderer,
                        'text': text,
                        'coverage': coverage,
                        'unique_grays': unique_grays,
                        'file_size_kb': file_size,
                        'width': img.width,
                        'height': img.height
                    }
                except Exception as e:
                    print(f"⚠ Failed to analyze {png_file}: {e}")
                    continue

        return quality_data

    def generate_markdown_report(self) -> str:
        """Generate comprehensive markdown report."""
        print("📊 Generating linra analysis report...")

        # Load all data
        benchmark_data = self.load_benchmark_data()
        pixel_diff_data = self.load_pixel_diff_data()
        quality_data = self.analyze_quality_metrics()

        # Build report
        lines = []
        lines.append("# Typf Linra Analysis Report")
        lines.append("")
        lines.append(f"**Generated:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        lines.append("")
        lines.append("---")
        lines.append("")

        # Section 1: Performance Summary
        if benchmark_data:
            lines.append("## 1. Performance Benchmarks")
            lines.append("")
            lines.append("### Overall Statistics")
            lines.append("")

            if 'summary' in benchmark_data:
                summary = benchmark_data['summary']
                lines.append(f"- **Total Runs**: {summary.get('total_runs', 0)}")
                lines.append(f"- **Successful**: {summary.get('successful', 0)}")
                lines.append(f"- **Failed**: {summary.get('failed', 0)}")
                lines.append(f"- **Success Rate**: {summary.get('success_rate', 0):.1f}%")
                lines.append("")

            # Top performers
            if 'benchmarks' in benchmark_data:
                benchmarks = benchmark_data['benchmarks']

                # Find fastest
                fastest = min(benchmarks, key=lambda x: x.get('avg_time_ms', float('inf')))
                lines.append(f"### ⚡ Fastest Configuration")
                lines.append("")
                lines.append(f"- **Backend**: {fastest['shaper']} + {fastest['renderer']}")
                lines.append(f"- **Text**: {fastest['text']}")
                lines.append(f"- **Avg Time**: {fastest['avg_time_ms']:.2f} ms")
                lines.append(f"- **Ops/sec**: {fastest['ops_per_sec']:.1f}")
                lines.append("")

                # Performance table
                lines.append("### Performance by Backend")
                lines.append("")
                lines.append("| Shaper | Renderer | Text | Avg Time (ms) | Ops/sec |")
                lines.append("|--------|----------|------|---------------|---------|")

                for b in sorted(benchmarks, key=lambda x: x.get('avg_time_ms', 0))[:10]:
                    lines.append(
                        f"| {b['shaper']} | {b['renderer']} | {b['text']} | "
                        f"{b['avg_time_ms']:.2f} | {b['ops_per_sec']:.1f} |"
                    )
                lines.append("")

        # Section 2: Visual Quality Analysis
        if pixel_diff_data:
            lines.append("## 2. Visual Quality Analysis")
            lines.append("")
            lines.append("### Renderer Similarity Matrix (PSNR)")
            lines.append("")
            lines.append("Higher PSNR = more similar rendering")
            lines.append("")

            for key, data in sorted(pixel_diff_data.items()):
                shaper = data['shaper']
                text = data['text']
                comparisons = data['comparisons']

                lines.append(f"#### {shaper.upper()} Shaper - {text.upper()} Text")
                lines.append("")
                lines.append("| Renderer 1 | Renderer 2 | MSE | PSNR (dB) | Max Diff |")
                lines.append("|------------|------------|-----|-----------|----------|")

                for comp in sorted(comparisons, key=lambda x: x['psnr'], reverse=True):
                    psnr_str = "∞" if comp['psnr'] == float('inf') else f"{comp['psnr']:.2f}"
                    lines.append(
                        f"| {comp['renderer1']} | {comp['renderer2']} | "
                        f"{comp['mse']:.2f} | {psnr_str} | {comp['max_diff']:.1f} |"
                    )
                lines.append("")

            # Best matches
            all_comparisons = []
            for data in pixel_diff_data.values():
                for comp in data['comparisons']:
                    comp['shaper'] = data['shaper']
                    comp['text'] = data['text']
                    all_comparisons.append(comp)

            if all_comparisons:
                best_matches = sorted(
                    [c for c in all_comparisons if c['psnr'] != float('inf')],
                    key=lambda x: x['psnr'],
                    reverse=True
                )[:5]

                lines.append("### 🏆 Most Similar Renderer Pairs")
                lines.append("")
                for i, comp in enumerate(best_matches, 1):
                    lines.append(
                        f"{i}. **{comp['renderer1']}** vs **{comp['renderer2']}** "
                        f"({comp['shaper']}/{comp['text']}): {comp['psnr']:.2f} dB"
                    )
                lines.append("")

                worst_matches = sorted(
                    all_comparisons,
                    key=lambda x: x['psnr']
                )[:5]

                lines.append("### ⚠️ Most Different Renderer Pairs")
                lines.append("")
                for i, comp in enumerate(worst_matches, 1):
                    psnr_str = "N/A" if comp['psnr'] == float('inf') else f"{comp['psnr']:.2f} dB"
                    lines.append(
                        f"{i}. **{comp['renderer1']}** vs **{comp['renderer2']}** "
                        f"({comp['shaper']}/{comp['text']}): {psnr_str}"
                    )
                lines.append("")

        # Section 3: Image Quality Metrics
        if quality_data:
            lines.append("## 3. Image Quality Metrics")
            lines.append("")

            # Group by renderer
            by_renderer = {}
            for key, data in quality_data.items():
                renderer = data['renderer']
                if renderer not in by_renderer:
                    by_renderer[renderer] = []
                by_renderer[renderer].append(data)

            # Calculate averages
            lines.append("### Average Quality by Renderer")
            lines.append("")
            lines.append("| Renderer | Avg Coverage | Avg Grays | Avg Size (KB) |")
            lines.append("|----------|--------------|-----------|---------------|")

            for renderer, items in sorted(by_renderer.items()):
                avg_coverage = sum(i['coverage'] for i in items) / len(items)
                avg_grays = sum(i['unique_grays'] for i in items) / len(items)
                avg_size = sum(i['file_size_kb'] for i in items) / len(items)

                lines.append(
                    f"| {renderer} | {avg_coverage:.2f}% | {avg_grays:.1f} | {avg_size:.2f} |"
                )
            lines.append("")

            # Best anti-aliasing
            best_aa = max(quality_data.values(), key=lambda x: x['unique_grays'])
            lines.append("### 🎨 Best Anti-Aliasing")
            lines.append("")
            lines.append(f"**{best_aa['renderer']}** ({best_aa['shaper']}/{best_aa['text']})")
            lines.append(f"- {best_aa['unique_grays']} unique gray levels")
            lines.append(f"- {best_aa['coverage']:.2f}% coverage")
            lines.append("")

        # Section 4: Recommendations
        lines.append("## 4. Recommendations")
        lines.append("")

        if benchmark_data and 'benchmarks' in benchmark_data:
            fastest = min(
                benchmark_data['benchmarks'],
                key=lambda x: x.get('avg_time_ms', float('inf'))
            )
            lines.append(f"### ⚡ For Maximum Performance")
            lines.append(f"Use **{fastest['shaper']}** + **{fastest['renderer']}**")
            lines.append(f"- Average render time: {fastest['avg_time_ms']:.2f} ms")
            lines.append("")

        if pixel_diff_data:
            lines.append("### 🎯 For Consistency")
            lines.append("Renderers with PSNR > 15 dB show good agreement.")
            lines.append("Consider using **opixa** or **skia** for consistent results.")
            lines.append("")

        if quality_data:
            lines.append("### 🎨 For Visual Quality")
            best_aa = max(quality_data.values(), key=lambda x: x['unique_grays'])
            lines.append(f"Use **{best_aa['renderer']}** for best anti-aliasing")
            lines.append(f"- {best_aa['unique_grays']} gray levels for smooth rendering")
            lines.append("")

        lines.append("---")
        lines.append("")
        lines.append("*Report generated by Typf Linra Analysis*")
        lines.append("")

        return "\n".join(lines)

    def generate_json_report(self) -> Dict[str, Any]:
        """Generate comprehensive JSON report."""
        return {
            'generated_at': datetime.now().isoformat(),
            'performance': self.load_benchmark_data(),
            'visual_quality': self.load_pixel_diff_data(),
            'image_quality': self.analyze_quality_metrics()
        }

    def save_reports(self):
        """Generate and save all reports."""
        print("\n📊 Typf Linra Analysis Report Generator")
        print("=" * 80)

        # Generate markdown report
        markdown = self.generate_markdown_report()
        md_path = self.output_dir / "linra_analysis.md"
        with open(md_path, 'w') as f:
            f.write(markdown)
        print(f"✅ Saved markdown report: {md_path}")

        # Generate JSON report
        json_data = self.generate_json_report()
        json_path = self.output_dir / "linra_analysis.json"
        with open(json_path, 'w') as f:
            json.dump(json_data, f, indent=2)
        print(f"✅ Saved JSON report: {json_path}")

        print("\n" + "=" * 80)
        print("✅ Report generation complete!")
        print(f"\n📄 View markdown report: {md_path}")
        print(f"📊 View JSON data: {json_path}")


def main():
    """Main entry point."""
    report = LinraReport()
    report.save_reports()
    return 0


if __name__ == "__main__":
    sys.exit(main())
