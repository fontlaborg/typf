//! Typf Core: Six stages from text to pixels
//!
//! Text enters as characters, exits as rendered images. This crate holds the pipeline
//! that makes that transformation possible through six distinct stages.
//!
//! ## The Pipeline
//!
//! Every piece of text follows the same journey:
//!
//! 1. **Input Parsing** - Raw text becomes structured data
//! 2. **Unicode Processing** - Scripts normalize, bidi resolves, segmentation happens
//! 3. **Font Selection** - The right font finds its way to each character
//! 4. **Shaping** - Characters transform into positioned glyphs
//! 5. **Rendering** - Glyphs become pixels or vectors
//! 6. **Export** - Final output emerges as PNG, SVG, or JSON
//!
//! ## Build Your First Pipeline
//!
//! ```rust,no_run
//! use typf_core::{Pipeline, RenderParams, ShapingParams};
//! use std::sync::Arc;
//!
//! # use typf_core::traits::*;
//! # use typf_core::context::PipelineContext;
//! # use typf_core::error::TypfError;
//! # struct MyShaper;
//! # impl Stage for MyShaper {
//! #     fn name(&self) -> &'static str { "test" }
//! #     fn process(&self, _ctx: PipelineContext) -> Result<PipelineContext, TypfError> { unimplemented!() }
//! # }
//! # impl Shaper for MyShaper {
//! #     fn name(&self) -> &'static str { "test" }
//! #     fn shape(&self, _: &str, _: Arc<dyn FontRef>, _: &ShapingParams)
//! #         -> typf_core::Result<typf_core::types::ShapingResult> { unimplemented!() }
//! # }
//! # struct MyRenderer;
//! # impl Stage for MyRenderer {
//! #     fn name(&self) -> &'static str { "test" }
//! #     fn process(&self, _ctx: PipelineContext) -> Result<PipelineContext, TypfError> { unimplemented!() }
//! # }
//! # impl Renderer for MyRenderer {
//! #     fn name(&self) -> &'static str { "test" }
//! #     fn render(&self, _: &typf_core::types::ShapingResult, _: Arc<dyn FontRef>, _: &RenderParams)
//! #         -> typf_core::Result<typf_core::types::RenderOutput> { unimplemented!() }
//! # }
//! # struct MyExporter;
//! # impl Stage for MyExporter {
//! #     fn name(&self) -> &'static str { "test" }
//! #     fn process(&self, _ctx: PipelineContext) -> Result<PipelineContext, TypfError> { unimplemented!() }
//! # }
//! # impl Exporter for MyExporter {
//! #     fn name(&self) -> &'static str { "test" }
//! #     fn export(&self, _: &typf_core::types::RenderOutput)
//! #         -> typf_core::Result<Vec<u8>> { unimplemented!() }
//! #     fn extension(&self) -> &'static str { "png" }
//! #     fn mime_type(&self) -> &'static str { "image/png" }
//! # }
//! # fn load_font() -> Arc<dyn FontRef> { unimplemented!() }
//!
//! let pipeline = Pipeline::builder()
//!     .shaper(Arc::new(MyShaper))
//!     .renderer(Arc::new(MyRenderer))
//!     .exporter(Arc::new(MyExporter))
//!     .build()?;
//!
//! let font = load_font();
//! let output = pipeline.process(
//!     "Hello, World!",
//!     font,
//!     &ShapingParams::default(),
//!     &RenderParams::default(),
//! )?;
//! # Ok::<(), typf_core::TypfError>(())
//! ```
//!
//! ## The Traits That Power Everything
//!
//! Want to add your own backend? Implement one of these:
//!
//! - [`Stage`] - The foundation every pipeline component builds upon
//! - [`Shaper`] - Where characters become glyphs
//! - [`Renderer`] - Where glyphs become images
//! - [`Exporter`] - Where images become files
//! - [`traits::FontRef`] - Your window into font data
//!
//! Data flows through the types in [`types`] - these structures carry
//! the results from one stage to the next.

use std::collections::HashSet;

pub mod cache;
pub mod context;
pub mod error;
pub mod glyph_cache;
pub mod linra;
pub mod pipeline;
pub mod shaping_cache;
pub mod traits;

pub use context::PipelineContext;
pub use error::{Result, TypfError};
pub use pipeline::{Pipeline, PipelineBuilder};
pub use traits::{Exporter, Renderer, Shaper, Stage};

// =============================================================================
// Security Limits
// =============================================================================

/// Maximum font size in pixels to prevent DoS attacks
///
/// Set very high (100K px) to catch only obvious attacks while
/// allowing legitimate large-format rendering use cases.
pub const MAX_FONT_SIZE: f32 = 100_000.0;

/// Maximum number of glyphs to render in a single operation
///
/// Set very high (10M glyphs) to catch only obvious attacks while
/// allowing legitimate bulk text processing.
pub const MAX_GLYPH_COUNT: usize = 10_000_000;

/// The data structures that power the pipeline
pub mod types {
    /// Unique identifier for a glyph within a font
    pub type GlyphId = u32;

    /// Which way the text flows
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Direction {
        LeftToRight,
        RightToLeft,
        TopToBottom,
        BottomToTop,
    }

    /// A glyph that knows exactly where it belongs
    #[derive(Debug, Clone, PartialEq)]
    pub struct PositionedGlyph {
        pub id: GlyphId,
        pub x: f32,
        pub y: f32,
        pub advance: f32,
        pub cluster: u32,
    }

    /// What emerges after shaping: glyphs positioned and ready to render
    #[derive(Debug, Clone)]
    pub struct ShapingResult {
        pub glyphs: Vec<PositionedGlyph>,
        pub advance_width: f32,
        pub advance_height: f32,
        pub direction: Direction,
    }

    /// The three forms output can take
    #[derive(Debug, Clone)]
    pub enum RenderOutput {
        Bitmap(BitmapData),
        Vector(VectorData),
        Json(String),
    }

    /// Raw pixel data from rasterized glyphs
    #[derive(Debug, Clone)]
    pub struct BitmapData {
        pub width: u32,
        pub height: u32,
        pub format: BitmapFormat,
        pub data: Vec<u8>,
    }

    /// How pixels are arranged in the bitmap
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BitmapFormat {
        Rgba8,
        Rgb8,
        Gray8,
        Gray1,
    }

    /// Scalable paths instead of pixels
    #[derive(Debug, Clone)]
    pub struct VectorData {
        pub format: VectorFormat,
        pub data: String,
    }

    /// Which vector format we're speaking
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum VectorFormat {
        Svg,
        Pdf,
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

    /// How text gets broken into manageable pieces
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

    /// Text that shares the same script, direction, and language
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

/// How shaping should behave
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

/// Which glyph data sources are allowed and in what order
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

/// How rendering should look
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

/// Target output for rendering operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RenderMode {
    /// Raster output (default)
    Bitmap,
    /// Vector output (currently SVG only)
    Vector(types::VectorFormat),
}

/// Simple RGBA color that works everywhere
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
