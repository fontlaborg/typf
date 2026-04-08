# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is typf

Typf is a modular text shaping and rendering engine. It takes a font file, a string, and parameters, then produces shaped, rasterized text as pixels or vector paths. The pipeline is: **Shaping** (text to positioned glyphs) **-> Rendering** (glyphs to pixels/vectors) **-> Export** (pixels to file format). There are 5 shapers x 7 renderers = 35 backend combinations, plus "linra" single-pass OS backends that bypass the intermediate step.

Made by [FontLab](https://www.fontlab.org/). Current version: 5.0.15. MSRV: Rust 1.75.

## Build & test commands

```bash
# Fast local checks (Rust only)
cargo fmt --check
cargo clippy --workspace --all-features -- -D warnings
cargo test --workspace                    # quick: default features
cargo test --workspace --all-features     # thorough: all features

# Full test suite (Rust + Python + linting)
./test.sh                    # delegates to scripts/test.sh
./test.sh --quick            # skip slow tests
./test.sh --rust             # Rust only
./test.sh --python           # Python only
./test.sh --lint             # linting only (no tests)

# Single crate / single test
cargo test --package typf-core
cargo test --package typf-core test_name

# Run examples (from repo root)
cargo run --example basic
cargo run --example harfbuzz --features shaping-hb

# Python bindings (from bindings/py/)
cd bindings/py && uv run --isolated --with pytest pytest tests/ -v

# Benchmarks
typf-bench -i test-fonts -l 0    # quick (~10s)
typf-bench -i test-fonts -l 1    # thorough

# Fuzz targets (4 targets, requires cargo-fuzz)
cargo fuzz run fuzz_unicode_process
cargo fuzz run fuzz_harfbuzz_shape
cargo fuzz run fuzz_pipeline
cargo fuzz run fuzz_font_parse
```

## Releasing

Version source of truth is the latest git tag (`vN.N.N`). The publish script stamps it into all Cargo.toml and pyproject.toml files.

```bash
gitnextver              # bump tag: v5.0.14 -> v5.0.15, pushes to remote
./publish.sh            # sync version into manifests, publish to crates.io + PyPI
./publish.sh --dry-run  # preview without uploading
./publish.sh sync       # stamp version into files without publishing
```

Pushing a tag triggers CI to build 8 Rust binaries + 40+ Python wheels and publish to crates.io (in dependency order) and PyPI.

## Workspace layout

```
typf/
  core/           -> typf-core       Core types, traits (Shaper/Renderer/Exporter/FontRef),
                                     pipeline, caching (Moka TinyLFU), FFI, error types
  main/           -> typf            Main library crate, re-exports + feature-flag wiring
  cli/            -> typf-cli        CLI binary (clap). Binary name: `typf`
  fontdb/         -> typf-fontdb     Font loading, TTC face index, metrics
  unicode/        -> typf-unicode    Bidi, script detection, segmentation
  input/          -> typf-input      Input parsing
  export/         -> typf-export     PNM/PNG export
  export-svg/     -> typf-export-svg SVG export
  backends/
    typf-shape-none/      Minimal LTR-only shaper (all platforms)
    typf-shape-hb/        HarfBuzz C via harfbuzz-rs (all platforms)
    typf-shape-hr/        Pure Rust HarfBuzz via harfrust (all platforms)
    typf-shape-icu-hb/    ICU normalization + HarfBuzz (all platforms)
    typf-shape-ct/        CoreText native shaper (macOS only)
    typf-render-opixa/    Pure Rust rasterizer with SIMD (all platforms)
    typf-render-skia/     tiny-skia renderer, color fonts (all platforms)
    typf-render-zeno/     Pure Rust zeno rasterizer, color fonts (all platforms)
    typf-render-vello/    GPU renderer via Vello (GPU required)
    typf-render-vello-cpu/ CPU Vello renderer (all platforms)
    typf-render-cg/       CoreGraphics native renderer (macOS only)
    typf-render-json/     JSON data exporter (all platforms)
    typf-render-svg/      SVG vector output
    typf-render-color/    Color glyph support (COLR/CPAL/SVG/bitmap)
    typf-os/              Linra OS backend trait
    typf-os-mac/          CoreText linra single-pass (macOS, 2.5x faster)
    typf-os-win/          DirectWrite backend (Windows)
  bindings/py/    -> typf-py         PyO3 Python bindings (cdylib)
  tools/typf-bench/ -> typf-bench    Benchmark framework
  fuzz/                              4 fuzz targets (excluded from workspace)
  external/                          Vendored deps like vello (excluded from workspace)
  examples/                          Runnable examples (basic, harfbuzz, pipeline, etc.)
```

## Architecture

### Pipeline model

`TextPipeline` chains: Shaper -> Renderer -> Exporter. Each stage is a trait in `typf-core`:

- **`Shaper`** trait: `shape(text, font, params) -> ShapingResult` (positioned glyphs)
- **`Renderer`** trait: `render(shaping_result, font, params) -> RenderOutput` (bitmap/vector)
- **`Exporter`** trait: `export(render_output) -> Vec<u8>` (file bytes)
- **`FontRef`** trait: abstraction over font data access (data bytes, metrics, glyph count, variation axes)

All processing traits require `Send + Sync` for concurrent pipeline execution.

### Caching

Uses Moka TinyLFU (scan-resistant). Two cache layers:
- **ShapingCache**: keyed on text + font hash + size + features + variations + language + script
- **GlyphCache**: keyed on font + glyph ID + size + render params (byte-weighted)

Caching is **disabled by default**. Enable via `cache_config::set_caching_enabled(true)`.

### Linra (single-pass)

`LinraRenderer` trait in `core/linra.rs` combines shaping + rendering in one OS call, bypassing the intermediate ShapingResult. Only implemented for macOS CoreText (`typf-os-mac`).

### FFI layer

`core/ffi.rs` exports C ABI types with `#[repr(C)]`, compile-time layout assertions, and `// SAFETY:` comments on every unsafe block. This is the reference standard for unsafe code in the project.

### Error handling

All operations return `Result<T, TypfError>`. Error types use `thiserror` with `#[from]` conversions. Hierarchy: `TypfError` -> `FontError | ShapingError | RenderError | ExportError | ConfigError`.

## Feature flags

The `typf` (main) crate gates backends behind feature flags:

- **`minimal`** = `shaping-none` + `render-opixa` (smallest build, ~500KB)
- **`default`** = `minimal` + `unicode` + `fontdb` + `export-pnm`
- **`full`** = all shapers + renderers + exports
- Shapers: `shaping-none`, `shaping-hb`, `shaping-icu-hb`, `shaping-ct` (alias: `shaping-mac`), `shaping-win`
- Renderers: `render-opixa`, `render-skia`, `render-zeno`, `render-vello-cpu`, `render-vello`, `render-cg` (alias: `render-mac`), `render-json`
- Exports: `export-pnm`, `export-png`, `export-svg`, `export-pdf`
- Other: `unicode`, `fontdb`, `simd`, `parallel`, `auto-backend`, `wasm`

macOS builds add `shaping-mac` + `render-mac`. The CLI crate has its own parallel feature set plus `linra-mac`/`linra-win` and `repl`.

## Workspace conventions

- **Workspace version**: single source in `[workspace.package]` of root Cargo.toml (currently 5.0.15), all crates inherit via `version.workspace = true`
- **Workspace deps**: centralized in root `[workspace.dependencies]` to prevent version drift
- **Lints**: `unwrap_used = "deny"`, `expect_used = "warn"`, `panic = "warn"` in workspace clippy lints
- **Formatting**: `rustfmt.toml` — edition 2021, max_width 100, 4-space indent, Unix newlines
- **License checking**: `deny.toml` with `cargo-deny` in CI
- **`this_file:` header**: every source file has a `this_file` comment near the top with its path relative to project root

## Platform-specific notes

- macOS: CoreText shaper (`typf-shape-ct`) and CoreGraphics renderer (`typf-render-cg`) use `objc2` crates. Thread-local storage for OS handles.
- Windows: DirectWrite backend (`typf-os-win`) — variable font support is incomplete (marked TODO).
- WASM: `typf/src/wasm.rs` exists but uses MockFont — non-functional for real fonts currently.
- ARM: NEON SIMD path in `typf-render-opixa/src/simd.rs` is incomplete (TODO) — falls back to scalar.

## Key files for understanding the codebase

- `core/src/traits.rs` — the 4 core traits that define the backend plugin system
- `core/src/pipeline.rs` — `TextPipeline` builder and 6-stage execution
- `core/src/lib.rs` — `ShapingParams`, `RenderParams`, `GlyphSourcePreference`, constants
- `core/src/ffi.rs` — C ABI, GPU mesh types, the gold standard for unsafe code
- `core/src/cache.rs` + `shaping_cache.rs` + `glyph_cache.rs` — caching system
- `main/src/lib.rs` — feature-flag wiring, public API surface
- `cli/src/main.rs` + `cli/src/commands/render.rs` — CLI entry point and main render command

## Commit style

```
type(scope): what changed
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`. Scope is the crate or area (e.g., `core`, `export`, `cli`).

## Known technical debt

See REVIEW.md (detailed code quality audit) and TASKS.md (actionable improvement plan). Key areas:
- Cache module has duplicate key types and vestigial L1/L2 naming from pre-Moka migration
- Dead code modules in CLI (`batch.rs`, `jsonl.rs`, `repl.rs`)
- SVG exporters silently swallow write errors (`let _ = write!(...)`)
- Doc/code value mismatches in `core/src/lib.rs` for bitmap dimension constants
