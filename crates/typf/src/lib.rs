//! TYPF: Text becomes art through six carefully crafted stages
//!
//! Every character tells a story. TYPF tells it through:
//! 1. Input parsing - Raw text finds structure
//! 2. Unicode processing - Scripts normalize, language emerges
//! 3. Font selection - The right font finds each character
//! 4. Text shaping - Characters learn their positions
//! 5. Glyph rendering - Positions become pixels or paths
//! 6. Export - The final format emerges
//!
//! ## Why TYPF?
//!
//! - **Swap any stage** - Need a different shaper? Just plug it in
//! - **Blazing fast** - SIMD and multi-level caching keep you responsive
//! - **Pick your backends** - HarfBuzz, CoreText, Skia, Orge - you choose
//! - **Memory safe** - All the performance, none of the unsafety
//!
//! ## Start Rendering
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
//! ## Build What You Need
//!
//! - `minimal` - Just the essentials: NoneShaper + OrgeRenderer
//! - `unicode` - Script detection, bidi, segmentation
//! - `fontdb` - System font discovery and caching
//! - `export-pnm` - PPM, PGM, PBM output formats
//! - `full` - Everything unlocked and ready to go

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

// Shaping backends
#[cfg(feature = "shaping-none")]
pub use typf_shape_none as shape_none;

#[cfg(feature = "shaping-hb")]
pub use typf_shape_hb as shape_hb;

#[cfg(feature = "shaping-ct")]
pub use typf_shape_ct as shape_ct;

#[cfg(feature = "shaping-icu-hb")]
pub use typf_shape_icu_hb as shape_icu_hb;

// Rendering backends
#[cfg(feature = "render-json")]
pub use typf_render_json as render_json;

#[cfg(feature = "render-orge")]
pub use typf_render_orge as render_orge;

#[cfg(feature = "render-cg")]
pub use typf_render_cg as render_cg;

#[cfg(feature = "render-skia")]
pub use typf_render_skia as render_skia;

#[cfg(feature = "render-zeno")]
pub use typf_render_zeno as render_zeno;

/// Everything you need to start rendering
pub mod prelude {
    pub use typf_core::{
        error::{Result, TypfError},
        traits::{Exporter, FontRef, Renderer, Shaper},
        types::{Direction, RenderOutput, ShapingResult},
        Color, Pipeline, RenderParams, ShapingParams,
    };
}
