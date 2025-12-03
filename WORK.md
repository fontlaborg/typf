# Current Work Session

## Session: Dec 3, 2025 - Benchmark Integration

### Completed

1. **Vello Benchmark Integration** ✓
   - Added `typf-render-vello-cpu` and `typf-render-vello` to typf-bench
   - Feature flags: `render-vello-cpu`, `render-vello`
   - Successfully ran benchmark with all 4 renderers

### Benchmark Results Summary (24px, 20 chars)

| Renderer | Standard Fonts | Color Fonts (SVG) | Notes |
|----------|---------------|-------------------|-------|
| **Opixa** | 230-280 μs | 350-440 μs | Fastest for standard text |
| **Vello CPU** | 500-1000 μs | 700-750 μs | Good color font handling |
| **Vello GPU** | ~10 ms | ~10-14 ms | GPU sync overhead dominates |
| **Skia** | 900-1500 μs | 700+ ms! | Very slow for SVG fonts |

**Key Insights:**
- Opixa remains fastest for standard text rendering (2-4x faster than alternatives)
- Vello CPU handles color fonts better than Skia
- Vello GPU overhead is constant (~10ms) - better suited for batch/large renders
- Skia has severe performance issues with SVG color fonts

### Test Summary

| Backend | Unit Tests | Integration Tests | Total |
|---------|------------|-------------------|-------|
| Vello CPU | 4 | 12 | 16 |
| Vello GPU | 3 | 12 | 15 |
| **Workspace** | - | - | **378** |

### Next Up

- Phase S.1: SDF Core & CPU Renderer (medium priority)
- Add Math font (STIX2Math) test to Vello backends
