<!-- this_file: WORK.md -->
# Current Work Session

**Session Date:** 2026-02-11
**Version:** 5.0.2
**Focus:** Render input-source exclusivity, size-capped text ingestion, and backend-token normalization micro-sprint

## Completed

- [x] Added explicit render input-source validation to reject ambiguous multi-source combinations (`positional text`, `--text`, `--text-file`)
- [x] Switched render `--text-file` and stdin ingestion to bounded reads using shared limit helper (`MAX_TEXT_CONTENT_BYTES=1_000_000`)
- [x] Normalized render `--shaper` and `--renderer` tokens (trim + lowercase + blank→`auto`) before backend selection
- [x] Added unit and CLI smoke coverage for ambiguous text-source rejection, oversized text-file rejection, and case-insensitive backend token acceptance

## Research Notes

- Rust std I/O read-capping behavior (`Read::take`) used by shared input-limit helper:
  https://doc.rust-lang.org/std/io/trait.Read.html#method.take
- Clap argument-grouping semantics (`ArgGroup`) reviewed for input-source exclusivity tradeoffs:
  https://docs.rs/clap/latest/clap/builder/struct.ArgGroup.html

## Verification Results

- `cargo test -p typf-cli --all-features` : PASS
- `./test.sh --quick` : PASS

## Notes

- Existing unrelated repository changes were preserved.

## Next

- No active scratch tasks in this session.
