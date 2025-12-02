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

Phase 1 — Core API for Glyph Source Selection
- [x] Add a `GlyphSourcePreference` struct (ordered allowlist + deny set) in `typf-core` and thread it through `RenderParams` and the CLI flags (`--glyph-source prefer=A,B --glyph-source deny=COLR,SVG`).
- [x] Define canonical source enum values: glyf, CFF, CFF2, COLR0, COLR1, SBIX, CBDT, EBDT, SVG; set default ordering (vector outlines first, then color, then bitmaps) matching current behavior.
- Ensure pipeline picks the first available allowed source per glyph and records which source was used (for debugging/JSON output).

Phase 2 — SVG Emission from tiny-skia and zeno
- Factor shared outline-to-curve abstraction (kurbo path builder) so both raster and vector outputs consume the same geometry.
- Add `RenderMode::Vector(VectorFormat::Svg)` paths to the skia and zeno renderers that bypass rasterization and emit SVG path data (reusing `typf-render-svg` writer where possible).
- Implement bbox-driven viewport sizing identical to raster paths; include transforms (y-flip) and variable font locations.
- Add snapshot tests comparing skia/zeno SVG output to `typf-render-svg` for reference strings and tall/complex glyph cases.

Phase 3 — resvg/usvg Integration for `SVG ` Table
- Introduce a small adapter crate/module that extracts `SVG ` table fragments via skrifa, feeds them into usvg for parsing, and hands resvg a render tree.
- Expose a renderer hook: when `SVG` source is selected for a glyph, rasterize via resvg into the current backend surface (pixmap for skia/zeno/opixa) respecting scale, baseline, and palette.
- Cache parsed usvg trees per glyph to avoid repeated XML parsing across runs.
- Add tests using fonts with `SVG ` glyphs to validate placement, bounding boxes, and fallback when SVG is denied or missing.

Phase 4 — COLRv0/COLRv1 Support via skrifa
- Use skrifa’s ColorPainter/OutlinePaint to traverse COLR layers/paints and map them to tiny-skia/zeno paint primitives (solids, linear/radial gradients, transforms).
- Support CPAL palette selection from `RenderParams` and default palette 0; honor user opt-out through glyph-source preferences.
- Validate fallback: if COLR not allowed, fall back to outline source or bitmap per preference order.
- Add regression tests for COLRv0 layered glyphs and COLRv1 gradients/affine transforms (e.g., Noto Color Emoji, test fonts in repo).

Phase 5 — Renderer Wiring & CLI UX
- [x] Thread `GlyphSourcePreference` through opixa, skia, zeno, svg, and json renderers; ensure color/svg branches are behind feature flags (`resvg` optional dep).
- Extend CLI help and examples showing how to prefer outlines, disallow color, or force bitmap sources.
- Add logging/metrics hooks to surface which source was chosen per glyph when `--verbose` or JSON output is enabled.

Phase 6 — QA, Performance, and Documentation
- Benchmarks: measure impact of resvg + COLR on render latency in `typf-bench` with representative strings; set guardrails for acceptable regressions.
- Run `cargo fmt`, `cargo clippy -- -D warnings`, and full `cargo test --workspace --all-features` including new snapshot tests.
- Update `README.md`, `ARCHITECTURE.md`, `TODO.md`, and `WORK.md` with new capabilities, defaults, and flag descriptions.
- Add troubleshooting notes for missing system SVG/COLR support and feature-flag build errors.

Risks & Mitigations
- Resvg dependency size/perf: gate behind feature flag and use lazy initialization + cache.
- Coordinate system mismatches (y-up vs y-down, viewBox vs em square): include golden snapshots and metrics dumps in tests; centralize transforms.
- Palette/variation mismatches: add assertions comparing skrifa metrics against rendered bbox for COLR/SVG glyphs.
- Backward compatibility: default glyph-source ordering must preserve prior outline-first behavior; provide integration tests to prevent regressions.
