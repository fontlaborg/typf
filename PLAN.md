<!-- this_file: PLAN.md -->
# Typf Plan (Index)

**Version:** 5.0.1  
**Updated:** 2025-12-16  
**Status:** Critical rendering issues resolved; remaining work is mainly integration/interop and low-priority follow-ups.

## TLDR

- Rendering backends are in a good state for color fonts on Skia/Zeno/Vello-CPU; Vello-GPU color fonts remain an upstream limitation.
- The remaining quality work is about consistency (baselines), clearer contracts (Stage 4/5 interop), and targeted integrations (Rust layout engines, Python FFI).
- The authoritative detailed plan is split into `PLANSTEPS/` documents; `TODO.md` is the flat actionable backlog.

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
- Preferred execution order is: baseline consistency → clarify shaped/geometry contracts → targeted Rust/Python integrations → optional SDF/platform expansion.
