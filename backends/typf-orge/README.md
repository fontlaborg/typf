# typf-orge

**orge - ultra-smooth unhinted font rasterization engine**

## Overview

This crate provides a specialized scan converter for supersmooth, unhinted font rendering. It focuses exclusively on the rasterization algorithm (NOT hinting), providing production-quality glyph rendering with:

- **Scanline rasterization** with edge tables
- **Bézier curve subdivision** (quadratic and cubic)
- **Sub-pixel precision** (26.6 fixed-point)
- **Fill rules** (non-zero winding, even-odd)
- **skrifa integration** (OutlinePen trait)

## Status

✅ **Core implementation complete** (Week 9 of Phase 2 - Renaming)

- 76 tests passing (62 in orge + 14 in typf-icu-hb)
- Zero warnings
- ~2,200 lines of code
- Ready for production use

## Architecture

```
typf-orge/
├── fixed.rs           # F26Dot6 fixed-point (26.6 format)
├── edge.rs            # Edge lists for scanline algorithm
├── scan_converter.rs  # Main rasterization engine
├── curves.rs          # Bézier subdivision (quadratic, cubic)
├── grayscale.rs       # Anti-aliasing via oversampling
└── lib.rs             # Public API
```

## Usage

```rust
use typf_orge::scan_converter::ScanConverter;
use typf_orge::fixed::F26Dot6;

// Create 64x64 bitmap
let mut sc = ScanConverter::new(64, 64);

// Draw rectangle
sc.move_to(F26Dot6::from_int(10), F26Dot6::from_int(10));
sc.line_to(F26Dot6::from_int(50), F26Dot6::from_int(10));
sc.line_to(F26Dot6::from_int(50), F26Dot6::from_int(50));
sc.line_to(F26Dot6::from_int(10), F26Dot6::from_int(50));
sc.close();

// Render to bitmap
let mut bitmap = vec![0u8; 64 * 64];
sc.render_mono(&mut bitmap);
```

### With skrifa

```rust
use skrifa::FontRef;
use skrifa::outline::OutlinePen;

let font_data = std::fs::read("font.ttf")?;
let font = FontRef::new(&font_data)?;

let mut sc = ScanConverter::new(64, 64);

// Get glyph outline
let glyph_id = font.charmap().map('A').unwrap();
font.outline_glyphs()
    .get(glyph_id)
    .unwrap()
    .draw(skrifa::instance::Size::unscaled(), &mut sc)?;

// Render
let mut bitmap = vec![0u8; 64 * 64];
sc.render_mono(&mut bitmap);
```

## Features

### Implemented ✅

- F26Dot6 fixed-point arithmetic
- Edge table management
- Active edge list
- Scanline rasterization
- Non-zero winding fill rule
- Even-odd fill rule
- Quadratic Bézier subdivision
- Cubic Bézier subdivision
- skrifa OutlinePen integration
- Monochrome rendering
- Grayscale rendering (2x2, 4x4, 8x8 oversampling)
- Integration with typf-icu-hb

### TODO 📋

- Smart dropout control (Simple mode implemented)
- Performance optimization (SIMD, profiling)
- Comparison test scripts

## Performance

Benchmarks (100 edges):

- `Vec::with_capacity()` + `push()`: **280 ns**
- `sort_by_x()`: **92 ns**
- Edge table lookup: **~1 ns per scanline**

Total overhead is <1% of render budget. No custom memory pool needed.

## Design Decisions

1. **Fixed-point:** 26.6 format for 1/64 pixel precision
2. **Edge storage:** Vec<Edge> (fast, simple, cache-friendly)
3. **Flatness threshold:** 4/64 = 1/16 pixel (good quality/performance balance)
4. **Fill rule:** Non-zero winding recommended for fonts
5. **Coordinate system:** Y-flip handled in OutlinePen (font space → graphics space)

## Documentation

See project documentation:

- `WORK.md` - Development progress
- `ALLOCATION.md` - Memory allocation strategy
- `TODO.md` - Task tracking
- `report/06.md` - orge port strategy

## Testing

```bash
# Unit tests
cargo test --package typf-orge

# Integration tests
cargo test --package typf-icu-hb --features orge

# All tests
cargo test --workspace --all-features

# Benchmarks
cargo bench --package typf-orge
```

All 76 tests pass. Zero warnings.

## License

Same as typf workspace (see root LICENSE file).

## Credits

Advanced scan conversion algorithms ported to Rust for the typf project.
