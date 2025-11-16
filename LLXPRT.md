# Development Guidelines: TYPF Project

## I. CORE PHILOSOPHY: The Rendering Engine

`typf` is the foundational rendering engine. Its development priorities are **performance**, **correctness**, and **API stability**. It must be a reliable, reusable, and panic-free library.

- **Performance**: We aim for state-of-the-art performance. This means minimizing allocations, using efficient data structures (e.g., `DashMap` for concurrency), and leveraging SIMD where appropriate. Every change should be considered for its performance impact.
- **Correctness**: The engine must produce accurate and high-fidelity renderings. We rely on extensive testing, including visual regression tests (SSIM) and fuzzing, to ensure correctness.
- **API Stability**: As a library, `typf`'s public API is its contract. Breaking changes should be avoided unless absolutely necessary and must be clearly documented in a migration guide.

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
