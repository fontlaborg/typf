//! Linra rendering: shape and render in a single pass
//!
//! For maximum performance, some platform APIs can shape AND render text in a
//! single operation. This module provides the trait and types for such linra
//! backends.
//!
//! ## Why Linra Rendering?
//!
//! Traditional pipeline: Shape → Extract Glyphs → Render Each Glyph
//! Linra pipeline: Shape + Render in One Call
//!
//! On macOS, CoreText's CTLineDraw shapes and renders atomically.
//! On Windows, DirectWrite's DrawTextLayout does the same.
//!
//! The linra approach:
//! - Eliminates glyph extraction overhead
//! - Allows the OS to optimize internally
//! - Can leverage hardware acceleration

use std::sync::Arc;

use crate::error::Result;
use crate::traits::FontRef;
use crate::types::RenderOutput;
use crate::Color;

/// Combined parameters for linra shape+render operations
///
/// This replaces separate ShapingParams and RenderParams when using
/// a linra renderer that handles both steps internally.
#[derive(Debug, Clone)]
pub struct LinraRenderParams {
    /// Font size in points
    pub size: f32,
    /// Text direction
    pub direction: crate::types::Direction,
    /// Text color
    pub foreground: Color,
    /// Background color (None = transparent)
    pub background: Option<Color>,
    /// Padding around the rendered text
    pub padding: u32,
    /// Variable font axis values like [("wght", 700.0), ("wdth", 100.0)]
    pub variations: Vec<(String, f32)>,
    /// OpenType feature settings like [("liga", 1), ("kern", 1)]
    pub features: Vec<(String, u32)>,
    /// Language code for shaping (e.g., "en", "ar", "zh")
    pub language: Option<String>,
    /// Script tag for shaping (e.g., "latn", "arab")
    pub script: Option<String>,
    /// Enable antialiasing
    pub antialias: bool,
    /// Extra spacing between characters (in points, can be negative)
    pub letter_spacing: f32,
}

impl Default for LinraRenderParams {
    fn default() -> Self {
        Self {
            size: 16.0,
            direction: crate::types::Direction::LeftToRight,
            foreground: Color::black(),
            background: None,
            padding: 0,
            variations: Vec::new(),
            features: Vec::new(),
            language: None,
            script: None,
            antialias: true,
            letter_spacing: 0.0,
        }
    }
}

impl LinraRenderParams {
    /// Create params with a specific font size
    pub fn with_size(size: f32) -> Self {
        Self {
            size,
            ..Default::default()
        }
    }

    /// Convert to separate ShapingParams for compatibility
    pub fn to_shaping_params(&self) -> crate::ShapingParams {
        crate::ShapingParams {
            size: self.size,
            direction: self.direction,
            language: self.language.clone(),
            script: self.script.clone(),
            features: self.features.clone(),
            variations: self.variations.clone(),
            letter_spacing: self.letter_spacing,
        }
    }

    /// Convert to separate RenderParams for compatibility
    pub fn to_render_params(&self) -> crate::RenderParams {
        crate::RenderParams {
            foreground: self.foreground,
            background: self.background,
            padding: self.padding,
            antialias: self.antialias,
            variations: self.variations.clone(),
        }
    }
}

/// Linra text renderer: shapes AND renders in a single operation
///
/// Implementations of this trait bypass the separate shaper/renderer pipeline
/// to achieve maximum performance through platform-native APIs.
///
/// ## Platform Implementations
///
/// - **macOS**: `CoreTextLinraRenderer` uses CTLineDraw
/// - **Windows**: `DirectWriteLinraRenderer` uses DrawTextLayout
///
/// ## Usage
///
/// ```rust,no_run
/// use typf_core::linra::{LinraRenderer, LinraRenderParams};
/// use typf_core::traits::FontRef;
/// use std::sync::Arc;
///
/// fn render_text<R: LinraRenderer>(
///     renderer: &R,
///     text: &str,
///     font: Arc<dyn FontRef>,
/// ) -> typf_core::Result<typf_core::types::RenderOutput> {
///     let params = LinraRenderParams::with_size(24.0);
///     renderer.render_text(text, font, &params)
/// }
/// ```
pub trait LinraRenderer: Send + Sync {
    /// The renderer's name (e.g., "coretext-linra", "directwrite-linra")
    fn name(&self) -> &'static str;

    /// Shape and render text in a single operation
    ///
    /// This method performs both text shaping (character→glyph mapping,
    /// positioning, feature application) and rendering (rasterization)
    /// in a single pass through the platform's native text API.
    ///
    /// # Arguments
    ///
    /// * `text` - The text string to render
    /// * `font` - Font to use for rendering
    /// * `params` - Combined shaping and rendering parameters
    ///
    /// # Returns
    ///
    /// Rendered output as a bitmap or vector format
    fn render_text(
        &self,
        text: &str,
        font: Arc<dyn FontRef>,
        params: &LinraRenderParams,
    ) -> Result<RenderOutput>;

    /// Clear any internal caches
    fn clear_cache(&self) {}

    /// Check if this renderer supports a given output format
    fn supports_format(&self, format: &str) -> bool {
        matches!(format, "bitmap" | "rgba")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linra_params_default() {
        let params = LinraRenderParams::default();
        assert_eq!(params.size, 16.0);
        assert!(params.antialias);
        assert!(params.background.is_none());
    }

    #[test]
    fn test_linra_params_with_size() {
        let params = LinraRenderParams::with_size(24.0);
        assert_eq!(params.size, 24.0);
    }

    #[test]
    fn test_params_conversion() {
        let linra = LinraRenderParams {
            size: 32.0,
            variations: vec![("wght".to_string(), 700.0)],
            features: vec![("liga".to_string(), 1)],
            language: Some("en".to_string()),
            ..Default::default()
        };

        let shaping = linra.to_shaping_params();
        assert_eq!(shaping.size, 32.0);
        assert_eq!(shaping.variations.len(), 1);
        assert_eq!(shaping.features.len(), 1);

        let render = linra.to_render_params();
        assert_eq!(render.variations.len(), 1);
    }
}
