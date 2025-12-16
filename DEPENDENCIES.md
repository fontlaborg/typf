<!-- this_file: DEPENDENCIES.md -->
# Dependencies

This project has a deliberately broad dependency surface because it targets multiple shaping/rendering backends across platforms. This document records the **major external dependencies** and why they exist.

Source of truth for versions: `Cargo.toml` (`[workspace.dependencies]`) and individual crate `Cargo.toml` files.

## Rust (workspace)

### Font parsing + font tables

- `skrifa`, `read-fonts`: Font parsing, metrics, outlines, and OpenType table access (including color tables).

### Text shaping

- `harfbuzz_rs`: HarfBuzz shaping backend (`typf-shape-hb`).
- `harfrust`: Rust shaping experiments / alternative backend (`typf-shape-hr`).
- `icu`, `icu_properties`, `icu_segmenter`: Unicode normalization/segmentation for `icu-hb` shaping.

### Rendering + geometry

- `kurbo`: Curve/path geometry primitives used across renderers.
- `tiny-skia`: CPU rasterization used directly and as an engine in `typf-render-color`.
- `resvg`, `usvg`: OpenType-SVG glyph rendering (`typf-render-color` SVG feature).
- `png`: PNG decode/encode for bitmap glyphs and exports.

### GPU / hybrid rendering (vendored)

- `wgpu`: GPU abstraction for the `vello` renderer backend.
- Vendored `vello_*` crates under `external/vello/…`: GPU (hybrid) and CPU rendering engines used by `typf-render-vello` / `typf-render-vello-cpu`.

### Caching + concurrency

- `moka`: Bounded cache with TinyLFU admission (scan-resistant), used for shaping/rendering caches.
- `parking_lot`: Low-overhead synchronization primitives.
- `rayon`: Optional parallelism for CPU-heavy workloads.
- `lru`: Legacy/auxiliary LRU usage (gradually being replaced by `moka`).

### Error handling + logging

- `thiserror`: Typed error enums.
- `anyhow`: Application-level error composition (primarily CLIs/tools).
- `log`, `env_logger`: Logging facade + default logger.

### Platform bindings

- `objc2` + `objc2-*` crates: macOS CoreText/CoreGraphics integration.

## Python

- `maturin`: Builds the `typfpy` extension module from Rust.
- `fire`: Python CLI wrapper.

## Policy

- Prefer well-maintained, widely-used crates over bespoke code.
- Avoid adding new dependencies unless they remove complexity or are required for a backend.
