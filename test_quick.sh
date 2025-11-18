#!/bin/bash
# Quick test script to compare OrgeHB vs SkiaHB outputs
# Made by FontLab https://www.fontlab.com/

set -e

echo "Building Python bindings..."
cd python
maturin develop --release --features "python,icu,mac" --quiet
cd ..

echo ""
echo "Testing OrgeHB vs SkiaHB..."
python3 << 'EOF'
import typf

text = "Test"

print("\n=== OrgeHB ===")
result_orgehb = typf.shape_text(text, "Georgia", 48.0, backend="orgehb")
print(f"Advance: {result_orgehb.advance:.2f}")
print(f"BBox: {result_orgehb.bbox.width:.2f} x {result_orgehb.bbox.height:.2f}")
print(f"Glyphs: {len(result_orgehb.glyphs)}")
for i, g in enumerate(result_orgehb.glyphs):
    print(f"  [{i}] id={g.id:3d} x={g.x:7.2f} y={g.y:7.2f} adv={g.advance:7.2f}")

print("\n=== SkiaHB ===")
result_skiahb = typf.shape_text(text, "Georgia", 48.0, backend="skiahb")
print(f"Advance: {result_skiahb.advance:.2f}")
print(f"BBox: {result_skiahb.bbox.width:.2f} x {result_skiahb.bbox.height:.2f}")
print(f"Glyphs: {len(result_skiahb.glyphs)}")
for i, g in enumerate(result_skiahb.glyphs):
    print(f"  [{i}] id={g.id:3d} x={g.x:7.2f} y={g.y:7.2f} adv={g.advance:7.2f}")

print("\n=== Comparison ===")
print(f"Advance ratio: {result_orgehb.advance / result_skiahb.advance:.4f}x")
print(f"BBox width ratio: {result_orgehb.bbox.width / result_skiahb.bbox.width:.4f}x")
print(f"BBox height ratio: {result_orgehb.bbox.height / result_skiahb.bbox.height:.4f}x")
EOF
