# Changelog

Changes to TypF.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0//),
project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html/).

## [Unreleased]

### Fixed
- **Memory Leak in FontDatabase** (Critical - Issue #450)
  - Added path-based deduplication to prevent duplicate font loading
  - New `path_cache: HashMap<PathBuf, Arc<Font>>` tracks loaded fonts by path
  - Canonicalized paths ensure reliable deduplication
  - Added `clear()` method to reset database state
  - Added `font_count()` method for diagnostics
  - Impact: Prevents unbounded memory growth when same font is loaded repeatedly

### Added
- **Backend Documentation**: Comprehensive comparison tables for shapers, renderers, and export formats in README.md
- **Quickstart Example**: New `examples/quickstart_backends.rs` demonstrating all major backend combinations with conditional compilation
- **Cargo Integration**: Registered quickstart_backends example in Cargo.toml for easy execution

### Fixed
- **macOS Backend Detection**: Both `typf` and `typfpy` CLIs now correctly report macOS-native backends (CoreText shaper, CoreGraphics renderer) in `info` command output
- **Rust CLI Feature Flags**: Fixed mismatch between feature flag names (`shaping-mac`/`render-mac`) and actual backend packages (`typf-shape-ct`/`typf-render-cg`)
- **Python CLI Dynamic Detection**: Replaced hardcoded backend lists with dynamic probing of available backends
- **Test Script Font Keys**: Fixed KeyError in benchmark script by correcting font dictionary keys ("kalnia" → "kalniav", "notoarabic" → "notoara", "notosans" → "notosan")
- **Documentation**: Fixed duplicate heading in WORK.md

### Improved
- **Platform-Aware Builds**: Enhanced build.sh with platform-specific feature selection for macOS backends
- **Backend Aliases**: Added support for mac/ct/cg aliases in CLI backend selection
- **Documentation Structure**: Better organization of backend comparison information

## [2.0.0] - 2025-11-21

**Major Release**: Complete rewrite with modular architecture and multiple backend support.

### Added

**Core Architecture**
- Six-stage pipeline: Input → Unicode → Font → Shaping → Rendering → Export
- Trait-based backends with selective compilation via Cargo features
- Builder pattern with comprehensive error handling
- Multi-level caching system (L1 glyph cache, font database)

**Shaping Backends** (4)
- `NoneShaper` - Simple LTR advancement
- `HarfBuzz` - Complex scripts (Arabic, Devanagari, Hebrew, Thai, CJK)
- `ICU-HarfBuzz` - Unicode normalization + HarfBuzz
- `CoreText` - Native macOS shaping with caching

**Rendering Backends** (5)
- `Opixa` - Pure Rust rasterization with SIMD
- `Skia` - Anti-aliased rendering via tiny-skia
- `Zeno` - Pure Rust 2D rasterization with 256x anti-aliasing
- `CoreGraphics` - Native macOS rendering
- `JSON` - HarfBuzz-compatible shaping data export

**Export Formats**
- Bitmap: PNG, PNM (PPM/PGM/PBM) with alpha
- Vector: SVG with glyph paths
- Data: JSON with shaping metrics and glyph info

**Command-Line Interface**
- Rust CLI: Clap v4 with subcommands (`info`, `render`, `batch`)
- Python CLI: Click v8 with identical feature parity
- 30+ command-line options for full pipeline control
- Unicode escape sequences, color parsing, font feature specs

**Language Bindings**
- Python: PyO3 bindings with maturin packaging
- Rust: Full API with feature-gated backends
- Cross-platform wheel support (Linux, macOS, Windows)

**Performance**
- SIMD blending: 12.5 GB/s (AVX2), 8.4 GB/s (SSE4.1)
- Shaping: ~5µs/100 chars (None), ~45µs/100 chars (HarfBuzz + Arabic)
- L1 cache: ~40ns, Glyph rasterization: ~0.8µs/glyph at 16px
- Binary: ~500KB (minimal build, stripped)

**Platform Support**
- Linux (x86_64, aarch64)
- macOS (x86_64, Apple Silicon)
- Windows (x86_64)
- WASM (basic)

**Testing & Quality**
- 446 tests passing (206 unit + 240 integration)
- Property-based testing with proptest
- Golden snapshot tests for regression detection
- Fuzz testing infrastructure
- 100% success rate across 20 backend combinations
- All outputs verified (JSON, PNG, SVG)

**Documentation**
- `README.md` - Updated with v2.0 CLI syntax
- `CLI_MIGRATION.md` - Complete v1.x to v2.0 migration guide
- `RELEASE_NOTES_v2.0.0.md` - Comprehensive release documentation
- `RELEASE_CHECKLIST.md` - Publishing procedures
- `FEATURES.md` - Feature matrix
- Complete API documentation

### Changed
- **CLI**: Migrated from manual parsing to Clap v4 (Rust) and Fire to Click v8 (Python)
- **Performance**: 71% reduction in compiler warnings (24 → 7)
- **Build system**: Clean compilation with selective feature gating

### Fixed
- **Rendering**: Fixed baseline positioning (BASELINE_RATIO 0.75)
- **Rendering**: Fixed Y-coordinate collapse in Zeno renderer
- **Rendering**: Fixed descender cropping in Opixa, Skia, Zeno
- **Build**: Removed dead code warnings in legacy REPL/batch modules

## [Future Roadmap]

### v2.1 (Planned)
- Windows DirectWrite shaping
- WOFF, WOFF2 font support
- Better variable font controls

### v2.2 (Planned)
- Advanced text layout (justification, hyphenation)
- Color font support (COLR, CBDT, SBIX)
- Mobile performance optimizations

### v3.0 (Future)
- WebGPU rendering
- Advanced typography (floating anchors, GPOS/GSUB extensions)
- Plugin system for custom shapers/renderers

---

*81 development rounds completed over multiple months. Full backend matrix verification. Production-ready with comprehensive testing and documentation.*
