# TYPF API Stability Policy

## Overview

This document defines the API stability guarantees for TYPF, including versioning policy, deprecation process, and compatibility commitments. TYPF follows Semantic Versioning 2.0.0 to provide clear expectations for API changes.

## Semantic Versioning

TYPF uses [Semantic Versioning 2.0.0](https://semver.org/) with version numbers in the format:

```
MAJOR.MINOR.PATCH
```

- **MAJOR**: Incompatible API changes
- **MINOR**: Backwards-compatible functionality additions
- **PATCH**: Backwards-compatible bug fixes

### Pre-release Versions

Pre-release versions are marked with suffixes:

```
0.8.0-alpha.1   # Alpha release
0.8.0-beta.2    # Beta release
0.8.0-rc.1      # Release candidate
```

**Note**: APIs in pre-release versions (0.x.y) may change without notice.

## API Categories

### 1. Stable APIs ✅

These APIs are covered by stability guarantees:

#### Rust API
```rust
// Core types and traits
pub struct FontEngine { ... }
pub trait Backend { ... }
pub struct Font { ... }
pub struct RenderConfig { ... }

// Primary functions
pub fn render(text: &str, font: &Font) -> Result<Bitmap>
pub fn load_font(path: &Path) -> Result<Font>
pub fn list_system_fonts() -> Vec<FontInfo>
```

#### C API
```c
// Core functions
TypfEngine* typf_engine_create(const char* backend);
TypfBitmap* typf_render(TypfEngine*, const char*, const char*);
void typf_engine_free(TypfEngine*);
void typf_bitmap_free(TypfBitmap*);
```

#### Python API
```python
# Core classes
class Engine:
    def render(self, text: str, font: str) -> Bitmap
    def set_backend(self, backend: str) -> None

class Font:
    def load(path: str) -> Font
    def metrics() -> FontMetrics
```

### 2. Experimental APIs ⚠️

These APIs are subject to change:

- GPU rendering functions
- WebAssembly bindings
- Streaming render APIs
- Custom shaper interfaces

Experimental APIs are marked with:
```rust
#[doc(hidden)]
#[cfg(feature = "experimental")]
pub fn experimental_feature() { ... }
```

### 3. Internal APIs ❌

Not covered by stability guarantees:

- Any items in `internal` or `private` modules
- Functions/types not marked `pub`
- Test utilities
- Benchmark harnesses

## Version Compatibility Matrix

| TYPF Version | Rust Version | Python Version | ABI Compatibility |
|-------------|--------------|----------------|-------------------|
| 0.8.x       | 1.70+        | 3.8+           | C ABI v1          |
| 0.9.x       | 1.70+        | 3.8+           | C ABI v1          |
| 1.0.x       | 1.75+        | 3.9+           | C ABI v2          |

## Breaking Change Policy

### What Constitutes a Breaking Change

#### Breaking (Requires Major Version Bump):
- Removing a public function, type, or trait
- Changing function signatures (parameters or return types)
- Changing trait requirements
- Removing enum variants
- Changing struct field visibility or types
- Changing error types in `Result`

#### Non-Breaking (Minor Version):
- Adding new public functions
- Adding new types or traits
- Adding enum variants (if marked `#[non_exhaustive]`)
- Adding default trait methods
- Adding new feature flags
- Performance improvements

#### Bug Fixes (Patch Version):
- Fixing incorrect behavior
- Security patches
- Documentation updates
- Internal refactoring

### Deprecation Process

1. **Announcement** (Minor Version)
   ```rust
   #[deprecated(since = "0.9.0", note = "Use `new_function` instead")]
   pub fn old_function() { ... }
   ```

2. **Migration Period** (At least 2 minor versions)
   - Deprecated items remain functional
   - Migration guide provided in changelog
   - Warnings emitted at compile time

3. **Removal** (Next Major Version)
   - Deprecated items removed
   - Breaking change documented in release notes

## Minimum Supported Versions

### Rust (MSRV)

Current MSRV: **1.70.0**

- MSRV changes are considered minor version bumps
- 6-month deprecation notice for MSRV increases
- CI tests against MSRV and latest stable

### Python

Minimum Python: **3.8**

- Follows Python EOL schedule
- 3-month notice before dropping Python version

### Operating Systems

Tier 1 Support (CI tested):
- Ubuntu 20.04+
- macOS 11+
- Windows 10+

Tier 2 Support (best effort):
- Older OS versions
- BSD variants
- ARM architectures

## Feature Stability

### Stable Features

Always available:
```toml
[features]
default = ["backend-orge"]
backend-orge = []     # ✅ Stable
c-api = []           # ✅ Stable
python = []          # ✅ Stable
```

### Optional Features

May require additional dependencies:
```toml
backend-harfbuzz = []  # ✅ Stable
backend-coretext = []  # ✅ Stable (macOS only)
backend-skia = []      # ⚠️ Experimental
```

## Migration Guides

### Upgrading from 0.7.x to 0.8.x

```rust
// Before (0.7.x)
let engine = FontEngine::new(Backend::Default);

// After (0.8.x)
let engine = FontEngine::new(Backend::Orge);  // 'Default' renamed to 'Orge'
```

### Upgrading from 0.8.x to 0.9.x

```rust
// Before (0.8.x)
engine.render(text, font)

// After (0.9.x)
engine.render(text, font, config)  // Added optional config parameter
```

## API Lifecycle

```
Experimental → Preview → Stable → Deprecated → Removed
     ↓           ↓         ↓          ↓           ↓
   0.x.0      0.x.0     1.0.0      1.x.0       2.0.0
```

## Compatibility Testing

### Test Matrix

We test compatibility across:
- Last 3 Rust stable releases
- All supported Python versions
- All Tier 1 operating systems
- All stable backend combinations

### Integration Tests

```bash
# Run compatibility tests
cargo test --features compat-tests

# Test C ABI compatibility
./scripts/test-c-abi.sh

# Test Python bindings
pytest tests/test_api_stability.py
```

## Long-term Support (LTS)

### LTS Releases

Starting with 1.0.0, every 3rd minor version is LTS:
- 1.0.x - LTS (2 years)
- 1.3.x - LTS (2 years)
- 1.6.x - LTS (2 years)

LTS versions receive:
- Security patches
- Critical bug fixes
- No new features
- No API changes

## Reporting Compatibility Issues

### Bug Reports

File issues at: https://github.com/fontlaborg/typf/issues

Include:
- TYPF version
- Platform details
- Minimal reproduction
- Expected vs actual behavior

### Security Issues

Report security issues privately to: security@fontlab.com

## Version Support Timeline

| Version | Release Date | End of Support | Status      |
|---------|-------------|----------------|-------------|
| 0.7.x   | 2025-09-01  | 2025-12-01     | Deprecated  |
| 0.8.x   | 2025-11-01  | 2026-05-01     | Current     |
| 0.9.x   | 2026-02-01  | 2026-08-01     | Planned     |
| 1.0.x   | 2026-05-01  | 2028-05-01     | LTS Planned |

## Ecosystem Compatibility

### FontSimi Integration

TYPF maintains compatibility with FontSimi:

| TYPF Version | Compatible FontSimi Versions |
|-------------|----------------------------|
| 0.8.x       | 2.3.x, 2.4.x               |
| 0.9.x       | 2.4.x, 2.5.x               |
| 1.0.x       | 3.0.x+                     |

### Package Managers

Published to:
- crates.io (Rust)
- PyPI (Python)
- npm (WebAssembly) - planned

## API Stability Checklist

Before each release:

- [ ] Run `cargo semver-checks`
- [ ] Check C ABI compatibility
- [ ] Update Python type stubs
- [ ] Run integration test suite
- [ ] Update migration guide
- [ ] Document breaking changes
- [ ] Update compatibility matrix
- [ ] Tag with appropriate version

## FAQ

### Q: When will 1.0 be released?
A: Target Q2 2026, after comprehensive field testing.

### Q: Can I use TYPF in production with 0.x versions?
A: Yes, but be prepared for API changes between minor versions.

### Q: How do I know which APIs are stable?
A: Check the documentation - stable APIs are marked with ✅.

### Q: Will you backport fixes to older versions?
A: Only security fixes for LTS versions.

### Q: How much notice for breaking changes?
A: Minimum 2 minor versions (typically 6 months).

---

**Policy Version**: 1.0.0
**Last Updated**: November 17, 2025
**Next Review**: February 2026

For questions about this policy, contact the maintainers at typf@fontlab.com.