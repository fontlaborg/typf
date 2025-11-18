// this_file: backends/typf-core/src/backend_trait.rs

#![deny(missing_docs)]

//! The core backend trait and related types for abstracting over different text rendering backends.

use crate::{Bitmap, Font, RenderOptions};
use crate::types::ShapingResult;

/// A point in 2D space.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct Point {
    /// The x-coordinate.
    pub x: f32,
    /// The y-coordinate.
    pub y: f32,
}

/// Font metrics.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct FontMetrics {
    /// The number of font units per em.
    pub units_per_em: u16,
    /// The ascender in font units.
    pub ascender: i16,
    /// The descender in font units.
    pub descender: i16,
    /// The line gap in font units.
    pub line_gap: i16,
}

/// Features supported by a backend.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BackendFeatures {
    /// Whether the backend supports monochrome rendering.
    pub monochrome: bool,
    /// Whether the backend supports grayscale rendering.
    pub grayscale: bool,
    /// Whether the backend supports subpixel rendering.
    pub subpixel: bool,
    /// Whether the backend supports color emoji.
    pub color_emoji: bool,
}

/// A trait for abstracting over different text rendering backends.
///
/// Implementations of this trait provide the core functionality for
/// text shaping, rasterization, and font metrics calculation.
pub trait DynBackend: Send + Sync {
    /// Returns the name of the backend.
    fn name(&self) -> &'static str;

    /// Shapes the given text using the provided font.
    fn shape_text(&self, text: &str, font: &Font) -> ShapingResult;

    /// Renders a single glyph to a bitmap.
    fn render_glyph(&self, font: &Font, glyph_id: u32, options: RenderOptions) -> Option<Bitmap>;

    /// Renders shaped text to a bitmap.
    fn render_shaped_text(&self, shaped_text: &ShapingResult, options: RenderOptions) -> Option<Bitmap>;

    /// Calculates the font metrics for the given font.
    fn font_metrics(&self, font: &Font) -> FontMetrics;

    /// Returns the features supported by this backend (e.g., monochrome, grayscale).
    fn supported_features(&self) -> BackendFeatures;
}