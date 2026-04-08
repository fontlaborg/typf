## Dark Matter: Hidden Couplings

Found 20 file pairs that frequently co-change but have no import relationship:

| File A | File B | NPMI | Co-Changes | Lift |
|--------|--------|------|------------|------|
| GEMINI.md | LLXPRT.md | 1.000 | 9 | 16.44 |
| backends/typf-render-opixa/src/grayscale.rs | backends/typf-render-opixa/src/scan_converter.rs | 1.000 | 3 | 49.33 |
| backends/typf-render-vello-cpu/tests/integration.rs | backends/typf-render-vello/src/lib.rs | 1.000 | 3 | 49.33 |
| backends/typf-render-vello-cpu/tests/integration.rs | backends/typf-render-vello/tests/integration.rs | 1.000 | 3 | 49.33 |
| backends/typf-render-vello/src/lib.rs | backends/typf-render-vello/tests/integration.rs | 1.000 | 3 | 49.33 |
| docs/08-performance-fundamentals/index.html | docs/17-export-formats/index.html | 1.000 | 7 | 21.14 |
| docs/01-introduction/index.html | docs/04-installation/index.html | 1.000 | 6 | 24.67 |
| docs/01-introduction/index.html | docs/05-six-stage-pipeline/index.html | 1.000 | 6 | 24.67 |
| docs/01-introduction/index.html | docs/07-memory-management/index.html | 1.000 | 6 | 24.67 |
| docs/01-introduction/index.html | docs/09-harfbuzz-shaping/index.html | 1.000 | 6 | 24.67 |
| docs/01-introduction/index.html | docs/10-platform-shapers/index.html | 1.000 | 6 | 24.67 |
| docs/01-introduction/index.html | docs/11-icu-harfbuzz-composition/index.html | 1.000 | 6 | 24.67 |
| docs/01-introduction/index.html | docs/12-none-shaper/index.html | 1.000 | 6 | 24.67 |
| docs/01-introduction/index.html | docs/13-skia-renderer/index.html | 1.000 | 6 | 24.67 |
| docs/01-introduction/index.html | docs/15-platform-renderers/index.html | 1.000 | 6 | 24.67 |
| docs/01-introduction/index.html | docs/16-zeno-renderer/index.html | 1.000 | 6 | 24.67 |
| docs/01-introduction/index.html | docs/18-rust-api/index.html | 1.000 | 6 | 24.67 |
| docs/01-introduction/index.html | docs/19-python-api/index.html | 1.000 | 6 | 24.67 |
| docs/02-quick-start/index.html | docs/20-cli-interface/index.html | 1.000 | 7 | 21.14 |
| docs/04-installation/index.html | docs/05-six-stage-pipeline/index.html | 1.000 | 6 | 24.67 |

These pairs likely share an architectural concern invisible to static analysis.
Consider adding explicit documentation or extracting the shared concern.