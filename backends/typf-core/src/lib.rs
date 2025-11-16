// this_file: backends/typf-core/src/lib.rs

//! Core traits and types for the typf text rendering engine.

pub mod cache;
pub mod error;
pub mod surface;
pub mod traits;
pub mod types;
pub mod utils;

pub use cache::FontCache;
pub use error::TypfError;
pub use surface::{RenderSurface, SurfaceFormat};
pub use traits::{Backend, FontShaper, GlyphRenderer, TextSegmenter};
pub use types::{
    Bitmap, Features, Font, Glyph, RenderFormat, RenderOptions, RenderOutput, SegmentOptions,
    ShapingResult, SvgOptions, TextRun,
};

/// Result type for typf operations
pub type Result<T> = std::result::Result<T, TypfError>;
