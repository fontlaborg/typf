# Security Policy

## Supported Versions

| Version | Supported          | Notes |
| ------- | ------------------ | ----- |
| 2.0.x   | :white_check_mark: | Current version, full support |
| 1.x.x   | :x:                | End of life, no updates |

Security fixes go to current minor version only. Run the latest stable release.

## Reporting a Vulnerability

Don't report security issues in public GitHub issues.

Email **security@fontlab.com** with:

1. Vulnerability type (buffer overflow, code injection, DoS)
2. Affected component (shaping backend, renderer, font loader)
3. Steps to reproduce
4. Proof of concept code (if available)
5. Potential impact
6. Suggested fix (if you have one)

### Timeline

- **Acknowledgment**: 48 hours
- **Initial assessment**: 1 week
- **Status updates**: Weekly until resolved
- **Fix timeline**: Critical: 2 weeks, High: 4 weeks

### Disclosure

We coordinate disclosure:

1. Acknowledge within 48 hours
2. Investigate and assess within 1 week
3. Work on fix with updates
4. Coordinate disclosure timing with you
5. Credit you (unless you prefer anonymity)

Allow 90 days to fix before public disclosure.

## Security Risks

### 1. Font File Parsing

Malformed font files can cause crashes, hangs, or memory corruption.

**How we protect**:
- Font parsing uses tested libraries (`read-fonts`, `skrifa`)
- Bounds checking on all array accesses
- Fuzzing with `cargo-fuzz` on font loading paths
- Memory-safe Rust prevents buffer overflows

**What you should do**:
- Only load fonts from trusted sources
- Validate font files before loading in production
- Use sandboxing when processing user-uploaded fonts

### 2. Text Input Handling

Malicious text input can cause denial of service or exploit Unicode edge cases.

**How we protect**:
- Unicode normalization (NFC) before processing
- Input length limits enforced
- Timeout protection for complex script shaping
- Stack overflow protection via tail recursion elimination

**What you should do**:
- Sanitize user input before rendering
- Implement rate limiting for text rendering endpoints
- Set maximum text length for user-provided content

### 3. Memory Safety

Risk: Memory corruption, use-after-free, data races.

**How we protect**:
- Rust's ownership system prevents most memory safety issues
- All `unsafe` code is documented and audited
- Thread-safe data structures (`Arc`, `DashMap`) for concurrency
- Miri testing for unsafe code validation

**Unsafe code audit**:
```bash
# Find all unsafe blocks
rg "unsafe" --type rust --stats

# Current count: <10 unsafe blocks (as of v2.0.0)
# All unsafe code is in SIMD intrinsics with documented safety invariants
```

### 4. Dependency Security

Risk: Vulnerabilities in third-party dependencies.

**How we protect**:
- `cargo-audit` runs in CI on every commit
- `cargo-deny` checks for vulnerable, unmaintained, or banned dependencies
- Minimal dependency tree (prefer std library)
- Security advisories monitored via RustSec

**Dependency auditing**:
```bash
# Check for known vulnerabilities
cargo audit

# Check dependency policies
cargo deny check

# List all dependencies
cargo tree --depth 1
```

### 5. SIMD Safety

Risk: Incorrect SIMD usage can cause UB or crashes on unsupported CPUs.

**How we protect**:
- Runtime CPU feature detection (no assumptions)
- Scalar fallbacks for all SIMD code
- Alignment guarantees for SIMD loads/stores
- SIMD code gated by `#[target_feature]`

**Example safe SIMD usage**:
```rust
#[target_feature(enable = "avx2")]
unsafe fn blend_avx2(src: &[u8], dst: &mut [u8]) {
    // SAFETY: Function can only be called if AVX2 is available
    // due to target_feature guard and runtime check
    // ...
}

// Runtime dispatch
if is_x86_feature_detected!("avx2") {
    unsafe { blend_avx2(src, dst) }
} else {
    blend_scalar(src, dst)  // Safe fallback
}
```

### 6. Integer Overflow

Risk: Integer overflows in size calculations can lead to buffer overruns.

**How we protect**:
- Debug builds have overflow checks enabled
- Checked arithmetic in critical paths
- Saturating operations where appropriate
- Fuzzing for edge cases

**Example safe arithmetic**:
```rust
// Instead of: let size = width * height;
let size = width.checked_mul(height)
    .ok_or(TypfError::InvalidDimensions)?;
```

### 7. Resource Exhaustion

Risk: Malicious input can cause excessive memory or CPU usage.

**How we protect**:
- Cache size limits (configurable)
- Timeouts for long-running operations
- Memory pool bounds
- LRU eviction prevents unbounded growth

**What you should do**:
```rust
use typf::CacheConfig;

// Set cache limits to prevent DoS
let config = CacheConfig::builder()
    .max_shaped_entries(1000)
    .max_glyph_entries(10_000)
    .max_memory_mb(100)
    .build();

let typf = Typf::with_config(config);
```

### 8. Platform-Specific APIs

Risk: Platform APIs (CoreText, DirectWrite) may have security issues.

**How we protect**:
- Minimal surface area (only use required APIs)
- Error handling for all platform calls
- No elevated privileges required
- Sandboxing-compatible (no private APIs)

### 9. FFI Safety (Python Bindings)

Risk: Incorrect FFI can cause crashes or undefined behavior.

**How we protect**:
- PyO3 provides memory-safe FFI abstractions
- All panics are caught at FFI boundary
- GIL management prevents data races
- Input validation on all FFI entry points

**Example panic safety**:
```rust
#[pyfunction]
fn render_text(text: &str) -> PyResult<Vec<u8>> {
    // PyO3 automatically catches panics and converts to Python exceptions
    std::panic::catch_unwind(|| {
        // Rust code that might panic
    }).map_err(|_| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
        "Internal error"
    ))
}
```

### 10. WASM Security

Risk: WASM builds may have different security characteristics.

**How we protect**:
- No file system access in WASM builds
- No network access
- Memory limits enforced by runtime
- No `unsafe` in WASM-specific code

## Security Limitations

1. **No sandboxing**: TypF runs in the same process as the caller
   - Use OS-level sandboxing for untrusted input

2. **No font signature verification**: We don't verify font authenticity
   - Implement your own signature checking

3. **Limited DoS protection**: Complex text can be slow
   - Implement timeouts and rate limiting

4. **Cache poisoning**: Shared caches can be poisoned with malicious entries
   - Use separate cache instances for untrusted input

## Security Best Practices

### 1. Validate Input

```rust
fn render_user_text(text: &str) -> Result<Vec<u8>> {
    // 1. Check length
    if text.len() > 10_000 {
        return Err(TypfError::TextTooLong);
    }

    // 2. Sanitize (remove control characters)
    let sanitized: String = text.chars()
        .filter(|c| !c.is_control() || c.is_whitespace())
        .collect();

    // 3. Render with timeout
    let typf = Typf::new();
    typf.render(&sanitized, ...)
}
```

### 2. Isolate Font Loading

```rust
use std::time::Duration;

fn load_user_font(path: &Path) -> Result<Arc<Font>> {
    // 1. Check file size
    let metadata = std::fs::metadata(path)?;
    if metadata.len() > 50_000_000 {  // 50MB limit
        return Err(TypfError::FontTooLarge);
    }

    // 2. Load with timeout (use separate thread/process)
    let font = Font::from_file(path)?;

    // 3. Validate basic properties
    if font.units_per_em() == 0 {
        return Err(TypfError::InvalidFont);
    }

    Ok(Arc::new(font))
}
```

### 3. Rate Limiting

```rust
use std::time::{Duration, Instant};
use std::collections::HashMap;

struct RateLimiter {
    requests: HashMap<String, Vec<Instant>>,
    max_per_minute: usize,
}

impl RateLimiter {
    fn check(&mut self, user_id: &str) -> bool {
        let now = Instant::now();
        let cutoff = now - Duration::from_secs(60);

        let requests = self.requests.entry(user_id.to_string())
            .or_insert_with(Vec::new);

        // Remove old requests
        requests.retain(|&time| time > cutoff);

        // Check limit
        if requests.len() >= self.max_per_minute {
            return false;
        }

        requests.push(now);
        true
    }
}
```

### 4. Separate Cache Instances

```rust
// DON'T: Share cache between trusted and untrusted input
let global_typf = Typf::new();  // Shared cache

// DO: Separate cache instances
let trusted_typf = Typf::new();
let untrusted_typf = Typf::new();

// Or: Clear cache after processing untrusted input
typf.clear_cache();
```

## Security Testing

### 1. Fuzzing

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Run font loading fuzzer
cargo fuzz run font_loading

# Run text shaping fuzzer
cargo fuzz run text_shaping

# Run renderer fuzzer
cargo fuzz run rendering
```

### 2. Static Analysis

```bash
# Clippy with security lints
cargo clippy --all-features -- -D warnings -W clippy::all

# Check for unsafe code
cargo geiger

# Dependency audit
cargo audit
```

### 3. Miri (UB Detection)

```bash
# Install Miri
rustup +nightly component add miri

# Run tests with Miri
cargo +nightly miri test
```

### 4. Address Sanitizer

```bash
# Build with address sanitizer
RUSTFLAGS="-Z sanitizer=address" cargo +nightly test

# Memory sanitizer
RUSTFLAGS="-Z sanitizer=memory" cargo +nightly test
```

## Security Checklist

Before submitting security-sensitive code:

- [ ] All `unsafe` blocks have `SAFETY` comments
- [ ] Bounds checks on all array accesses
- [ ] Integer overflow protection (checked arithmetic)
- [ ] Input validation on all public APIs
- [ ] Error handling (no panics in library code)
- [ ] Fuzz tests for new parsing code
- [ ] Documentation of security implications
- [ ] No new dependencies without audit

## Security Configuration

### Recommended `Cargo.toml`:

```toml
[profile.release]
# Enable security features
opt-level = 3
lto = "fat"
panic = "abort"  # Smaller binary, no unwinding
strip = true     # Remove symbols

[profile.dev]
# Keep overflow checks in dev
overflow-checks = true
```

### Recommended runtime config:

```rust
use typf::{Typf, CacheConfig, RenderConfig};

let config = RenderConfig::builder()
    // Limit resource usage
    .max_text_length(100_000)
    .max_line_width(10_000)
    .timeout(Duration::from_secs(5))
    // Security hardening
    .disable_file_access()  // No file:// URLs in SVG
    .sandbox_mode(true)     // Restrictive mode
    .build();

let typf = Typf::with_config(config);
```

## Incident Response

1. **Assess severity** (Critical, High, Medium, Low)
2. **Develop fix** (private branch, no public discussion)
3. **Test thoroughly** (including regression tests)
4. **Coordinate disclosure** with reporter and users
5. **Release patch** (expedited release process)
6. **Publish advisory** (GitHub Security Advisories)
7. **Notify users** (release notes, email, social media)
8. **Post-mortem** (how did this happen? how can we prevent it?)

## Contact

- **Security issues**: security@fontlab.com
- **General bugs**: [GitHub Issues](https://github.com/fontlaborg/typf/issues)
- **General questions**: [GitHub Discussions](https://github.com/fontlaborg/typf/discussions)

## Attribution

We credit security researchers who help keep TypF safe in:

- Security advisories
- CHANGELOG.md
- Release notes
- Hall of Fame (if you prefer)

Thanks for helping keep TypF secure!

---

*Last Updated: 2025-11-18*
*Security Policy Version: 1.0*
