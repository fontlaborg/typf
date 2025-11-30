# Releasing typf

This document describes how to release a new version of typf.

## Prerequisites

- Rust toolchain (stable)
- `cargo-edit` installed: `cargo install cargo-edit`
- `uv` for Python environment: `curl -LsSf https://astral.sh/uv/install.sh | sh`
- `maturin` installed: `uv tool install maturin` or `uvx maturin`
- Repository secrets configured in GitHub:
  - `CRATES_IO_TOKEN` - API token for crates.io
  - `PYPI_API_TOKEN` - API token for PyPI (or trusted publisher configured)

## Quick Release

```bash
# 1. Ensure you're on main with clean working directory
git checkout main
git pull origin main
git status  # Should be clean

# 2. Update version from git tag
./scripts/set-version.sh 2.4.0

# 3. Verify build
./scripts/test.sh

# 4. Commit version bump
git add -A
git commit -m "v2.4.0"

# 5. Create and push tag
git tag v2.4.0
git push origin main --tags
```

The tag push triggers GitHub Actions which will:
1. Build binaries for 8 platforms (Linux/macOS/Windows, x64/arm64)
2. Build Python wheels for 40+ configurations
3. Create a GitHub release with all artifacts
4. Publish to crates.io (in dependency order)
5. Publish to PyPI

## Manual Release (Local)

For testing or when CI is unavailable:

```bash
# Dry run (no actual publishing)
./scripts/publish.sh --dry-run

# Publish only to crates.io
./scripts/publish.sh --crates

# Publish only to PyPI
./scripts/publish.sh --pypi

# Full publish (requires tokens)
CRATES_IO_TOKEN=xxx PYPI_API_TOKEN=xxx ./scripts/publish.sh
```

## Version Format

We use semantic versioning: `MAJOR.MINOR.PATCH`

- **MAJOR**: Breaking API changes
- **MINOR**: New features, backward compatible
- **PATCH**: Bug fixes, backward compatible

Prerelease versions: `2.4.0-alpha.1`, `2.4.0-beta.1`, `2.4.0-rc.1`

## Publishing Order

Crates are published in dependency order with 30s delays:

```
Layer 0: typf-core, typf-unicode
Layer 1: typf-fontdb, typf-input, typf-export, typf-shape-none,
         typf-render-opixa, typf-render-json, typf-render-svg
Layer 2: typf-export-svg, typf-shape-hb, typf-render-color,
         typf-render-zeno, typf-render-skia, typf-os
Layer 3: typf-shape-icu-hb, typf-shape-ct, typf-render-cg,
         typf-os-mac, typf-os-win
Layer 4: typf
Layer 5: typf-cli, typf-bench
```

## Platform Matrix

### Rust Binaries

| Target | OS | Arch |
|--------|-----|------|
| x86_64-unknown-linux-gnu | Linux | x64 |
| x86_64-unknown-linux-musl | Linux | x64 (static) |
| aarch64-unknown-linux-gnu | Linux | arm64 |
| aarch64-unknown-linux-musl | Linux | arm64 (static) |
| x86_64-apple-darwin | macOS | x64 |
| aarch64-apple-darwin | macOS | arm64 |
| x86_64-pc-windows-msvc | Windows | x64 |
| aarch64-pc-windows-msvc | Windows | arm64 |

### Python Wheels

- Python: 3.9, 3.10, 3.11, 3.12, 3.13
- Platforms: manylinux, musllinux, macOS, Windows
- Architectures: x86_64/amd64, aarch64/arm64

## Troubleshooting

### crates.io publish fails

- Wait 30-60s between dependent crates
- Check if crate already exists at that version
- Verify `CRATES_IO_TOKEN` is valid

### PyPI publish fails

- Check if wheel already exists at that version
- Verify trusted publisher is configured or `PYPI_API_TOKEN` is valid
- Check wheel filename format

### Version mismatch

Run `./scripts/set-version.sh` to sync all manifests:
```bash
./scripts/set-version.sh 2.4.0
cargo check --workspace  # Verify
```

## Scripts Reference

| Script | Purpose |
|--------|---------|
| `scripts/set-version.sh` | Sync version from git tag or argument |
| `scripts/build.sh` | Build Rust workspace + Python wheel |
| `scripts/test.sh` | Run fmt, clippy, tests, pytest |
| `scripts/publish.sh` | Publish to crates.io and PyPI |
