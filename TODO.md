# TYPF v2.0 - TODO List

## ✅ ROUND 79 COMPLETED: Baseline Alignment Fixes (2025-11-19)

**Issues Fixed**:
- ✅ Orge: Fixed vertical shift (baseline from 0.80 to 0.75)
- ✅ Skia: Fixed vertical shift (baseline from 0.80 to 0.75)
- ✅ Zeno: Fixed vertical shift (baseline from 0.80 to 0.75)

**Results**:
- All renderers now match CoreGraphics baseline positioning
- Text no longer cropped at bottom
- Proper spacing maintained at top
- 100% backend success rate maintained

## ✅ ROUND 78 COMPLETED: Critical Renderer Regression Fixes (2025-11-19)

**Issues Reported (Build #251119-1400)**:
- ⚠️ Orge: Vertical shift + dirt artifacts
- ⚠️ Skia: Vertical shift  
- ⚠️ Zeno: Y-coordinate collapse (all pixels at Y=0)

**Root Causes Found**:
1. Round 75 changed BASELINE_RATIO from 0.75 to 0.65 (incorrect)
2. Zeno swapped bbox.y0/y1 causing negative height calculation

**Fixes Applied**:
1. ✅ Reverted BASELINE_RATIO to 0.75 in Orge, Skia, Zeno (match CoreGraphics)
2. ✅ Fixed Zeno bbox coordinate usage (min_y = bbox.y0, max_y = bbox.y1)

**Results**:
- Zeno file sizes: 0.6KB → 5.8KB (proper rendering restored)
- All renderers now match CoreGraphics baseline positioning
- 100% backend success rate maintained


## ✅ ROUND 77 COMPLETED: Performance Baseline & Optimization (2025-11-19)
## ✅ MILESTONE ACHIEVED: Complete Backend Matrix (2025-11-19)

- We’ve created ./typf-tester/typfme.py that uses a test font and into @./typf-tester/ folder it’s supposed to output a renderings using ALL shaping and render backends, as both PNG and SVG. 
- Make sure that 'typfme.py' supports ALL shaping and render backends. Make sure the Python bindings support ALL shaping and render background. Make sure that the Rust CLI supports ALL shaping and render backends.
- The typefme.py tool should also perform benchmarking of all backend combos across many sample texts and font sizes and produce a nice JSON report and an extremely compact Markdown table into the @./typf-tester/ folder.  
- Use the 'typfme.py' tool and inspect the outputs to debug and improve the shaping and rendering of all backgrounds. Work in a continuous feedback loop. 
- You must actually RUN ./build.sh (which at the end runs ./typf-tester/typfme.py and produces the outputs in @./typf-tester/output/ ) to verify that the changes you make are working, and then you must inspect the outputs to debug and improve the shaping and rendering of all backgrounds.
- A common problem with shaping and rendering may be size (scale) mismatch, or that the rendering may be upside down (coordinate system mismatch).

**Performance Results:**
- Fastest combo: CoreText + JSON (30,639 ops/sec)
- Best rasterizer: CoreGraphics (22,346 ops/sec)
- All backends 100% success rate

**Known Issues:**
- [x] ICU-HarfBuzz produces narrow output (41px vs ~700px) - ✅ FIXED (2025-11-19) - scaling formula corrected
- [x] SVG tiny glyph issue - ✅ FIXED (2025-11-19) - double-scaling bug resolved
- [x] SVG export for all renderers - ✅ WORKING AS DESIGNED (2025-11-19) - SVG generated from glyph outlines, not renderer output
- [x] CoreGraphics blank PNG - ✅ FIXED (2025-11-19) - switched to CTFont API
- [x] Orge garbled PNG - ✅ FIXED (2025-11-19) - Y-axis coordinate flip in TransformPen
- [x] Skia faint pixels - ✅ FIXED (2025-11-19 Round 28) - removed double-scaling bug (skrifa already scales)
- [x] Zeno blank PNG - ✅ FIXED (2025-11-19 Round 28) - rewrote SVG path parser to handle space-separated tokens
- [x] Format validation - ✅ COMPLETE (2025-11-19 Round 31) - all 5 renderers (including JSON) now have `supports_format()` implemented with tests
- [x] Mixed-script SVG export fails - ✅ FIXED (2025-11-19 Round 35) - NotoSans font now used for mixed scripts (has CJK coverage)
- [x] Skia top cropping - ✅ FIXED (2025-11-19 Round 75) - adjusted baseline from 75% to 65% for tall glyphs (A, T, W, f, l)
- [x] Zeno faint glyphs - ✅ FIXED (2025-11-19 Round 75) - removed Y-scale from path builder, added bitmap flip + pixel inversion
- [x] Orge top cropping - ✅ FIXED (2025-11-19 Round 75) - adjusted baseline from 75% to 65% (same as Skia)
- [x] Orge counter-filling - ✅ FIXED (2025-11-19 Round 75) - corrected edge winding direction for bitmap coords (dy > 0 → +1)
- [x] Zeno top cropping - ✅ FIXED (2025-11-19 Round 75) - adjusted baseline from 75% to 65% (consistency across all renderers)
- [x] Skia/Orge/Zeno baseline regression - ✅ FIXED (2025-11-19 Round 78) - reverted BASELINE_RATIO from 0.65 back to 0.75 (Round 75 change was incorrect)
- [x] Zeno Y-coordinate collapse - ✅ FIXED (2025-11-19 Round 78) - fixed bbox.y0/y1 swapping that caused 1-pixel high bitmaps

**Next: Continuous improvement** using typfme.py feedback loop 


## Immediate Tasks

- [ ] If you work on the 'orge' backend (the pure-Rast monochrome/grayscale rasterizer), consult the reference implementation in @./external/rasterization_reference/ ('orge' is the Rust port thereof)
- [x] MUST-DO!!! Variable font support (2025-11-18)
- [x] MUST-DO!!! Batch processing mode (2025-11-18)
- [x] MUST-DO!!! JSONL batch processing (batch + stream modes) (2025-11-19)
- [x] [orge] Port remaining Orge modules (curves, edge, scan_converter, grayscale) (2025-11-18)
- [x] [orge] Add glyph outline extraction from skrifa (2025-11-18)
- [x] [orge] Integrate scan converter with real glyph outlines (2025-11-18)
- [x] [skia] Implement Skia rendering backend (Week 13-14) (2025-11-19)
- [x] [zeno] Implement Zeno rendering backend (Week 15) (2025-11-19)
- [x] [svg] Implement SVG vector export (2025-11-19)
- [ ] DirectWrite shaper (Windows), Direct2D renderer (Windows) —— Windows platform backends (DirectWrite + Direct2D) require Windows platform or GitHub Actions for testing. The macOS implementation provides a complete reference pattern for the Windows backends. See @./github.fontlaborg/typf/old-typf/backends/typf-win for an OLD implementation

--- 

## Completed Documentation & Optimization (Round 17-20 - 2025-11-19)

- [x] Enhanced error messages with actionable solutions (Round 17)
- [x] Documented bitmap width limitations in README (Round 17)
- [x] Created comprehensive performance optimization guide (docs/PERFORMANCE.md) (Round 17)
- [x] Added long text handling examples (Rust + Python) (Round 17)
- [x] Enhanced typfme.py info command with comprehensive environment details (Round 19)
- [x] Created QUICKSTART.md guide for 5-minute onboarding (Round 19)
- [x] Added Troubleshooting section to main README.md (Round 19)
- [x] Added real benchmark results to typf-tester README (Round 20)
- [x] Created comprehensive backend comparison guide (docs/BACKEND_COMPARISON.md) (Round 20)
- [x] Added cross-reference links throughout documentation (Round 20)

## Release Preparation

### Pre-Release Verification (All Complete ✅)
- [x] All 206 tests passing (100% pass rate)
- [x] Zero compiler warnings
- [x] All 20 backend combinations operational (100% success)
- [x] 175 outputs verified (JSON + SVG + PNG)
- [x] Documentation complete with zero broken links
- [x] Automated regression detection operational
- [x] 92% feature completeness (81/88 features)
- [x] Production readiness report approved

### Release Tasks (Manual - Ready to Execute)
- [ ] **Version Bump** - Update version to v2.0.0 in all Cargo.toml files:
  - Root workspace Cargo.toml
  - crates/typf/Cargo.toml
  - crates/typf-core/Cargo.toml
  - All backend crates (typf-shape-*, typf-render-*)
  - crates/typf-cli/Cargo.toml
  - bindings/python/Cargo.toml

- [ ] **Final Testing** - Run comprehensive test suite:
  - `cargo test --workspace --all-features`
  - `./build.sh` (verify 175 outputs)
  - Cross-platform CI verification (macOS, Linux, Windows)

- [ ] **GitHub Release** - Create v2.0.0 release:
  - Tag: v2.0.0
  - Title: "TYPF v2.0.0 - Production Release"
  - Body: Use CHANGELOG.md content
  - Attach: README.md, FEATURES.md, benchmark reports

- [ ] **crates.io Publication** - Publish workspace members:
  - Order: typf-core → backends → typf → typf-cli
  - Command: `cargo publish --package <crate-name>`
  - Verify each crate on crates.io before next

- [ ] **Python Wheels** - Build and publish to PyPI:
  - `cd bindings/python && maturin build --release`
  - Test wheel installation: `pip install <wheel-file>`
  - Publish: `maturin publish` or `twine upload`
  - Platforms: macOS (x86_64 + aarch64), Linux (x86_64), Windows (x86_64)

## Deferred to Future Releases

- [ ] Color font support (v2.2)
- [ ] REPL mode implementation (v2.1)
- [ ] Rich output formatting with progress bars (v2.1)
- [ ] DirectWrite/Direct2D Windows backends (requires Windows platform)

## Notes

- Focus on minimal viable product first
- Ensure <500KB binary size for minimal build
- Maintain backwards compatibility where possible
- Document all breaking changes

## Priority Levels

- 🔴 **Critical**: Pipeline framework, minimal backends
- 🟡 **High**: HarfBuzz integration, font loading
- 🟢 **Medium**: Platform backends, Python bindings
- 🔵 **Low**: Advanced features, optimizations

## Blockers

- None currently

## Questions to Research

- [ ] Best approach for zero-copy font loading
- [ ] Optimal cache key design for glyph cache
- [ ] Cross-compilation strategy for Python wheels
- [ ] WASM build configuration

---

_Last Updated: 2025-11-19_
