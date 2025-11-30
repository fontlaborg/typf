# Fuzz Testing - Crash Typf Before Your Users Do

Fuzzing throws random, malformed data at your code to find panics, crashes, and security vulnerabilities that normal testing misses. Typf handles complex Unicode text, font files, and rendering pipelines - perfect candidates for fuzzing to ensure robustness.

## 🚀 Quick Start

```bash
# From project root - 60 seconds of fuzzing
./scripts/fuzz.sh fuzz_unicode_process 60

# Or dive deeper with direct cargo-fuzz commands
cd fuzz
cargo fuzz run fuzz_unicode_process
```

## 🎯 Fuzz Targets

### `fuzz_unicode_process` - Unicode Bombardment
**Goal**: Find Unicode processing crashes in normalization, bidirectional text, and script detection.

**What gets tested:**
- NFC normalization (character composition/decomposition)
- Bidirectional algorithm failure modes
- Script detection edge cases
- Text segmentation bugs
- Invalid UTF-8 sequences

**Why it matters**: A single Unicode bug can crash your entire app when users paste text from different sources.

### `fuzz_harfbuzz_shape` - Font Shaping Stress Test
**Goal**: Ensure malformed text can't crash the professional HarfBuzz shaping engine.

**What gets tested:**
- Complex script shaping (Arabic, Hindi, Thai)
- OpenType feature application
- Right-to-left and left-to-right text mixing
- Font loading and parsing
- Glyph positioning algorithms

**Why it matters**: HarfBuzz is complex C++ code that processes untrusted text - a perfect fuzzing target.

### `fuzz_pipeline` - Architecture Robustness
**Goal**: Test Typf's pipeline framework with minimal backends to isolate architectural bugs.

**What gets tested:**
- Pipeline builder pattern stability
- Stage execution and error propagation
- Context management between stages
- Parameter validation
- Component lifecycle management

**Why it matters**: Pipeline bugs can affect every text rendering operation, regardless of which backends you use.

## ⚙️ Setup & Installation

```bash
# Install cargo-fuzz (one-time setup)
cargo install cargo-fuzz

# Typf's fuzz targets are already initialized
cd fuzz
```

## 🏃 Running Fuzz Tests

### Basic Fuzzing
```bash
cd fuzz
cargo fuzz run fuzz_unicode_process
```

### Time-Limited Fuzzing (Recommended for CI)
```bash
# Run for 60 seconds then stop
cargo fuzz run fuzz_unicode_process -- -max_total_time=60
```

### Continuous Fuzzing (For Deep Testing)
```bash
# Run multiple targets in parallel
cargo fuzz run fuzz_unicode_process -- -jobs=4 &
cargo fuzz run fuzz_harfbuzz_shape -- -jobs=4 &
cargo fuzz run fuzz_pipeline -- -jobs=4 &
wait
```

### Custom Corpus Testing
```bash
# Use your own test cases as starting points
cargo fuzz run fuzz_unicode_process corpus/mixed_scripts/
```

## 🔍 When Crashes Happen

### Reproduce the Crash
```bash
# Test the exact input that caused the crash
cargo fuzz run fuzz_unicode_process fuzz/artifacts/fuzz_unicode_process/crash-abc123
```

### Minimize the Crash Case
```bash
# Automatically reduce the input to the smallest crashing case
cargo fuzz cmin fuzz_unicode_process
```

### Debug Deep with GDB
```bash
# Build with debugging symbols and attach GDB
cargo fuzz run -O fuzz_unicode_process -- crash-abc123
gdb target/x86_64-unknown-linux-gnu/release/fuzz_unicode_process
```

### Fix and Verify
1. **Fix the bug** in the target code
2. **Reproduce the crash** to confirm it's fixed
3. **Add the minimized crash** to the corpus
4. **Run the fuzzer again** to ensure no regressions

## 🔄 Continuous Fuzzing

### GitHub Actions Integration
Add fuzzing to your CI to catch regressions early:

```yaml
# .github/workflows/fuzz.yml
name: Security & Robustness Fuzzing
on: [push, pull_request]

jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Install cargo-fuzz
      run: cargo install cargo-fuzz
    - name: Run comprehensive fuzz tests
      run: |
        cd fuzz
        for target in fuzz_unicode_process fuzz_harfbuzz_shape fuzz_pipeline; do
          echo "Fuzzing $target..."
          cargo fuzz run $target -- -max_total_time=60 || {
            echo "Fuzz failures detected in $target"
            exit 1
          }
        done
```

### OSS-Fuzz for Industry-Scale Testing
For continuous professional fuzzing, integrate with Google's OSS-Fuzz:

1. **Submit to OSS-Fuzz** - Get free 24/7 fuzzing on Google's infrastructure
2. **Daily reports** - Automatic bug reports with minimized test cases
3. **Coverage tracking** - Measure your fuzzing effectiveness over time
4. **Sanitizer variety** - Test with AddressSanitizer, MemorySanitizer, and more

## 🎯 Fuzzing Best Practices

### Write Effective Targets
```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // 1. Transform raw bytes into your domain
    let text = String::from_utf8_lossy(data);

    // 2. Filter out time-wasters early
    if text.is_empty() || text.len() > 10_000 {
        return;
    }

    // 3. Exercise the code you want to protect
    let result = my_function(&text);

    // 4. Optional: Verify invariants aren't violated
    if let Ok(output) = result {
        assert!(output.is_valid());
    }
});
```

### Build a Smart Corpus
1. **Seed with diversity**: Include real-world edge cases and problematic inputs
2. **Let evolution work**: libFuzzer mutates and discovers new crash patterns
3. **Minimize regularly**: `cargo fuzz cmin` removes redundant test cases
4. **Share your findings**: Check in interesting crashes to the corpus

### Maximize Performance
1. **Reject early**: Filter out inputs that would waste CPU cycles
2. **Timeout wisely**: Use `-timeout=5` to prevent stuck fuzzers
3. **Enable sanitizers**: AddressSanitizer catches memory bugs, UBSan catches undefined behavior
4. **Track coverage**: Use `-print_coverage=1` to see if you're exercising new code paths

## 🛡️ Security Testing with Sanitizers

### AddressSanitizer (ASan) - Memory Safety Guardian
Catches the most common memory bugs:
- Use-after-free and use-after-return
- Heap, stack, and global buffer overflows
- Memory leaks and double-free

```bash
cargo fuzz run --sanitizer=address fuzz_unicode_process
```

### MemorySanitizer (MSan) - Uninitialized Memory Hunter
Finds reads of uninitialized memory that can cause unpredictable behavior:

```bash
cargo fuzz run --sanitizer=memory fuzz_unicode_process
```

### UndefinedBehaviorSanitizer (UBSan) - Undefined Behavior Detector
Catches subtle C++/Rust undefined behaviors that compilers miss:

```bash
cargo fuzz run --sanitizer=undefined fuzz_unicode_process
```

## 📊 Coverage Analysis

### Generate Coverage Reports
See which parts of your code the fuzzer is actually exercising:

```bash
cargo fuzz coverage fuzz_unicode_process
```

### Visual Coverage Analysis
```bash
# Install coverage tools
cargo install cargo-binutils
rustup component add llvm-tools-preview

# Generate beautiful HTML coverage report
cargo fuzz coverage fuzz_unicode_process
llvm-cov show target/x86_64-unknown-linux-gnu/coverage/x86_64-unknown-linux-gnu/release/fuzz_unicode_process \
    -instr-profile=coverage/fuzz_unicode_process/coverage.profdata \
    -format=html > coverage.html

# Open in browser to see which lines were exercised
open coverage.html
```

**Goal**: Aim for 80%+ coverage of critical text processing code paths.

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
