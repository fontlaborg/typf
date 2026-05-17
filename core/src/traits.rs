//! The contracts that bind every backend together.
//!
//! Typf's pipeline has three interchangeable steps, each defined by a trait:
//!
//! ```text
//! "Hello, مرحبا"  ──[Shaper]──▶  positioned glyphs  ──[Renderer]──▶  pixels/paths  ──[Exporter]──▶  PNG/SVG/…
//! ```
//!
//! Swap any step without touching the others. Five shaper implementations ×
//! seven renderer implementations = 35 valid backend combinations, all wired
//! through these four traits.
//!
//! ## The Players
//!
//! - [`Stage`] — The foundation every pipeline component builds upon
//! - [`FontRef`] — Your window into font data and metrics
//! - [`Shaper`] — Converts Unicode text into positioned glyphs (step 1)
//! - [`Renderer`] — Draws those glyphs into pixels or vector paths (step 2)
//! - [`Exporter`] — Encodes the rendered output into a file format (step 3)
//!
//! ## What "shaping" means
//!
//! Text is not a sequence of pictures. "A" in isolation looks different from
//! "A" kerned next to "V". Arabic letters change shape depending on whether
//! they start, continue, or end a word. Hindi consonants stack vertically into
//! ligatures. Even "fi" in a well-made Latin font is a single merged glyph.
//!
//! A [`Shaper`] reads the font's OpenType rules (GSUB/GPOS tables) and figures
//! out *which* glyph IDs to use, in *what order*, at *what positions*. The
//! result is a [`ShapingResult`][crate::types::ShapingResult]: a flat list of
//! glyph IDs each carrying an (x, y) offset from the run origin.
//!
//! ## What "rendering" means
//!
//! Once you know which glyphs go where, you need to draw them. Each glyph in
//! a font is stored as a set of Bézier curves (the outline). A [`Renderer`]
//! reads those curves and either rasterizes them into a pixel grid (bitmap
//! output) or traces them as SVG paths (vector output).
//!
//! ## What "exporting" means
//!
//! The rendered pixels or paths live in memory as a [`RenderOutput`][crate::types::RenderOutput].
//! An [`Exporter`] encodes that into bytes you can write to disk — PNG, PNM,
//! SVG, JSON, etc.

// this_file: crates/typf-core/src/traits.rs

use crate::{error::Result, types::*, PipelineContext, RenderParams, ShapingParams};
use std::sync::Arc;

/// Every pipeline dancer learns these same steps
///
/// Implement Stage and your component can join the six-stage procession
/// that transforms text into rendered output.
///
/// ```ignore
/// struct MyStage;
///
/// impl Stage for MyStage {
///     fn name(&self) -> &'static str {
///         "my-stage"
///     }
///
///     fn process(&self, context: PipelineContext) -> Result<PipelineContext> {
///         // Transform the context, pass it forward
///         Ok(context)
///     }
/// }
/// ```
pub trait Stage: Send + Sync {
    /// Who are you? Used for debugging and logging
    fn name(&self) -> &'static str;

    /// Do your work and pass the context forward
    ///
    /// Take the context, make your changes, and return it for the next stage.
    fn process(&self, context: PipelineContext) -> Result<PipelineContext>;
}

/// A uniform window into font data, regardless of the underlying font library.
///
/// Shapers and renderers do not care whether the font was loaded by `fontdb`,
/// `skrifa`, `ttf-parser`, or a custom loader. They only care about this
/// abstraction: give me the raw bytes, tell me the coordinate scale, look up a
/// glyph ID by character, and tell me how wide that glyph is.
///
/// ## Font coordinate units
///
/// Every measurement inside a font file is in "font units" — an arbitrary
/// integer coordinate space whose scale is set by `units_per_em`. A glyph
/// that is 700 font-units wide in a font with `units_per_em = 1000` is 70% of
/// one em wide. At `font_size = 48px`, that glyph is `48 × 0.70 = 33.6px`.
///
/// Type 1 and most OTF (CFF) fonts use 1000 units/em.
/// TrueType (glyf) fonts typically use 2048 units/em.
///
/// ```ignore
/// struct MyFont {
///     data: Vec<u8>,
///     // ... your internal state
/// }
///
/// impl FontRef for MyFont {
///     fn data(&self) -> &[u8] {
///         &self.data
///     }
///
///     fn units_per_em(&self) -> u16 {
///         1000
///     }
///
///     fn glyph_id(&self, ch: char) -> Option<GlyphId> {
///         // Look up the Unicode character in the font's cmap table.
///         Some(42)
///     }
///
///     fn advance_width(&self, glyph_id: GlyphId) -> f32 {
///         // Read the hmtx table for this glyph's horizontal advance.
///         500.0
///     }
/// }
/// ```
pub trait FontRef: Send + Sync {
    /// The raw font file bytes (TTF/OTF/WOFF data).
    ///
    /// Shapers pass this directly to their underlying library (HarfBuzz, skrifa,
    /// etc.). Keep the slice alive as long as the font is in use.
    fn data(&self) -> &[u8];

    /// Shared font bytes for zero-copy downstream consumption.
    ///
    /// Implementations that store their bytes in an `Arc` SHOULD override this
    /// to avoid per-call allocations in downstream libraries that require shared
    /// ownership (e.g. Vello's `FontData`). Default returns `None`.
    fn data_shared(&self) -> Option<Arc<dyn AsRef<[u8]> + Send + Sync>> {
        None
    }

    /// The font's internal coordinate scale (units per em).
    ///
    /// Divide any font-unit measurement by this value to get a fraction of one
    /// em, then multiply by the target `font_size` in pixels to get pixels.
    /// Common values: 1000 (CFF/Type 1), 2048 (TrueType).
    fn units_per_em(&self) -> u16;

    /// Font-wide metrics in font units: ascent, descent, and line gap.
    ///
    /// Backends that parse OpenType tables SHOULD provide this so callers can
    /// compute baselines without depending on a specific font parser. Returns
    /// `None` if the implementation cannot or does not parse these tables.
    fn metrics(&self) -> Option<FontMetrics> {
        None
    }

    /// Map a Unicode character to the font-specific glyph ID that represents it.
    ///
    /// This is a cmap lookup. Returns `None` when the font does not contain
    /// a glyph for `ch` (the font is missing that character). Note that shapers
    /// may return *different* glyph IDs for the same character after applying
    /// OpenType substitution — this method only reflects the raw cmap mapping.
    fn glyph_id(&self, ch: char) -> Option<GlyphId>;

    /// Horizontal advance of `glyph_id` in font units.
    ///
    /// This is the cursor-advance distance read from the `hmtx` table: how far
    /// right the pen moves after drawing this glyph. Multiply by
    /// `(font_size / units_per_em)` to convert to pixels.
    fn advance_width(&self, glyph_id: GlyphId) -> f32;

    /// Total number of glyphs in the font (from the `maxp` table).
    ///
    /// Used to validate shaper output — a glyph ID ≥ `glyph_count` is invalid.
    /// Returns `None` if the implementation cannot provide this cheaply.
    fn glyph_count(&self) -> Option<u32> {
        None
    }

    /// Variable font axes from the `fvar` table.
    ///
    /// A variable font encodes a continuous design space along named axes such
    /// as `wght` (weight), `wdth` (width), `slnt` (slant), or `ital` (italic).
    /// Each axis has a min, default, and max value; callers pass specific
    /// coordinates via [`ShapingParams::variations`][crate::ShapingParams] and
    /// [`RenderParams::variations`][crate::RenderParams].
    ///
    /// Returns `None` for static (non-variable) fonts or if the font file
    /// cannot be parsed. Returns `Some(empty vec)` if the `fvar` table exists
    /// but declares no axes.
    fn variation_axes(&self) -> Option<Vec<VariationAxis>> {
        None
    }

    /// Returns `true` if this font has at least one variable axis.
    fn is_variable(&self) -> bool {
        self.variation_axes().is_some_and(|axes| !axes.is_empty())
    }
}

/// Step 1 of the pipeline: turn Unicode text into positioned glyphs.
///
/// A shaper reads the font's OpenType layout tables and answers: *which glyph
/// IDs should appear, in what order, at what (x, y) offsets?*
///
/// That question is trivial for ASCII — one codepoint, one glyph, advance by
/// the glyph width. It is not trivial for:
///
/// - **Arabic/Hebrew**: runs right-to-left; letters have up to four contextual
///   forms (isolated, initial, medial, final) selected from the GSUB table.
/// - **Devanagari/Bengali**: consonant clusters combine into conjuncts; vowel
///   signs reorder visually relative to their base consonant.
/// - **Thai/Khmer**: no spaces between words; vowels and tone marks stack
///   above or below their base letter.
/// - **Latin**: ligatures (`fi` → one glyph), kerning (`AV` tighter than `AX`),
///   small-caps, old-style figures — all driven by GSUB/GPOS feature tags.
///
/// ## Available implementations
///
/// | Crate | Name | What it uses |
/// |---|---|---|
/// | `typf-shape-none` | `"none"` | Simple LTR passthrough — no OpenType features |
/// | `typf-shape-hb` | `"harfbuzz"` | HarfBuzz C library via FFI — industry standard |
/// | `typf-shape-hr` | `"harfrust"` | Pure Rust HarfBuzz port — zero C deps |
/// | `typf-shape-icu-hb` | `"icu-hb"` | ICU normalization + HarfBuzz — full Unicode pipeline |
/// | `typf-shape-ct` | `"coretext"` | macOS CoreText — native Apple typography |
pub trait Shaper: Send + Sync {
    /// Identify yourself in logs and error messages.
    fn name(&self) -> &'static str;

    /// Turn `text` into a list of positioned glyphs using `font` and `params`.
    ///
    /// The returned [`ShapingResult`] contains one [`PositionedGlyph`][crate::types::PositionedGlyph]
    /// per rendered glyph (which may differ in count from the input codepoints —
    /// ligatures merge multiple codepoints; some scripts expand one codepoint
    /// into several glyphs).
    fn shape(
        &self,
        text: &str,
        font: Arc<dyn FontRef>,
        params: &ShapingParams,
    ) -> Result<ShapingResult>;

    /// Returns `true` if this shaper can correctly handle the named script.
    ///
    /// Script names follow ISO 15924 four-letter codes: `"Arab"`, `"Deva"`,
    /// `"Latn"`, `"Thai"`, etc. Returns `false` by default; implementations
    /// should explicitly declare what they support.
    fn supports_script(&self, _script: &str) -> bool {
        false
    }

    /// Flush any internally cached shaping results.
    fn clear_cache(&self) {}
}

/// Step 2 of the pipeline: draw positioned glyphs into pixels or vector paths.
///
/// A renderer receives a [`ShapingResult`] (the list of positioned glyph IDs)
/// and a font, then reads each glyph's outline from the font file (Bézier
/// curves stored in the `glyf`, `CFF`, or `CFF2` table), and either:
///
/// - **Rasterizes** the curves into a pixel grid with anti-aliasing, or
/// - **Traces** the curves into SVG path data for vector output.
///
/// The output is a [`RenderOutput`] — either a [`BitmapData`][crate::types::BitmapData]
/// (RGBA pixels) or [`VectorData`][crate::types::VectorData] (SVG/PDF text).
///
/// ## Available implementations
///
/// | Crate | Name | Notes |
/// |---|---|---|
/// | `typf-render-opixa` | `"opixa"` | Pure Rust, SIMD-optimized, monochrome only |
/// | `typf-render-skia` | `"skia"` | tiny-skia (Rust port of Skia), full color font support |
/// | `typf-render-zeno` | `"zeno"` | Pure Rust zeno rasterizer, 256-level AA, color fonts |
/// | `typf-render-vello-cpu` | `"vello-cpu"` | Vello CPU path, pure Rust, COLR/bitmap color fonts |
/// | `typf-render-vello` | `"vello"` | Vello GPU (Metal/Vulkan/DX12), outline-only today |
/// | `typf-render-cg` | `"coregraphics"` | macOS CoreGraphics — native Apple rendering |
/// | `typf-render-json` | `"json"` | Emits HarfBuzz-compatible JSON instead of pixels |
pub trait Renderer: Send + Sync {
    /// Identify yourself in logs and error messages.
    fn name(&self) -> &'static str;

    /// Draw `shaped` glyphs using `font`'s outlines and return the visual output.
    ///
    /// Implementations must respect `params.output` (bitmap vs vector) and
    /// `params.glyph_sources` (which glyph table to prefer for color fonts).
    fn render(
        &self,
        shaped: &ShapingResult,
        font: Arc<dyn FontRef>,
        params: &RenderParams,
    ) -> Result<RenderOutput>;

    /// Returns `true` if this renderer can produce the named output format.
    ///
    /// Format strings match file extensions: `"png"`, `"svg"`, `"json"`.
    /// Returns `false` by default; implementations should declare support.
    fn supports_format(&self, _format: &str) -> bool {
        false
    }

    /// Free up any internally cached glyph bitmaps or path data.
    fn clear_cache(&self) {}
}

/// Step 3 of the pipeline: encode rendered output as bytes ready to write to disk.
///
/// An exporter takes a [`RenderOutput`] (pixels or vector paths already in
/// memory) and serializes it into the final file format — PNG, PNM, SVG, JSON,
/// etc. The returned `Vec<u8>` can be written directly to a file or streamed
/// over a network.
pub trait Exporter: Send + Sync {
    /// Identify yourself in logs and error messages.
    fn name(&self) -> &'static str;

    /// Encode `output` and return the file bytes.
    fn export(&self, output: &RenderOutput) -> Result<Vec<u8>>;

    /// File extension for this format, without the leading dot (e.g. `"png"`).
    fn extension(&self) -> &'static str;

    /// MIME type for HTTP Content-Type headers (e.g. `"image/png"`).
    fn mime_type(&self) -> &'static str;
}
