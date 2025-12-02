---
title: Architecture Overview
icon: lucide/box
tags:
  - Architecture
  - Design
  - Pipeline
---

# Architecture Overview

Typf v2.4.5 turns text into pixels through six stages with enhanced memory management and backend detection.

## The Pipeline

```mermaid
graph LR
    A[Text] --> B[Unicode]
    B --> C[Font]
    C --> D[Shape]
    D --> E[Render]
    E --> F[Export]
```

Each stage does one job:

1. **Input**: Parses your text and settings
2. **Unicode**: Detects scripts, handles bidi, segments text
3. **Font**: Finds and loads the right font for your text
4. **Shape**: Positions characters as glyphs
5. **Render**: Draws pixels from glyphs
6. **Export**: Writes files in the format you need

## Backends

Typf plugs different engines into each stage:

**Shaping Backends:**
- HarfBuzz (cross-platform)
- CoreText (macOS)
- DirectWrite (Windows)
- ICU-HB (advanced scripts)
- None (testing)

**Rendering Backends:**
- Skia (vectors)
- CoreGraphics (macOS)
- Direct2D (Windows)
- Opixa (pure Rust)
- Zeno (GPU)
- JSON (data export)

All backends implement the same traits, so you can swap them without changing your code.

## Memory Strategy

Fonts are memory-mapped and intentionally leaked to avoid copying. Glyphs cache in an LRU hierarchy. Arc handles sharing between threads safely. **v2.4.5**: Fixed memory leaks in FontDatabase via path-based deduplication (Issue #450).

## Performance

SIMD accelerates pixel operations. Text chunks process in parallel. Cache coherence prevents redundant work.

## Configuration

Feature flags control what compiles. Runtime settings choose backends. You build only what you need. **v2.4.5**: Improved backend detection for macOS CoreText/CoreGraphics and fixed feature flag mismatches between CLI and backend packages.

## Error Handling

Clear error types tell you exactly what failed. No silent fallbacks - if a backend isn't compiled, you get an explicit error message.

## Testing

Unit tests verify components. Integration tests check the full pipeline. Property tests catch edge cases. Fuzz tests find crashes.

## Why This Works

Six stages separate concerns. Backends plug into a common interface. Rust's type system prevents invalid states. Cache hierarchy eliminates redundant work.

Next: [Six-Stage Pipeline](05-six-stage-pipeline.md) dives deeper into each stage.
