# Plan — SVG/COLR Support Across Pure-Rust Backends

Scope: deliver SVG emission and color glyph support across tiny-skia and zeno while adding configurable glyph-source selection for all non-OS renderers.

Assumptions
- Current non-native renderers: opixa, skia (tiny-skia), zeno, svg exporter; OS renderers (coregraphics/directwrite/coretext-linra) stay unchanged.
- skrifa already exposes COLRv0/v1, SVG table access, and ColorPainter APIs we can reuse.
- Test fonts on hand include COLRv0, COLRv1, SVG, sbix, CBDT/EBDT samples.

Objectives
1) Emit SVG from tiny-skia and zeno backends as a vector alternative to typf-render-svg.
2) Integrate usvg/resvg to read and rasterize the `SVG ` table in non-OS renderers.
3) Provide COLRv0 and COLRv1 rendering in pure-Rust backends using skrifa’s color hooks.
4) Add a renderer-level glyph-source preference/deny mechanism covering glyf, CFF, CFF2, COLR, sbix, CBDT/EBDT, and `SVG ` so callers can opt out of color glyphs or pick a priority order.

Phase 0 — Recon & Spike
- Inventory current path-building code in `backends/typf-render-skia` and `backends/typf-render-zeno` to see what can be reused for vector export and COLR painting.
- Verify what `typf-render-color` already implements; decide whether to extend it or fold capabilities into the core renderers.
- Spike a minimal `SVG ` table decode using skrifa + usvg to confirm parsing and coordinate handling; record pitfalls (viewBox, units-per-em scaling, font-origin).

Phase 1 — Core API for Glyph Source Selection ✓ COMPLETE
- [x] Add a `GlyphSourcePreference` struct (ordered allowlist + deny set) in `typf-core` and thread it through `RenderParams` and the CLI flags (`--glyph-source prefer=A,B --glyph-source deny=COLR,SVG`).
- [x] Define canonical source enum values: glyf, CFF, CFF2, COLR0, COLR1, SBIX, CBDT, EBDT, SVG; set default ordering (vector outlines first, then color, then bitmaps) matching current behavior.
- [x] Pipeline picks first available allowed source per glyph; source recorded via debug logging in skia/zeno renderers.

Phase 2 — SVG Emission from tiny-skia and zeno ✓ COMPLETE
- [x] Kurbo path builder used in skia renderer; SvgRenderer handles all vector output.
- [x] `RenderMode::Vector(VectorFormat::Svg)` supported in skia and zeno renderers via delegation to SvgRenderer.
- [x] Bbox-driven viewport sizing with y-flip transforms implemented in SvgRenderer.
- [x] CLI smoke tests verify SVG output functionality.

Phase 3 — resvg/usvg Integration for `SVG ` Table ✓ COMPLETE
- [x] `svg.rs` module extracts SVG table via skrifa, handles gzip compression, parses with usvg.
- [x] `render_svg_glyph()` renders via resvg into pixmap respecting scale.
- [x] Integrated into `render_glyph_with_preference()` when `GlyphSource::Svg` is selected.
- [x] Tests for SVG fonts (Nabla, Twitter emoji) included.

Phase 4 — COLRv0/COLRv1 Support via skrifa ✓ COMPLETE
- [x] `TinySkiaColorPainter` implements skrifa ColorPainter API with solid/gradient/transform support.
- [x] COLRv0 and COLRv1 rendering via `render_glyph_with_preference()`.
- [x] CPAL palette selection from `RenderParams::color_palette`.
- [x] Fallback to outline/bitmap per preference order.

Phase 5 — Renderer Wiring & CLI UX ✓ COMPLETE
- [x] Thread `GlyphSourcePreference` through opixa, skia, zeno, svg renderers; ensure color/svg branches are behind feature flags (`resvg` optional dep).
- [x] Debug logging shows glyph source chosen per glyph (skia/zeno backends).
- [x] CLI help with examples for `--glyph-source prefer=X deny=Y` added.
- Note: JSON renderer outputs shaping data only (not render metadata); OS renderers (cg, linra) bypass preference system by design.

Phase 6 — QA, Performance, and Documentation ✓ COMPLETE
- [x] Benchmarks: measured resvg + COLR impact (opixa ~0.3ms, skia ~1.2ms outline, ~4ms COLR, ~700ms SVG).
- [x] Fixed color glyph fallback bug: `try_color_glyph()` now returns `Ok(None)` for missing glyphs.
- [x] Run `cargo fmt`, `cargo clippy -- -D warnings`, and full `cargo test --workspace` — all pass.
- [x] Updated README.md with color glyph support tables and `--glyph-source` CLI examples.
- [x] Updated ARCHITECTURE.md with Color Glyph Rendering section and GlyphSource priority order.

Risks & Mitigations
- Resvg dependency size/perf: gate behind feature flag and use lazy initialization + cache.
- Coordinate system mismatches (y-up vs y-down, viewBox vs em square): include golden snapshots and metrics dumps in tests; centralize transforms.
- Palette/variation mismatches: add assertions comparing skrifa metrics against rendered bbox for COLR/SVG glyphs.
- Backward compatibility: default glyph-source ordering must preserve prior outline-first behavior; provide integration tests to prevent regressions.
