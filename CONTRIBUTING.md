# Contributing to TYPF

Thank you for your interest in contributing to TYPF! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Testing](#testing)
- [Code Style](#code-style)
- [Commit Guidelines](#commit-guidelines)
- [Pull Request Process](#pull-request-process)
- [Project Structure](#project-structure)

## Code of Conduct

We are committed to providing a welcoming and inclusive environment. Please be respectful and constructive in all interactions.

## Getting Started

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- For HarfBuzz support:
  - **macOS**: `brew install harfbuzz`
  - **Linux**: `sudo apt-get install libharfbuzz-dev`
  - **Windows**: HarfBuzz binaries included via `harfbuzz-sys`

### Clone and Build

```bash
git clone https://github.com/fontlaborg/typf.git
cd typf
cargo build --workspace
```

### Run Tests

```bash
cargo test --workspace
```

### Run Examples

```bash
cargo run --example basic
cargo run --example formats
cargo run --example harfbuzz --features shaping-hb
```

## Development Workflow

### 1. Read the Documentation

Before making changes, familiarize yourself with:

- `README.md` - Project overview
- `ARCHITECTURE.md` - System design
- `PLAN.md` - Implementation roadmap
- `PLAN/00.md` - Comprehensive architecture docs

### 2. Check Existing Issues

- Browse [existing issues](https://github.com/fontlaborg/typf/issues)
- Comment if you want to work on something
- Ask questions if anything is unclear

### 3. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/bug-description
```

### 4. Make Your Changes

Follow the code style guidelines (see below) and write tests for new functionality.

### 5. Test Your Changes

```bash
# Run all tests
cargo test --workspace

# Run specific test
cargo test --package typf-core test_name

# Run with all features
cargo test --workspace --all-features

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --workspace --all-features -- -D warnings
```

### 5.5. Set Up Pre-Commit Hooks (Optional but Recommended)

Install the pre-commit hook to automatically check your code before committing:

```bash
# Install the pre-commit hook
cp .github/hooks/pre-commit.sample .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

The pre-commit hook will automatically:
- Check code formatting with `cargo fmt`
- Run `cargo clippy` with warnings as errors
- Run all tests
- Warn about debugging statements (`dbg!`, `println!`, `eprintln!`)

To bypass the hook temporarily (not recommended):
```bash
git commit --no-verify
```

### 6. Commit Your Changes

Follow the commit guidelines (see below).

### 7. Submit a Pull Request

Push your branch and create a pull request on GitHub.

## Testing

### Test Categories

1. **Unit Tests**: Test individual functions and modules
   ```bash
   cargo test --package typf-core
   ```

2. **Integration Tests**: Test complete workflows
   ```bash
   cargo test --test integration_test
   ```

3. **Example Tests**: Verify examples compile and run
   ```bash
   cargo test --examples
   ```

### Writing Tests

Place tests in the same file as the code being tested:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_name() {
        // Arrange
        let input = create_test_data();
        
        // Act
        let result = function_to_test(input);
        
        // Assert
        assert_eq!(result, expected_value);
    }
}
```

### Test Guidelines

- ✅ Test edge cases (empty input, None, negative numbers, etc.)
- ✅ Use descriptive test names: `test_when_condition_then_outcome`
- ✅ Keep tests focused on one thing
- ✅ Use test fixtures for complex setups
- ❌ Don't test implementation details
- ❌ Don't duplicate tests unnecessarily

## Code Style

### Formatting

We use `rustfmt` with default settings:

```bash
cargo fmt --all
```

### Linting

We use `clippy` with `-D warnings` (all warnings are errors):

```bash
cargo clippy --workspace --all-features -- -D warnings
```

### Naming Conventions

- **Crates**: `typf-feature-name` (kebab-case)
- **Modules**: `module_name` (snake_case)
- **Types**: `TypeName` (PascalCase)
- **Functions**: `function_name` (snake_case)
- **Constants**: `CONSTANT_NAME` (SCREAMING_SNAKE_CASE)

### Documentation

- Add rustdoc comments to all public items
- Include examples in documentation
- Document panics and errors

```rust
/// Renders text using the specified font.
///
/// # Arguments
///
/// * `text` - The text to render
/// * `font` - Font reference
/// * `params` - Rendering parameters
///
/// # Returns
///
/// Returns a `RenderOutput` containing the rendered bitmap.
///
/// # Errors
///
/// Returns `RenderError::InvalidDimensions` if the output size is invalid.
///
/// # Examples
///
/// ```
/// use typf_core::*;
///
/// let rendered = render_text("Hello", font, &params)?;
/// ```
pub fn render_text(text: &str, font: Arc<dyn FontRef>, params: &RenderParams) -> Result<RenderOutput> {
    // Implementation
}
```

### Code Organization

- Keep functions small (<20 lines ideally)
- Keep files focused (<200 lines ideally)
- Limit indentation depth (max 3 levels)
- Extract complex logic into helper functions

### Error Handling

- Use `Result<T, E>` for fallible operations
- Use `thiserror` for error types
- Provide helpful error messages

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

### Performance Considerations

- Profile before optimizing
- Use benchmarks to validate improvements
- Document performance-critical code
- Avoid premature optimization

## Commit Guidelines

### Commit Message Format

```
type(scope): short description

Longer description if needed.

- Detail 1
- Detail 2
```

### Types

- **feat**: New feature
- **fix**: Bug fix
- **docs**: Documentation changes
- **style**: Code style changes (formatting, etc.)
- **refactor**: Code restructuring
- **perf**: Performance improvements
- **test**: Adding or updating tests
- **chore**: Maintenance tasks

### Examples

```
feat(export): add PNG export support

Implements PNG export using the image crate with support for
RGBA, RGB, Gray8, and Gray1 color spaces.

- Added PngExporter struct
- Implemented color space conversion
- Added 4 comprehensive tests
```

```
fix(shaping): correct glyph ID mapping for TTC fonts

Fixed an issue where glyph IDs were incorrectly mapped when
using TrueType Collection (TTC) fonts.

Fixes #123
```

## Pull Request Process

### Before Submitting

1. ✅ All tests pass
2. ✅ Code is formatted (`cargo fmt`)
3. ✅ No clippy warnings (`cargo clippy`)
4. ✅ Documentation is updated
5. ✅ CHANGELOG.md is updated (for user-facing changes)

### PR Description Template

```markdown
## Description

Brief description of the changes.

## Type of Change

- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing

How the changes were tested.

## Checklist

- [ ] Tests pass
- [ ] Code formatted
- [ ] Clippy clean
- [ ] Documentation updated
- [ ] CHANGELOG.md updated (if user-facing)
```

### Review Process

1. Automated checks must pass (CI/CD)
2. At least one maintainer approval required
3. Address review feedback
4. Squash commits if requested
5. Maintainer will merge when ready

## Project Structure

```
typf/
├── crates/
│   ├── typf/           # Main library crate
│   ├── typf-core/      # Core traits and types
│   ├── typf-input/     # Input parsing
│   ├── typf-unicode/   # Unicode processing
│   ├── typf-fontdb/    # Font management
│   ├── typf-export/    # Export formats
│   └── typf-cli/       # CLI application
├── backends/
│   ├── typf-shape-none/    # Minimal shaper
│   ├── typf-shape-hb/      # HarfBuzz shaper
│   └── typf-render-orge/   # Orge renderer
├── bindings/
│   └── python/         # Python bindings (PyO3)
├── examples/           # Usage examples
├── PLAN/               # Architecture documentation
└── docs/               # Additional documentation
```

### Adding a New Crate

1. Create crate in appropriate directory (`crates/` or `backends/`)
2. Add to workspace in root `Cargo.toml`
3. Add feature flag if optional
4. Update `ARCHITECTURE.md`
5. Add tests and documentation

### Adding a New Backend

1. Create crate in `backends/typf-{stage}-{name}/`
2. Implement the appropriate trait (`Shaper`, `Renderer`, etc.)
3. Add feature flag: `{stage}-{name}`
4. Update backend registry
5. Add comprehensive tests
6. Document in `PLAN/02.md`

## Areas Needing Contributions

### High Priority

- **Platform Backends**: CoreText (macOS), DirectWrite (Windows)
- **Orge Renderer**: Anti-aliasing implementation
- **Documentation**: API docs, user guides
- **Testing**: More test coverage, property-based tests

### Medium Priority

- **Skia Integration**: Alternative rendering backend
- **Variable Font Support**: Font variations
- **Color Font Support**: COLR/CPAL tables
- **Performance**: SIMD optimizations for ARM

### Low Priority

- **Additional Exporters**: PDF, WebP
- **CLI Enhancements**: REPL mode, rich output
- **Benchmarks**: More comprehensive benchmark suite

## Getting Help

- **Questions**: Open a [GitHub Discussion](https://github.com/fontlaborg/typf/discussions)
- **Bugs**: Open an [Issue](https://github.com/fontlaborg/typf/issues)
- **Chat**: Join our Discord server (link in README)

## License

By contributing to TYPF, you agree that your contributions will be licensed under both the MIT License and Apache License 2.0.

---

Thank you for contributing to TYPF! 🎨✨
