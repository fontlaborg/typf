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
- [x] JSON schema version in typf-render-json output
- [x] Deprecate render_simple with warning (stub font masks errors)
- [x] Remove trivial add() placeholder from typf-cli lib.rs
- [x] Add workspace-level lint configuration
- [x] Fix unwrap() in L2Cache with const fallback

---

## Pending

### Phase 1: Security & Reliability

- [ ] Add fuzz target for skrifa font parsing (fuzz/fuzz_targets/fuzz_font_parse.rs)
- [ ] Add fuzz target for read-fonts table parsing
- [ ] Add corpus of malformed font files (fuzz/corpus/malformed/)
- [ ] Integrate font fuzzing with CI (cargo-fuzz)
- [ ] Add font size limits to prevent DoS (max 10000px)
- [ ] Add glyph count limits for rendering (max 100K glyphs)
- [ ] Improve dimension validation error messages
- [ ] Add timeout configuration for font operations
- [ ] Add configurable memory limits for font loading
- [ ] Add operation timeout configuration to RenderParams
- [ ] Document resource limit configuration in README

### Phase 2: Windows Backend Completion

- [ ] Audit feature gaps: typf-os-win vs typf-os-mac
- [ ] Add missing DirectWrite features (variable fonts, color glyphs)
- [ ] Add Windows CI runner to GitHub Actions
- [ ] Add Windows-specific documentation
- [ ] Evaluate need for Direct2D standalone renderer
- [ ] Implement typf-render-win if valuable for non-linra use cases

### Phase 3: Testing Infrastructure

- [ ] Add image comparison library (pixelmatch-rs or similar)
- [ ] Generate golden images for all test fonts
- [ ] Add visual diff CI step
- [ ] Add script rendering tests (Arabic, CJK, Devanagari)
- [ ] Add Windows CI runner (GitHub Actions)
- [ ] Add Linux CI runner
- [ ] Verify macOS CI runner configuration
- [ ] Test all backends on each platform
- [ ] Add criterion benchmark baselines
- [ ] Add CI step to detect performance regressions (>10% slowdown)
- [ ] Document performance expectations per backend

### Phase 4: Developer Experience

- [ ] Add API stability markers (stable/experimental) to public items
- [ ] Add backend development guide (CONTRIBUTING_BACKENDS.md)
- [ ] Add troubleshooting FAQ
- [ ] Add migration guide template
- [ ] Audit feature flag dependencies
- [ ] Remove unused feature combinations
- [ ] Document recommended feature sets
- [ ] Consider feature flag presets (minimal, standard, full)
- [ ] Add optional tracing integration
- [ ] Add structured logging option
- [ ] Add metrics export option (prometheus-compatible)

### Backlog (Low Priority)

- [ ] Memory profiling integration (heaptrack)
- [ ] Third-party extension examples
- [ ] WASM target improvements
- [ ] PDF export backend
