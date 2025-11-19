# TYPF v2.0 - TODO List

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
- [ ] Orge rasterizer quality - Known limitation, produces functional but slightly rough output
- [ ] Don't silently fall back to other renderers if the primary renderer doesn't support the requested output format, or if something fails. 

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

## Deferred later issues

- [ ] Color font support
- [ ] REPL mode implementation (connect to rendering pipeline)
- [ ] Rich output formatting with progress bars

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
