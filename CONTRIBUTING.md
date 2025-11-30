# Contributing to Typf

How to contribute code, documentation, and improvements to Typf.

## Code of Conduct

Be respectful. Be constructive.

## Getting Started

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- HarfBuzz support:
  - **macOS**: `brew install harfbuzz`
  - **Linux**: `sudo apt-get install libharfbuzz-dev`
  - **Windows**: Included via `harfbuzz-sys`

### Setup

```bash
git clone https://github.com/fontlaborg/typf.git
cd typf
cargo build --workspace
cargo test --workspace
```

### Examples

```bash
cargo run --example basic
cargo run --example formats
cargo run --example harfbuzz --features shaping-hb
```

## Development Workflow

### 1. Read Documentation

- `README.md` - What Typf does
- `ARCHITECTURE.md` - How it works
- `PLAN.md` - What we're building
- `PLAN/00.md` - Technical details

### 2. Check Issues

Browse [existing issues](https://github.com/fontlaborg/typf/issues), comment on what you want to work on.

### 3. Create Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/bug-description
```

### 4. Make Changes

Follow the style guidelines. Write tests for your code.

### 5. Test Changes

```bash
cargo test --workspace
cargo test --package typf-core test_name
cargo test --workspace --all-features
cargo fmt --all -- --check
cargo clippy --workspace --all-features -- -D warnings
```

### 6. Pre-Commit Hooks (Optional)

```bash
cp .github/hooks/pre-commit.sample .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

Automatically checks formatting, warnings, tests, and debug statements.

To bypass (not recommended):
```bash
git commit --no-verify
```

### 7. Commit & Submit

Follow the commit format below. Push your branch. Open a pull request.

## Testing

### Test Types

1. **Unit Tests**: Individual functions/modules
   ```bash
   cargo test --package typf-core
   ```

2. **Integration Tests**: Complete workflows
   ```bash
   cargo test --test integration_test
   ```

3. **Example Tests**: Verify examples compile/run
   ```bash
   cargo test --examples
   ```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_name() {
        let input = create_test_data();
        let result = function_to_test(input);
        assert_eq!(result, expected_value);
    }
}
```

### Test Guidelines

- ✅ Test edge cases (empty, None, negative values)
- ✅ Name tests clearly: `test_when_condition_then_outcome`
- ✅ Keep each test focused on one thing
- ✅ Use fixtures for complex setup
- ❌ Don't test how code works internally
- ❌ Don't write the same test twice

## Code Style

### Formatting

```bash
cargo fmt --all
```

### Linting

```bash
cargo clippy --workspace --all-features -- -D warnings
```

### Naming

- **Crates**: `typf-feature-name` (kebab-case)
- **Modules**: `module_name` (snake_case)
- **Types**: `TypeName` (PascalCase)
- **Functions**: `function_name` (snake_case)
- **Constants**: `CONSTANT_NAME` (SCREAMING_SNAKE_CASE)

### Documentation

Add rustdoc comments to public items. Include examples. Document panics and errors.

```rust
/// Renders text using the specified font.
///
/// # Arguments
/// * `text` - The text to render
/// * `font` - Font reference
/// * `params` - Rendering parameters
///
/// # Returns
/// Returns a `RenderOutput` containing the rendered bitmap.
///
/// # Errors
/// Returns `RenderError::InvalidDimensions` if the output size is invalid.
///
/// # Examples
/// ```
/// use typf_core::*;
/// let rendered = render_text("Hello", font, &params)?;
/// ```
pub fn render_text(text: &str, font: Arc<dyn FontRef>, params: &RenderParams) -> Result<RenderOutput> {
    // Implementation
}
```

### Organization

- Functions: <20 lines
- Files: <200 lines
- Indentation: max 3 levels
- Extract complex logic into helpers

### Error Handling

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Invalid dimensions: {width}x{height}")]
    InvalidDimensions { width: u32, height: u32 },
    #[error("Rendering failed: {0}")]
    RenderFailed(String),
}
```

### Performance

- Profile first, then optimize
- Use benchmarks to prove your improvements
- Mark performance-critical code clearly
- Don't optimize code that isn't slow

## Commit Guidelines

### Format

```
type(scope): what you changed

Why you changed it.

- Specific change 1
- Specific change 2
```

### Types

- **feat**: New feature
- **fix**: Bug fix
- **docs**: Documentation changes
- **style**: Code style changes
- **refactor**: Code restructuring
- **perf**: Performance improvements
- **test**: Adding/updating tests
- **chore**: Maintenance tasks

### Examples

```
feat(export): add PNG export support

Added PNG export using the image crate. Supports RGBA, RGB, Gray8, and Gray1.

- Created PngExporter struct
- Added color space conversion
- Wrote 4 tests
```

```
fix(shaping): fix glyph ID mapping for TTC fonts

Glyph IDs were wrong for TrueType Collection fonts.

Fixes #123
```

## Pull Request Process

### Before Submitting

1. ✅ All tests pass
2. ✅ Code formatted (`cargo fmt`)
3. ✅ No clippy warnings
4. ✅ Documentation updated
5. ✅ CHANGELOG.md updated (if users will notice the change)

### PR Template

```markdown
## Description

Brief description of changes.

## Type of Change

- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing

How changes were tested.

## Checklist

- [ ] Tests pass
- [ ] Code formatted
- [ ] Clippy clean
- [ ] Documentation updated
- [ ] CHANGELOG.md updated (if user-facing)
```

### Review Process

1. CI checks must pass
2. A maintainer must approve
3. Fix the feedback you get
4. Squash commits if asked
5. Maintainer merges

## Project Structure

```
typf/
├── crates/
│   ├── typf/           # Main library
│   ├── typf-core/      # Core traits and types
│   ├── typf-input/     # Input parsing
│   ├── typf-unicode/   # Unicode processing
│   ├── typf-fontdb/    # Font management
│   ├── typf-export/    # Export formats
│   └── typf-cli/       # CLI application
├── backends/
│   ├── typf-shape-none/    # Minimal shaper
│   ├── typf-shape-hb/      # HarfBuzz shaper
│   └── typf-render-opixa/   # Opixa renderer
├── bindings/
│   └── python/         # Python bindings (PyO3)
├── examples/           # Usage examples
├── PLAN/               # Architecture documentation
└── docs/               # Additional documentation
```

### Adding a New Crate

1. Create crate in `crates/` or `backends/`
2. Add to workspace in root `Cargo.toml`
3. Add feature flag if optional
4. Update `ARCHITECTURE.md`
5. Add tests and documentation

### Adding a New Backend

1. Create crate in `backends/typf-{stage}-{name}/`
2. Implement appropriate trait (`Shaper`, `Renderer`, etc.)
3. Add feature flag: `{stage}-{name}`
4. Update backend registry
5. Add comprehensive tests
6. Document in `PLAN/02.md`

## Contribution Areas

### High Priority

- **Platform Backends**: CoreText (macOS), DirectWrite (Windows)
- **Opixa Renderer**: Add anti-aliasing
- **Documentation**: API docs, user guides
- **Testing**: More coverage, property-based tests

### Medium Priority

- **Skia Integration**: Alternative rendering backend
- **Variable Font Support**: Font variations
- **Color Font Support**: COLR/CPAL tables
- **Performance**: SIMD optimizations for ARM

### Low Priority

- **Additional Exporters**: PDF, WebP
- **CLI Enhancements**: REPL mode, rich output
- **Benchmarks**: Comprehensive benchmark suite

## Getting Help

- **Questions**: [GitHub Discussions](https://github.com/fontlaborg/typf/discussions)
- **Bugs**: [Open an issue](https://github.com/fontlaborg/typf/issues)
- **Chat**: Discord (link in README)

## License

Your contributions are licensed under MIT and Apache 2.0.
