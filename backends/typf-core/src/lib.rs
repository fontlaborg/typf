// this_file: backends/typf-core/src/lib.rs

#![deny(missing_docs)]

//! Core traits and types for the typf text rendering engine.
//!
//! This crate provides the foundational infrastructure for all typf backends:
//! - [`Backend`] trait for text segmentation, shaping, and rendering
//! - [`FontCache`] for efficient font and glyph management
//! - [`TypfError`] for unified error handling
//! - Common types like [`Font`], [`Glyph`], and [`RenderOptions`]

pub mod backend_trait;
pub use backend_trait::{DynBackend, BackendFeatures, Point, FontMetrics};

pub mod cache;
pub mod error;
pub mod surface;
pub mod traits;
pub mod types;
pub mod utils;

pub use cache::{FontCache, FontCacheConfig};
pub use error::TypfError;
pub use surface::{RenderSurface, SurfaceFormat};
pub use traits::{Backend as CoreBackendTrait, FontShaper, GlyphRenderer, TextSegmenter}; // Renamed Backend
pub use types::{
    Bitmap, Features, Font, Glyph, RenderFormat, RenderOptions, RenderOutput, SegmentOptions,
    ShapingResult, SvgOptions, TextRun,
};

/// Result type for typf operations
pub type Result<T> = std::result::Result<T, TypfError>;
