# Typf Roadmap — Quality Sprint (Dec 2025)

Goal: make the advertised six-stage pipeline, bidi correctness, and export/render paths reliable enough for release without adding new features. Priorities follow P0 (blocker) → P1 (high) → P2 (medium).

P0 — Correctness Blockers (must land before anything else)
- Fix SVG embedding: replace `crates/typf-export/src/svg.rs::bitmap_to_png` with png crate or `PngExporter`; validate IHDR/IDAT/IEND + CRC; add embed/external snapshot tests including Gray1 and short-buffer regressions.
- Bidi accuracy: rewrite `create_bidi_runs` in `crates/typf-unicode` to index by scalars; add mixed-script fixtures (Arabic+Latin+emoji, Hebrew+numbers, Thai marks) and property test vs `unicode-bidi::BidiInfo::reorder_line`; expose `--direction auto` in CLI.
- Pipeline truthfulness: either wire Input/Unicode/FontSelection stages or remove them from docs; route CLI through Pipeline; enforce shaper/renderer/exporter presence and capability checks.
- Font handling: remove StubFont fallback in CLI/Python; integrate typf-fontdb with TTC face index; guard font size and data length; surface clear FontLoad errors; CLI/Batch smoke tests for missing font and successful render.

P1 — Rendering, metrics, and caching
- Canvas sizing: compute width/height from ascent/descent/bbox across opixa/skia/zeno/SVG; include padding/clamping; add tall-glyph snapshots (emoji, Thai, Arabic marks) to prevent clipping.
- Glyph ID and color propagation: keep 32-bit glyph IDs through vector/export paths; preserve variations and CPAL palette selection in SVG/vector renderers.
- Shaping cache: connect SharedShapingCache to HB/ICU shapers; switch cache keys to stable font IDs (not full byte hashes); add capacity settings + hit/miss stats; benchmark cache hit rate on sample corpus.
- FontDB hygiene: drop Box::leak, honor TTC face index, bound font data size, return structured errors; verify advance width uses requested size/units.

P2 — Export, testing, CI, docs
- Exporters: add bounds checks for PNM/PNG Gray1 and short buffers; version JSON schema; snapshot tests for SVG/vector/JSON outputs.
- Test coverage: backend goldens (bitmap hashes for opixa/skia/zeno, SVG snapshots, JSON schema validation); CLI smoke for info/render/batch (success + bad input); Python parity tests for PNG/SVG/JSON once exposed.
- CI matrix: cargo fmt --all --check; cargo clippy --workspace --all-features -D warnings; cargo test --workspace across minimal/default/full feature sets (skip platform-only when unavailable).
- Documentation: write ARCHITECTURE.md explaining actual pipeline/backends; align README/CONTRIBUTING/RELEASING with font discovery, cache flags, renderer limits; note Python feature gaps.

Execution Checklist (per change set)
- Write failing test first, then minimal fix.
- Run `cargo fmt && cargo clippy -D warnings && cargo test --workspace` (or targeted subsets when platform-gated).
- Record findings and test results in WORK.md during work, then clear it at end.
- Update TODO.md checkboxes to match progress; keep NEXTTASK.md untouched.

Milestones
- M1: All P0 items fixed with tests; CLI uses Pipeline; no stub font fallback; bidi/PNG regressions covered.
- M2: Renderer sizing, cache wiring, fontdb hygiene, 32-bit glyph propagation complete with goldens.
- M3: Export/CLI/Python parity tests and CI matrix running; docs updated to match reality.
