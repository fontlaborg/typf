# Typf Improvement Plan — 1 Dec 2025

Scope (one sentence): Get the workspace compiling again, make pipeline/backends honest and correct, and deliver tag-driven releases (Cargo + hatch-vcs) with macOS build script parity and push-button publishing to crates.io and PyPI.

Principles
- Minimal fixes first: restore build + tests before refactors.
- Capability honesty: prefer explicit errors over silent fallbacks.
- One source of truth for versions: git tags → Cargo + hatch-vcs.

Critical blockers to clear first
1) Fix `typf-cli` compile break  
   - Add `typf-unicode` dependency and reintroduce `resolve_direction` helper that routes `--direction auto` through `UnicodeProcessor` with language/script hints.  
   - Re-run `cargo test --workspace` and gate on green.
2) Align version source  
   - Adopt `hatch-vcs` in `pyproject.toml` and ensure Python wheels derive version from git tags `vN.N.N` (PEP 621 `dynamic = ["version"]` + hatch config).  
   - Keep `workspace.package.version` in Cargo synced via `cargo set-version` driven by tags; remove hard-coded `"2.0.0-dev"` in Python module and CLI banners.

Feature quality and correctness
3) Pipeline truthfulness  
   - Either wire Input/Unicode/FontSelection stages using `typf-unicode` and `typf-fontdb` or document the pipeline as three stages; make CLI reuse `Pipeline::process` instead of bespoke flow.  
   - Introduce capability tables per shaper/renderer/exporter; fail early when unsupported combinations are chosen.
4) Vector/SVG fidelity  
   - Base canvas sizing on ascent/descent/bbox; propagate glyph IDs > u16, variations, and CPAL palette into SVG/Skia/Zeno; share parsed font faces to avoid per-glyph reparsing.  
   - Add snapshot tests for tall glyphs, emoji, COLR, and large glyph IDs across renderers/exporters.
5) Caching & guardrails  
   - Replace font-buffer hash keys with stable identity (path + face index + checksum), add size bounds/eviction metrics, and expose hit/miss stats.  
   - Add canvas dimension guards before allocation in all renderers/exporters.

Bindings & usability
6) Python parity and safety  
   - Reuse font handles across calls; add TTC face index, JSON/vector exporters, and optional stub font (off by default).  
   - Mirror CLI direction auto-detection; surface renderer capability errors.  
   - Add pytest coverage for TTC, SVG/PNG/JSON export, missing-font error paths.

Release, CI, and tooling (per user requirements)
7) macOS-friendly build script  
   - Trim `build.sh` to a reproducible macOS path: install Homebrew deps, run `cargo build --release --workspace`, build Python wheels via `uvx maturin build`, and avoid system-wide `uv pip --system`.  
   - Add a CI smoke job to run `./build.sh` on macOS 13/14.
8) GitHub Actions release on `vN.N.N`  
   - Keep single workflow that: sets Cargo workspace version from tag, ensures `hatch-vcs` resolves Python version, builds Rust binaries and maturin wheels for relevant targets, signs/checksums, and cuts GitHub release.  
   - Add gates so wheels are uploaded to PyPI and crates to crates.io only when tag matches semver and tests/clippy/fmt pass.
9) Publishing scripts  
   - Update `publish.sh` to read version from git tag, verify clean tree, run `cargo publish` across crates in topo order, then `uvx hatch build`/`uvx hatch publish`.  
   - Document manual steps in `RELEASING.md`, including dry-run flags and credentials.

Testing & documentation
10) Testing matrix  
    - Add clippy -D warnings, fmt, and feature matrix (minimal/default/full, skia/zeno, macOS linra when runner available).  
    - Add snapshot diffs for exporters; keep fuzz targets gated but runnable in CI nightly.  
11) Docs/logs  
    - Update README/ARCHITECTURE to reflect real pipeline behavior and supported outputs (JSON schema, SVG limits).  
    - Add versioning/release section explaining git-tag → Cargo/hatch-vcs flow; update WORK.md and CHANGELOG.md per iteration.

Order of execution
- Unblock build (1), align versions (2), then pipeline honesty (3) to avoid rework.  
- Vector/caching fixes (4–5) before Python parity (6) to reuse APIs.  
- Release/tooling work (7–9) after functional correctness; docs/tests (10–11) throughout.
