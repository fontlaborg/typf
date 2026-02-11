Top risks
- Formatting and linting gates are failing, which can block CI/merge and hide real regressions.
- Rust style/Clippy warnings may indicate correctness or API misuse issues that haven’t been enforced.
- Sanity suite is the only failing category, so quick “green” signals (smoke/unit) may be misleading.

Probable root causes
- `rustfmt`/`cargo fmt` mismatch with repo config (rust-toolchain, config file, or version drift).
- `clippy` warnings newly introduced or newly elevated to deny/warn by config.
- Toolchain inconsistency between local and CI (nightly vs stable, feature flags).

Concrete next actions
- Run locally: `cargo fmt --all -- --check` and `cargo clippy --all-targets --all-features -D warnings` to reproduce.
- Verify toolchain: check `rust-toolchain.toml` and ensure local matches CI; update if needed.
- Fix or suppress only justified Clippy lints; avoid blanket `allow` unless documented.
- Re-run sanity tests only to confirm: the two failed checks should go green.

Confidence: I’m certain.