// this_file: backends/typf-core/src/error.rs

//! Error types for the typf rendering engine.
//!
//! This module provides the unified error type [`TypfError`] used throughout
//! the typf rendering engine. All public API functions return `Result<T, TypfError>`.
//!
//! # Error Handling Strategy
//!
//! - **No panics in library code**: All errors are returned as `Result` values
//! - **Structured errors**: Each variant provides context-specific information
//! - **Source chain preservation**: Original errors are preserved via `#[source]`
//! - **Clear error messages**: All variants have descriptive `Display` implementations
//!
//! # Examples
//!
//! ```no_run
//! use typf_core::{TypfError, Result};
//! use std::path::PathBuf;
//!
//! fn load_font(path: &str) -> Result<()> {
//!     // Font loading that might fail
//!     let font_data = std::fs::read(path)
//!         .map_err(|e| TypfError::font_load(PathBuf::from(path), e))?;
//!
//!     Ok(())
//! }
//! ```

use std::path::PathBuf;
use thiserror::Error;

/// Main error type for typf operations
///
/// This enum covers all error cases that can occur during font rendering:
/// - Font loading and discovery failures
/// - Text shaping and layout errors
/// - Rendering and rasterization failures
/// - Parameter validation errors
/// - Backend availability issues
///
/// Each variant provides context-specific information to help with debugging.
#[derive(Error, Debug)]
pub enum TypfError {
    /// Font loading error
    #[error("Failed to load font: {path:?}")]
    FontLoadError {
        /// Path that failed to load.
        path: PathBuf,
        /// Underlying IO or parsing error.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Font not found
    #[error("Font not found: {name}")]
    FontNotFound {
        /// Name of the font that could not be resolved.
        name: String,
    },

    /// Invalid font data
    #[error("Invalid font data")]
    InvalidFontData,

    /// Shaping error
    #[error("Failed to shape text: {reason}")]
    ShapingError {
        /// Explanation for the shaping failure.
        reason: String,
    },

    /// Rendering error
    #[error("Failed to render: {reason}")]
    RenderError {
        /// Explanation for the rendering failure.
        reason: String,
    },

    /// Segmentation error
    #[error("Failed to segment text: {reason}")]
    SegmentationError {
        /// Explanation for the segmentation failure.
        reason: String,
    },

    /// Invalid parameter
    #[error("Invalid parameter: {name} = {value}")]
    InvalidParameter {
        /// Name of the invalid parameter.
        name: String,
        /// Value supplied for the parameter.
        value: String,
    },

    /// Backend not available
    #[error("Backend not available: {name}")]
    BackendNotAvailable {
        /// Name of the backend that is unavailable.
        name: String,
    },

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl TypfError {
    /// Create a new font load error with source error chain
    ///
    /// This preserves the original error context, which is useful for debugging.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use typf_core::TypfError;
    /// # use std::path::PathBuf;
    /// let path = PathBuf::from("/path/to/font.ttf");
    /// let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    /// let err = TypfError::font_load(path, io_err);
    /// ```
    pub fn font_load(
        path: PathBuf,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::FontLoadError {
            path,
            source: Box::new(source),
        }
    }

    /// Create a new shaping error
    ///
    /// Used when HarfBuzz or other shaping engines fail to process text.
    ///
    /// # Example
    ///
    /// ```
    /// # use typf_core::TypfError;
    /// let err = TypfError::shaping("Invalid script code");
    /// assert_eq!(err.to_string(), "Failed to shape text: Invalid script code");
    /// ```
    pub fn shaping(reason: impl Into<String>) -> Self {
        Self::ShapingError {
            reason: reason.into(),
        }
    }

    /// Create a new rendering error
    ///
    /// Used when rasterization or output generation fails.
    ///
    /// # Example
    ///
    /// ```
    /// # use typf_core::TypfError;
    /// let err = TypfError::render("Out of memory during rasterization");
    /// ```
    pub fn render(reason: impl Into<String>) -> Self {
        Self::RenderError {
            reason: reason.into(),
        }
    }

    /// Create a new segmentation error
    ///
    /// Used when ICU-based text segmentation fails.
    ///
    /// # Example
    ///
    /// ```
    /// # use typf_core::TypfError;
    /// let err = TypfError::segmentation("Failed to detect script boundaries");
    /// ```
    pub fn segmentation(reason: impl Into<String>) -> Self {
        Self::SegmentationError {
            reason: reason.into(),
        }
    }

    /// Create a new generic error
    ///
    /// Use sparingly - prefer specific error variants when possible.
    ///
    /// # Example
    ///
    /// ```
    /// # use typf_core::TypfError;
    /// let err = TypfError::other("Unexpected internal state");
    /// ```
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}
