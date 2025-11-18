## Description

<!-- Provide a clear and concise description of your changes -->

## Type of Change

<!-- Mark the relevant option with an 'x' -->

- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Performance improvement
- [ ] Documentation update
- [ ] Code refactoring (no functional changes)
- [ ] Build/CI changes
- [ ] Other (please describe):

## Related Issues

<!-- Link to related issues using #issue_number -->

Fixes #
Related to #

## Changes Made

<!-- List the key changes in this PR -->

-
-
-

## Testing

<!-- Describe how you tested your changes -->

### Test Environment
- **OS**: <!-- e.g., macOS 14.0, Ubuntu 22.04, Windows 11 -->
- **Rust Version**: <!-- e.g., 1.75.0 -->
- **Architecture**: <!-- e.g., x86_64, aarch64 -->

### Tests Run

- [ ] `cargo test --workspace` (all tests pass)
- [ ] `cargo test --workspace --all-features` (all features enabled)
- [ ] `cargo test --workspace --no-default-features --features minimal` (minimal build)
- [ ] New tests added for new functionality
- [ ] Existing tests updated for changed functionality

### Manual Testing

<!-- Describe any manual testing performed -->

```bash
# Example commands used for testing


```

## Performance Impact

<!-- Required for performance-related changes, optional otherwise -->

### Benchmarks Run

- [ ] No performance impact expected
- [ ] Benchmarks run and results reviewed
- [ ] Performance regression detected (justified below)
- [ ] Performance improvement measured

### Benchmark Results

<!-- If you ran benchmarks, paste relevant results here -->

```
# Before


# After


```

## Code Quality Checklist

- [ ] Code follows the style guidelines in [CONTRIBUTING.md](../CONTRIBUTING.md)
- [ ] `cargo fmt --all` ran successfully (no formatting changes needed)
- [ ] `cargo clippy --workspace --all-features -- -D warnings` passes with no warnings
- [ ] `cargo deny check` passes (dependencies audited)
- [ ] Documentation updated (if applicable)
- [ ] CHANGELOG.md updated (if user-facing changes)
- [ ] No new compiler warnings introduced
- [ ] All `unsafe` blocks have `SAFETY` comments explaining why they're safe

## Documentation

- [ ] Rustdoc comments added/updated for public APIs
- [ ] Code examples included in documentation
- [ ] README.md updated (if applicable)
- [ ] ARCHITECTURE.md updated (if architectural changes)
- [ ] Examples updated or added (if API changes)

## Security Considerations

<!-- Review the security checklist in SECURITY.md -->

- [ ] No new unsafe code (or all unsafe code is documented and justified)
- [ ] Input validation added for user-facing APIs
- [ ] No panics in library code (only in truly unrecoverable situations)
- [ ] Integer overflow protection (checked arithmetic where needed)
- [ ] Fuzz tests considered for parsing/input handling code
- [ ] No new dependencies (or dependencies reviewed and justified)

## Breaking Changes

<!-- If this is a breaking change, describe the migration path -->

### What breaks

<!-- Describe what existing code will no longer work -->

### Migration guide

<!-- Explain how users should update their code -->

```rust
// Before


// After


```

## Screenshots/Examples

<!-- If applicable, add screenshots or example output -->

## Checklist Before Requesting Review

- [ ] Self-review completed
- [ ] Code compiles cleanly on all platforms (Linux, macOS, Windows if possible)
- [ ] All tests pass locally
- [ ] Branch is up to date with main/master
- [ ] Commit messages follow conventional commits format (optional but recommended)
- [ ] No debugging code or commented-out code left in
- [ ] Feature flags used appropriately (if adding new backends/features)

## Additional Notes

<!-- Any additional context, concerns, or questions for reviewers -->

## Reviewer Notes

<!-- For reviewers: Add any notes or concerns during review -->

---

**Thank you for contributing to TYPF!** 🎉

Please ensure you've read [CONTRIBUTING.md](../CONTRIBUTING.md) before submitting.
