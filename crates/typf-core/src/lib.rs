//! TYPF Core - Pipeline framework and trait definitions
//!
//! This crate provides the core abstractions and types for the TYPF text rendering pipeline.
//!
//! # Overview
//!
//! TYPF uses a six-stage pipeline architecture:
//!
//! 1. **Input Parsing** - Text and parameters
//! 2. **Unicode Processing** - Normalization, bidi, segmentation
//! 3. **Font Selection** - Font database, TTC support
//! 4. **Shaping** - Glyph selection and positioning
//! 5. **Rendering** - Rasterization or vector output
//! 6. **Export** - Final format (PNG, SVG, JSON, etc.)
//!
//! # Quick Start
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
//! // Build a pipeline
//! let pipeline = Pipeline::builder()
//!     .shaper(Arc::new(MyShaper))
//!     .renderer(Arc::new(MyRenderer))
//!     .exporter(Arc::new(MyExporter))
//!     .build()?;
//!
//! // Process text
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
//! # Traits
//!
//! Implement these traits to create custom backends:
//!
//! - [`Stage`] - Base trait for all pipeline stages
//! - [`Shaper`] - Text shaping backend
//! - [`Renderer`] - Rendering backend
//! - [`Exporter`] - Export format backend
//! - [`traits::FontRef`] - Font data access
//!
//! # Types
//!
//! See the [`types`] module for core data structures.

pub mod cache;
pub mod context;
pub mod error;
pub mod pipeline;
pub mod traits;

pub use context::PipelineContext;
pub use error::{Result, TypfError};
pub use pipeline::{Pipeline, PipelineBuilder};
pub use traits::{Exporter, Renderer, Shaper, Stage};

/// Core types and utilities
pub mod types {
    /// A glyph ID
    pub type GlyphId = u32;

    /// Text direction
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Direction {
        LeftToRight,
        RightToLeft,
        TopToBottom,
        BottomToTop,
    }

    /// A positioned glyph
    #[derive(Debug, Clone, PartialEq)]
    pub struct PositionedGlyph {
        pub id: GlyphId,
        pub x: f32,
        pub y: f32,
        pub advance: f32,
        pub cluster: u32,
    }

    /// Shaping result
    #[derive(Debug, Clone)]
    pub struct ShapingResult {
        pub glyphs: Vec<PositionedGlyph>,
        pub advance_width: f32,
        pub advance_height: f32,
        pub direction: Direction,
    }

    /// Rendering output formats
    #[derive(Debug, Clone)]
    pub enum RenderOutput {
        Bitmap(BitmapData),
        Vector(VectorData),
        Json(String),
    }

    /// Bitmap data
    #[derive(Debug, Clone)]
    pub struct BitmapData {
        pub width: u32,
        pub height: u32,
        pub format: BitmapFormat,
        pub data: Vec<u8>,
    }

    /// Bitmap formats
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BitmapFormat {
        Rgba8,
        Rgb8,
        Gray8,
        Gray1,
    }

    /// Vector data (SVG, PDF, etc.)
    #[derive(Debug, Clone)]
    pub struct VectorData {
        pub format: VectorFormat,
        pub data: String,
    }

    /// Vector formats
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum VectorFormat {
        Svg,
        Pdf,
    }

    /// Options for text segmentation
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

    /// A run of text with uniform properties
    #[derive(Debug, Clone)]
    pub struct TextRun {
        pub text: String,
        pub start: usize,
        pub end: usize,
        pub script: icu_properties::Script,
        pub language: String,
        pub direction: Direction,
    }
}

/// Shaping parameters
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

/// Rendering parameters
#[derive(Debug, Clone)]
pub struct RenderParams {
    pub foreground: Color,
    pub background: Option<Color>,
    pub padding: u32,
    pub antialias: bool,
}

impl Default for RenderParams {
    fn default() -> Self {
        Self {
            foreground: Color::black(),
            background: None,
            padding: 0,
            antialias: true,
        }
    }
}

/// Color representation
#[derive(Debug, Clone, Copy, PartialEq)]
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
