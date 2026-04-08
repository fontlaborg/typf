//! Core traits and shared types for Typf.
//!
//! This crate defines the common contract used by the rest of the Typf
//! workspace: pipeline stages, shaping and rendering traits, shared error
//! types, caches, and the data structures that move from one stage to the
//! next.
//!
//! Typf models text rendering as six conceptual steps:
//!
//! 1. read input text,
//! 2. analyse script and direction,
//! 3. choose a font,
//! 4. shape characters into positioned glyphs,
//! 5. render those glyphs into pixels or vector data,
//! 6. export the result.
//!
//! The default pipeline currently exposes the shaping, rendering, and export
//! steps directly. The earlier stages still matter because other crates may use
//! them for bidi resolution, text segmentation, or font fallback.
//!
//! To extend Typf, implement one or more of these traits:
//!
//! - [`Stage`] for a general pipeline step,
//! - [`Shaper`] for text shaping,
//! - [`Renderer`] for glyph rendering,
//! - [`Exporter`] for serialization,
//! - [`traits::FontRef`] for access to font data.
//!
//! Shared values passed between those steps live in [`types`].

use std::collections::HashSet;

/// Default maximum bitmap dimension (width): 16,777,216 pixels (16M)
///
/// This prevents memory explosions from pathological fonts or extreme render sizes.
/// The width can be very large for long text runs.
pub const DEFAULT_MAX_BITMAP_WIDTH: u32 = 16 * 1024 * 1024;

/// Default maximum bitmap height: 16k pixels
///
/// Height is more strictly limited than width since vertical overflow is rarer
/// and tall bitmaps are often pathological.
pub const DEFAULT_MAX_BITMAP_HEIGHT: u32 = 16 * 1024;

/// Default maximum total bitmap pixels: 1 Gpix
///
/// This caps total memory regardless of aspect ratio.
/// A 4-megapixel RGBA8 bitmap consumes 16 MB.
pub const DEFAULT_MAX_BITMAP_PIXELS: u64 = 1024 * 1024 * 1024;

pub fn get_max_bitmap_width() -> u32 {
    std::env::var("TYPF_MAX_BITMAP_WIDTH")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_MAX_BITMAP_WIDTH)
}

pub fn get_max_bitmap_height() -> u32 {
    std::env::var("TYPF_MAX_BITMAP_HEIGHT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_MAX_BITMAP_HEIGHT)
}

pub fn get_max_bitmap_pixels() -> u64 {
    std::env::var("TYPF_MAX_BITMAP_PIXELS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_MAX_BITMAP_PIXELS)
}

pub mod cache;
pub mod cache_config;
pub mod context;
pub mod error;
pub mod ffi;
pub mod glyph_cache;
pub mod linra;
pub mod pipeline;
pub mod shaping_cache;
pub mod traits;

pub use context::PipelineContext;
pub use error::{Result, TypfError};
pub use pipeline::{Pipeline, PipelineBuilder};
pub use traits::{Exporter, Renderer, Shaper, Stage};

/// Maximum font size in pixels to prevent DoS attacks.
///
/// Set very high (100K px) to catch only obvious attacks while
/// allowing legitimate large-format rendering use cases.
pub const MAX_FONT_SIZE: f32 = 100_000.0;

/// Maximum number of glyphs to render in a single operation.
///
/// Set very high (10M glyphs) to catch only obvious attacks while
/// allowing legitimate bulk text processing.
pub const MAX_GLYPH_COUNT: usize = 10_000_000;

/// The data structures that move through the pipeline.
///
/// These types carry the output of one stage into the next, so they are shared
/// across shapers, renderers, exporters, and bindings.
pub mod types {
    /// Unique identifier for a glyph within a font.
    pub type GlyphId = u32;

    /// Minimal, stable font-wide metrics in font units.
    ///
    /// These are intended for layout/baseline decisions by consumers that only have a `FontRef`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FontMetrics {
        /// Units per em (typically 1000 or 2048).
        pub units_per_em: u16,
        /// Distance from baseline to top of highest glyph.
        pub ascent: i16,
        /// Distance from baseline to bottom of lowest glyph (usually negative).
        pub descent: i16,
        /// Recommended gap between lines.
        pub line_gap: i16,
    }

    /// A variable font axis definition.
    ///
    /// Describes one axis of variation in a variable font (e.g., weight, width, slant).
    /// Values are in font design units unless otherwise specified.
    #[derive(Debug, Clone, PartialEq)]
    pub struct VariationAxis {
        /// 4-character tag (e.g., "wght", "wdth", "slnt", "ital").
        pub tag: String,
        /// Human-readable name (if available from name table).
        pub name: Option<String>,
        /// Minimum axis value.
        pub min_value: f32,
        /// Default axis value.
        pub default_value: f32,
        /// Maximum axis value.
        pub max_value: f32,
        /// Whether this is a hidden axis (not shown to users).
        pub hidden: bool,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Direction {
        /// Standard Latin, Cyrillic, etc.
        LeftToRight,
        /// Arabic, Hebrew, etc.
        RightToLeft,
        /// Vertical CJK (not fully supported yet).
        TopToBottom,
        /// Vertical (rare).
        BottomToTop,
    }

    /// One shaped glyph with its final position inside the run.
    #[derive(Debug, Clone, PartialEq)]
    pub struct PositionedGlyph {
        /// The glyph ID in the font.
        pub id: GlyphId,
        /// X position of the glyph origin (relative to run start).
        pub x: f32,
        /// Y position of the glyph origin.
        pub y: f32,
        /// How much to advance the cursor after this glyph.
        pub advance: f32,
        /// The Unicode cluster index this glyph belongs to.
        pub cluster: u32,
    }

    /// Output from the shaping stage, ready for rendering.
    #[derive(Debug, Clone)]
    pub struct ShapingResult {
        /// The list of positioned glyphs.
        pub glyphs: Vec<PositionedGlyph>,
        /// Total width of the shaped run.
        pub advance_width: f32,
        /// Total height of the shaped run (usually 0 for horizontal text).
        pub advance_height: f32,
        /// Overall direction of the run.
        pub direction: Direction,
    }

    #[derive(Debug, Clone)]
    pub enum RenderOutput {
        /// Rasterized bitmap (PNG, PBM, etc.).
        Bitmap(BitmapData),
        /// Serialized vector format (SVG, PDF).
        Vector(VectorData),
        /// JSON representation of glyph data.
        Json(String),
        /// Path geometry for GPU pipelines and tessellators.
        Geometry(GeometryData),
    }

    impl RenderOutput {
        /// Returns the approximate heap size in bytes of this render output.
        ///
        /// Used by byte-weighted caches to enforce memory limits.
        pub fn byte_size(&self) -> usize {
            match self {
                RenderOutput::Bitmap(b) => b.byte_size(),
                RenderOutput::Vector(v) => v.data.len(),
                RenderOutput::Json(s) => s.len(),
                RenderOutput::Geometry(g) => g.byte_size(),
            }
        }

        /// Returns true if this is geometry output suitable for GPU consumption.
        pub fn is_geometry(&self) -> bool {
            matches!(self, RenderOutput::Geometry(_))
        }
    }

    #[derive(Debug, Clone)]
    pub struct BitmapData {
        pub width: u32,
        pub height: u32,
        pub format: BitmapFormat,
        pub data: Vec<u8>,
    }

    impl BitmapData {
        /// Returns the heap size in bytes of the pixel data.
        pub fn byte_size(&self) -> usize {
            self.data.len()
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BitmapFormat {
        Rgba8,
        Rgb8,
        Gray8,
        Gray1,
    }

    #[derive(Debug, Clone)]
    pub struct VectorData {
        pub format: VectorFormat,
        pub data: String,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum VectorFormat {
        Svg,
        Pdf,
    }

    // =========================================================================
    // Stage 5 Geometry Types (for GPU pipelines and external tessellators)
    // =========================================================================

    /// A single path operation for vector glyph outlines.
    ///
    /// These primitives match the common subset of path operations supported by
    /// Cairo, Skia, CoreGraphics, Direct2D, and wgpu tessellators.
    ///
    /// Coordinates are in font units (typically 1000 or 2048 per em).
    /// Scale by `(font_size / units_per_em)` to convert to pixels.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum PathOp {
        /// Move to a new position without drawing (M x y)
        MoveTo { x: f32, y: f32 },
        /// Draw a straight line to a point (L x y)
        LineTo { x: f32, y: f32 },
        /// Quadratic Bézier curve (Q cx cy x y)
        QuadTo { cx: f32, cy: f32, x: f32, y: f32 },
        /// Cubic Bézier curve (C c1x c1y c2x c2y x y)
        CubicTo {
            c1x: f32,
            c1y: f32,
            c2x: f32,
            c2y: f32,
            x: f32,
            y: f32,
        },
        /// Close the current subpath (Z)
        Close,
    }

    /// Glyph path data with positioning for a single glyph.
    ///
    /// Contains the vector outline of a glyph and its rendered position.
    /// Can be consumed by external tessellators or GPU pipelines.
    #[derive(Debug, Clone, PartialEq)]
    pub struct GlyphPath {
        /// Glyph ID in the font
        pub glyph_id: GlyphId,
        /// X position of the glyph origin (in rendered coordinates)
        pub x: f32,
        /// Y position of the glyph origin (in rendered coordinates)
        pub y: f32,
        /// Path operations defining the glyph outline (in font units)
        pub ops: Vec<PathOp>,
    }

    /// Geometry output for GPU pipelines and vector consumers.
    ///
    /// This provides path operations that can be:
    /// - Tessellated by external libraries (lyon, earcutr)
    /// - Converted to GPU meshes for wgpu/Vulkan
    /// - Used for hit testing and text selection
    /// - Exported to vector formats
    ///
    /// # Coordinate Systems
    ///
    /// - Path ops are in font units (unscaled)
    /// - Glyph positions (x, y) are in rendered coordinates (scaled to font_size)
    /// - Consumer must apply `font_size / units_per_em` scaling to path ops
    #[derive(Debug, Clone)]
    pub struct GeometryData {
        /// Glyph paths with their positions
        pub glyphs: Vec<GlyphPath>,
        /// Total advance width in rendered coordinates
        pub advance_width: f32,
        /// Total advance height in rendered coordinates
        pub advance_height: f32,
        /// Font units per em (for scaling path ops to rendered coordinates)
        pub units_per_em: u16,
        /// Font size used for positioning (pixels)
        pub font_size: f32,
    }

    impl GeometryData {
        /// Returns the approximate heap size in bytes.
        pub fn byte_size(&self) -> usize {
            self.glyphs
                .iter()
                .map(|g| {
                    std::mem::size_of::<GlyphPath>() + g.ops.len() * std::mem::size_of::<PathOp>()
                })
                .sum()
        }

        /// Returns an iterator over glyph paths.
        pub fn iter(&self) -> impl Iterator<Item = &GlyphPath> {
            self.glyphs.iter()
        }

        /// Scale factor to convert font units to rendered coordinates.
        pub fn scale(&self) -> f32 {
            self.font_size / self.units_per_em as f32
        }
    }

    /// The source type of glyph data in a font
    ///
    /// Different glyph types require different rendering approaches:
    /// - Outlines can be scaled and exported to SVG paths
    /// - Bitmaps are pre-rendered at specific sizes
    /// - COLR/SVG glyphs contain color information
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum GlyphType {
        /// Standard vector outline (glyf/CFF/CFF2 tables)
        Outline,
        /// Color glyph using COLR/CPAL tables (v0 or v1)
        Colr,
        /// Color glyph using embedded SVG documents
        Svg,
        /// Embedded bitmap glyph (CBDT/CBLC tables - Google format)
        BitmapCbdt,
        /// Embedded bitmap glyph (EBDT/EBLC tables - legacy format)
        BitmapEbdt,
        /// Embedded bitmap glyph (sbix table - Apple format)
        BitmapSbix,
        /// Glyph with no data (space, missing glyph, etc.)
        Empty,
    }

    impl GlyphType {
        /// Returns true if this glyph type contains vector outline data
        pub fn has_outline(&self) -> bool {
            matches!(self, GlyphType::Outline | GlyphType::Colr)
        }

        /// Returns true if this glyph type contains bitmap data
        pub fn is_bitmap(&self) -> bool {
            matches!(
                self,
                GlyphType::BitmapCbdt | GlyphType::BitmapEbdt | GlyphType::BitmapSbix
            )
        }

        /// Returns true if this glyph type contains color information
        pub fn is_color(&self) -> bool {
            matches!(
                self,
                GlyphType::Colr | GlyphType::Svg | GlyphType::BitmapCbdt | GlyphType::BitmapSbix
            )
        }
    }

    #[derive(Debug, Clone)]
    pub struct SegmentOptions {
        pub language: Option<String>,
        pub bidi_resolve: bool,
        pub font_fallback: bool,
        pub script_itemize: bool,
    }

    impl Default for SegmentOptions {
        fn default() -> Self {
            Self {
                language: None,
                bidi_resolve: true,
                font_fallback: false,
                script_itemize: true,
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct TextRun {
        pub text: String,
        pub start: usize,
        pub end: usize,
        pub script: icu_properties::props::Script,
        pub language: String,
        pub direction: Direction,
    }
}

#[derive(Debug, Clone)]
pub struct ShapingParams {
    pub size: f32,
    pub direction: types::Direction,
    pub language: Option<String>,
    pub script: Option<String>,
    pub features: Vec<(String, u32)>,
    pub variations: Vec<(String, f32)>,
    pub letter_spacing: f32,
}

impl Default for ShapingParams {
    fn default() -> Self {
        Self {
            size: 16.0,
            direction: types::Direction::LeftToRight,
            language: None,
            script: None,
            features: Vec::new(),
            variations: Vec::new(),
            letter_spacing: 0.0,
        }
    }
}

impl ShapingParams {
    /// Validate shaping parameters against security limits
    ///
    /// Returns an error if:
    /// - Font size is not finite (`NaN`, `+/-inf`)
    /// - Font size exceeds [`MAX_FONT_SIZE`] (currently 100,000 pixels)
    /// - Font size is negative or zero
    ///
    /// # Example
    ///
    /// ```
    /// use typf_core::ShapingParams;
    ///
    /// let params = ShapingParams { size: 48.0, ..Default::default() };
    /// params.validate().expect("valid params");
    ///
    /// let bad_params = ShapingParams { size: 200_000.0, ..Default::default() };
    /// assert!(bad_params.validate().is_err());
    /// ```
    pub fn validate(&self) -> Result<(), error::ShapingError> {
        if !self.size.is_finite() {
            return Err(error::ShapingError::BackendError(
                "Font size must be finite".to_string(),
            ));
        }
        if self.size <= 0.0 {
            return Err(error::ShapingError::BackendError(
                "Font size must be positive".to_string(),
            ));
        }
        if self.size > MAX_FONT_SIZE {
            return Err(error::ShapingError::FontSizeTooLarge(
                self.size,
                MAX_FONT_SIZE,
            ));
        }
        Ok(())
    }
}

/// Validate glyph count against security limits
///
/// Returns an error if glyph count exceeds [`MAX_GLYPH_COUNT`] (currently 10M).
/// Call this before rendering to prevent resource exhaustion from malicious input.
///
/// # Example
///
/// ```
/// use typf_core::{validate_glyph_count, types::ShapingResult};
///
/// // In a renderer, before processing:
/// // validate_glyph_count(shaped.glyphs.len())?;
/// ```
pub fn validate_glyph_count(count: usize) -> Result<(), error::RenderError> {
    if count > MAX_GLYPH_COUNT {
        return Err(error::RenderError::GlyphCountTooLarge(
            count,
            MAX_GLYPH_COUNT,
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum GlyphSource {
    Glyf,
    Cff,
    Cff2,
    Colr0,
    Colr1,
    Svg,
    Sbix,
    Cbdt,
    Ebdt,
}

const DEFAULT_GLYPH_SOURCES: [GlyphSource; 9] = [
    GlyphSource::Glyf,
    GlyphSource::Cff2,
    GlyphSource::Cff,
    GlyphSource::Colr1,
    GlyphSource::Colr0,
    GlyphSource::Svg,
    GlyphSource::Sbix,
    GlyphSource::Cbdt,
    GlyphSource::Ebdt,
];

/// Preference ordering and deny list for glyph sources
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlyphSourcePreference {
    pub prefer: Vec<GlyphSource>,
    pub deny: HashSet<GlyphSource>,
}

impl GlyphSourcePreference {
    /// Build a preference list with an optional deny set.
    ///
    /// - Empty `prefer` uses the default outline-first order.
    /// - Duplicates are removed while keeping first-seen order.
    /// - Denied sources are removed from the preferred list.
    pub fn from_parts(
        prefer: Vec<GlyphSource>,
        deny: impl IntoIterator<Item = GlyphSource>,
    ) -> Self {
        let deny: HashSet<GlyphSource> = deny.into_iter().collect();
        let source_order = if prefer.is_empty() {
            DEFAULT_GLYPH_SOURCES.to_vec()
        } else {
            prefer
        };

        let mut seen = HashSet::new();
        let mut normalized = Vec::new();

        for source in source_order {
            if deny.contains(&source) {
                continue;
            }
            if seen.insert(source) {
                normalized.push(source);
            }
        }

        Self {
            prefer: normalized,
            deny,
        }
    }

    /// Effective order with current denies applied.
    pub fn effective_order(&self) -> Vec<GlyphSource> {
        self.prefer
            .iter()
            .copied()
            .filter(|src| !self.deny.contains(src))
            .collect()
    }
}

impl Default for GlyphSourcePreference {
    fn default() -> Self {
        Self::from_parts(DEFAULT_GLYPH_SOURCES.to_vec(), [])
    }
}

#[derive(Debug, Clone)]
pub struct RenderParams {
    pub foreground: Color,
    pub background: Option<Color>,
    pub padding: u32,
    pub antialias: bool,
    /// Variable font variations like [("wght", 700.0), ("wdth", 100.0)]
    /// Variable fonts need specific coordinates to render correctly
    pub variations: Vec<(String, f32)>,
    /// CPAL color palette index for COLR color glyphs (0 = default palette)
    pub color_palette: u16,
    /// Allowed glyph sources (order + deny list)
    pub glyph_sources: GlyphSourcePreference,
    /// Desired render output mode (bitmap or vector)
    pub output: RenderMode,
}

impl Default for RenderParams {
    fn default() -> Self {
        Self {
            foreground: Color::black(),
            background: None,
            padding: 0,
            antialias: true,
            variations: Vec::new(),
            color_palette: 0,
            glyph_sources: GlyphSourcePreference::default(),
            output: RenderMode::Bitmap,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderMode {
    /// Raster output (default)
    Bitmap,
    /// Vector output (currently SVG only)
    Vector(types::VectorFormat),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn black() -> Self {
        Self::rgba(0, 0, 0, 255)
    }

    pub const fn white() -> Self {
        Self::rgba(255, 255, 255, 255)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic)]
mod tests {
    use super::types::*;
    use super::{error::ShapingError, ShapingParams, MAX_FONT_SIZE};

    #[test]
    fn test_path_op_size() {
        // PathOp should be reasonably compact for efficient storage
        assert!(std::mem::size_of::<PathOp>() <= 32);
    }

    #[test]
    fn test_geometry_data_byte_size() {
        let geometry = GeometryData {
            glyphs: vec![
                GlyphPath {
                    glyph_id: 1,
                    x: 0.0,
                    y: 0.0,
                    ops: vec![
                        PathOp::MoveTo { x: 0.0, y: 0.0 },
                        PathOp::LineTo { x: 100.0, y: 0.0 },
                        PathOp::Close,
                    ],
                },
                GlyphPath {
                    glyph_id: 2,
                    x: 50.0,
                    y: 0.0,
                    ops: vec![
                        PathOp::MoveTo { x: 0.0, y: 0.0 },
                        PathOp::QuadTo {
                            cx: 50.0,
                            cy: 100.0,
                            x: 100.0,
                            y: 0.0,
                        },
                        PathOp::Close,
                    ],
                },
            ],
            advance_width: 100.0,
            advance_height: 0.0,
            units_per_em: 1000,
            font_size: 16.0,
        };

        // byte_size should be non-zero and reasonable
        let size = geometry.byte_size();
        assert!(size > 0);
        assert!(size < 1024); // Should be small for this test case
    }

    #[test]
    fn test_geometry_data_scale() {
        let geometry = GeometryData {
            glyphs: vec![],
            advance_width: 0.0,
            advance_height: 0.0,
            units_per_em: 2048,
            font_size: 16.0,
        };

        let scale = geometry.scale();
        assert!((scale - 16.0 / 2048.0).abs() < 0.0001);
    }

    #[test]
    fn test_render_output_geometry_variant() {
        let geometry = GeometryData {
            glyphs: vec![GlyphPath {
                glyph_id: 42,
                x: 0.0,
                y: 0.0,
                ops: vec![PathOp::MoveTo { x: 0.0, y: 0.0 }, PathOp::Close],
            }],
            advance_width: 50.0,
            advance_height: 0.0,
            units_per_em: 1000,
            font_size: 24.0,
        };

        let output = RenderOutput::Geometry(geometry);
        assert!(output.is_geometry());
        assert!(output.byte_size() > 0);
    }

    #[test]
    fn test_shaping_params_validate_when_non_finite_size_then_error() {
        for size in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let params = ShapingParams {
                size,
                ..Default::default()
            };
            let error = params.validate().expect_err("non-finite size must fail");
            match error {
                ShapingError::BackendError(message) => {
                    assert!(
                        message.contains("finite"),
                        "expected finite-size guidance, got: {}",
                        message
                    );
                },
                other => panic!("expected BackendError, got {:?}", other),
            }
        }
    }

    #[test]
    fn test_shaping_params_validate_when_positive_size_then_ok() {
        let params = ShapingParams {
            size: 24.0,
            ..Default::default()
        };
        params
            .validate()
            .expect("positive finite size should validate");
    }

    #[test]
    fn test_shaping_params_validate_when_size_above_max_then_error() {
        let params = ShapingParams {
            size: MAX_FONT_SIZE + 1.0,
            ..Default::default()
        };
        let error = params
            .validate()
            .expect_err("oversized font size must fail validation");
        match error {
            ShapingError::FontSizeTooLarge(size, max) => {
                assert!(
                    size > max,
                    "expected reported size {} to exceed max {}",
                    size,
                    max
                );
            },
            other => panic!("expected FontSizeTooLarge, got {:?}", other),
        }
    }
}
