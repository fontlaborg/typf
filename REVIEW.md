Typf Code Quality Review — 30 Nov 2025

Confidence: I believe (~85%); static reading only, no builds/tests executed.

Scope & Method
- Read README/PLAN/TODO plus source in typf-core, typf-unicode, typf-fontdb, typf-export, typf-export-svg, typf-cli, typf crate, Python bindings, and major backends (opixa, skia, zeno, json, none, hb). No code modifications made; target directory /Users/adam/Developer/vcs/github.fontlaborg/typf.

Executive Summary
- Architecture advertises a six-stage pipeline and multi-backend rendering, but pipeline stages are mostly stubs and the CLI bypasses them; real dataflow is shaper→renderer→exporter only.
- Two correctness blockers: SvgExporter writes PNG signature followed by raw bitmap bytes (missing IHDR/IDAT/IEND), producing invalid PNG payloads; bitmap direction analysis slices bidi levels with byte indices rather than scalar indices, so multi-byte text can mis-evaluate direction or panic. citeturn9search0turn9search1
- System-wide gaps: stub font fallback hides missing fonts; canvas sizing is heuristic and clips tall glyphs; shaping cache is unused and hashes entire font buffers; fontdb leaks allocations and ignores TTC face indices; exporters lack bounds checks; JSON/SVG outputs are unversioned; tests/CI coverage is minimal.

Strengths
- Clear trait surfaces for Shaper/Renderer/Exporter and linra path; code is readable with doc comments and example-heavy tests in several modules.
- Backends are feature-gated and mostly separated; HarfBuzz shaper exercises features/language/script paths; renderers use skrifa/tiny-skia/zeno consistently.

Blocking Defects (P0)
- SvgExporter::bitmap_to_png writes PNG magic bytes then raw RGBA, omitting required IHDR/IDAT/IEND chunks and CRCs, yielding invalid embedded PNGs. citeturn9search0
- UnicodeProcessor::create_bidi_runs slices unicode_bidi::BidiInfo::levels using byte start/end indices from char_indices; levels are per scalar value, so multi-byte characters can misalign levels or panic on slice boundaries. citeturn9search1
- typf-cli render path falls back to StubFont when no font is supplied, silently fabricating metrics and masking missing-font errors; batch reuses this path.
- Pipeline default stages (InputParsingStage, UnicodeProcessingStage, FontSelectionStage) are no-ops; Pipeline::process bypasses stages entirely, so documented six-stage contract is not honored.

Crate-by-Crate Findings
- typf-core: Pipeline::process ignores stages and runs shaper→renderer→exporter directly; PipelineBuilder default stages are empty TODOs; execute() sets backends then runs stages without validating renderer/exporter presence; traits::supports_script/format default to true, so capability checks are unreliable; context holds raw strings without validation; cache.rs uses DefaultHasher over full font/glyph buffers and never wired to backends; shaping_cache.rs hashes entire font data per call, lacks eviction metrics; linra params lack color_palette and reuse same variations list for render/shaping.
- typf-unicode: process() normalizes and segments but grapheme breaks unused; detect_scripts stores byte ranges; create_bidi_runs uses byte slicing on levels (P0); create_simple_runs duplicates text slices; options.bidi_resolve default true but no paragraph direction override; property tests do not cover mixed-script RTL with multi-byte clusters.
- typf-fontdb: Font::from_data leaks cloned buffers via Box::leak and clones again for Vec storage; always loads index 0 in TTC; advance_width rescales to 1000 UPM regardless of requested size; FontDatabase::load_font_data lacks size limits, error variants coarse; default_font silently set to first load; no validation against malformed fonts.
- typf-export: PnmExporter/JsonExporter functional but Pnm export iterates width*height without guarding short buffers; Gray1 paths can read beyond buffer; png::export_bitmap trusts dimensions and converts formats but does not clamp width*height overflow; SvgExporter invalid PNG embed (P0), base64 encoder home-grown without padding validation, external-image path hardcodes output.png.
- typf-export-svg (vector renderer): SvgRenderer recomputes skrifa FontRef per glyph and casts GlyphId to u16 (drops >65k); baseline fixed at 0.8h; padding fixed; no bbox-based canvas sizing; missing palette/variation propagation; extract_glyph_path re-parses font for each glyph.
- typf-cli: render::run bypasses typf-core Pipeline; StubFont fallback; parse_direction treats auto as LTR with no bidi detection; decode_unicode_escapes substitutes U+FFFD on malformed \\uXXXX silently; renderer selection defaults to opixa even when JSON requested unless format overrides; batch processor builds RenderArgs but lacks validation; info command advertises backends regardless of compiled features; JSON output path unimplemented but errors late.
- typf crate: re-export crate with feature gates but no doctests or integration assertions; promises six-stage pipeline not backed by tests.
- Renderers: opixa/skia/zeno compute canvas width from advances plus padding, height from advance_height heuristics; no overflow clamp aside from max_size; lack bbox/ascent/descent sizing leading to potential clipping of tall marks/emoji; JSON renderer emits prettified JSON without schema/version; render-color/OS renderers not covered by tests.
- Shapers: none shaper returns supports_script=false yet cli auto selects it; harfbuzz shaper uses 26.6 scaling but no shaping cache hookup; icu-hb not inspected deeply but shares cache gap.
- OS/backends (coretext/coregraphics/os-mac/os-win): code present but untested here; capability detection defaults optimistic; errors often mapped to TypfError::Other.
- Python bindings: always reload font from disk (no cache), ignore TTC face index, only PNM exporter exposed, vector/PNG/SVG/JSON parity missing; errors reported as strings; no GIL-safety notes for multithreaded use.

Testing & Tooling
- Unit tests cover happy paths for many small modules but miss mixed-script/bidi, exporter corruption, renderer sizing, cache behavior, fontdb leaks, CLI/Batch smoke, and feature-matrix compilation.
- No CI config committed to enforce fmt/clippy/tests; fuzz harness present but unused; golden assets under test-fonts unused.

Documentation Accuracy
- README and crate docs describe a functional six-stage pipeline, auto font discovery, JSON output, and backends that are not implemented as documented; architecture docs (ARCHITECTURE.md) absent; Python parity claims in README not matched by bindings.

Immediate Remediation Suggestions
- Replace SvgExporter::bitmap_to_png with png crate encoder or reuse PngExporter; add embed/external snapshot tests covering Gray1 and short buffers. citeturn9search0
- Rework create_bidi_runs to map unicode_bidi levels by scalar index and add fixtures for Arabic+Latin+emoji plus property test vs unicode-bidi reorder_line. citeturn9search1
- Remove StubFont fallback; plumb typf-fontdb with TTC index; fail fast on missing font; add CLI/Batch smoke tests.
- Wire Pipeline stages or simplify docs; route CLI through Pipeline; add capability guards on supports_script/supports_format.
- Compute canvas size from ascent/descent/bbox in renderers; add tall mark/emoji snapshots; propagate >16-bit glyph IDs in SVG/vector paths.
- Bound caches by ID instead of hashing full font data; expose hit/miss stats; integrate with hb/icu shapers.
- Harden exporters (bounds checks, Row-stride for Gray1/PNM); version JSON/SVG outputs.
- Add workspace CI matrix (fmt, clippy -D warnings, cargo test --workspace with feature sets) and backend/CLI/Python parity tests.

Wait, but…
- Platform backends (CT/CG/Win) not exercised here; risks inferred (~60% confidence).
- Renderer clipping inferred from sizing formulas; needs golden image confirmation.
- Cache performance concerns assume typical font sizes; benchmark before final eviction policy.
