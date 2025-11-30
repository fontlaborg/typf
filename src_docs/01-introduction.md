---
title: Introduction
icon: lucide/book-open
tags:
  - Introduction
  - Overview
---

Your text breaks. Arabic renders backwards, Hindi characters collide, Thai glyphs disappear.

Typf fixes this.

## What Typf does

Takes broken text and makes it right. Processes Unicode strings through six stages:

```
Input → Unicode → Fonts → Shape → Render → Export
```

Mix and match backends:
- **Shapers**: HarfBuzz, CoreText, DirectWrite, or None
- **Renderers**: Skia, Opixa, Zeno, CoreGraphics, or JSON
- **Exports**: PNG, SVG, PNM, or structured data

## Why you need it

**HarfBuzz** shapes text but can't render it. **Skia** renders everything but weighs 10MB. **Platform APIs** lock you to one OS.

Typf gives you the right tool for the job. Use only what you need.

## Performance

- **<1ms** rendering for most text
- **>10 GB/s** throughput with SIMD
- **<500KB** minimal builds
- **Zero-copy** font loading

## Where it works

- **Desktop apps**: Text editors, design tools
- **Servers**: Generate images, process documents
- **Embedded**: Resource-constrained devices
- **Web**: Browser rendering via WebAssembly

## Next

[Quick Start](02-quick-start.md) - Get Typf running in 5 minutes
