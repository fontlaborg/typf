# TypF Release Checklist

This document outlines the process for releasing a new version of TYPF.

## Pre-Release (1-2 weeks before)

### Code Freeze
- [ ] Announce code freeze in project communications
- [ ] Create release branch: `git checkout -b release/vX.Y.Z`
- [ ] Update `CHANGELOG.md` with all changes since last release
- [ ] Update version numbers in all `Cargo.toml` files
- [ ] Update version in `bindings/python/pyproject.toml`
- [ ] Update version in Python bindings `src/lib.rs` (`__version__`)

### Testing
- [ ] Run full test suite: `cargo test --workspace --all-features`
- [ ] Run tests on all platforms (macOS, Linux, Windows)
- [ ] Run minimal build test: `cargo test --workspace --no-default-features --features minimal`
- [ ] Run benchmarks: `cargo bench --workspace --all-features`
- [ ] Compare benchmark results with previous release
- [ ] Test Python bindings: `cd bindings/python && maturin develop && pytest`
- [ ] Test all examples: `cargo test --examples`
- [ ] Run examples manually to verify output
- [ ] Test CLI: `cargo run --package typf-cli -- --help`

### Documentation
- [ ] Review and update README.md
- [ ] Review and update ARCHITECTURE.md
- [ ] Generate API docs: `cargo doc --workspace --all-features --no-deps`
- [ ] Review rustdoc output for correctness
- [ ] Update migration guide (if breaking changes)
- [ ] Update Python documentation
- [ ] Check all links in documentation

### Quality Checks
- [ ] Run `cargo fmt --all -- --check`
- [ ] Run `cargo clippy --workspace --all-features -- -D warnings`
- [ ] Run `cargo deny check` (dependencies, licenses, advisories)
- [ ] Check binary size: `cargo build --release --package typf-cli --no-default-features --features minimal`
- [ ] Verify <500KB binary size for minimal build
- [ ] Run security audit: `cargo audit`
- [ ] Check for outdated dependencies: `cargo outdated`

### Performance Validation
- [ ] Run performance benchmarks
- [ ] Verify no regressions (use `scripts/bench-compare.sh`)
- [ ] Check SIMD performance targets:
  - [ ] Blending >10GB/s (release build)
  - [ ] L1 cache access <50ns
  - [ ] Simple Latin shaping <10µs/100 chars
  - [ ] Complex Arabic shaping <50µs/100 chars
- [ ] Profile memory usage for 1M character rendering (<100MB)

## Release Candidate (1 week before)

### Build Release Candidate
- [ ] Tag RC: `git tag -a vX.Y.Z-rc.1 -m "Release candidate 1"`
- [ ] Push tag: `git push origin vX.Y.Z-rc.1`
- [ ] Build Python wheels: `cd bindings/python && maturin build --release`
- [ ] Test Python wheels on all platforms
- [ ] Create draft GitHub release with RC tag

### Community Testing
- [ ] Announce RC in project communications
- [ ] Request community testing
- [ ] Collect feedback
- [ ] Fix critical bugs (create RC2 if needed)

## Final Release

### Pre-Release Checks
- [ ] All RC issues resolved
- [ ] All tests passing on CI
- [ ] Documentation reviewed and approved
- [ ] CHANGELOG.md finalized
- [ ] Version numbers confirmed in all files

### Create Release
- [ ] Merge release branch to main: `git merge release/vX.Y.Z`
- [ ] Tag release: `git tag -a vX.Y.Z -m "Release X.Y.Z"`
- [ ] Push to GitHub: `git push origin main --tags`

### Publish Packages

#### Rust Crates
- [ ] Publish crates in order (dependencies first):
  ```bash
  cd crates/typf-core && cargo publish
  cd ../typf-input && cargo publish
  cd ../typf-unicode && cargo publish
  cd ../typf-fontdb && cargo publish
  cd ../typf-export && cargo publish
  cd ../../backends/typf-shape-none && cargo publish
  cd ../typf-shape-hb && cargo publish
  cd ../typf-render-orge && cargo publish
  cd ../../crates/typf && cargo publish
  cd ../typf-cli && cargo publish
  ```
- [ ] Wait for crates.io to index
- [ ] Verify crates on crates.io

#### Python Package
- [ ] Build wheels for all platforms:
  - [ ] macOS x86_64
  - [ ] macOS ARM64
  - [ ] Linux x86_64
  - [ ] Linux ARM64
  - [ ] Windows x86_64
- [ ] Test wheels on each platform
- [ ] Publish to PyPI: `maturin publish`
- [ ] Verify package on PyPI: `pip install typf`

### GitHub Release
- [ ] Create GitHub release from tag
- [ ] Copy CHANGELOG entry to release notes
- [ ] Add migration notes if breaking changes
- [ ] Attach binary builds (Linux, macOS, Windows)
- [ ] Publish release

### Post-Release

#### Documentation
- [ ] Publish docs to GitHub Pages (if configured)
- [ ] Update docs.rs (automatic for crates.io)
- [ ] Update README.md badges (if needed)

#### Communication
- [ ] Announce release on:
  - [ ] GitHub Discussions
  - [ ] Twitter/X
  - [ ] Reddit (r/rust)
  - [ ] Discord/Slack
  - [ ] Project website
- [ ] Update project roadmap
- [ ] Close release milestone on GitHub
- [ ] Create next milestone

#### Cleanup
- [ ] Delete release branch: `git branch -d release/vX.Y.Z`
- [ ] Archive old documentation versions
- [ ] Update benchmark baseline
- [ ] Update dependency lock files

## Hotfix Release (Critical Bugs)

For critical bugs that need immediate release:

1. Create hotfix branch from tag: `git checkout -b hotfix/vX.Y.Z+1 vX.Y.Z`
2. Apply fix and update version
3. Follow abbreviated release process:
   - [ ] Run tests
   - [ ] Update CHANGELOG
   - [ ] Create tag
   - [ ] Publish packages
   - [ ] Create GitHub release
   - [ ] Announce hotfix
4. Merge hotfix back to main

## Version Numbering

Follow Semantic Versioning (semver):
- **MAJOR** (X.0.0): Breaking changes
- **MINOR** (X.Y.0): New features, backwards compatible
- **PATCH** (X.Y.Z): Bug fixes, backwards compatible

## Release Schedule

- **Major releases**: Planned, announced 1 month in advance
- **Minor releases**: Every 4-6 weeks
- **Patch releases**: As needed for bug fixes
- **Release candidates**: 1 week before major/minor releases

## Support Policy

- **Current major version**: Full support
- **Previous major version**: Security fixes for 6 months
- **Older versions**: Unsupported

## Emergency Contacts

- Release Manager: [Name]
- Security Contact: security@fontlab.com
- Infrastructure: [Team]

---

*Last Updated: 2025-11-18*
