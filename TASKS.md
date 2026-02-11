<!-- this_file: TASKS.md -->
# Typf Plan (Index)

**Version:** 5.0.2
**Updated:** 2026-02-11
**Status:** All v5.0.2 tasks complete. Post-5.0.2 maintenance, quality-hygiene, validation, parser, and batch-hardening micro-sprints complete.

## TLDR

**v5.0.2 Complete.** All planned quality work is done:

- **Rendering**: Color fonts work on Skia/Zeno/Vello-CPU; Vello-GPU has documented limitations (upstream)
- **Stage 4/5 Contracts**: Stable C-ABI types for shaping (`PositionedGlyphC`, `ShapingResultC`) and rendering (`Vertex2D`, `VertexUV`, `VertexColor`, `RenderMesh`)
- **GPU Integration**: Zero-copy mesh upload patterns documented with `wgpu_mesh_upload.rs` example
- **Python FFI**: `ShapedGlyphs` class with Pycairo integration, path primitives, font metrics
- **Rust Integration**: `external_layout_integration.rs` shows typf as rasterization backend for cosmic-text/parley
- **Testing**: 490 tests including visual regression (21 SSIM tests covering all renderer pairs)
- **Platform Docs**: Vello-GPU platform matrix in `src_docs/26-vello-gpu-platform-support.md`

The authoritative detailed plan is split into `PLANSTEPS/` documents; `TODO.md` is the flat actionable backlog.

### Post-v5.0.2 Maintenance Sprint (2026-02-11)

- Verification entrypoint standardized with repo-root `./test.sh`
- Rust formatting checks fixed to use `cargo fmt --check` (avoids vendored Vello path breakage triggered by `--all`)
- CI lint workflow formatting check aligned with local test script

### Post-v5.0.2 JSONL Quality Sprint (2026-02-11)

- JSONL `text.features` is now parsed and validated before shaping
- JSONL job spec now accepts canonical `version` and legacy `_version`
- JSONL batch execution now runs in parallel with deterministic output ordering
- JSONL parallel execution no longer does redundant index/sort post-processing

### Post-v5.0.2 Quality Hygiene Sprint (2026-02-11)

- JSONL feature tags now enforce OpenType ASCII-byte constraints (`0x20..0x7E`)
- JSONL ordering tests now include a high-cardinality parallel regression case
- Python `render_simple` tests now assert `DeprecationWarning` explicitly (warning-clean suite)

### Post-v5.0.2 CLI Validation Micro-Sprint (2026-02-11)

- JSONL batch input now rejects unsupported API versions (major `2.x` required)
- JSONL `text.direction` now validates allowed values (`ltr|rtl|ttb|btt`) instead of silently defaulting
- Render CLI OpenType feature parsing now enforces exactly 4 printable ASCII tag characters

### Post-v5.0.2 Parser Delimiter Micro-Sprint (2026-02-11)

- Render CLI token parsing now accepts mixed comma/tab/newline separators for OpenType features
- Render CLI variation-axis parsing now accepts mixed comma/tab/newline separators
- Glyph-source list parsing now accepts mixed comma/tab/newline separators

### Post-v5.0.2 Batch Hardening Micro-Sprint (2026-02-11)

- Batch command now validates output filename pattern requires `{}` placeholder
- Batch command now rejects unsafe output paths (`..`, absolute paths, missing file name) outside `--output`
- Batch command now rejects unsupported `format` values and unknown JSON fields in batch jobs

## Plan Steps (authoritative details)

1. `PLANSTEPS/01-rendering-quality-status.md`
2. `PLANSTEPS/02-external-ecosystems.md`
3. `PLANSTEPS/03-api-extension-typf-core.md`
4. `PLANSTEPS/04-api-extension-typfpy.md`
5. `PLANSTEPS/05-integration-recipes.md`
6. `PLANSTEPS/06-color-font-integration.md`
7. `PLANSTEPS/07-architecture-thesis.md`
8. `PLANSTEPS/08-rust-ecosystem-integration.md`
9. `PLANSTEPS/09-python-ecosystem-and-api-amendments.md`

## Execution

- Action items live in `TODO.md`.
- v5.0.2 execution completed: baseline consistency → Stage 4/5 contracts → Rust/Python integrations → visual regression testing.
- SDF is explicitly out of scope for v5.x.
