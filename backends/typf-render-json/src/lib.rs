//! JSON rendering backend for TYPF
//!
//! Outputs shaping results in HarfBuzz-compatible JSON format.
//! This is useful for debugging and for applications that need to process
//! shaping results programmatically.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use typf_core::{
    error::Result,
    traits::{FontRef, Renderer, Stage},
    types::{RenderOutput, ShapingResult},
    RenderParams,
};

/// HarfBuzz-compatible glyph information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HbGlyphInfo {
    pub g: u32,    // glyph ID
    pub cl: usize, // cluster
    pub ax: i32,   // x advance (in font units)
    pub ay: i32,   // y advance (in font units)
    pub dx: i32,   // x offset (in font units)
    pub dy: i32,   // y offset (in font units)
}

/// JSON output format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonOutput {
    pub glyphs: Vec<HbGlyphInfo>,
    pub direction: String,
    pub script: Option<String>,
    pub language: Option<String>,
    pub advance: f32,
}

/// JSON renderer backend
pub struct JsonRenderer;

impl JsonRenderer {
    /// Create a new JSON renderer
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Stage for JsonRenderer {
    fn name(&self) -> &'static str {
        "JSON"
    }

    fn process(
        &self,
        ctx: typf_core::context::PipelineContext,
    ) -> Result<typf_core::context::PipelineContext> {
        // JSON renderer doesn't process pipeline context directly
        Ok(ctx)
    }
}

impl Renderer for JsonRenderer {
    fn name(&self) -> &'static str {
        "JSON"
    }

    fn supports_format(&self, format: &str) -> bool {
        // JSON renderer only produces JSON output, not bitmaps or vectors
        matches!(format.to_lowercase().as_str(), "json")
    }

    fn render(
        &self,
        shaped: &ShapingResult,
        _font: Arc<dyn FontRef>,
        _params: &RenderParams,
    ) -> Result<RenderOutput> {
        // Convert PositionedGlyphs to HbGlyphInfo
        let glyphs: Vec<HbGlyphInfo> = shaped
            .glyphs
            .iter()
            .map(|g| {
                // Convert to font units (assuming 64 subpixel units per pixel)
                let scale = 64.0; // HarfBuzz uses 1/64 pixel units
                HbGlyphInfo {
                    g: g.id,
                    cl: g.cluster as usize,
                    ax: (g.advance * scale) as i32,
                    ay: 0,
                    dx: (g.x * scale) as i32,
                    dy: (g.y * scale) as i32,
                }
            })
            .collect();

        // Create JSON output
        let output = JsonOutput {
            glyphs,
            direction: format!("{:?}", shaped.direction),
            script: None,   // TODO: Extract from params
            language: None, // TODO: Extract from params
            advance: shaped.advance_width,
        };

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&output).map_err(|e| {
            typf_core::error::TypfError::RenderingFailed(
                typf_core::error::RenderError::BackendError(e.to_string()),
            )
        })?;

        // Return as JSON output
        Ok(RenderOutput::Json(json))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use typf_core::types::{Direction, PositionedGlyph};

    struct MockFont;
    impl FontRef for MockFont {
        fn data(&self) -> &[u8] {
            &[]
        }
        fn units_per_em(&self) -> u16 {
            1000
        }
        fn glyph_id(&self, _ch: char) -> Option<u32> {
            Some(1)
        }
        fn advance_width(&self, _glyph_id: u32) -> f32 {
            500.0
        }
    }

    #[test]
    fn test_json_renderer() {
        use typf_core::traits::Renderer;
        let renderer = JsonRenderer::new();
        assert_eq!(Renderer::name(&renderer), "JSON");

        let shaped = ShapingResult {
            glyphs: vec![
                PositionedGlyph {
                    id: 65,
                    x: 0.0,
                    y: 0.0,
                    advance: 10.0,
                    cluster: 0,
                },
                PositionedGlyph {
                    id: 66,
                    x: 10.0,
                    y: 0.0,
                    advance: 10.0,
                    cluster: 1,
                },
            ],
            advance_width: 20.0,
            advance_height: 16.0,
            direction: Direction::LeftToRight,
        };

        let font = Arc::new(MockFont);
        let params = RenderParams::default();

        let result = renderer.render(&shaped, font, &params);
        assert!(result.is_ok());

        if let Ok(RenderOutput::Json(json)) = result {
            assert!(json.contains("\"g\""));
            assert!(json.contains("\"cl\""));
            assert!(json.contains("\"ax\""));
        }
    }

    #[test]
    fn test_json_renderer_empty() {
        let renderer = JsonRenderer::new();
        let shaped = ShapingResult {
            glyphs: vec![],
            advance_width: 0.0,
            advance_height: 0.0,
            direction: Direction::LeftToRight,
        };

        let font = Arc::new(MockFont);
        let params = RenderParams::default();

        let result = renderer.render(&shaped, font, &params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_supports_format() {
        let renderer = JsonRenderer::new();

        // JSON renderer only supports JSON output
        assert!(renderer.supports_format("json"));
        assert!(renderer.supports_format("JSON"));

        // It doesn't support bitmap or vector formats
        assert!(!renderer.supports_format("bitmap"));
        assert!(!renderer.supports_format("rgba"));
        assert!(!renderer.supports_format("svg"));
        assert!(!renderer.supports_format("png"));
        assert!(!renderer.supports_format("unknown"));
    }
}
