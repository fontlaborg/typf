<!-- this_file: WORK.md -->
# Current Work Session

**Session Date:** 2026-02-11
**Version:** 5.0.2
**Focus:** Post-v5.0.2 maintenance sprint (verification workflow)

## Sprint Tasks

- [x] Fix Rust formatting check path in `scripts/test.sh` by using `cargo fmt --check`
- [x] Align CI lint formatting command in `.github/workflows/ci.yml`
- [x] Add repo-root `./test.sh` wrapper as stable test entrypoint

## Verification Results

- `bash -n scripts/test.sh`: PASS
- `bash -n test.sh`: PASS
- `cargo fmt --check`: PASS
- `cargo clippy --workspace --all-features -- -D warnings`: PASS
- `./scripts/test.sh --rust`: PASS
- `./test.sh --rust --lint`: PASS

## Notes

- Standardized formatting command is `cargo fmt --check`.
- `cargo fmt --all --check` is no longer used in local/CI verification because the `--all` mode traverses vendored Vello workspace metadata and fails in this repository snapshot.

## Next

- No open items in this sprint.
