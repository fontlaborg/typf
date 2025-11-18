//! TYPF v2.0 - A modular text rendering pipeline
//!
//! TYPF provides a six-stage pipeline for text rendering:
//! 1. Input parsing and validation
//! 2. Unicode processing and normalization
//! 3. Font selection and loading
//! 4. Text shaping
//! 5. Glyph rendering
//! 6. Output export
//!
//! # Features
//!
//! - **Modular Architecture**: Each stage can be replaced with different implementations
//! - **Performance**: SIMD optimizations and multi-level caching
//! - **Flexibility**: Support for multiple shaping and rendering backends
//! - **Safety**: Memory-safe with minimal unsafe code
//!
//! # Example
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
//! ```
//!
//! # Feature Flags
//!
//! - `minimal`: Minimal build with NoneShaper and OrgeRenderer
//! - `unicode`: Unicode processing support
//! - `fontdb`: Font database support
//! - `export-pnm`: PNM export formats (PPM, PGM, PBM)
//! - `full`: All features enabled

pub use typf_core::{error, traits, Color, Pipeline, RenderParams, ShapingParams};

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

#[cfg(feature = "shaping-none")]
pub use typf_shape_none as shape_none;

#[cfg(feature = "render-orge")]
pub use typf_render_orge as render_orge;

/// Common imports for typical usage
pub mod prelude {
    pub use typf_core::{
        error::{Result, TypfError},
        traits::{Exporter, FontRef, Renderer, Shaper},
        types::{Direction, RenderOutput, ShapingResult},
        Color, Pipeline, RenderParams, ShapingParams,
    };
}
