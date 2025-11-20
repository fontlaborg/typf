# TypF Documentation Index

**Complete documentation sitemap for TypF text shaping and rendering library**

Community project by FontLab - https://www.fontlab.org/

---

## 🚀 Getting Started (New Users Start Here!)

**If you're new to TYPF, follow this path:**

1. **[README.md](../README.md)** - Project overview, quick start, installation
2. **[QUICKSTART.md](../typf-tester/QUICKSTART.md)** - 5-minute hands-on tutorial
3. **[Examples](../examples/README.md)** - Working code examples (start with `simple`)

---

## 📚 Core Documentation

### Essential Reading

| Document | Purpose | Audience |
|----------|---------|----------|
| **[README.md](../README.md)** | Project overview, features, building | Everyone |
| **[QUICKSTART.md](../typf-tester/QUICKSTART.md)** | 5-minute onboarding guide | New users |
| **[ARCHITECTURE.md](../ARCHITECTURE.md)** | System design, pipeline details | Developers |
| **[BACKEND_COMPARISON.md](BACKEND_COMPARISON.md)** | Backend selection guide | Decision makers |
| **[PERFORMANCE.md](PERFORMANCE.md)** | Optimization strategies | Performance engineers |

### Reference Documentation

| Document | Purpose |
|----------|---------|
| **[BENCHMARKS.md](../BENCHMARKS.md)** | Performance targets and methodology |
| **[CHANGELOG.md](../CHANGELOG.md)** | Release notes and version history |
| **[PROJECT_STATUS.md](../PROJECT_STATUS.md)** | Current project status |

---

## 🎯 By Use Case

### "I Want to Use TYPF"

**Path**: New User → Basic Usage → Advanced Features

1. [README.md](../README.md) - Install and build
2. [QUICKSTART.md](../typf-tester/QUICKSTART.md) - Get started in 5 minutes
3. [examples/README.md](../examples/README.md) - See working code
4. [BACKEND_COMPARISON.md](BACKEND_COMPARISON.md) - Choose the right backend
5. [README.md#troubleshooting](../README.md#troubleshooting) - Fix common issues

### "I Want to Optimize Performance"

**Path**: Understand → Measure → Optimize

1. [BACKEND_COMPARISON.md](BACKEND_COMPARISON.md) - Backend performance comparison
2. [PERFORMANCE.md](PERFORMANCE.md) - Optimization strategies (6 strategies)
3. [QUICKSTART.md](../typf-tester/QUICKSTART.md) - Run benchmarks
4. [examples/long_text_handling.rs](../examples/long_text_handling.rs) - Handle edge cases

### "I Want to Contribute"

**Path**: Understand → Setup → Contribute

1. [ARCHITECTURE.md](../ARCHITECTURE.md) - Understand the system
2. [CONTRIBUTING.md](../CONTRIBUTING.md) - Development guidelines
3. [SECURITY.md](../SECURITY.md) - Security practices
4. [RELEASE.md](../RELEASE.md) - Release process

### "I'm Migrating from Another Library"

**Path**: Compare → Migrate → Verify

1. [BACKEND_COMPARISON.md#migration-guide](BACKEND_COMPARISON.md#migration-guide) - From cosmic-text, rusttype
2. [ARCHITECTURE.md](../ARCHITECTURE.md) - Understand TypF architecture
3. [examples/](../examples/README.md) - See equivalent code patterns

---

## 📖 Documentation by Category

### Getting Started
- **[README.md](../README.md)** - Project overview and quick start
- **[QUICKSTART.md](../typf-tester/QUICKSTART.md)** - 5-minute onboarding
- **[examples/README.md](../examples/README.md)** - Working code examples
- **[README.md#troubleshooting](../README.md#troubleshooting)** - Common issues

### Performance & Optimization
- **[PERFORMANCE.md](PERFORMANCE.md)** - Comprehensive optimization guide
- **[BACKEND_COMPARISON.md](BACKEND_COMPARISON.md)** - Backend selection & comparison
- **[BENCHMARKS.md](../BENCHMARKS.md)** - Performance targets & methodology
- **[typf-tester/QUICKSTART.md#benchmarking](../typf-tester/QUICKSTART.md)** - Run benchmarks

### Architecture & Design
- **[ARCHITECTURE.md](../ARCHITECTURE.md)** - System design & pipeline
- **[0PLAN.md](../0PLAN.md)** - Implementation plan overview
- **[PLAN/](../PLAN/)** - Detailed plan sections (00-09)

### Development & Contributing
- **[CONTRIBUTING.md](../CONTRIBUTING.md)** - Development guidelines
- **[SECURITY.md](../SECURITY.md)** - Security policy
- **[RELEASE.md](../RELEASE.md)** - Release checklist

### Project Management
- **[CHANGELOG.md](../CHANGELOG.md)** - Version history
- **[PROJECT_STATUS.md](../PROJECT_STATUS.md)** - Current status
- **[TODO.md](../TODO.md)** - Task tracking
- **[WORK.md](../WORK.md)** - Work log

---

## 💻 Code Examples

### Rust Examples

Located in `examples/` directory:

| Example | Features | Complexity |
|---------|----------|------------|
| **[simple.rs](../examples/simple.rs)** | Basic pipeline | Beginner |
| **[minimal.rs](../examples/minimal.rs)** | Minimal build | Beginner |
| **[backend_comparison.rs](../examples/backend_comparison.rs)** | Compare backends | Intermediate |
| **[variable_fonts.rs](../examples/variable_fonts.rs)** | Variable fonts | Intermediate |
| **[svg_export_example.rs](../examples/svg_export_example.rs)** | SVG vector output | Intermediate |
| **[all_formats.rs](../examples/all_formats.rs)** | All export formats | Intermediate |
| **[long_text_handling.rs](../examples/long_text_handling.rs)** | Bitmap limits | Advanced |

See [examples/README.md](../examples/README.md) for detailed documentation.

### Python Examples

Located in `bindings/python/examples/`:

- **[simple_render.py](../bindings/python/examples/simple_render.py)** - Basic rendering
- **[advanced_render.py](../bindings/python/examples/advanced_render.py)** - Advanced features
- **[long_text_handling.py](../bindings/python/examples/long_text_handling.py)** - Handle long texts

---

## 🧪 Testing & Benchmarking

### Testing Tools

| Tool | Purpose | Documentation |
|------|---------|---------------|
| **typfme.py** | Comprehensive testing & benchmarking | [typf-tester/README.md](../typf-tester/README.md) |
| **cargo test** | Unit & integration tests | [README.md#testing](../README.md#testing) |

### Benchmark Commands

```bash
# Full benchmark suite
python typf-tester/typfme.py bench

# Shaping-only performance
python typf-tester/typfme.py bench-shaping

# Rendering-only performance
python typf-tester/typfme.py bench-rendering

# Text length scaling
python typf-tester/typfme.py bench-scaling
```

See [QUICKSTART.md](../typf-tester/QUICKSTART.md) for complete benchmarking guide.

---

## 🔧 Technical Documentation

### API Documentation

- **Rust API**: Run `cargo doc --open --all-features`
- **Python API**: See [bindings/python/README.md](../bindings/python/README.md)

### Implementation Details

| Topic | Document |
|-------|----------|
| Six-stage pipeline | [ARCHITECTURE.md#pipeline](../ARCHITECTURE.md) |
| Backend architecture | [BACKEND_COMPARISON.md](BACKEND_COMPARISON.md) |
| Font loading | [ARCHITECTURE.md#font-loading](../ARCHITECTURE.md) |
| Unicode processing | [ARCHITECTURE.md#unicode](../ARCHITECTURE.md) |
| Cache architecture | [PERFORMANCE.md#caching](PERFORMANCE.md) |
| SIMD optimizations | [PERFORMANCE.md#simd](PERFORMANCE.md) |

---

## 📊 Performance Data

### Benchmark Results

**Real performance data from comprehensive testing:**

| Metric | Value | Source |
|--------|-------|--------|
| Shaping (NONE) | 36.3µs | [BACKEND_COMPARISON.md](BACKEND_COMPARISON.md) |
| Shaping (HarfBuzz) | 46.6µs | [BACKEND_COMPARISON.md](BACKEND_COMPARISON.md) |
| Rendering (Orge @48px) | 1122µs | [typf-tester/README.md](../typf-tester/README.md) |
| Throughput | 2,400 ops/sec | [typf-tester/README.md](../typf-tester/README.md) |

See [PERFORMANCE.md](PERFORMANCE.md) for complete performance analysis.

---

## 🐛 Troubleshooting

### Common Issues

**Quick links to solutions:**

1. **Build errors** → [README.md#troubleshooting](../README.md#troubleshooting)
2. **Runtime errors** → [README.md#troubleshooting](../README.md#troubleshooting)
3. **Performance issues** → [PERFORMANCE.md#common-pitfalls](PERFORMANCE.md)
4. **Backend selection** → [BACKEND_COMPARISON.md#troubleshooting](BACKEND_COMPARISON.md)
5. **Testing issues** → [QUICKSTART.md#troubleshooting](../typf-tester/QUICKSTART.md)

---

## 📦 Package-Specific Documentation

### Rust Crates

- **typf** - Main library ([Cargo.toml](../crates/typf/Cargo.toml))
- **typf-core** - Core types ([Cargo.toml](../crates/typf-core/Cargo.toml))
- **typf-unicode** - Unicode processing ([Cargo.toml](../crates/typf-unicode/Cargo.toml))
- **typf-fontdb** - Font database ([Cargo.toml](../crates/typf-fontdb/Cargo.toml))
- **typf-export** - Export formats ([Cargo.toml](../crates/typf-export/Cargo.toml))
- **typf-cli** - Command-line interface ([Cargo.toml](../crates/typf-cli/Cargo.toml))

### Python Package

- **typf** - Python bindings ([bindings/python/README.md](../bindings/python/README.md))

---

## 🗺️ Documentation Map (Visual)

```
TypF Documentation Hierarchy:

START HERE
│
├─ New User Path
│  ├─ README.md (overview)
│  ├─ QUICKSTART.md (5-min tutorial)
│  ├─ examples/simple.rs (first code)
│  └─ BACKEND_COMPARISON.md (choose backend)
│
├─ Performance Path
│  ├─ BACKEND_COMPARISON.md (backend selection)
│  ├─ PERFORMANCE.md (optimization)
│  ├─ BENCHMARKS.md (targets)
│  └─ typfme.py (measurement)
│
├─ Development Path
│  ├─ ARCHITECTURE.md (design)
│  ├─ CONTRIBUTING.md (guidelines)
│  ├─ examples/ (patterns)
│  └─ tests/ (validation)
│
└─ Reference Path
   ├─ API docs (cargo doc)
   ├─ CHANGELOG.md (history)
   ├─ PROJECT_STATUS.md (current state)
   └─ This INDEX.md (sitemap)
```

---

## 🔗 External Resources

- **GitHub Repository**: https://github.com/fontlab/typf
- **Issue Tracker**: https://github.com/fontlab/typf/issues
- **FontLab Website**: https://www.fontlab.org/
- **Community Forum**: https://forum.fontlab.com/

---

## 📝 Document Maintenance

**Last Updated**: 2025-11-19 (Round 21)

**Coverage**: This index covers all documentation as of TypF.0-dev

**Missing Documentation?** Please open an issue if you find broken links or missing docs.

---

**Community project by FontLab** - Professional font editing software
https://www.fontlab.org/
