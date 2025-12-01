Typf Code Quality Review — 1 Dec 2025  
Confidence: I believe (≈90%). Tests: `cargo test --workspace --quiet` (pass at 1 Dec 2025, 13:XX local run). No code changes made; review only.

Scope & method  
- Workspace: `/Users/adam/Developer/vcs/github.fontlaborg/typf`.  
- Read README/PLAN/TODO/WORK/ARCHITECTURE/CONTRIBUTING/RELEASING, skimmed CHANGELOG.  
- Inspected core crates (`typf-core`, `typf-export`, `typf-export-svg`, `typf-fontdb`, `typf-unicode`, `typf-input`, `typf-cli`, `typf` facade), major backends (hb/icu-hb/ct shapers; opixa/skia/zeno/cg/render-color renderers; linra OS backends), Python bindings, build/publish scripts, and CI hints.  
- Focused on invariants, error typing, caching, capability signalling, performance pitfalls, API parity, and test coverage.  
- No edits; all observations are from the checked-in code as of this date.

Executive assessment  
- Architecture is modular and mostly type-safe; backends implement narrow traits. The six-stage pipeline is largely aspirational—Input/Unicode/FontSelection stages are pass-through and CLI bypasses them, so docs overstate capability.  
- Rendering/vector paths remain correctness-risky: canvas sizing based on advance not bbox, glyph IDs clamped to u16 in several exporters, variations/palette data dropped, and fonts re-parsed per glyph in SVG exporters.  
- Caching and identity are weak: font caches hash raw buffers (expensive, unstable), no size bounds or metrics, and render caches are opaque to users.  
- Capability honesty is low: `supports_*` defaults to `true`, so unsupported combinations fail late.  
- Python bindings lag: forced LTR, reload fonts per call, no TTC index or JSON/vector parity, and `render_simple` hides missing-font errors via stub font. Version string is hard-coded `2.0.0-dev`.  
- Release/tooling story is fragmented: Cargo version is 2.0.0, Python uses dynamic placeholder, scripts install system-wide deps via `uv pip --system`, and GH Actions mutates Cargo version but not hatch-vcs; no tag-to-version guardrails.

Rust workspace: crate-by-crate highlights  
- `crates/typf-core`  
  - `pipeline.rs`: `process()` enforces presence of shaper/renderer/exporter and propagates errors cleanly. Default builder injects six stages but three are no-ops; CLI uses its own flow. Lacks capability validation across backends. Tests cover happy/negative paths but no property tests for empty fonts or cache sizing.  
  - `traits.rs`: default `supports_script`/`supports_format` return `true`, masking lack of support; `FontRef::glyph_count` optional so shapers can emit out-of-range IDs unnoticed.  
  - `cache.rs`/`shaping_cache.rs`: L1/L2 caches work but key is full font buffer hash (costly, not identity-stable) with unbounded size and no metrics/eviction signals.  
  - `linra.rs`: maps linra params but ignores palette/optical size; capability detection minimal.  
  - Types and errors are crisp; docs sometimes imply more pipeline behavior than exists.  
- `crates/typf-unicode`  
  - `UnicodeProcessor::process` correctly maps byte→char indices and bidi levels; grapheme cluster results are computed then discarded. `detect_scripts` keeps `Common` sticky until first specific script; language copy-through only.  
  - Tests cover RTL/mixed scripts/segmentation; no stress tests for surrogate pairs or invalid UTF-8 (input assumed valid).  
- `crates/typf-fontdb`  
  - `Font::from_file/from_data_index` validate via `read-fonts`, honor `face_index`, and avoid leaks; error typing is coarse (`FontError`).  
  - `advance_width` normalizes to 1000 UPM, diverging from actual UPM and can distort metrics for fonts with non-1000 UPM; no size cap on input data.  
- `crates/typf-export`  
  - `png.rs` validates lengths for RGBA/RGB/Gray8/Gray1; Gray1 bit-walk is bounds-checked. Uses `image` encoder; no streaming/stride options.  
  - `svg.rs` (embed PNG) re-parses bitmap and uses bespoke base64 encoder; maintenance risk vs `base64` crate. Padding handled; no metadata or viewBox control.  
  - `json.rs` emits unversioned schema; not exercised by CLI.  
  - `pnm` exporters are correct but slow (per-pixel expansion).  
- `crates/typf-export-svg`  
  - `SvgExporter::export` re-parses font for every glyph (`extract_glyph_path`), sizes canvas from advance height with fixed padding, clamps glyph IDs to `u16`, ignores variations and CPAL palettes; tall glyphs/emoji/large glyph IDs can clip or be wrong. Tests are thin snapshots.  
- `crates/typf-cli`  
  - CLI now restores direction auto-detect via `typf-unicode`; error messages are clear. Still hand-assembles pipeline instead of `Pipeline::process`, so the six-stage contract is unused.  
  - Batch runner counts per-job errors but doesn’t validate font existence per job; JSON renderer deliberately unsupported. SVG fallback wiring is explicit. Logging UX is good.  
- `crates/typf` facade  
  - Re-exports backends and WASM mock font; docs claim fully pluggable pipeline, but facade defaults to None+Opixa for WASM. Limited tests.  
- `crates/typf-input`  
  - Placeholder `add()` only; no integration. Dead weight until wired or removed.  
- `crates/typf-bench`  
  - Bench harness depends on local fonts; no guards for missing files; not wired into CI.  

Backends  
- Shapers  
  - `typf-shape-hb`/`icu-hb`: solid direction/language/feature handling; shaping cache integration exists but cache keys share global font-buffer hash and no eviction signals. Tests include mixed scripts/features.  
  - `typf-shape-ct`: uses CoreText; limited tests and no feature coverage; capability detection coarse.  
  - `typf-shape-none`: deterministic passthrough; safe for debug.  
- Renderers  
  - `typf-render-opixa`: robust bounds checks, supports Gray1/antialias; allocates per-call RGBA buffers (no pooling).  
  - `typf-render-skia`/`typf-render-zeno`: surface sizing by advance not bbox; large glyph IDs and CPAL/variations dropped; shares code paths with SVG export but re-parses fonts. Tests mostly smoke.  
  - `typf-render-cg` and `typf-render-color`: handle COLRv0/v1, sbix, CBDT/SVG; rely on platform availability; tests are ignored on non-mac/Windows targets.  
  - `typf-render-json`: emits glyph lists without schema validation; not used by CLI.  
- Linra (OS single-pass)  
  - `typf-os-mac`: CTLineDraw path with AA/letter spacing; capability checks shallow; no palette/variation support.  
  - `typf-os-win`: mostly stubbed when not on Windows; risk of divergence when enabled.  

Python bindings (`bindings/python/src/lib.rs`)  
- `Typf` constructor selects shaper/renderer but reloads fonts every call and forces `Direction::LeftToRight`; no TTC index, no feature or variation plumbing.  
- `render_text` returns dict with RGBA8 only; no format negotiation.  
- `render_to_svg` gated behind `export-svg`, re-parses font per glyph, ignores palette/variation.  
- `render_simple` fabricates a stub font, masking missing-font errors and producing untrue metrics.  
- Version exposed as `"2.0.0-dev"` constant; not tied to workspace version or git tags.  

Tooling / release / docs  
- `build.sh` installs Python deps twice (venv + `--system`), references `typf-py` (nonexistent), always runs docs/tests/benchmarks, and assumes Homebrew fonts; not reproducible or CI-friendly.  
- `publish.sh` sets Cargo version from tag but does not ensure Python version matches; assumes clean `main` and pushes crates/wheels without topo ordering or dry-run guardrails.  
- `.github/workflows/release.yml` builds binaries and maturin wheels on `v*` tags, mutates Cargo version, but Python version remains dynamic placeholder (no hatch-vcs). No check that tag matches workspace version.  
- Docs (README/ARCHITECTURE) over-promise six-stage pipeline and SVG/vector fidelity; capability tables lack per-backend caveats.  

Quality risks (ranked)  
1) Capability honesty: default-true `supports_*` and pass-through pipeline hide unsupported combos until late failure.  
2) Vector/SVG correctness: bbox-free sizing, glyph ID truncation, palette/variation loss, per-glyph font reparse → clipping and wrong output.  
3) Caching/identity: hashing font buffers without bounds or metrics risks memory blowups and poor perf; no eviction visibility.  
4) Python parity/safety: forced LTR, per-call font reload, stub font masking errors, no TTC/JSON/vector parity; hard-coded version.  
5) Versioning/release: Cargo/Python/tag mismatch; build/publish scripts non-reproducible; GH Actions lacks guards.  
6) Dead/placeholder code: `typf-input` unused; bench harness depends on local fonts; documentation drifts from reality.  

Opportunities / quick wins  
- Add capability matrices + early validation in CLI and `Pipeline::process`.  
- Switch font identity keys to (path, face index, checksum) with size bounds + metrics; expose hit/miss counters.  
- Bbox-based canvas sizing and 32-bit glyph IDs across renderers/exporters; share parsed font handles to avoid reparse.  
- Python: reuse font handles, allow TTC index, honor direction auto-detect, remove stub default, add JSON/SVG/PNG parity, and source version from git tag via hatch-vcs.  
- Release: single source of truth (git tag -> Cargo workspace version -> hatch-vcs); `build.sh` minimal macOS recipe; `publish.sh` dry-run + topo publish; CI smoke for macOS build script.  
- Documentation: align pipeline description with actual flow; state capability gaps per backend; add JSON schema versioning.  

Second-pass self-check  
- Re-read modules to avoid overclaim; limited to observed code paths.  
- Risks above map to concrete code lines (pipeline pass-through, SVG glyph clamp, cache key).  
- Tests are currently green; noted missing coverage rather than assuming failure.  
- No refactoring proposed here—plan moves to PLAN.md for action.  
