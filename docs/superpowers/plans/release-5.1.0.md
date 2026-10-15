<!-- this_file: docs/superpowers/plans/release-5.1.0.md -->
# Release Typf 5.1.0

## Global Constraints

- Publish the current project as version 5.1.0 on PyPI under `typf` and all publishable Typf workspace crates on crates.io.
- Support standard CPython 3.11 and higher using prebuilt wheels for Linux glibc and musl (x86_64/aarch64), macOS (x86_64/arm64), and Windows x86_64. Verify Python 3.11, 3.12, 3.13, and 3.14.
- Preserve current functionality and the existing `typfpy` Python import as compatibility where feasible. `import typf` must work.
- Never expose credentials, private logs, machine paths, or development scratch files in public artifacts.
- Preserve untracked `.codegraph/`. No unrelated review/refactoring work. Publication and Git pushes are authorized by the release objective.
- Implementation and review use at most one child agent concurrently. Children must not spawn agents.

## Task 1: Prepare release source, packaging and automation

Own all required release code and documentation changes except this plan and `.omc/` or `.superpowers/` coordination artifacts. You are not alone in the repository; preserve other edits. Work on the existing release branch. Do not push, tag, or publish.

1. Align workspace package versions, internal dependency versions, lockfiles, active installation docs, and Python metadata at 5.1.0. Do not rewrite historical changelog entries or unrelated package versions.
2. Publish Python distribution as `typf`; make `import typf` work while retaining `typfpy` compatibility. Set Python >=3.11, classifiers through 3.14, and stable ABI `abi3-py311` where compatible. Use official PyO3/maturin documentation before changes. Avoid unnecessary binding rewrites.
3. Repair build/release automation: build from `bindings/py`, fail on version mismatches and publish failures, cover all workspace crates in dependency order, and produce audited wheels/sdist. Add explicit manual build-only execution so CI wheels can be downloaded and published locally if trusted publishing is unavailable. Modern supported runners, no obsolete macos-13. Do not depend on unavailable CI secrets for build-only execution.
4. Remove dependency on untracked `external/` sources for release builds. Match the Vello revision with published upstream crates or an appropriate distributable source strategy; preserve backends. All path dependencies to publishable packages need version requirements. Inspect metadata, dry-run packaging, platform cfgs, and source payload.
5. Add focused regression checks for the release contract and meaningful smoke rendering. Run relevant Rust tests, formatting and clippy, Python tests on a built wheel, and release validators. Start by checking the baseline Rust test process/log `/tmp/typf-baseline-test.log` (the controller owns that process).
6. Update CHANGELOG.md, WORK.md and relevant install documentation with exact results and limitations. Commit scoped changes, report tests and unresolved concerns to `.superpowers/sdd/release-5.1.0/task-1-report.md`. Return short status and commit IDs.

## Task 2: Verify and publish

Controller owns credentials, release delivery and live checks. Review the Task 1 diff independently before publication; fix issues through the implementer and re-review. Run build-only CI, inspect all artifacts and smoke tests, publish all crates in dependency order and upload wheels/sdist to PyPI with available credentials without printing them. Push release source and tag, verify exact remote commit and registry versions. Verify wheel-only clean installation on Python 3.11 through 3.14, platform coverage and published SHA-256 hashes. Mark the goal complete only after all requirements have fresh authoritative evidence.
