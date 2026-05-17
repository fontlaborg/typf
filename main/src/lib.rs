//! Typf — modular text shaping and rendering engine.
//!
//! This crate is the public entry point. It re-exports everything from
//! `typf-core` and conditionally re-exports each backend crate under its
//! feature flag, so callers only pay for what they enable.
//!
//! ## The pipeline at a glance
//!
//! ```text
//! "Hello, مرحبا, 你好!"
//!          │
//!          ▼
//!   ┌─────────────┐
//!   │   Shaper    │  Reads font's OpenType rules. Decides which glyphs to
//!   │             │  use, handles Arabic joins, Hindi stacking, Latin kerning.
//!   └──────┬──────┘
//!          │  ShapingResult: glyph IDs + (x, y) positions
//!          ▼
//!   ┌─────────────┐
//!   │  Renderer   │  Reads glyph outlines (Bézier curves) from the font.
//!   │             │  Rasterizes into pixels, or traces into SVG paths.
//!   └──────┬──────┘
//!          │  RenderOutput: RGBA bitmap or SVG/JSON data
//!          ▼
//!   ┌─────────────┐
//!   │  Exporter   │  Encodes pixels/paths as PNG, PNM, SVG, JSON bytes.
//!   └─────────────┘
//! ```
//!
//! Every box is swappable. Five shaper implementations × seven renderer
//! implementations = 35 valid backend combinations, all sharing the same
//! trait contracts defined in `typf-core`.
//!
//! ## Quick start
//!
//! ```ignore
//! use typf::prelude::*;
//! use typf::Pipeline;
//!
//! let pipeline = Pipeline::builder()
//!     .shaper(my_shaper)
//!     .renderer(my_renderer)
//!     .exporter(my_exporter)
//!     .build()?;
//!
//! let bytes = pipeline.process("Hello", font, &ShapingParams::default(), &RenderParams::default())?;
//! ```
//!
//! ## Feature flags
//!
//! | Flag | What you get |
//! |---|---|
//! | `minimal` | `shaping-none` + `render-opixa` — ~500 KB binary, no C deps |
//! | `default` | `minimal` + `unicode` + `fontdb` + `export-pnm` |
//! | `full` | Every shaper, renderer, and exporter |
//! | `shaping-hb` | HarfBuzz C library — industry-standard, all scripts |
//! | `shaping-hr` | Pure Rust HarfBuzz port — zero C deps |
//! | `shaping-icu-hb` | ICU normalization + HarfBuzz — full Unicode pipeline |
//! | `shaping-ct` / `shaping-mac` | macOS CoreText (macOS only) |
//! | `render-skia` | tiny-skia — full color font support |
//! | `render-zeno` | Pure Rust zeno rasterizer — 256-level AA, color fonts |
//! | `render-vello-cpu` | Vello CPU — pure Rust, COLR/bitmap support |
//! | `render-vello` | Vello GPU — Metal/Vulkan/DX12, outline-only today |
//! | `render-cg` / `render-mac` | macOS CoreGraphics (macOS only) |
//! | `export-png` | PNG output |
//! | `export-svg` | SVG output (23× faster than PNG) |

pub use typf_core::{
    cache_config, error, traits, validate_glyph_count, Color, GlyphSource, GlyphSourcePreference,
    Pipeline, RenderMode, RenderParams, ShapingParams, MAX_FONT_SIZE, MAX_GLYPH_COUNT,
};

#[cfg(feature = "input")]
pub use typf_input as input;

#[cfg(feature = "unicode")]
pub use typf_unicode as unicode;

#[cfg(feature = "fontdb")]
pub use typf_fontdb as fontdb;

#[cfg(feature = "export-pnm")]
pub use typf_export as export;

#[cfg(feature = "wasm")]
pub mod wasm;

// ── Shaping backends ────────────────────────────────────────────────────────
//
// Each shaper implements `typf_core::traits::Shaper`. Pick one based on your
// platform constraints, script requirements, and dependency budget.

/// Minimal LTR-only passthrough shaper.
///
/// No OpenType feature processing: one Unicode codepoint → one glyph, advance
/// by the font's default advance width. Use for ASCII-only debug builds or as
/// a zero-dependency baseline. Enabled by default via the `minimal` feature.
#[cfg(feature = "shaping-none")]
pub use typf_shape_none as shape_none;

/// HarfBuzz text shaper (C library via FFI).
///
/// The industry-standard shaper used by Chrome, Firefox, LibreOffice, and
/// virtually every cross-platform app that handles complex scripts. Supports
/// all 200+ Unicode scripts, full GSUB/GPOS feature processing, and variable
/// fonts. Requires a C toolchain at build time.
#[cfg(feature = "shaping-hb")]
pub use typf_shape_hb as shape_hb;

/// macOS CoreText native shaper (macOS only).
///
/// Delegates to Apple's text engine — the same one used by Cocoa, Safari, and
/// the macOS UI. Excellent variable font support, native Apple typography feel,
/// and 2.5× faster than the separate shaper+renderer path when combined with
/// the CoreText linra backend. Only available on macOS.
#[cfg(feature = "shaping-ct")]
pub use typf_shape_ct as shape_ct;

/// ICU Unicode normalization + HarfBuzz shaping.
///
/// Runs ICU's Unicode normalization (NFC/NFD), bidirectional analysis, and
/// script detection on the input text before passing it to HarfBuzz. Use this
/// when you need to handle every Unicode edge case: combining characters,
/// mixed-direction strings, or text from arbitrary sources that may not already
/// be normalized.
#[cfg(feature = "shaping-icu-hb")]
pub use typf_shape_icu_hb as shape_icu_hb;

// ── Rendering backends ───────────────────────────────────────────────────────
//
// Each renderer implements `typf_core::traits::Renderer`. They all accept the
// same `ShapingResult` and `RenderParams`; pick based on quality, color font
// support, and platform constraints.

/// Debug renderer: emits HarfBuzz-compatible JSON instead of pixels.
///
/// Useful for inspecting shaper output, comparing glyph positions with
/// reference tools, or building custom pipelines that consume glyph data
/// rather than rendered images.
#[cfg(feature = "render-json")]
pub use typf_render_json as render_json;

/// Opixa: pure Rust rasterizer with optional SIMD acceleration.
///
/// Reads TrueType/CFF/CFF2 outlines and rasterizes them into a monochrome or
/// grayscale bitmap. Fast and dependency-free; does not support color glyphs
/// (COLR/SVG/bitmap). The default renderer in the `minimal` feature set.
#[cfg(feature = "render-opixa")]
pub use typf_render_opixa as render_opixa;

/// CoreGraphics renderer (macOS only).
///
/// Renders through Apple's CoreGraphics framework — the same engine that draws
/// macOS's UI. Produces pixel-perfect results that match what users see in
/// native apps, including support for sbix bitmap glyphs and COLR color fonts.
#[cfg(feature = "render-cg")]
pub use typf_render_cg as render_cg;

/// tiny-skia renderer: production-quality rasterization in pure Rust.
///
/// Uses the `tiny-skia` crate (a Rust port of Skia's path rasterizer) to
/// produce anti-aliased bitmaps. Supports color fonts: COLR v0/v1, SVG glyphs
/// (via resvg), and bitmap glyphs (CBDT/sbix). Good all-round choice when
/// color font support matters.
#[cfg(feature = "render-skia")]
pub use typf_render_skia as render_skia;

/// Vello CPU renderer: modern 2D rendering without a GPU.
///
/// Uses the `vello_cpu` crate for high-quality rendering with 256-level
/// anti-aliasing and native glyph caching. Supports COLR color fonts and
/// bitmap glyphs. Pure Rust — no GPU required. Good for server-side rendering
/// where GPUs are unavailable.
#[cfg(feature = "render-vello-cpu")]
pub use typf_render_vello_cpu as render_vello_cpu;

/// Zeno renderer: pure Rust, 256-level anti-aliasing, color fonts.
///
/// Uses the `zeno` path rasterizer — a pure Rust rasterizer that produces
/// output comparable to Skia. Supports COLR v0/v1, SVG glyphs (via resvg),
/// and bitmap glyphs. No C dependencies; runs everywhere.
#[cfg(feature = "render-zeno")]
pub use typf_render_zeno as render_zeno;

/// Everything you need to start rendering
pub mod prelude {
    pub use typf_core::{
        error::{Result, TypfError},
        traits::{Exporter, FontRef, Renderer, Shaper},
        types::{Direction, RenderOutput, ShapingResult},
        Color, GlyphSource, GlyphSourcePreference, Pipeline, RenderMode, RenderParams,
        ShapingParams,
    };
}
