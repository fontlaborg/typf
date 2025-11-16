// this_file: crates/typf-render/src/lib.rs

// TODO: Add #![deny(missing_docs)] after documenting all public items (41 items)
//! Rendering utilities for typf text engine.

pub mod batch;
pub mod outlines;
pub mod output;
pub mod perf;
pub mod svg;

pub use batch::{BatchItem, BatchRenderer, BatchResult};
pub use outlines::{glyph_outline, GlyphOutline, OutlineCommand};
pub use output::{ImageOutput, OutputError};
pub use perf::{BufferPool, MetricType, PerfMetrics, PerfScope, PerfStats};
pub use svg::SvgRenderer;
