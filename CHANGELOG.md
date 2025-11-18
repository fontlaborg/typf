# Changelog

All notable changes to TYPF v2.0 will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Documentation**: Comprehensive `BENCHMARKS.md` with performance targets, methodology, current results, and optimization guides
- **Documentation**: Complete `SECURITY.md` with vulnerability reporting, security considerations, and best practices
- **Documentation**: Professional `CONTRIBUTING.md` with development workflows, code standards, and contributor guidelines
- **Documentation**: `RELEASE.md` with complete release checklist and procedures
- **Documentation**: Enhanced rustdoc in `typf-core` with module-level examples and usage guidance
- **GitHub**: Issue templates for bug reports and feature requests
- **GitHub**: Issue template configuration directing users to discussions and docs
- **Scripts**: `scripts/bench-compare.sh` for benchmark regression detection
- **Scripts**: `scripts/bench.sh` for simple benchmark execution
- **Config**: `deny.toml` for dependency security auditing with cargo-deny
- **Export**: PNG format support with proper color space conversion (production-ready)
- **Export**: SVG format support with embedded bitmaps
- **Export**: JSON format (HarfBuzz-compatible shaping results)
- **Python**: Fire CLI with 4 commands (`render`, `shape`, `info`, `version`)
- **Python**: Comprehensive README (300+ lines) with installation and usage
- **Python**: Example scripts (simple and advanced rendering)
- **Examples**: Complete `examples/README.md` documenting all 4 Rust examples
- **Examples**: `all_formats.rs` demonstrating PNG, SVG, and all PNM formats
- **Examples**: `formats.rs` and `pipeline.rs` showing API patterns
- **WASM**: Build support with wasm-bindgen configuration
- **Testing**: Property-based testing with proptest (7 tests for Unicode normalization, bidi, script detection)
- **Testing**: Golden snapshot tests for HarfBuzz shaping (5 regression detection tests)
- **GitHub**: Pull request template with comprehensive quality checklist
- **CI**: cargo-audit security scanning in GitHub Actions
- **Scripts**: `scripts/count-tests.sh` to automatically update test count badge
- **Scripts**: `scripts/profile-memory.sh` for automated memory profiling with Valgrind/heaptrack
- **Scripts**: `scripts/fuzz.sh` for running fuzz tests with cargo-fuzz
- **Config**: `.editorconfig` for consistent formatting across editors
- **Config**: `rustfmt.toml` for standardized Rust code formatting
- **Docs**: `docs/MEMORY.md` - comprehensive memory profiling guide (200+ lines)
- **Fuzz**: Complete fuzz testing infrastructure with 3 targets (unicode, harfbuzz, pipeline)
- **CLI**: REPL mode scaffold with interactive command interface (--features repl)
- **CI**: GitHub Actions workflow for automated fuzz testing (daily + manual trigger)
- **Hooks**: Pre-commit hook template for automated code quality checks (.github/hooks/)

### Changed
- **README**: Updated test count to 107 (was 95)
- **README**: Updated to reflect current state (all export formats, Python bindings complete)
- **README**: Added links to BENCHMARKS.md, SECURITY.md, and RELEASE.md
- **README**: Corrected performance metrics (12.5 GB/s SIMD, ~5µs/100 chars shaping)
- **README**: Updated backend status (ICU-HB complete, PNG/SVG/JSON complete)
- **CONTRIBUTING.md**: Added pre-commit hook installation instructions
- **.gitignore**: Added entries for fuzz artifacts and profiling data

### Fixed
- Rustdoc warnings in `typf-core` (unresolved link to `FontRef`)
- Doctest in `typf-core` (missing `process()` and backend `name()` implementations)
- Performance test threshold in `typf-render-orge` (lowered for CI environments)

## [2.0.0-dev] - 2025-11-18

### Added
- **Core**: Six-stage pipeline architecture (Input → Unicode → Font → Shaping → Rendering → Export)
- **Core**: Trait-based backend system (`Stage`, `Shaper`, `Renderer`, `Exporter`)
- **Core**: Builder pattern for pipeline construction
- **Core**: Comprehensive error types (`TypfError`, `ShapingError`, `RenderError`)
- **Shaping**: NoneShaper (simple LTR advancement)
- **Shaping**: HarfBuzz integration with complex script support (Arabic, Devanagari, Hebrew, Thai, CJK)
- **Shaping**: OpenType feature support (liga, kern, smcp, etc.)
- **Unicode**: ICU integration for text normalization (NFC)
- **Unicode**: Bidirectional text support with level resolution
- **Unicode**: Text segmentation (grapheme, word, line breaking)
- **Font**: Real font loading via `read-fonts` and `skrifa`
- **Font**: TrueType Collection (.ttc) support
- **Font**: Font metrics and advance width calculation
- **Font**: Arc-based memory management for zero-copy
- **Rendering**: Orge rasterizer with scanline conversion
- **Rendering**: SIMD optimizations (AVX2, SSE4.1, partial NEON)
- **Rendering**: Parallel rendering with Rayon
- **Cache**: Multi-level cache system (L1/L2/L3)
- **Cache**: DashMap for concurrent access
- **Cache**: LRU eviction policy
- **Export**: PNM formats (PPM, PGM, PBM)
- **CLI**: Rust CLI with Clap argument parsing
- **Build**: Cargo feature flags (minimal, default, full)
- **Build**: Selective compilation (pay only for what you use)
- **CI**: GitHub Actions with multi-platform matrix
- **CI**: Code coverage with tarpaulin
- **CI**: Security auditing with cargo-deny and cargo-audit
- **Tests**: 90 tests across all modules
- **Tests**: Integration tests for end-to-end pipeline
- **Benchmarks**: Criterion.rs benchmark suite
- **Benchmarks**: Performance targets achieved (>10GB/s SIMD, <10µs shaping, <50ns cache)
- **Python**: PyO3 bindings with Pythonic API
- **Docs**: ARCHITECTURE.md with system design
- **Docs**: Consolidated `.gitignore` for Rust/Python workspace
- **Docs**: CLAUDE.md with workspace snapshot and Rust/PyO3 guardrails

### Performance
- Binary size: ~500KB (minimal build, stripped)
- SIMD blending: 12.5 GB/s (AVX2), 8.4 GB/s (SSE4.1)
- Simple shaping: ~5µs/100 chars (NoneShaper)
- Complex shaping: ~45µs/100 chars (HarfBuzz with Arabic)
- L1 cache hit: ~40ns
- Glyph rasterization: ~0.8µs/glyph at 16px

### Platform Support
- Linux (x86_64, aarch64)
- macOS (x86_64, Apple Silicon)
- Windows (x86_64)
- WASM (basic support)
