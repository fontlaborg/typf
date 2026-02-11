<!-- this_file: TASKS.md -->
# Typf Plan (Index)

**Version:** 5.0.2
**Updated:** 2026-02-11
**Status:** All v5.0.2 tasks complete. Post-5.0.2 maintenance sprints complete.

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
