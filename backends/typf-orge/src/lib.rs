// this_file: backends/typf-orge/src/lib.rs

#![deny(unsafe_code)]

//! orge - ultra-smooth unhinted glyph rasterization.
//!
//! This crate provides a specialized scan converter for supersmooth, unhinted
//! font rendering. It focuses ONLY on the scan conversion algorithm, NOT on hinting.
//!
//! ## Safety
//!
//! This crate is 100% safe Rust with no `unsafe` blocks. The `#![deny(unsafe_code)]`
//! attribute ensures this property is maintained.
//!
//! ## Architecture
//!
//! - `fixed`: F26Dot6 fixed-point arithmetic (26.6 format)
//! - `edge`: Edge lists for scan line algorithm
//! - `curves`: Bézier curve subdivision
//! - `scan_converter`: Main rasterization algorithm
//! - `dropout`: Dropout control for thin features
//! - `grayscale`: Anti-aliasing via oversampling

pub mod curves;
pub mod edge;
pub mod fixed;
pub mod grayscale;
pub mod renderer;
pub mod scan_converter;
// pub mod dropout;

// Re-export main types
pub use renderer::{GlyphRasterizer, Image, OrgeError, Result};

/// Fill rule for scan conversion.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FillRule {
    /// Non-zero winding rule (recommended for fonts).
    NonZeroWinding,
    /// Even-odd rule.
    EvenOdd,
}

/// Dropout control mode.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DropoutMode {
    /// No dropout control.
    None,
    /// Simple dropout (fill gaps in thin stems).
    Simple,
    /// Smart dropout (perpendicular scan + stub detection).
    Smart,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
