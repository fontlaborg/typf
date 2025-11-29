# TODO

## Completed
- [x] Into README describe the 'feature mix' of shapers and renderers (see Font Feature Support Matrix and Glyph Format Support tables)
- [x] Perform extensive research on the feasibility of adding support for SVG glyphs, COLR v0 and COLR v1 (see WORK.md)
- [x] Into PLAN write an extensive plan to improve the feature mix (see PLAN.md)

## Color Font Support (P1)

### COLR v0 (Layered Color Glyphs)
- [x] Create `typf-render-color` backend with skrifa ColorPainter
- [x] Implement transform stack for glyph transforms
- [x] Implement clip stack for glyph clipping (uses OutlinePen → tiny-skia Mask)
- [x] Handle CPAL palette lookups (basic implementation done)
- [x] Add `--palette` CLI option for palette selection (already existed as `--color-palette`/`-p`, wired to RenderParams)
- [x] Test with Noto Color Emoji (11 tests passing)

### SVG Glyphs
- [x] Add resvg/usvg dependencies (workspace + typf-render-color with `svg` feature)
- [x] Implement SVG table glyph extraction (`svg::get_svg_document`)
- [x] Handle gzip-compressed SVG documents (auto-detected via magic bytes)
- [x] Integrate resvg for SVG→bitmap rendering (`svg::render_svg_glyph`)
- [x] Test with Twemoji SVG (TwitterColorEmoji.subset.ttf)

## Color Font Support (P2)

### COLR v1 (Gradients)
- [x] Extend ColorPainter for linear gradients
- [x] Extend ColorPainter for radial gradients
- [~] Extend ColorPainter for sweep gradients (fallback to solid color, tiny-skia lacks native sweep gradient)
- [x] Implement all CompositeMode blend modes (28 modes mapped)
- [x] Variable COLR axis support (`render_color_glyph_with_variations`)

### Bitmap Glyphs
- [x] Implement sbix bitmap extraction (via skrifa BitmapStrikes)
- [x] Implement CBDT/CBLC bitmap extraction (via skrifa BitmapStrikes)
- [x] Automatic size selection algorithm (`glyph_for_size`)
- [x] Fallback to outline when bitmap unavailable (`render_bitmap_glyph_or_outline`)

## Enhancements (P3)

### SVG Export from Renderers
- [ ] Skia renderer SVG output mode
- [ ] Zeno renderer SVG output mode
- [ ] Gradient preservation in SVG output
- [ ] Bitmap embedding in SVG for emoji

## Ongoing
- [ ] For Python, use `uv`, `uv add`, `uv pip`, `uv run python -m`
