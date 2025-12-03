# TODO

## Completed

- [x] Phase 1: Core API for Glyph Source Selection
- [x] Phase 2: SVG Emission from tiny-skia and zeno (via SvgRenderer delegation)
- [x] Phase 3: resvg/usvg Integration for SVG table
- [x] Phase 4: COLRv0/COLRv1 Support via skrifa
- [x] Phase 5: Renderer Wiring & CLI UX (GlyphSourcePreference, logging, CLI help)
- [x] Remove unused objc2/objc2-foundation dependencies from typf-shape-ct
- [x] Fix clippy warnings across workspace
- [x] Phase 6: QA, Performance, and Documentation
  - [x] Benchmarks in typf-bench with resvg + COLR impact
  - [x] Fix color glyph fallback bug in skia/zeno renderers
  - [x] Update README.md with new capabilities (glyph-source flags, color support)
  - [x] Update ARCHITECTURE.md with glyph source flow

## Quality Improvements (Dec 3, 2025)

- [x] Python bindings: direction auto-detect (was forced LTR)
- [x] Python bindings: workspace version instead of hard-coded "2.0.0-dev"
- [x] Trait defaults: `supports_script`/`supports_format` now return `false` (capability honesty)
- [x] Fix glyph ID truncation in typf-export-svg (u16 → u32)
- [x] Python bindings: TTC face index support

## Pending

(No pending tasks)
