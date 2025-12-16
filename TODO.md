<!-- this_file: TODO.md -->
# TODO (derived from PLAN.md)

**Version:** 5.0.1  
**Updated:** 2025-12-16  
**Source:** `PLANSTEPS/` (split from `PLAN.md`)

- [x] Keep `PLAN.md` as an index (TLDR + links) and keep `PLANSTEPS/` authoritative
- [x] Add/maintain a single flat backlog here; avoid nested lists
- [x] Add `DEPENDENCIES.md` (major dependencies + rationale)
- [x] Align Cargo workspace versioning with git tags/docs (v5.0.1)

- [x] Baseline standardization: inventory baseline math for Opixa/Skia/Zeno/Vello(-cpu) and document deltas vs CoreGraphics
- [x] Baseline standardization: decide on one baseline contract (font metrics vs per-glyph bounds) and write it down
- [x] Baseline standardization: implement the chosen contract consistently across renderers (or document why not)
- [x] Baseline standardization: add regression tests that compare baseline placement across at least 2 renderers using shared fixtures

- [x] Vello-GPU color fonts: confirm current behavior (blank output) and ensure CLI/docs steer users to `vello-cpu` for color fonts
- [x] Vello-GPU color fonts: add a clear runtime warning/error when `vello` is selected and the chosen glyph source is bitmap/COLR (avoid silent blank renders)
- [x] Vello-GPU color fonts: track upstream status (issue link + minimal reproduction) and periodically re-test

- [x] Glyph source model: confirm `GlyphSource` covers `Glyf`, `Cff`, `Cff2`, `Colr0`, `Colr1`, `Svg`, `Sbix`, `Cbdt`, `Ebdt`
- [x] Glyph source selection: ensure `typf-render-color` tries sources in `GlyphSourcePreference` priority order with correct fallback
- [x] Bitmap sources: ensure availability checks never depend on outline presence (`sbix`/`CBDT`/`EBDT` often have empty outlines)
- [x] Bitmap decoding: centralize and harden decoding in `typf-render-color/src/bitmap.rs` (sbix PNG, CBDT/EBDT formats)
- [x] SVG renderer: add/verify opt-in bitmap embedding so color glyphs can be represented in SVG output when paths are not available
- [x] SVG renderer: when bitmap embedding disabled, define behavior for color glyphs (outline fallback vs placeholder) and test it
- [x] Skia/Zeno: verify they delegate complex glyph composition to `typf-render-color` (no duplicated parsing/compositing logic)
- [ ] Color fixtures: expand regression coverage for COLR(v0/v1), SVG, sbix, CBDT with known-problem glyphs (cutoffs, padding, flips)

- [ ] Stage 4 (shaping) contract: define a stable shaped-glyph output contract for zero-copy consumers (layout + FFI)
- [ ] Stage 4 output: define a C-ABI-safe glyph struct (repr(C), alignment, no padding surprises) and conversion from internal glyphs
- [ ] Stage 4 output: add a “decoupled glyph iterator” API so layout engines can consume shaping output without owning the pipeline

- [ ] Stage 5 (rendering) contract: define an optional geometry output path (mesh/path ops) for GPU pipelines and vector consumers
- [ ] Stage 5 output: define a `RenderMesh`/vertex ABI suitable for zero-copy upload (repr(C) + `zerocopy` markers)
- [ ] Stage 5 output: define a minimal path-op iterator API for external tessellators (avoid new frameworks)

- [x] Font bytes access: ensure `FontRef` exposes raw font bytes without copies for downstream libraries (Stage 3 interop)
- [x] Font metadata access: expose a minimal, stable font-metrics API surface for consumers (ascent/descent/units_per_em, etc.)

- [ ] Python FFI: expose a zero-copy shaped-glyph view for Pycairo-style consumers (buffer protocol / NumPy view)
- [ ] Python FFI: expose vector path primitives for ReportLab-style consumers (minimal command list API)
- [ ] Python API: expose font metrics/variations metadata for tooling (fontTools auditing, layout decisions)

- [ ] Rust integration: validate `typf` shaping → `cosmic-text`/`parley` handoff feasibility and document a supported integration boundary
- [ ] Rust integration: add at least one real example that demonstrates consuming shaped glyphs from `typf` in a layout engine
- [ ] WGPU integration: prototype a zero-copy mesh upload path (types + example) without committing to a full GPU framework

- [ ] SDF: decide whether SDF is in-scope; if yes, define minimal types + one CPU renderer crate (`typf-render-sdf`)
- [ ] SDF: implement outline→SDF generation with a constrained API and add correctness tests on a small fixture set

- [ ] Platform support: define a minimal test matrix for vello-gpu across Linux/Windows and capture results in docs
- [ ] Platform support: evaluate WASM/WebGPU constraints and explicitly document what is and is not supported

- [x] Verification: run full workspace tests + clippy and record results in `WORK.md`
