# Current Work Session

## Completed P0 Items (Quality Sprint)

### 1. SVG Embedding Fix (typf-export)
- **File**: `crates/typf-export/src/svg.rs`
- **Issue**: `bitmap_to_png` wrote invalid PNG (magic bytes + raw data)
- **Fix**: Extracted proper PNG encoding to `encode_bitmap_to_png()` in `png.rs`, shared between `PngExporter` and `SvgExporter`
- **Added**: Buffer size validation, fails fast on malformed/short buffers
- **Tests**: Added 6 new tests including Gray1 format, short-buffer errors, embedded PNG validation

### 2. Bidi/Script Indexing Fix (typf-unicode)
- **File**: `crates/typf-unicode/src/lib.rs`
- **Issue**: `create_bidi_runs` indexed `bidi_info.levels` by byte position but levels are char-indexed
- **Fix**: Convert byte positions to char positions before accessing levels
- **Tests**: Added 5 new tests for mixed scripts (Arabic+Latin+Emoji, Hebrew+numbers, Thai marks, multibyte boundaries)

### 3. Pipeline Documentation Update (typf-core)
- **File**: `crates/typf-core/src/pipeline.rs`
- **Issue**: Docs claimed 6 functional stages but first 3 were stubs
- **Fix**: Updated docs to be truthful: `process()` is recommended (direct Shape→Render→Export), `execute()` runs stages with first 3 as pass-throughs
- **Tests**: All 9 pipeline tests pass

### 4. StubFont Removal (typf-cli)
- **File**: `crates/typf-cli/src/commands/render.rs`
- **Issue**: CLI silently used StubFont with fake metrics when no font provided
- **Fix**: Removed StubFont, `load_font()` now returns clear error with example usage
- **Note**: Python `render_simple` kept (explicit opt-in function name)

## Completed P1 Items

### 5. ShapingCache Integration (typf-shape-hb, typf-shape-icu-hb)
- **Files**: `backends/typf-shape-hb/src/lib.rs`, `backends/typf-shape-icu-hb/src/lib.rs`
- **Issue**: ShapingCache was defined in typf-core but not used by shapers
- **Fix**:
  - Added optional `cache: Option<SharedShapingCache>` field to both shapers
  - Added `with_cache()` and `with_shared_cache()` constructors
  - Check cache before shaping, insert results after shaping
  - Added `cache_stats()` and `cache_hit_rate()` methods
  - Implemented `clear_cache()` trait method
- **Tests**: Added 7 new tests for HarfBuzz shaper, 5 for ICU-HarfBuzz:
  - Cache hit/miss verification
  - Stats tracking
  - Shared cache across shapers
  - Clear cache functionality
  - Different params = different cache keys
  - Normalization before caching (ICU-HB specific)

### 6. Canvas Sizing Fix (All Renderers)
- **Files**: `backends/typf-render-skia/src/lib.rs`, `backends/typf-render-zeno/src/lib.rs`, `backends/typf-render-opixa/src/lib.rs`, `backends/typf-render-svg/src/lib.rs`
- **Issue**: All renderers used `advance_height * 1.2` or `height * 0.8` heuristics for canvas sizing, causing clipping of tall glyphs (emoji, Thai marks, Arabic diacritics)
- **Fix**: Changed to multi-phase rendering approach in all renderers:
  - **Phase 1**: Render/extract all glyph paths, collect actual min_y/max_y bounds
  - **Phase 2**: Calculate canvas/viewBox dimensions from actual content bounds
  - **Phase 3**: Composite glyphs with correct baseline positioning
- **Details**:
  - **Skia**: Uses `bearing_y` from `GlyphBitmap` for bounds tracking, stores rendered glyphs in `RenderedGlyph` struct
  - **Zeno**: Same as Skia, skips empty glyphs (width/height == 0) during bounds calculation
  - **Opixa**: Uses `glyph_bitmap.top` instead of `bearing_y`, otherwise same approach
  - **SVG**: Modified `SvgPathBuilder` to track Y coordinates during path construction, returns bounds with `finish_with_bounds()`
- **Tests**: All renderers pass existing tests (Skia: 6, Zeno: 10, Opixa: 69, SVG: 3)

## Test Results
```
cargo test --workspace: All tests pass
cargo clippy --workspace: No warnings
```

## Files Modified
- `crates/typf-export/src/png.rs` - Added `encode_bitmap_to_png()` with buffer validation
- `crates/typf-export/src/svg.rs` - Use proper PNG encoding, added tests
- `crates/typf-export/src/lib.rs` - Export `encode_bitmap_to_png`
- `crates/typf-export/Cargo.toml` - Added base64 dev-dependency
- `crates/typf-unicode/src/lib.rs` - Fixed byte-to-char indexing in bidi, added tests
- `crates/typf-core/src/pipeline.rs` - Updated docs and stage comments
- `crates/typf-cli/src/commands/render.rs` - Removed StubFont, require font file
- `backends/typf-shape-hb/src/lib.rs` - Added cache integration (optional caching)
- `backends/typf-shape-icu-hb/src/lib.rs` - Added cache integration (optional caching)
- `backends/typf-render-skia/src/lib.rs` - Multi-phase rendering with actual bounds
- `backends/typf-render-zeno/src/lib.rs` - Multi-phase rendering with actual bounds
- `backends/typf-render-opixa/src/lib.rs` - Multi-phase rendering with actual bounds
- `backends/typf-render-svg/src/lib.rs` - Two-phase rendering with viewBox from glyph bounds
- `TODO.md` - Marked P0 and P1 items complete
