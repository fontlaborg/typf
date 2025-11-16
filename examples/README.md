---
this_file: typf/examples/README.md
---

# typf Examples

This directory contains examples demonstrating all rendering paths in typf.

## Quick Start

```bash
# Run direct orge example (single glyph, no ICU-HB overhead)
cargo run --example direct_orge_single_glyph

# Run full text rendering with ICU-HB + orge
cargo run --example full_text_icu_hb_orge --features orge

# Run full text rendering with ICU-HB + tiny-skia
cargo run --example full_text_icu_hb_orge --no-default-features --features tiny-skia-renderer
```

## Examples

### Pattern 1: Full Unicode Text Line Rendering

**`full_text_icu_hb_orge.rs`** - Complete text processing pipeline
- ICU segmentation (grapheme, word, line breaks)
- HarfBuzz shaping (complex scripts, ligatures)
- skrifa font parsing (TrueType, CFF, variable fonts)
- orge rendering (ultra-smooth unhinted)

**Use when:**
- Rendering complete text lines
- Need complex script support (Arabic, Devanagari, etc.)
- Need BiDi text handling
- Need OpenType feature control (liga, kern, etc.)
- Working with variable fonts

### Pattern 2: Direct Single Glyph Rendering

**`direct_orge_single_glyph.rs`** - High-performance direct path
- Unicode codepoint → skrifa → orge
- NO ICU overhead
- NO HarfBuzz overhead
- ~100μs per glyph target

**Use when:**
- Rendering individual glyphs
- Building glyph atlases
- Need maximum performance
- Don't need text processing
- Have pre-shaped glyph IDs

### Platform-Specific Examples (Future)

**`full_text_coretext.rs`** (macOS only)
- Native CoreText rendering
- Includes platform hinting
- Best integration on macOS

**`full_text_directwrite.rs`** (Windows only)
- Native DirectWrite rendering
- Includes platform hinting
- Best integration on Windows

## Rendering Paths Comparison

| Path | Unicode | Shaping | Hinting | Speed | Use Case |
|------|---------|---------|---------|-------|----------|
| CoreText | ✅ | ✅ | ✅ | Fast | macOS native |
| DirectWrite | ✅ | ✅ | ✅ | Fast | Windows native |
| ICU-HB + orge | ✅ | ✅ | ❌ | Medium | Cross-platform text |
| Direct orge | ❌ | ❌ | ❌ | Very Fast | Single glyphs |

## Font Support

**All examples support:**
- ✅ Variable fonts (OpenType Variations)
- ✅ Static fonts
- ✅ TrueType outlines (quadratic Bézier)
- ✅ CFF/CFF2 outlines (cubic Bézier)
- ❌ Color fonts (out of scope)

## Building

```bash
# Build all examples
cargo build --examples

# Build with specific renderer
cargo build --examples --features orge
cargo build --examples --no-default-features --features tiny-skia-renderer

# Build with all features (for comparison)
cargo build --examples --all-features
```

## Testing

```bash
# Run example with test data
cargo run --example direct_orge_single_glyph

# Run with specific font
FONT_PATH=testdata/fonts/NotoSans-Regular.ttf cargo run --example direct_orge_single_glyph

# Run with variable font
FONT_PATH=testdata/fonts/RobotoFlex-VariableFont_wght.ttf cargo run --example direct_orge_single_glyph
```

## Performance Notes

### Direct orge Path
- **Target:** <100μs per glyph (48pt, simple shapes)
- **Overhead:** Minimal (skrifa parsing + scan conversion)
- **Memory:** ~50KB per glyph cache entry
- **Best for:** Glyph atlases, icon rendering, single character display

### Full ICU-HB + orge Path
- **Target:** <1ms per text line (typical)
- **Overhead:** ICU segmentation + HarfBuzz shaping
- **Memory:** <50MB cache (fonts + glyphs)
- **Best for:** Paragraphs, complex scripts, full text layout

## Architecture

```
┌─────────────────────────────────────────┐
│         Pattern 1: Full Text            │
│                                         │
│  Text → ICU → HarfBuzz → skrifa → orge │
│                                         │
└─────────────────────────────────────────┘

┌─────────────────────────────────────────┐
│      Pattern 2: Direct Glyph            │
│                                         │
│      Codepoint → skrifa → orge          │
│                                         │
└─────────────────────────────────────────┘
```

## Common Issues

### Font Not Found
```
Error: Font "Noto Sans" not found
```
**Solution:** Install system fonts or specify path to TTF/OTF file

### Variable Font Not Working
```
Warning: Axis 'wght' not found in font
```
**Solution:** Verify font has variations: `otfinfo -i font.ttf`

### Rendering Quality Issues
```
Bitmap appears jagged
```
**Solution:** Use grayscale rendering instead of monochrome:
```rust
render_grayscale_direct(width, height, GrayscaleLevel::Level4x4, ...)
```

## See Also

- **ARCHITECTURE.md** - Complete system architecture
- **CLAUDE.md** - Development guidelines
- **backends/typf-orge/README.md** - orge backend details
- **backends/typf-icu-hb/README.md** - ICU-HB backend details
