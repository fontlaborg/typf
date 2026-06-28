<!-- this_file: src_docs/28-gpu-vs-cpu-vello.md -->
# GPU vs CPU Vello

Typf ships **two** renderers built on the [Vello](https://github.com/linebender/vello)
engine, and they are routinely confused. They share path-rasterization code but
target completely different execution environments.

| | `vello` (GPU) | `vello-cpu` (CPU) |
|---|---|---|
| Crate | `typf-render-vello` | `typf-render-vello-cpu` |
| Feature flag | `render-vello` | `render-vello-cpu` |
| Executes on | GPU via `wgpu` (Metal / Vulkan / DX12) | CPU only, pure Rust |
| Requires a GPU | **Yes** | No |
| Throughput | 10K+ ops/sec (large text) | ~3.5K ops/sec |
| Colour glyphs | ⚠️ Outline-only today (no COLR / SVG / bitmap) | COLR + bitmap supported |
| Typical use | High-throughput / large canvases on a GPU host | Servers, CI, headless, no-GPU machines |

## Which one should I use?

- **No GPU, or running headless/CI:** use `vello-cpu`. It is pure Rust, needs no
  drivers, and renders colour fonts.
- **You have a GPU and render lots of large text:** use `vello` for the
  throughput win — but note its colour-glyph limitations below.

```bash
# GPU path (needs a working GPU + driver)
typf render "Hello" --renderer vello -o out.png

# CPU path (pure Rust, works anywhere)
typf render "Hello" --renderer vello-cpu -o out.png
```

## GPU Vello colour-glyph limitation

In this repository the GPU `vello` backend rasterizes glyph **outlines** only,
filled with a single solid foreground paint. It does **not** currently handle:

- `COLR` v1 (layered / gradient colour glyphs)
- `SVG`-table glyphs
- Bitmap strikes (`sbix` / `CBDT` / `EBDT`)

Such glyphs render as plain monochrome outlines (or blank if they have no
outline). For colour fonts, use `vello-cpu`, `skia`, or `zeno` instead. This is
documented at the source level in `convert_glyphs` and `render` inside
`backends/typf-render-vello/src/lib.rs`.

## Why two crates rather than a runtime flag?

The GPU backend pulls in `wgpu` and a live graphics device, which is a hard
dependency that many deployment targets (containers, CI runners, minimal
servers) cannot satisfy. Keeping the CPU renderer as a separate crate behind its
own feature flag means a `vello-cpu` build carries none of the GPU stack.

## Headless / CI note

The GPU `vello` backend needs an adapter at runtime; on machines without one its
constructor returns `VelloError::NoAdapter`. Tests and pipelines that must run
headless should select `vello-cpu` (or another CPU renderer) rather than relying
on the GPU path being available.
