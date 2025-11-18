# Fuzz Testing

This directory contains fuzz tests for TYPF using `cargo-fuzz` and `libFuzzer`.

## Quick Start

```bash
# From project root
./scripts/fuzz.sh fuzz_unicode_process 60

# Or run directly
cd fuzz
cargo fuzz run fuzz_unicode_process
```

## Available Targets

### 1. `fuzz_unicode_process`

Fuzzes the Unicode processing pipeline (normalization, bidi, script detection).

**Coverage:**
- NFC normalization
- Bidirectional text resolution
- Script detection
- Segmentation

**Seed corpus:**
- Latin, Arabic, Chinese, Hebrew text
- Emoji sequences
- Mixed scripts

### 2. `fuzz_harfbuzz_shape`

Fuzzes the HarfBuzz shaping backend.

**Coverage:**
- Complex script shaping
- OpenType feature application
- LTR and RTL text
- Various languages

**Seed corpus:**
- Simple Latin
- Complex Arabic
- Japanese/CJK
- Mixed scripts

### 3. `fuzz_pipeline`

Fuzzes the complete six-stage pipeline.

**Coverage:**
- Pipeline builder
- Stage execution
- Error handling
- Context passing

**Seed corpus:**
- Simple text
- Complex text with numbers
- Edge cases

## Installation

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Initialize fuzz testing (already done)
cargo fuzz init
```

## Usage

### Run Single Target

```bash
cd fuzz
cargo fuzz run fuzz_unicode_process
```

### Run with Timeout

```bash
cargo fuzz run fuzz_unicode_process -- -max_total_time=60
```

### Run with Custom Corpus

```bash
cargo fuzz run fuzz_unicode_process corpus/fuzz_unicode_process
```

### Parallel Fuzzing

```bash
# Run 4 parallel jobs
cargo fuzz run fuzz_unicode_process -- -jobs=4
```

## Analyzing Crashes

### Reproduce Crash

```bash
cargo fuzz run fuzz_unicode_process fuzz/artifacts/fuzz_unicode_process/crash-abc123
```

### Minimize Crash Input

```bash
cargo fuzz cmin fuzz_unicode_process
```

### Debug with GDB

```bash
cargo fuzz run -O fuzz_unicode_process -- crash-abc123
gdb target/x86_64-unknown-linux-gnu/release/fuzz_unicode_process
```

## Continuous Fuzzing

### CI Integration

```yaml
# .github/workflows/fuzz.yml
name: Fuzz
on: [push, pull_request]

jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Install cargo-fuzz
      run: cargo install cargo-fuzz
    - name: Run fuzz tests
      run: |
        cd fuzz
        for target in fuzz_unicode_process fuzz_harfbuzz_shape fuzz_pipeline; do
          cargo fuzz run $target -- -max_total_time=60 || exit 1
        done
```

### OSS-Fuzz Integration

TYPF can be integrated with [OSS-Fuzz](https://github.com/google/oss-fuzz) for continuous fuzzing:

1. Create `oss-fuzz` directory
2. Add build script
3. Submit to OSS-Fuzz

## Best Practices

### Writing Fuzz Targets

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // 1. Convert input
    let text = String::from_utf8_lossy(data);

    // 2. Reject invalid inputs early
    if text.is_empty() || text.len() > 10_000 {
        return;
    }

    // 3. Fuzz the target
    let result = my_function(&text);

    // 4. Assert invariants (optional)
    if let Ok(output) = result {
        assert!(output.is_valid());
    }
});
```

### Corpus Management

1. **Start with diverse seeds:** Cover common cases and edge cases
2. **Let fuzzer discover:** libFuzzer will mutate and expand corpus
3. **Minimize periodically:** `cargo fuzz cmin` removes redundant inputs
4. **Share corpus:** Check in interesting findings

### Performance Tips

1. **Limit input size:** Reject very large inputs early
2. **Avoid timeouts:** Set reasonable `-timeout` values
3. **Use sanitizers:** Enable AddressSanitizer and UndefinedBehaviorSanitizer
4. **Profile coverage:** Use `-print_coverage=1` to track progress

## Sanitizers

### AddressSanitizer (ASan)

Detects:
- Use-after-free
- Heap buffer overflow
- Stack buffer overflow
- Global buffer overflow
- Use-after-return

```bash
cargo fuzz run --sanitizer=address fuzz_unicode_process
```

### MemorySanitizer (MSan)

Detects uninitialized memory reads:

```bash
cargo fuzz run --sanitizer=memory fuzz_unicode_process
```

### UndefinedBehaviorSanitizer (UBSan)

Detects undefined behavior:

```bash
cargo fuzz run --sanitizer=undefined fuzz_unicode_process
```

## Coverage

### Generate Coverage Report

```bash
cargo fuzz coverage fuzz_unicode_process
```

### View Coverage

```bash
# Install llvm-cov
cargo install cargo-binutils
rustup component add llvm-tools-preview

# Generate HTML report
cargo fuzz coverage fuzz_unicode_process
llvm-cov show target/x86_64-unknown-linux-gnu/coverage/x86_64-unknown-linux-gnu/release/fuzz_unicode_process \
    -instr-profile=coverage/fuzz_unicode_process/coverage.profdata \
    -format=html > coverage.html
```

## Troubleshooting

### "No corpus found"

```bash
# Create corpus directory
mkdir -p corpus/fuzz_unicode_process
echo "test" > corpus/fuzz_unicode_process/seed1.txt
```

### "Unable to find libFuzzer"

```bash
# Reinstall cargo-fuzz
cargo install --force cargo-fuzz
```

### "Fuzzer timeout"

Increase timeout:
```bash
cargo fuzz run target -- -timeout=5
```

## Resources

- [libFuzzer Documentation](https://llvm.org/docs/LibFuzzer.html)
- [cargo-fuzz Book](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [Rust Fuzz Book](https://rust-fuzz.github.io/book/)
- [OSS-Fuzz](https://google.github.io/oss-fuzz/)
