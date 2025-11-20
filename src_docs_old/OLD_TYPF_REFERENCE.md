# Old-TypF v1.x Reference Guide for TypF

This document maps TypF's six-stage pipeline architecture to the existing old-typf codebase, enabling efficient refactoring and reuse of proven implementations.

**Last Updated:** 2025-11-18
**Status:** Complete reference mapping ready for Phase 1 implementation

## Quick Reference

| Stage | Purpose | Old-TypF Location | Reuse Strategy |
|-------|---------|-------------------|-----------------|
| 1. Input | Parsing & validation | `crates/typf-api/src/session.rs` | Adapt validation patterns |
| 2. Unicode | Normalization, script detection | `crates/typf-unicode/src/lib.rs` | **REUSE AS-IS** ✓ |
| 3. Font | Loading with caching | `crates/typf-fontdb/src/` | Adapt LRU/mmap patterns |
| 4. Shaping | Text layout | `backends/typf-*-hb/src/` | Refactor into traits |
| 5. Rendering | Rasterization | `backends/typf-orge/src/` | Refactor into traits |
| 6. Export | Format conversion | `crates/typf-render/src/output.rs` | Adapt format handlers |

## Detailed Component Mapping

### Stage 1: Input Parsing & Validation

**Files to Reference:**
- `old-typf/crates/typf-api/src/session.rs` - SessionBuilder validation
- `old-typf/crates/typf-api/src/backend.rs` - Backend option validation
- `old-typf/crates/typf-core/src/types.rs` - Type definitions

**Key Patterns:**
- Use builder pattern for options collection
- Validate font size, text content early
- Check feature availability at compile time with `#[cfg(...)]` guards
- Error messages must include recovery suggestions

**Migration Notes:**
- Move validation logic from SessionBuilder::build() to new InputSpec::validate()
- Combine Backend enum validation with ShapingBackend/RenderBackend enums
- Preserve all error types from old-typf/crates/typf-core/src/error.rs

---

### Stage 2: Unicode Processing

**Files to Reference:**
- `old-typf/crates/typf-unicode/src/lib.rs` - **COMPLETE IMPLEMENTATION**

**Status:** ✓ **PRODUCTION READY** - Copy this crate directly with minimal changes

**Key Components:**
- `UnicodeProcessor` struct with normalization, script detection, bidi analysis
- `TextRun` representation with script, direction, language metadata
- Uses `icu_segmenter` for line/word breaking
- Uses `unicode_bidi::BidiInfo` for UAX #9 analysis
- Handles BCP 47 language tags

**What to Copy:**
```bash
cp -r old-typf/crates/typf-unicode typf/crates/typf-unicode
```

**Adaptation Required:**
- Update error types to use new TypfError enum (if changed)
- Ensure feature flags align with Cargo.toml (`features = ["icu-segmenter", "bidi"]`)
- Add tests using new pipeline InputSpec/PreprocessedText types

**No Changes Needed:**
- Core algorithms (script detection, normalization)
- BCP 47 handling
- Bidirectional text analysis
- Text segmentation logic

---

### Stage 3: Font Selection & Loading

**Files to Reference:**
- `old-typf/crates/typf-fontdb/src/lib.rs` - FontDatabase interface
- `old-typf/crates/typf-fontdb/src/font_cache.rs` - FontLoader implementation
- `old-typf/backends/typf-core/src/lib.rs` - Font metadata extraction

**Key Patterns:**
- `FontDatabase::global()` - Process-wide singleton with lazy initialization
- `FontDatabase::resolve()` - Convert Font spec to FontHandle
- `FontLoader::load()` - Handles memmap2 + fontdb + caching
- LRU eviction with `lru` crate (size limits)
- Concurrent access with `DashMap<String, Arc<FontHandle>>`

**Critical Implementation Details:**

```rust
// From old-typf/crates/typf-fontdb/src/lib.rs:
1. load_font_data_from_path() - Walk directories and load .ttf/.otf/.ttc/.otc files
2. Extra font directories via environment variables or hardcoded paths
3. System font loading via fontdb::Database::load_system_fonts()
4. Cache with Arc<FontHandle> to share loaded data
```

**Adaptation Points:**
- Rename `FontHandle` → `LoadedFont` (for consistency with v2.0)
- Add `skrifa::Font` instance alongside `read_fonts::FontRef`
- Extract variation normalization logic from old-typf backends
- Ensure `FontKey` hashing is consistent across platforms

**Dependencies Needed:**
- `read-fonts` 0.36+ (already in old-typf)
- `skrifa` 0.39+ (already in old-typf)
- `fontdb` for system discovery
- `memmap2` for zero-copy loading
- `lru` for cache eviction
- `dashmap` for concurrent access

**Performance Characteristics:**
- Zero-copy via memmap2 on most systems
- LRU cache prevents unbounded memory growth
- System font discovery cached after first access
- Feature availability checking before loading (NO silent fallbacks)

---

### Stage 4: Shaping

**Files to Reference:**

**HarfBuzz + ICU (Cross-platform default):**
- `old-typf/backends/typf-icu-hb/src/lib.rs` - Proven composition pattern
- Shows how to chain ICU preprocessing → HarfBuzz shaping
- Uses icu_segmenter + unicode_bidi for preprocessing

**Platform-Specific:**
- `old-typf/backends/typf-mac/src/lib.rs` - CoreText shaping (macOS)
- `old-typf/backends/typf-win/src/lib.rs` - DirectWrite shaping (Windows)
- Both implement integrated shape + render pipelines

**Key Patterns:**

1. **Buffer Management** (from typf-icu-hb)
   - Reuse HarfBuzz buffers via object pool
   - Clear before each use
   - Set script, direction, language on buffer

2. **Feature Application**
   - Convert OpenType feature tags (u32) to HarfBuzz Feature structs
   - Apply before calling `hb_shape()`
   - Store in HashMap<Tag, u32> for serialization

3. **Caching**
   - Key: (text, font_key, script, direction, language, features, size)
   - Value: Vec<ShapedGlyph> + metrics
   - Use DashMap for thread-safe concurrent access
   - Size limit via LRU eviction

4. **Variable Font Support** (from old-typf/crates/typf-fontdb)
   - Normalize variation axes using skrifa
   - Pass normalized coords to HarfBuzz via `hb_font_set_variation()`
   - Validate requested variations exist (fail if not available)

**Migration Strategy:**

```rust
// OLD: Backend implements DynBackend trait from typf-api
impl DynBackend for HarfBuzzBackend {
    fn shape_text(...) -> ShapingResult { ... }
}

// NEW: Backend implements Shaper trait from typf-core
impl Shaper for HarfBuzzShaper {
    fn shape(...) -> Result<ShapingResult> { ... }
}
```

**Do NOT Reuse Directly:**
- Old-typf uses monolithic backend pattern
- v2.0 separates Shaper trait from Renderer trait
- Extract shaping logic, discard render logic

---

### Stage 5: Rendering

**Files to Reference:**

**Orge Rasterizer (CPU scan converter):**
- `old-typf/backends/typf-orge/src/lib.rs` - Full implementation
- Custom F26.6 fixed-point rasterizer (made by FontLab)
- Glyph cache with DashMap
- Canvas compositing with color + transparency
- **Reference**: `backends/typf-orge/src/renderer.rs`

**TinySkia (Cross-platform SVG/bitmap):**
- `old-typf/backends/typf-skiahb/src/renderer.rs` - Skia integration
- Bitmap rendering with anti-aliasing
- SVG output via path extraction
- Uses `lyon_geom` for path operations

**Output Formats:**
- `old-typf/crates/typf-render/src/output.rs` - PNG, SVG, JSON
- PNG export via `png` crate
- SVG templating
- JSON shaping results (HarfBuzz-compatible format)

**Key Patterns:**

1. **Glyph Rasterization Cache**
   ```rust
   Key: GlyphCacheKey { glyph_id, font_key, size }
   Value: RasterizedGlyph { width, height, left, top, data }
   ```
   - Critical for performance (1000s of glyphs per frame)
   - Use DashMap for concurrent access
   - Size limit via LRU eviction

2. **Canvas/Pixmap Management**
   - Calculate combined bounding box from shaped glyphs
   - Create blank canvas (white or transparent background)
   - Composite each glyph at x_offset, y_offset
   - Handle color + transparency (RGBA8 format)

3. **Format Handlers**
   - Separate encoder for each format (PNG, SVG, JSON)
   - PNG: Use `png` crate encoder
   - SVG: Generate path data with transform
   - JSON: Serialize glyph array with 26.6 fixed-point format

**Migration Notes:**
- Extract rasterization logic from DynBackend implementations
- Create separate Renderer trait implementations
- Preserve glyph cache patterns (proven efficient)
- Add format handlers as Exporter trait implementations

---

### Stage 6: Export

**Files to Reference:**
- `old-typf/crates/typf-render/src/output.rs` - Format handling
- `old-typf/crates/typf-render/src/svg.rs` - SVG generation
- `old-typf/backends/typf-orge/src/` - PNG writing patterns

**Key Components:**

1. **PNG Export**
   - Wrap bitmap data with width, height
   - Use `png` crate encoder
   - Support RGBA8, RGB8, Grayscale

2. **SVG Export**
   - Generate `<svg>` wrapper with viewBox
   - Create `<path>` elements for each glyph
   - Apply `transform="translate(x, y)"`
   - Set fill color from foreground option

3. **JSON Export**
   - Serialize ShapingResult in HarfBuzz-compatible format
   - Glyph array with id, cluster, offsets, advances
   - Font metadata (family, size, features)
   - Metrics (total advance, line height)

**Migration Strategy:**
- Create Exporter trait (takes RenderOutput → Write)
- Implement PngExporter, SvgExporter, JsonExporter
- Register via feature flags in Cargo.toml

---

## Cross-Cutting Concerns

### Error Handling

**Reference:** `old-typf/crates/typf-core/src/error.rs`

**Key Principles:**
- Use `thiserror` for error types (library code)
- Use `anyhow::Result` for application code
- NO silent fallbacks - explicit TypfError variants
- Rich error context with file paths, line numbers

**Error Types to Preserve:**
```rust
TypfError::FeatureNotCompiled(backend_name)
TypfError::UnsupportedBackendCombination(shaping, render)
TypfError::FontNotFound(family)
TypfError::UnsupportedScript(script)
```

**Example from old-typf:**
```rust
if !cfg!(feature = "shaping-icu-hb") && backend == Backend::IcuHb {
    return Err(TypfError::FeatureNotCompiled("shaping-icu-hb"));
}
```

### Caching Architecture

**Reference:** `old-typf/crates/typf-fontdb/src/font_cache.rs`

**Three-Level Cache:**
1. **L1 Cache** (< 50ns) - Per-instance glyph bitmap cache
   - DashMap<GlyphCacheKey, Arc<RasterizedGlyph>>
   - No eviction needed for small fonts
2. **L2 Cache** (~ 1µs) - Font instance + shaping result cache
   - LRU eviction per size limit
   - Keyed by (font_key, size, features)
3. **L3 Cache** (persistent) - System font database
   - Lazy-loaded singleton
   - System fonts enumerated once per process

**Key Pattern from old-typf:**
```rust
let mut cache = DashMap::new();
if let Some(cached) = cache.get(&key) {
    return Ok(cached.clone());
}
let result = expensive_operation()?;
cache.insert(key, Arc::new(result.clone()));
```

### Concurrency & Memory

**Thread Safety Pattern:**
- Arc<T> for shared ownership
- DashMap for lock-free concurrent access
- parking_lot::RwLock for reader-heavy workloads
- NO global mutable state (use lazy_once::OnceCell)

**Memory Management:**
- Box::leak() for font data (intentional, cached globally)
- Arc<ReadonlyData> for glyph bitmaps
- LRU eviction when cache size exceeds limit
- Benchmarks verify no memory leaks (run with valgrind)

---

## External Reference Implementations

**Location:** `/Users/adam/Developer/vcs/github.fontlaborg/typf/external/`

When stuck on implementation details, reference:
- **fontations/** - read-fonts and skrifa source code
- **icu4x/** - ICU4X reference for Unicode processing
- **tiny-skia/** - Rasterization algorithms
- **harfrust/** - HarfBuzz Rust binding patterns
- **zeno/** - Alternative vector graphics rasterizer
- **fontgrep(c)/** - Font querying patterns

---

## Phase 1 Implementation Checklist

### Week 1: Workspace & Core Structure

- [ ] Clone typf-unicode directly from old-typf
- [ ] Extract error types from old-typf/crates/typf-core/src/error.rs
- [ ] Create typf-input crate with InputSpec validation
- [ ] Create typf-core crate with trait definitions
- [ ] Set up feature flags in Cargo.toml (reference old-typf/Cargo.toml)

### Week 2-3: Font Loading & Unicode

- [ ] Implement typf-fontdb with patterns from old-typf
  - Copy FontDatabase::global() pattern
  - Adapt FontLoader::load() for new LoadedFont type
  - Use proven LRU + DashMap caching
- [ ] Verify typf-unicode works with new pipeline types
- [ ] Add integration tests: text → normalized runs

### Week 4: Minimal Backends

- [ ] Extract NoneShaper from old-typf backends
- [ ] Implement OrgeRenderer from old-typf/backends/typf-orge
- [ ] Add PNM export (simple bitmap format)
- [ ] Test minimal pipeline: Input → Unicode → Font → None → Orge → PNG

---

## Dependency Checklist

All dependencies already in old-typf's Cargo.toml. Do NOT add new crates:

✓ `read-fonts` 0.36+ - Font parsing
✓ `skrifa` 0.39+ - Font metrics
✓ `fontdb` 0.21+ - System fonts
✓ `memmap2` 0.9+ - Zero-copy loading
✓ `lru` 0.12+ - Cache eviction
✓ `dashmap` 6.1+ - Concurrent hash map
✓ `harfbuzz_rs` 2.0+ - Text shaping
✓ `icu_segmenter` 2.1+ - Text segmentation
✓ `unicode_bidi` 0.3+ - Bidirectional text
✓ `kurbo` 0.11+ - Path geometry
✓ `png` 0.17+ - PNG export
✓ `pyo3` 0.22+ - Python bindings
✓ `thiserror` 1.0+ - Error handling
✓ `rayon` 1.10+ - Parallelization

---

## Critical "Do Not" List

1. **Do NOT** import ttf-parser (old-typf already uses read-fonts)
2. **Do NOT** create new dependency crates (everything proven exists)
3. **Do NOT** refactor error handling prematurely (use old-typf patterns)
4. **Do NOT** optimize before testing (correctness first)
5. **Do NOT** remove old-typf/ until v2.0 is feature-complete
6. **Do NOT** silence errors with `unwrap()` or `expect()` in libraries

---

## Success Criteria for Phase 1

- [ ] Minimal pipeline renders Latin text with NoneShaper + OrgeRenderer
- [ ] All old-typf tests still pass when run against old-typf codebase
- [ ] v2.0 pipeline produces identical output to old-typf for test cases
- [ ] No compiler warnings (clippy clean)
- [ ] Binary size < 500KB for minimal build
- [ ] Document any breaking changes from v1.x in CHANGELOG.md

---

*End of Reference Guide*
*For questions, refer to PLAN/00-09 and CLAUDE.md Section X*
