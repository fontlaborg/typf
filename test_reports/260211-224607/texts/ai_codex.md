Top risks  
- CI is red with 7 required failures; merges are unsafe and releases are blocked.  
- Core build + tests (CLI build, unit + integration + doc tests) failing means basic correctness and API integrity are unverified.  
- Formatting + clippy failing implies code quality gates are broken and may hide real regressions.

Probable root causes  
- Recent changes broke Rust build or tests; clippy/fmt failures suggest style or lint violations in new/modified code.  
- Test discovery/listing failure hints at misconfigured test targets, missing features, or broken workspace setup.  
- Doc tests failing often indicate outdated docs or example code drift.

Concrete next actions  
1) Run the failing checks locally in order: `smoke_build_cli`, `sanity_fmt`, `sanity_clippy`, `sanity_list_tests`, then `unit_*` to capture first error.  
2) Fix format and clippy first; re-run to reduce noise.  
3) If `sanity_list_tests` fails, validate workspace config (Cargo.toml, features, test targets) before fixing tests.  
4) After build passes, fix unit/integration/doc tests; update docs/examples if doctests are stale.  
5) Re-run full suite and confirm all required failures are cleared.

Confidence: I’m certain about the priority/risk; I believe about the likely root causes (need logs to be sure).