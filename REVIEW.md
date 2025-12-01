Typf Code Quality Review — 1 Dec 2025  
Confidence: I believe (≈80%). Tests: `cargo test --workspace --quiet` fails at `crates/typf-cli` compile (unresolved `typf_unicode` import and missing `resolve_direction`), so runtime status is unknown.

Scope & method  
- Working dir: `/Users/adam/Developer/vcs/github.fontlaborg/typf`.  
- Read README/PLAN/TODO/WORK/ARCHITECTURE, inspected Rust crates, backends, Python bindings, scripts (`build.sh`, `publish.sh`, `.github/workflows`).  
- Reviewed function and type definitions with emphasis on invariants, error handling, feature gating, and doc/test alignment. No code changes made.

Executive take  
- Architecture is coherent and modular; most crates have sensible error types and unit tests. However the current workspace does not compile because `typf-cli` references an undeclared `typf_unicode` dependency and a missing helper (`resolve_direction`).  
- Early pipeline stages remain pass-through; CLI manually orchestrates shaping/render/export instead of the six-stage pipeline. Capability flags default to permissive “supports everything,” so unsupported combinations surface late.  
- Vector/SVG paths still size canvases heuristically and drop variation/palette/glyph>u16 data; caches hash full font buffers and expose no metrics.  
- Python bindings reload fonts per call, lack TTC index and JSON/vector parity, and expose a stub font helper that masks missing-font errors.  
- Release/version story is fragmented: Cargo workspaces set version to 2.0.0, Python uses `dynamic = ["version"]` without hatch-vcs, and scripts/workflows perform ad-hoc `cargo set-version`.

Immediate blockers  
- `crates/typf-cli/src/commands/render.rs` fails to compile: `use typf_unicode::{UnicodeOptions, UnicodeProcessor};` lacks a dependency entry, and helper `resolve_direction` is not defined. The test run aborts at compile time.  
- Version sources disagree (Cargo 2.0.0, Python dynamic but unset, CLI uses `env!("CARGO_PKG_VERSION")`), so tag-based releases cannot be trusted yet.

Rust workspace review (by crate/module)  
- `crates/typf-core`:  
  - `pipeline.rs::process/execute` correctly enforces presence of shaper/renderer/exporter and propagates errors, but Input/Unicode/FontSelection stages are no-ops so the “six-stage” story over-promises.  
  - `traits::{Shaper, Renderer, Exporter}` expose `supports_*` defaults returning true; no capability matrix means unsupported combos fail downstream.  
  - `cache.rs` and `shaping_cache.rs` provide L1/L2 caches with promotion; keys hash full font buffers (expensive, unbounded, not identity-stable) and do not report metrics or eviction.  
  - `linra.rs` maps linra params to shaping/render params but ignores palette/optical size; reasonable defaults otherwise.  
  - Tests cover builder wiring and stage ordering; no property tests for cache key stability or size guardrails.  
- `crates/typf-unicode`:  
  - `UnicodeProcessor::process` correctly converts byte → char indices for bidi levels; options enable script detection, normalization, bidi resolve. Grapheme detection is computed then discarded (untapped).  
  - `detect_scripts` treats `Script::Common` as sticky until a specific script appears; language is copied verbatim (empty by default).  
  - Word/line segmentation via ICU; no locale override beyond `language`. Property/unit tests cover RTL, mixed scripts, segmentation.  
- `crates/typf-fontdb`:  
  - `Font::from_file/from_data_index` validates via `read-fonts`, honors `face_index`, and avoids leaks. Error typing is coarse (`FontError`) but safe; no size/format caps on loaded data.  
  - `advance_width` normalizes widths to 1000 UPM, which may diverge from actual requested size; consistent but surprising.  
- `crates/typf-export`:  
  - `png.rs::encode_bitmap_to_png` validates buffer length and handles RGBA/RGB/Gray8/Gray1; Gray1 bit-walking is bounds-guarded. Uses `image` encoder; fast-path absent but correctness solid.  
  - `svg.rs::SvgExporter` embeds PNG via custom base64 encoder; maintains padding and color but re-parses bitmap; bespoke base64 is maintenance risk vs `base64` crate.  
  - `json.rs` serializes shaping output without schema/version tagging; no CLI path exercises it.  
  - `pnm` exporter expands per-pixel; slow but correct for debug.  
- `crates/typf-export-svg`:  
  - `SvgExporter::export` re-parses font per glyph (`extract_glyph_path`), sizes canvas from advance height with fixed padding, and clamps glyph IDs to u16, dropping variation/palette data. Tall ascenders/emoji can clip; no bbox-based sizing. Tests are minimal snapshots.  
- `crates/typf-cli`:  
  - CLI flow hand-builds pipeline; resolves direction via missing helper; attempts to use `UnicodeProcessor` but dependency absent. Shaper/renderer selection is clear with feature gating, and SVG fallback to `SvgRenderer` is explicit.  
  - Batch runner counts per-job errors but does not validate font presence per job; JSON output intentionally unsupported.  
  - Logging and progress UX are good; compile break currently blocks use.  
- `crates/typf`: facade re-exports backends and WASM mock font. Docs describe fully pluggable pipeline that does not exist in CLI; WASM build exposes only None+Opixa.  
- `crates/typf-input`: placeholder `add()` and trivial test; not wired anywhere.  
- `crates/typf-bench`: benches wire HB+Opixa; rely on local fonts; not part of failed build.  

Backends  
- Shapers:  
  - `typf-shape-hb`/`typf-shape-icu-hb` honor direction, features, variations; integrate shaping cache; coverage/axis handling is solid.  
  - `typf-shape-ct` maps CoreText coverage and supports scripts by default; limited tests.  
  - `typf-shape-none` deterministic passthrough for debugging; safe.  
- Renderers:  
  - `typf-render-opixa` is well-tested; clamps canvas bounds and supports Gray1/antialias modes; allocates per-glyph RGBA buffers (could pool).  
  - `typf-render-skia` and `typf-render-zeno` support vector output but size surfaces from advances, not bbox; glyph>u16 and palette/variation data dropped.  
  - `typf-render-cg` and `typf-render-color` handle COLRv0/v1, sbix, CBDT/SVG with detection; smoke tests only.  
  - `typf-render-json` emits glyph lists without schema; unchecked by CLI.  
- OS linra renderers (`typf-os-mac`, `typf-os-win`) provide combined shape+render APIs with basic capability tests; DirectWrite path stubbed on non-Windows targets.  

Python bindings (`bindings/python/src/lib.rs`)  
- `Typf` class exposes shaper/renderer selection and bitmap export; each call reloads fonts and forces LTR direction. No TTC index, no JSON/vector parity, and no shared font handles.  
- `render_simple` uses a stub font that fabricates glyph metrics, potentially hiding missing-font errors.  
- `render_to_svg` available only with `export-svg` feature; still re-parses font and ignores variations/palettes.  
- Module version is hard-coded to `"2.0.0-dev"`; not sourced from git tags or Cargo.  

Tooling, scripts, and release flow  
- `build.sh` is macOS-aware but installs both venv and system packages with `uv pip`; runs heavy docs/tests/benchmarks unconditionally and references `typf-py` (nonexistent). No validation of required Homebrew deps.  
- `publish.sh` relies on `cargo set-version` output and `uv publish`; no verification that Python/Rust versions match git tag; assumes clean `main`.  
- `.github/workflows/release.yml` triggers on `v*` and builds Rust binaries plus maturin wheels. Version is injected via `cargo set-version` during workflow but Python still uses a dynamic placeholder; no hatch-vcs or PEP 621 `version` source, so PyPI wheel version may drift.  
- No CI check ensures `build.sh` works on macOS; no release dry-run combining crates.io + PyPI; no guard that `vN.N.N` tags map to semver in Cargo/Python.  

Quality risks and opportunities  
- Broken workspace compile prevents test execution; need to restore `typf-cli` dependency wiring and direction resolver.  
- Capability honesty: default-true `supports_*` and pass-through pipeline stages hide missing features until runtime; add explicit tables and early errors.  
- Vector/SVG correctness: bbox-based sizing, glyph>u16, variation/palette propagation, and shared font parsing are needed to avoid clipping and data loss.  
- Caching/observability: adopt stable font identity keys, bounds, and metrics.  
- Python parity: reuse fonts, add TTC index and JSON/vector exporters, remove stub default.  
- Versioning/release: unify version source from git tags via hatch-vcs (Python) and workspace metadata; align `build.sh`, `publish.sh`, and GH Actions with semver tag triggers.  

Second pass (self-check and refinements)  
- Re-read key modules for overstatement: confined claims to observed code paths; noted where tests are lacking.  
- Recorded the exact test failure and compile break instead of assuming previous green state.  
- Highlighted version-source drift and release automation gaps per the new requirements.  
- Confirmed no code changes were made.  
