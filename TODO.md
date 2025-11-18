# TYPF v2.0 - TODO List

## Immediate Tasks

- [ ] If you work on the 'orge' backend (the pure-Rast monochrome/grayscale rasterizer), consult the reference implementation in @./external/rasterization_reference/ ('orge' is the Rust port thereof)
- [ ] MUST-DO!!! Variable font support
- [ ] MUST-DO!!! Batch processing mode
- [ ] [orge] Port remaining Orge modules (curves, edge, scan_converter, grayscale)
- [ ] [orge] Add glyph outline extraction from skrifa
- [ ] [orge] Integrate scan converter with real
- [ ] DirectWrite shaper (Windows), Direct2D renderer (Windows) —— Windows platform backends (DirectWrite + Direct2D) require Windows platform or GitHub Actions for testing. The macOS implementation provides a complete reference pattern for the Windows backends. See @./github.fontlaborg/typf/old-typf/backends/typf-win for an OLD implementation

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
- [ ] WASM build configuration
- [ ] Cross-compilation strategy for Python wheels

---

_Last Updated: 2025-11-18_
