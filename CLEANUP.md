# Repository Cleanup Plan (2025-11-30)

## Scope & Guardrails
- Keep: core crates/backends/bindings, tests/fuzz/benches/examples, `docs` + `src_docs`, fixtures in `test-fonts`/`benchmark-fonts`/`typf-tester` inputs, CI/workflow + build scripts.
- Remove: generated build products, cached tooling state, local virtualenvs, OS/editor cruft, benchmark/test output artifacts, vendored reference copies that duplicate crates.io sources.
- Do **not** delete test fixtures or documentation sources; verify any ambiguous file before removal.

## Proposed Cleanup Actions (no deletions run yet)
1) Build artifacts & virtualenvs
- Drop `target/` (9.7G, includes wheels, incremental caches, flycheck) via `cargo clean` + `rm -rf target`.
- Remove local envs: `.venv/`, `bindings/python/.venv/`, `typf-tester/.venv/`; reinstall with `uv venv` only when needed.
- Purge Python caches: `typf-tester/.mypy_cache/`, `typf-tester/__pycache__/`.

2) Generated outputs & logs
- Delete rendering outputs and reports: `typf-tester/output/**` (PNGs/SVG/JSON + summaries).
- Remove stray sample outputs: `examples/output.ppm`, `crates/typf/examples/output/test.ppm`, root `test.ppm`.
- Remove runtime/bench logs: `typf-bench-level1.log`, `typf-bench-quick.log`, `test_output.log`, `test_output.txt`, `test_output.log` (duplicates), `target/*.log` if any regenerate.
- Keep input specs (`test_batch_spec.json`, `test_jobs.jsonl`) and Rust test sources (`test_ops_calculation.rs`).

3) Tooling/OS cruft
- Delete `.cache/**` numeric entries, `.DS_Store` copies across repo, `.git/.DS_Store`, `.github/.DS_Store`, `.venv/.DS_Store`, `.claude/**`, `.cursorrules`, `.git/.smbdelete*`.
- Add/confirm `.gitignore` rules for `.DS_Store`, `.cache/`, `.venv/`, `target/`, `typf-tester/output/`, `*.log`, `*.ppm`, `*.whl` to prevent reappearance.

4) Vendored reference repositories
- The `external/` tree (≈1.3G) mirrors upstream projects for reference only (see `external/README.md`). Remove or convert to shallow submodules; rely on crates.io/git dependencies instead. Highlight `external/old-typf/` (36M) as legacy code ready for deletion after confirming no docs/tests reference it.

5) Miscellaneous generated assets
- Clear built wheels in `target/wheels/` once published or archived elsewhere.
- Remove empty placeholder `examples/output/` directory unless used by scripts; adjust scripts to create it on demand.
- Verify `llms.sh` / `llms.txt` are needed; if only ad-hoc experimentation, archive outside repo or delete.

## Validation Checklist (post-cleanup)
- Run `cargo test` and `cargo fmt --check` to ensure codebase unaffected.
- Run `uvx ruff check --fix .` and `pytest` inside `typf-tester` if bindings are kept.
- Rebuild wheels as needed via documented release process.
- Update `WORK.md` with actions taken and `CHANGELOG.md` if user-facing artifacts removed.
