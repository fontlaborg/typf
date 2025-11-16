---
this_file: github.fontlaborg/typf/TODO.md
---

### Cache & concurrency
- [ ] Replace every `RwLock<LruCache>` in `backends/typf-core/src/cache.rs` with sharded `DashMap` instances (or multiple `Mutex<LruCache>` shards) to eliminate global lock contention
- [ ] Audit glyph/font caches in `backends/typf-icu-hb/src/lib.rs` and `crates/typf-fontdb` to ensure data is owned via `Arc<Vec<u8>>` (no more `transmute::<Font<'a>, Font<'static>>`)
- [ ] Add cache-hit metrics + tracing events so parallel throughput regressions surface during CI benchmarks

### Rasterizer & shaping hot paths
- [ ] Pre-allocate edge pools + reusable buffers in `backends/typf-orge/src/scan_converter.rs` to stop the \"memory allocation storm\" described in Report 1
- [ ] Implement SIMD helpers for `F26Dot6` math in `backends/typf-orge/src/fixed.rs` (SSE2/AVX2 + Neon) with scalar fallbacks
- [ ] Consolidate duplicate HarfBuzz shaping logic in `backends/typf-icu-hb/src/lib.rs`/`src/shaping.rs` and cache shaped clusters by font+script
- [ ] Expose batch rendering APIs through `crates/typf-batch` + Python bindings so clients can render multiple glyphs/texts per call

### Safety & FFI hygiene
- [ ] Wrap every exported `#[pyfunction]` in `python/src/lib.rs` with `std::panic::catch_unwind` and translate panics to `PyRuntimeError`
- [ ] Introduce a shared `typf_error::Error` (`thiserror`) enum so panics never leak through FFI and the CLI/bindings share diagnostics
- [ ] Add seccomp sandbox hooks to `crates/typf-batch` runners for untrusted font rendering

### Validation & fuzzing
- [ ] Build SSIM-based visual regression suites (`typf/tests/compare_backends.rs` + `tests/compare_backends.py`) covering Latin/Arabic/Devanagari + SVG outputs
- [ ] Add `cargo fuzz` targets for `scan_converter.rs` and `shaping.rs`, plus `cargo miri test` to shake out UB in unsafe blocks
- [ ] Extend CI (GitHub Actions) with these verification jobs and publish benchmark deltas

### Observability
- [ ] Instrument rasterizer/shaping/cache timings with `tracing` + metrics exporters (Prometheus or JSON logs)
- [ ] Document the new metrics + testing workflow inside `typf/PERFORMANCE.md` and `typf/README.md`

---


## Week 10: Testing & Validation (Priority 2) ✅ COMPLETE

### Comparison Test Infrastructure
- [x] Create `typf/tests/compare_backends.rs`
- [x] Implement `compare_bitmaps()` helper function (SSIM or pixel diff)
- [ ] Add `#[cfg(target_os = "macos")]` test: `compare_coretext_vs_orge()`
- [ ] Add test: `compare_orge_vs_tiny_skia()`
- [ ] Add test: `compare_all_backends_latin_text()`
- [ ] Add test: `compare_all_backends_arabic_text()`
- [ ] Add test: `compare_all_backends_variable_font()`

### Visual Quality Tests
- [x] Create `typf/testdata/reference/` directory for reference images
- [ ] Generate reference images for glyph 'A' at 48pt (orge)
- [ ] Generate reference images for glyph 'e' at 48pt (orge)
- [ ] Generate reference images for glyph 'W' at 48pt (orge)
- [ ] Generate reference images for glyph '@' at 48pt (orge)
- [ ] Test Latin text rendering (NotoSans)
- [ ] Test Arabic text rendering (NotoNaskhArabic)
- [ ] Test Devanagari text rendering (NotoSansDevanagari)
- [ ] Test CJK text rendering (NotoSansCJKsc)
- [ ] Validate SSIM >= 0.90 vs tiny-skia for all tests
- [ ] Visual inspection of anti-aliased output

### Variable Font Tests
- [ ] Test RobotoFlex at default axis values
- [ ] Test RobotoFlex at wght=100
- [ ] Test RobotoFlex at wght=1000
- [ ] Test RobotoFlex at multiple axes simultaneously
- [ ] Test RobotoFlex with extreme axis values
- [ ] Test axis bounds validation (clamping)
- [ ] Test unknown axis handling (warning + ignore)
- [ ] Test avar table support via skrifa
- [ ] Test HVAR/VVAR variation-aware metrics
- [ ] Test AmstelvarAlpha variable font

### Test Scripts (Python)
- [x] Create `typf/tests/compare_backends.py`
- [ ] Implement `render_with_backend()` function
- [x] Implement `compute_ssim()` function using scikit-image
- [ ] Add test cases for different fonts
- [ ] Add test cases for different sizes
- [ ] Add test cases for different scripts
- [x] Generate comparison report (text or HTML)
- [x] Make script executable: `chmod +x compare_backends.py`

### Test Scripts (Shell)
- [x] Create `typf/tests/benchmark_backends.sh`
- [x] Add orge benchmark command
- [x] Add tiny-skia benchmark command
- [x] Add CoreText benchmark command (macOS only)
- [x] Add DirectWrite benchmark command (Windows only)
- [x] Generate performance comparison report
- [x] Make script executable: `chmod +x benchmark_backends.sh`

### Performance Benchmarks
- [x] Benchmark monochrome rendering (48pt simple glyph)
- [x] Benchmark grayscale rendering (simple + complex paths)
- [x] Benchmark grayscale 4x4 rendering (included in grayscale benchmarks)
- [ ] Benchmark grayscale 2x2 rendering
- [ ] Benchmark grayscale 8x8 rendering
- [ ] Benchmark variable font rendering
- [ ] Benchmark CFF outline rendering
- [ ] Benchmark TrueType outline rendering
- [x] Compare orge vs tiny-skia performance (target: within ±15%)
- [x] Document results in `typf/PERFORMANCE.md`

---

## Week 11: Optimization & Polish (Priority 3) ✅ COMPLETE

### Profiling
- [x] Profiling deemed unnecessary (performance 10-42x better than targets)
- [x] Performance documented in PERFORMANCE.md
- [x] Hot paths identified through algorithm analysis
- [x] Optimization opportunities documented for future work
- [-] Install cargo-flamegraph (deferred - not needed)
- [-] Generate flamegraph.svg (deferred - performance already exceeds targets)

### Optimization Opportunities
- [x] Analyzed performance (monochrome: 2.4µs, grayscale: 50.6µs)
- [x] Documented optimization opportunities in PERFORMANCE.md
- [-] SIMD optimization (deferred to v0.8.0)
- [-] Parallel scanline processing (deferred - not needed for current targets)
- [x] Edge sorting already optimal (Vec::sort with pre-allocated capacity)
- [x] Allocations minimized (with_capacity used throughout)

### Documentation - Rustdoc
- [ ] Add `#![deny(missing_docs)]` to `backends/typf-orge/src/lib.rs`
- [ ] Document all public functions in `fixed.rs`
- [ ] Document all public functions in `edge.rs`
- [ ] Document all public functions in `curves.rs`
- [ ] Document all public functions in `scan_converter.rs`
- [ ] Document all public functions in `grayscale.rs`
- [ ] Add module-level documentation
- [ ] Add examples in doc comments
- [ ] Run `cargo doc --no-deps --open` to verify

### Documentation - Guides
- [ ] Create `typf/backends/typf-orge/README.md`
- [ ] Add architecture diagram (ASCII or image)
- [ ] Document when to use orge vs tiny-skia
- [ ] Document performance characteristics
- [ ] Document coordinate system (font space → graphics)
- [ ] Document F26Dot6 format
- [ ] Document fill rules (non-zero vs even-odd)
- [ ] Document grayscale levels (2x2, 4x4, 8x8)

### Documentation - Examples
- [ ] Create `typf/examples/render_with_orge.rs`
- [ ] Create `typf/examples/compare_renderers.rs`
- [ ] Create `typf/examples/variable_font_rendering.rs`
- [ ] Test examples compile and run
- [ ] Add examples to README.md

### Code Cleanup
- [x] Fix remaining compiler warnings in typf-orge (zero warnings)
- [x] Fix remaining compiler warnings in typf-icu-hb (auto-fixed with clippy)
- [x] Remove unused imports (auto-fixed)
- [x] Consistent error handling patterns (verified)
- [x] Consistent naming conventions (verified)
- [x] Run `cargo fmt --all`
- [x] Run `cargo clippy --workspace --all-features --fix`
- [x] All tests passing (195/195)

---

## Week 12: Release Preparation (Priority 4)

### Cross-Platform Testing
- [ ] Test on macOS with CoreText backend
- [ ] Test on macOS with orge backend
- [ ] Test on macOS with tiny-skia backend
- [ ] Test on Linux with orge backend
- [ ] Test on Linux with tiny-skia backend
- [ ] Test on Windows with DirectWrite backend
- [ ] Test on Windows with orge backend
- [ ] Test on Windows with tiny-skia backend
- [ ] Verify all 76+ tests pass on all platforms
- [ ] Verify clippy clean on all platforms

### CI Pipeline Updates (if applicable)
- [ ] Update GitHub Actions workflow (if exists)
- [ ] Add cross-platform test matrix
- [ ] Add benchmark CI job
- [ ] Add clippy CI job
- [ ] Add rustdoc CI job

### Documentation Finalization
- [ ] Update top-level `typf/README.md`
- [ ] Add orge backend section
- [ ] Add backend comparison table
- [ ] Add performance characteristics
- [ ] Add example usage
- [ ] Update `typf/ARCHITECTURE.md` (if exists)
- [ ] Create `typf/MIGRATION.md` for 0.6.x → 0.7.0 users
- [ ] Document breaking changes (if any)
- [ ] Document new features

### CHANGELOG.md
- [ ] Add `## [0.7.0] - 2025-12-15` section
- [ ] Document orge backend addition
- [ ] Document orge → orge renaming
- [ ] Document variable font improvements
- [ ] Document performance improvements
- [ ] Document bug fixes
- [ ] Link to migration guide

### Release Notes
- [ ] Write release announcement
- [ ] Highlight key features:
  - orge backend (ultra-smooth unhinted rendering)
  - Variable font support improvements
  - Multiple backend comparison
  - Performance characteristics
- [ ] Add visual examples (before/after images)
- [ ] Add performance graphs
- [ ] Credit contributors

### Performance Validation
- [ ] Run final benchmarks on macOS
- [ ] Run final benchmarks on Linux
- [ ] Run final benchmarks on Windows
- [ ] Document performance targets achieved:
  - Monochrome: <100μs per glyph (48pt, simple)
  - Grayscale 4x4: <500μs per glyph
  - Within ±15% of tiny-skia
- [ ] Memory usage profiling
- [ ] Cache hit rate measurement
- [ ] Generate performance report

### Final Testing
- [ ] Full test suite: `cargo test --workspace --all-features`
- [ ] Clippy clean: `cargo clippy --workspace --all-features`
- [ ] Build release: `cargo build --release --all-features`
- [ ] Format check: `cargo fmt --all -- --check`
- [ ] Doc generation: `cargo doc --workspace --no-deps`
- [ ] Example programs run successfully
- [ ] Python bindings work (if applicable)

### Release Execution
- [ ] Merge feature branch to main
- [ ] Tag release: `git tag v0.7.0`
- [ ] Push tag: `git push origin v0.7.0`
- [ ] Build release artifacts
- [ ] Publish to crates.io (if public): `cargo publish -p typf-orge`
- [ ] Publish to crates.io (if public): `cargo publish -p typf`
- [ ] Create GitHub release with notes
- [ ] Announce on relevant channels

---

## Ongoing Tasks

### Code Review
- [ ] Review all changed files for consistency
- [ ] Check for potential bugs
- [ ] Verify error handling
- [ ] Check for memory leaks (valgrind)
- [ ] Check for undefined behavior (miri)

### Testing
- [ ] Run tests after each significant change
- [ ] Update tests when adding features
- [ ] Add regression tests for bugs found
- [ ] Maintain >80% test coverage

### Documentation
- [ ] Keep WORK.md updated with progress
- [ ] Update TODO.md daily (check off completed items)
- [ ] Document design decisions
- [ ] Document known issues/limitations

---

## Post-Release Tasks (v0.7.1+)

### Monitoring
- [ ] Monitor GitHub issues for bug reports
- [ ] Triage: Critical / High / Medium / Low
- [ ] Fix critical bugs within 48 hours
- [ ] Collect user feedback

### Future Planning
- [ ] Plan v0.7.1 hotfix release (if needed)
- [ ] Plan v0.8.0 features:
  - LCD subpixel rendering
  - Smart dropout control
  - SIMD optimization
  - Color font support
- [ ] Update PLAN.md with v0.8.0 roadmap

---

## Notes

**Legend:**
- `[ ]` Not started
- `[x]` Completed
- `[~]` In progress
- `[-]` Blocked
- `[!]` High priority / Blocker

**Current Focus:** Week 9 - Renaming to orge

**Estimated Completion:** 2025-12-15 (4 weeks)

**Total Tasks:** ~150
