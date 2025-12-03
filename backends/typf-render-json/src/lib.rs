//! JSON Renderer - When you need to see what the shaper really did
//!
//! Sometimes pixels aren't enough—you need the raw glyph data.
//! This renderer outputs shaping results in HarfBuzz-compatible JSON,
//! perfect for debugging text rendering issues or building custom pipelines.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use typf_core::{
    error::Result,
    traits::{FontRef, Renderer, Stage},
    types::{RenderOutput, ShapingResult},
    RenderParams,
};

/// Individual glyph data that matches HarfBuzz's JSON output format
///
/// Use this to compare Typf's output with HarfBuzz reference implementations
/// or to feed glyph data into custom rendering pipelines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HbGlyphInfo {
    pub g: u32,    // Glyph identifier in the font
    pub cl: usize, // Cluster mapping back to original text
    pub ax: i32,   // Horizontal advance (how far to move after this glyph)
    pub ay: i32,   // Vertical advance (for vertical text layouts)
    pub dx: i32,   // Horizontal offset from default position
    pub dy: i32,   // Vertical offset from default position
}

/// Schema version for JSON output format
pub const JSON_SCHEMA_VERSION: &str = "1.0";

/// Complete shaping output in a debug-friendly format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonOutput {
    /// Schema version for forward compatibility
    pub schema_version: String,
    pub glyphs: Vec<HbGlyphInfo>, // The positioned glyph sequence
    pub direction: String,        // Text direction that influenced shaping
    pub script: Option<String>,   // Script detection result (when available)
    pub language: Option<String>, // Language context (when available)
    pub advance: f32,             // Total width of the shaped text
}

/// The renderer that turns shaping results into structured data
///
/// Unlike bitmap renderers, this doesn't create pixels—it creates insight.
/// Perfect for testing, debugging, or feeding other rendering systems.
pub struct JsonRenderer;

impl JsonRenderer {
    /// Creates a renderer that speaks JSON instead of pixels
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
        // Transform Typf's PositionedGlyph into HarfBuzz-compatible format
        let glyphs: Vec<HbGlyphInfo> = shaped
            .glyphs
            .iter()
            .map(|g| {
                // HarfBuzz thinks in 1/64 pixel units (26.6 fixed point)
                // This conversion ensures compatibility with HB tools
                let scale = 64.0;
                HbGlyphInfo {
                    g: g.id,
                    cl: g.cluster as usize,
                    ax: (g.advance * scale) as i32,
                    ay: 0, // Typf doesn't yet support vertical advances
                    dx: (g.x * scale) as i32,
                    dy: (g.y * scale) as i32,
                }
            })
            .collect();

        // Package everything for downstream consumption
        let output = JsonOutput {
            glyphs,
            direction: format!("{:?}", shaped.direction),
            script: None,   // Future: extract from Unicode processing stage
            language: None, // Future: extract from input parameters
            advance: shaped.advance_width,
        };

        // Turn our structured data into a pretty JSON string
        let json = serde_json::to_string_pretty(&output).map_err(|e| {
            typf_core::error::TypfError::RenderingFailed(
                typf_core::error::RenderError::BackendError(e.to_string()),
            )
        })?;

        // Hand back the JSON for whatever comes next
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
