//! The contracts that bind every backend together
//!
//! Five traits, infinite possibilities. Each trait defines a role
//! in the pipeline, allowing you to swap implementations without
//! touching a single line of user code.
//!
//! ## The Players
//!
//! - [`Stage`] - The foundation every pipeline component builds upon
//! - [`FontRef`] - Your window into font data and metrics
/// - [`Shaper`] - Where characters become glyphs
/// - [`Renderer`] - Where glyphs become images
/// - [`Exporter`] - Where images become files

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

/// Your key to unlocking font secrets
///
/// Every font format speaks the same language through this trait.
/// TTF, OTF, WOFF - they all expose their data and metrics the same way.
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
///         1000 // Common for Type 1 fonts
///     }
///
///     fn glyph_id(&self, ch: char) -> Option<GlyphId> {
///         // Turn Unicode into font-specific glyph IDs
///         Some(42)
///     }
///
///     fn advance_width(&self, glyph_id: GlyphId) -> f32 {
///         // How far to move after this glyph
///         500.0
///     }
/// }
/// ```
pub trait FontRef: Send + Sync {
    /// Raw font bytes as they live in the file
    fn data(&self) -> &[u8];

    /// The font's internal coordinate system scale
    ///
    /// Used to convert between font units and rendered pixels.
    /// Type 1 fonts use 1000, TrueType often uses 2048.
    fn units_per_em(&self) -> u16;

    /// Find the glyph that represents this character
    ///
    /// Returns None when the font doesn't contain this character.
    fn glyph_id(&self, ch: char) -> Option<GlyphId>;

    /// How wide this glyph stands in font units
    ///
    /// This spacing determines how glyphs sit next to each other.
    fn advance_width(&self, glyph_id: GlyphId) -> f32;

    /// How many glyphs this font contains
    ///
    /// Useful for validation when shapers return glyph IDs.
    fn glyph_count(&self) -> Option<u32> {
        None // Not all implementations can provide this
    }
}

/// Where characters learn their positions
///
/// Text shaping is where script rules, font features, and character clusters
/// collide to produce perfectly positioned glyphs ready for rendering.
pub trait Shaper: Send + Sync {
    /// Identify yourself in logs and error messages
    fn name(&self) -> &'static str;

    /// Transform characters into positioned glyphs
    fn shape(
        &self,
        text: &str,
        font: Arc<dyn FontRef>,
        params: &ShapingParams,
    ) -> Result<ShapingResult>;

    /// Can you handle this script?
    fn supports_script(&self, _script: &str) -> bool {
        true // Optimistic by default
    }

    /// Flush any cached shaping data
    fn clear_cache(&self) {}
}

/// Where glyphs become visible
///
/// Rasterizers turn positioned glyphs into pixels. Vector renderers
/// turn them into paths. Both implement this trait.
pub trait Renderer: Send + Sync {
    /// Your renderer's signature
    fn name(&self) -> &'static str;

    /// Convert glyphs to visual output
    fn render(
        &self,
        shaped: &ShapingResult,
        font: Arc<dyn FontRef>,
        params: &RenderParams,
    ) -> Result<RenderOutput>;

    /// Do you understand this output format?
    fn supports_format(&self, _format: &str) -> bool {
        true // Assume we can handle anything
    }

    /// Free up rendering resources
    fn clear_cache(&self) {}
}

/// The final step: pixels become files
///
/// Exporters know how to encode rendered output into the format
/// users actually want - PNG, SVG, JSON, and more.
pub trait Exporter: Send + Sync {
    /// Who are you?
    fn name(&self) -> &'static str;

    /// Encode the rendered output as bytes
    fn export(&self, output: &RenderOutput) -> Result<Vec<u8>>;

    /// What file extension should be used?
    fn extension(&self) -> &'static str;

    /// What MIME type identifies your format?
    fn mime_type(&self) -> &'static str;
}
