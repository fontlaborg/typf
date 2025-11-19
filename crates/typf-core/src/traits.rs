//! Core trait definitions for TYPF
//!
//! This module defines the fundamental traits that power the TYPF text rendering pipeline.
//!
//! # Overview
//!
//! TYPF uses a trait-based architecture to allow swappable backends at each stage:
//! - [`Stage`] - Base trait for pipeline stages
//! - [`FontRef`] - Font data access interface
//! - [`Shaper`] - Text shaping (character to glyph conversion)
//! - [`Renderer`] - Glyph rendering (rasterization)
//! - [`Exporter`] - Output format conversion

use crate::{error::Result, types::*, PipelineContext, RenderParams, ShapingParams};
use std::sync::Arc;

/// A stage in the processing pipeline
///
/// All pipeline components implement this base trait to participate
/// in the six-stage text rendering pipeline.
///
/// # Example
/// ```ignore
/// struct MyStage;
///
/// impl Stage for MyStage {
///     fn name(&self) -> &'static str {
///         "my-stage"
///     }
///
///     fn process(&self, context: PipelineContext) -> Result<PipelineContext> {
///         // Process and return modified context
///         Ok(context)
///     }
/// }
/// ```
pub trait Stage: Send + Sync {
    /// Name of this stage for debugging and logging
    fn name(&self) -> &'static str;

    /// Process the pipeline context through this stage
    ///
    /// Takes ownership of the context, performs transformations,
    /// and returns the modified context for the next stage.
    fn process(&self, context: PipelineContext) -> Result<PipelineContext>;
}

/// Font reference for shaping and rendering
///
/// This trait provides a unified interface for accessing font data and metrics.
/// Implementations can wrap various font formats (TTF, OTF, WOFF, etc.).
///
/// # Example
/// ```ignore
/// struct MyFont {
///     data: Vec<u8>,
///     // ... other fields
/// }
///
/// impl FontRef for MyFont {
///     fn data(&self) -> &[u8] {
///         &self.data
///     }
///
///     fn units_per_em(&self) -> u16 {
///         1000 // typical value
///     }
///
///     fn glyph_id(&self, ch: char) -> Option<GlyphId> {
///         // Map character to glyph ID
///         Some(42)
///     }
///
///     fn advance_width(&self, glyph_id: GlyphId) -> f32 {
///         // Return advance width in font units
///         500.0
///     }
/// }
/// ```
pub trait FontRef: Send + Sync {
    /// Get raw font data (TTF/OTF bytes)
    fn data(&self) -> &[u8];

    /// Get units per em from the font's head table
    ///
    /// This value is used to scale font metrics to the desired size.
    /// Common values are 1000 (Type 1) or 2048 (TrueType).
    fn units_per_em(&self) -> u16;

    /// Map a Unicode character to a glyph ID
    ///
    /// Returns `None` if the character is not present in the font.
    fn glyph_id(&self, ch: char) -> Option<GlyphId>;

    /// Get the advance width for a glyph in font units
    ///
    /// The advance width determines the horizontal spacing between glyphs.
    fn advance_width(&self, glyph_id: GlyphId) -> f32;

    /// Get the total number of glyphs in the font
    ///
    /// This is useful for validating glyph IDs returned by shapers.
    /// Returns None if the glyph count cannot be determined.
    fn glyph_count(&self) -> Option<u32> {
        None // Default implementation returns None
    }
}

/// Text shaping backend
pub trait Shaper: Send + Sync {
    /// Name of this shaping backend
    fn name(&self) -> &'static str;

    /// Shape text into positioned glyphs
    fn shape(
        &self,
        text: &str,
        font: Arc<dyn FontRef>,
        params: &ShapingParams,
    ) -> Result<ShapingResult>;

    /// Check if a script is supported
    fn supports_script(&self, _script: &str) -> bool {
        true // Default: claim to support all scripts
    }

    /// Clear any internal caches
    fn clear_cache(&self) {}
}

/// Rendering backend
pub trait Renderer: Send + Sync {
    /// Name of this rendering backend
    fn name(&self) -> &'static str;

    /// Render shaped text
    fn render(
        &self,
        shaped: &ShapingResult,
        font: Arc<dyn FontRef>,
        params: &RenderParams,
    ) -> Result<RenderOutput>;

    /// Check if a format is supported
    fn supports_format(&self, _format: &str) -> bool {
        true // Default: claim to support all formats
    }

    /// Clear any internal caches
    fn clear_cache(&self) {}
}

/// Export backend for various output formats
pub trait Exporter: Send + Sync {
    /// Name of this exporter
    fn name(&self) -> &'static str;

    /// Export rendered output to bytes
    fn export(&self, output: &RenderOutput) -> Result<Vec<u8>>;

    /// Get the file extension for this format
    fn extension(&self) -> &'static str;

    /// Get MIME type
    fn mime_type(&self) -> &'static str;
}
