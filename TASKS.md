<!-- this_file: TASKS.md -->
# Typf Quality Improvement Plan

**Version:** 5.1.0 (Target)
**Status:** Planning
**Last Updated:** 2026-02-11

This plan addresses the technical debt and quality issues identified in the 2026-02-11 audit.

## Phase 1: Safety & Reliability (High Priority)

### 1.1 Eliminate `unwrap()` in Production Code
- [ ] **Audit `crates/typf-cli/`**:
    - [ ] Replace `BatchConfig::parse(&args).unwrap()` with proper error handling in `src/batch.rs`.
    - [ ] Replace all `unwrap()` calls in `src/jsonl.rs` with `Result` and `?`.
- [ ] **Audit `crates/typf-export/`**:
    - [ ] Replace `unwrap()` in base64 encoding logic in `src/svg.rs`.
    - [ ] Replace `unwrap()` in JSON serialization in `src/json.rs`.
- [ ] **Workspace Policy**:
    - [ ] Add `unwrap_used = "deny"` to `[workspace.lints.clippy]` in root `Cargo.toml`.
    - [ ] Add `expect_used = "warn"` to the same section.

### 1.2 Harden Error Handling
- [ ] **SVG Exporters**:
    - [ ] Replace `let _ = write!(...)` with `write!(...)?` in `crates/typf-export-svg/src/lib.rs`.
    - [ ] Replace `let _ = write!(...)` with `write!(...)?` in `backends/typf-render-svg/src/lib.rs`.
- [ ] **Pipeline & Export**:
    - [ ] Replace `unreachable!` with `Err(TypfError::...)` in `typf-export/src/png.rs` and `typf-core/src/pipeline.rs`.

### 1.3 Unsafe Code Audit
- [ ] **Documentation**:
    - [ ] Add `// SAFETY:` comments to all `unsafe` blocks in `typf-core/src/ffi.rs`.
    - [ ] Add `// SAFETY:` comments to all SIMD intrinsics in `typf-render-opixa/src/simd.rs`.
- [ ] **Verification**:
    - [ ] Audit `backends/typf-shape-ct/src/lib.rs` for correct ownership when using `Box::from_raw`.

## Phase 2: Refactoring & Maintainability (Medium Priority)

### 2.1 Decompose `typf-cli` Command Logic
- [ ] **Refactor `render` command**:
    - [ ] Extract input resolution (font paths, text files) to `crates/typf-cli/src/resolver.rs`.
    - [ ] Extract parameter validation (colors, sizes, features) to `crates/typf-cli/src/validation.rs`.
- [ ] **Refactor `batch` command**:
    - [ ] Break down the large `Job` processing loop in `src/jsonl.rs` into smaller, testable functions.

### 2.2 Crate Consolidation
- [ ] **Merge `typf-export-svg`**:
    - [ ] Move SVG export logic into the main `typf-export` crate.
    - [ ] Update all workspace dependencies and feature flags.
    - [ ] Remove the redundant `crates/typf-export-svg` directory.

### 2.3 Clean up Suppressed Lints
- [ ] **Audit `#[allow(dead_code)]`**:
    - [ ] Remove unused internal fields in `crates/typf-core/src/pipeline.rs`.
    - [ ] Remove unused SIMD variants in `backends/typf-render-opixa/src/simd.rs`.

## Phase 3: Documentation & API Quality (Low Priority)

### 3.1 Complete Public API Documentation
- [ ] **CLI Crates**:
    - [ ] Add `///` doc comments to all public functions in `crates/typf-cli/src/limits.rs`.
    - [ ] Add `///` doc comments to all public functions in `crates/typf-cli/src/commands/*.rs`.
- [ ] **Enable Documentation Lints**:
    - [ ] Add `#![warn(missing_docs)]` to `typf-core` and `typf-cli`.

## Phase 4: Validation & Testing

### 4.1 Expand Visual Regression Suite
- [ ] **Complex Scripts**:
    - [ ] Add SSIM tests for Arabic (RTL) shaping.
    - [ ] Add SSIM tests for Devanagari (Hindi) shaping.
    - [ ] Add SSIM tests for Thai shaping.

### 4.2 Fuzzing Expansion
- [ ] **Batch Input Fuzzer**:
    - [ ] Create a new fuzz target `fuzz_batch_jsonl` to identify potential panics in the JSONL parser.

### 4.3 CI Enhancements
- [ ] **MSRV Verification**:
    - [ ] Add a dedicated CI job that runs `cargo check` with Rust 1.75 to ensure no newer features are accidentally used.
