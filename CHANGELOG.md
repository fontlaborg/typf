# Changelog

All notable changes to typf will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Backend Benchmarking**: Created a new benchmark crate (`backend-benches`) to measure and compare the rendering speed of different backends (`CoreText`, `HarfBuzz`, `Orge`, etc.). This tool will help in identifying performance regressions and guiding optimization efforts.
- **Issue #104 - toy.py CLI Tool**: Added a Python CLI script (`toy.py`) using the `fire` library with two commands: `bench` (runs Rust benchmarks) and `render` (renders samples with all available backends, saves PNGs, then runs benchmarks). This provides a convenient interface for developers to test and benchmark the library. (2025-11-17)

### Changed
- **Architectural Refactoring**:
  - Introduced a two-tiered backend trait system to resolve cyclic dependencies between the `typf-api` crate and the backend implementation crates.
  - **`typf-core`**: Now defines a high-level, object-safe `DynBackend` trait that all backends must implement. This serves as the unified interface for the `typf-api` crate. The lower-level `CoreBackendTrait` (previously `Backend`) is now used for direct, backend-specific implementations. `typf-core` is now fully backend-agnostic.
  - **`typf-api`**: Now acts as the central factory for backends. It contains the `Backend` enum and the `create_backend`/`create_default_backend` functions, which construct `Box<dyn DynBackend>` instances based on enabled features.
  - **Backend Crates**: All backend crates (e.g., `typf-icu-hb`, `typf-mac`) now depend on `typf-core` and implement the `DynBackend` trait, providing a consistent interface for the `typf-api` session.

### Fixed
- **Issue #103 - Backend Refactoring Compilation Errors**: Fixed 50+ compilation errors that arose from the backend trait refactoring. All backend crates (`typf-orge`, `typf-pure`, `typf-icu-hb`, `typf-mac`, `typf-python`) now correctly use the `typf_core::traits::Backend` import path and the new `TypfError::render()` API. Updated type conversions (FWord→i16, f32→f64) and resolved trait method ambiguities. The benchmark crate was also updated to use the new Session API. (2025-11-17)

## [0.1.0] - 2025-11-15

### Summary
Initial release of typf - fast, cross-platform text rendering with native platform backends.
...