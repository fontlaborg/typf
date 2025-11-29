# TYPF Roadmap

## Vision

TYPF aims to be the most complete Rust text rendering library, supporting all major font formats and glyph types with multiple backend options.

---

## Phase 1: Color Font Foundation (COLR v0)

**Goal:** Enable solid-color layered glyph rendering via skrifa's ColorPainter API.

### Technical Approach

1. **Create `typf-render-color` backend**
   - Implement `skrifa::color::ColorPainter` trait
   - Use tiny-skia for layer compositing
   - Handle CPAL palette lookups

2. **ColorPainter Implementation**
   ```rust
   struct TinySkiaColorPainter {
       pixmap: Pixmap,
       transform_stack: Vec<Transform>,
       clip_stack: Vec<ClipMask>,
       palette: Vec<Color>,
   }

   impl ColorPainter for TinySkiaColorPainter {
       fn push_transform(&mut self, t: Transform) { ... }
       fn pop_transform(&mut self) { ... }
       fn push_clip_glyph(&mut self, glyph_id: GlyphId) { ... }
       fn push_clip_box(&mut self, box: BoundingBox<f32>) { ... }
       fn pop_clip(&mut self) { ... }
       fn fill(&mut self, brush: Brush) { ... }
       fn push_layer(&mut self, mode: CompositeMode) { ... }
       fn pop_layer(&mut self) { ... }
   }
   ```

3. **Integration with existing renderers**
   - Add color glyph detection in shaping phase
   - Route color glyphs to ColorPainter, others to standard path
   - Composite color and monochrome glyphs on same canvas

### Deliverables
- [x] `typf-render-color` crate with COLR v0 support
- [x] CPAL palette selection via CLI/API
- [x] Test suite with COLR v0 fonts (Noto Color Emoji, etc.) - 11 tests passing

---

## Phase 2: Gradient Support (COLR v1)

**Goal:** Full COLR v1 support including gradients and variable palettes.

### Technical Approach

1. **Extend ColorPainter for gradients**
   - Handle `Brush::LinearGradient`, `Brush::RadialGradient`, `Brush::SweepGradient`
   - Map to tiny-skia's gradient primitives
   - Support color stops and extend modes

2. **Variable font integration**
   - Pass variation coordinates to ColorGlyph::paint()
   - Handle ItemVariationStore for color variations

3. **CompositeMode support**
   - Implement all 27 Porter-Duff and blend modes
   - Use tiny-skia's BlendMode or custom compositing

### Deliverables
- [x] Linear gradient rendering
- [x] Radial gradient rendering
- [~] Sweep gradient rendering (fallback to solid, tiny-skia lacks native support)
- [x] Variable COLR glyph support (`render_color_glyph_with_variations`)
- [x] Full CompositeMode implementation (28 modes mapped)
- [x] Test suite with COLR v1 fonts (NotoColorEmojiColr1)

---

## Phase 3: SVG Glyph Rendering

**Goal:** Render SVG table glyphs using resvg.

### Technical Approach

1. **Add resvg dependency**
   ```toml
   [dependencies]
   resvg = "0.44"
   usvg = "0.44"
   ```

2. **SVG glyph detection and extraction**
   ```rust
   fn get_svg_glyph(font: &FontRef, glyph_id: GlyphId) -> Option<String> {
       let svg_table = font.svg()?;
       let doc_list = svg_table.document_list()?;
       for record in doc_list.iter() {
           if glyph_id >= record.start_glyph_id()
              && glyph_id <= record.end_glyph_id() {
               return Some(decompress_svg(record.svg_document()?));
           }
       }
       None
   }
   ```

3. **Render pipeline integration**
   - Check for SVG glyph before outline extraction
   - Parse SVG with usvg, render with resvg to pixmap
   - Handle gzip-compressed SVG documents
   - Position SVG at correct glyph location

### Deliverables
- [x] SVG table parsing and glyph lookup (`svg::get_svg_document`)
- [x] resvg integration for SVG rendering (`svg::render_svg_glyph`)
- [x] Gzip decompression for compressed SVGs (auto-detected via magic bytes)
- [x] Test suite with SVG fonts (Abelone, TwitterColorEmoji)

---

## Phase 4: Bitmap Glyph Support

**Goal:** Render embedded bitmap glyphs (CBDT, CBLC, sbix).

### Technical Approach

1. **Use skrifa::bitmap module**
   - Access bitmap strikes via `font.bitmap_strikes()`
   - Select appropriate size based on requested ppem
   - Extract PNG/glyph data

2. **Bitmap formats**
   - Apple sbix: PNG data with metrics
   - Google CBDT/CBLC: Raw bitmaps or PNG
   - EBDT/EBLC: Legacy monochrome/grayscale

3. **Size selection algorithm**
   ```rust
   fn select_bitmap_strike(font: &FontRef, ppem: u16) -> Option<BitmapStrike> {
       let strikes = font.bitmap_strikes();
       // Find exact match or next larger size
       strikes.iter()
           .filter(|s| s.ppem_x() >= ppem)
           .min_by_key(|s| s.ppem_x())
   }
   ```

### Deliverables
- [x] sbix bitmap glyph rendering (`bitmap::render_bitmap_glyph`)
- [x] CBDT/CBLC bitmap glyph rendering (`bitmap::render_bitmap_glyph`)
- [x] Automatic size selection (`glyph_for_size` with best-match algorithm)
- [x] Fallback to outline when bitmap unavailable (`render_bitmap_glyph_or_outline`)

---

## Phase 5: SVG Export Enhancements

**Goal:** Enable skia and zeno renderers to output SVG in addition to bitmaps.

### Technical Approach

1. **Abstract render target**
   ```rust
   enum RenderTarget {
       Bitmap(Pixmap),
       Svg(SvgBuilder),
   }
   ```

2. **Path-based rendering for SVG output**
   - Convert kurbo/zeno paths to SVG path strings
   - Preserve fill colors and gradients
   - Handle clipping and compositing

3. **Hybrid output**
   - Allow mixed bitmap+vector when needed
   - Embed bitmaps in SVG for emoji

### Deliverables
- [ ] Skia renderer SVG output mode
- [ ] Zeno renderer SVG output mode
- [ ] Gradient preservation in SVG
- [ ] Bitmap embedding option

---

## Implementation Priority

| Phase | Feature | Effort | Impact | Priority |
|-------|---------|--------|--------|----------|
| 1 | COLR v0 | Medium | High | P1 |
| 3 | SVG glyphs | Medium | High | P1 |
| 4 | Bitmap glyphs | Medium | Medium | P2 |
| 2 | COLR v1 | High | Medium | P2 |
| 5 | SVG export | Medium | Low | P3 |

---

## Dependencies

| Library | Version | Purpose |
|---------|---------|---------|
| skrifa | 0.39+ | ColorPainter, color module |
| read-fonts | 0.36+ | SVG table parsing |
| tiny-skia | 0.11+ | Rendering, compositing |
| resvg | 0.44+ | SVG glyph rendering |
| usvg | 0.44+ | SVG parsing |
| flate2 | 1.0+ | SVG decompression |

---

## Success Metrics

- [x] Noto Color Emoji renders correctly (COLR format) — `test_success_metric_noto_colr_emoji`
- [x] Apple Color Emoji sbix glyphs display at correct sizes — `test_success_metric_sbix_sizes`
- [x] Twemoji COLR v1 renders with gradients — `test_success_metric_colrv1_gradients`
- [x] Custom SVG fonts render accurately — `test_success_metric_svg_accuracy`
- [x] Performance: <10ms per glyph (well under 2x overhead) — `test_success_metric_performance`
