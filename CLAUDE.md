# Development Guidelines: TYPF Project

## I. CORE PHILOSOPHY: The Rendering Engine

`typf` is the foundational rendering engine providing cross-platform text layout and rasterization. Its development priorities are **performance**, **correctness**, and **API stability**.

### The Three Pillars

1. **Performance** - Sub-millisecond rendering for typical glyphs
   - Minimize allocations (object pooling for hot paths)
   - Efficient data structures (multi-shard LRU, DashMap for concurrency)
   - SIMD acceleration where appropriate (F26Dot6 math, edge processing)
   - Zero-copy font loading (memmap2)
   - Lock-free concurrency (avoid global RwLock bottlenecks)

2. **Correctness** - Pixel-perfect, high-fidelity output
   - Visual regression testing (SSIM-based comparisons)
   - Fuzzing critical paths (scan_converter, shaping, font parsing)
   - Miri checks for undefined behavior in unsafe blocks
   - Property-based testing (cache behavior, coordinate transformations)
   - Multi-script validation (Latin, Arabic, Devanagari, CJK)

3. **API Stability** - Library contract is sacred
   - No breaking changes without migration guide
   - Semantic versioning strictly enforced
   - Comprehensive rustdoc (100% public API coverage)
   - Example-driven documentation
   - Clear error types (`TypfError` enum, no panics in public API)

### Current Status (Nov 2025)

**Production-Ready:**
- ✅ All 3 platform backends (CoreText, DirectWrite, HarfBuzz)
- ✅ Multi-shard LRU caching (16 shards, lock contention eliminated)
- ✅ Python bindings via PyO3 (automatic backend selection)
- ✅ CLI with batch/stream/render subcommands
- ✅ 38+ integration tests passing
- ✅ SVG/PNG output with COLRv1 color font support

**In Progress:**
- ⏳ Orge rasterizer integration (core algorithm complete, backend wiring needed)
- ⏳ FFI panic handling (need std::panic::catch_unwind wrappers)
- ⏳ Visual regression framework (SSIM infrastructure planned)
- ⏳ Comprehensive README (currently minimal)

**Planned:**
- 📋 SIMD-accelerated fixed-point math
- 📋 cargo-fuzz + cargo-miri in CI
- 📋 Unified typf_error::Error enum
- 📋 seccomp sandboxing for untrusted fonts

## II. PROJECT STRUCTURE

`typf` is a Rust workspace with a layered architecture:

- **`backends/`**: Contains the platform-specific and platform-agnostic rendering backends.
  - `typf-core`: Core traits, types, and caching infrastructure.
  - `typf-icu-hb`: The main cross-platform backend using HarfBuzz for shaping.
  - `typf-orge`: Our custom, high-performance CPU rasterizer.
  - `typf-mac`/`typf-win`: Platform-native backends (CoreText/DirectWrite).
- **`crates/`**: Contains modular, reusable components.
  - `typf-api`: The high-level, unified public API.
  - `typf-batch`: Infrastructure for batch processing.
  - `typf-fontdb`: Font discovery and database management.
- **`python/`**: The PyO3-based Python bindings.
- **`src/`**: The `typf-cli` binary.

## III. TOOLING & WORKFLOW

### 3.1. Core Toolchain
- **Build & Test**: `cargo`
- **Formatting**: `cargo fmt` (non-negotiable)
- **Linting**: `cargo clippy` (with `-D warnings` to enforce high standards)
- **Benchmarking**: `criterion`

### 3.2. Development Workflow
1.  **Select a task** from the root `TODO.md` that is prefixed with `(typf)`.
2.  **Write a failing test.** This could be a unit test, an integration test, or a benchmark. For visual changes, create a regression test in `tests/compare_backends.rs`.
3.  **Implement the feature/fix.** Adhere to the principles of minimalism and safety.
4.  **Run all checks and tests** using the commands below.
5.  **Profile if necessary.** If you're working on a performance-sensitive area, use `cargo-flamegraph` or other profiling tools to validate your changes.

## IV. KEY COMMANDS

- **Format code**:
  ```bash
  cargo fmt --all
  ```
- **Check for warnings and lint issues**:
  ```bash
  cargo clippy --workspace --all-features -- -D warnings
  ```
- **Run all tests**:
  ```bash
  cargo test --workspace --all-features
  ```
- **Run benchmarks**:
  ```bash
  cargo bench --workspace
  ```
- **Build the Python extension**:
  ```bash
  cd python/
  maturin develop
  ```

## V. SPECIFIC GUIDELINES

- **Error Handling**: All public functions in `typf-api` and other core crates should return a `Result<T, typf_error::Error>`. Do not allow panics to cross API boundaries.
- **Feature Flags**: Use Cargo feature flags to manage optional dependencies and backends (e.g., `orge`, `tiny-skia-renderer`). Ensure the default feature set is sensible.
- **FFI (Python)**: When working in the `python/` directory, remember that you are in a guest environment. Catch all panics, convert errors to `PyErr`, and release the GIL for any long-running operations.
- **Documentation**: All public modules, types, and functions MUST have clear documentation (`rustdoc`). Add examples to show how to use the API.

---
**Focus:** Build a world-class rendering engine. Prioritize stability and performance above all else.
