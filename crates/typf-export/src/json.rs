//! JSON export format
//!
//! Exports shaping results to HarfBuzz-compatible JSON format.

use serde::{Deserialize, Serialize};
use typf_core::{
    error::{ExportError, Result},
    traits::Exporter,
    types::{Direction, RenderOutput, ShapingResult},
};

/// JSON exporter for shaping results
///
/// Produces HarfBuzz-compatible JSON output for debugging and testing.
///
/// # Examples
///
/// ```ignore
/// use typf_export::JsonExporter;
///
/// let exporter = JsonExporter::new();
/// let json = exporter.export(&render_output)?;
/// println!("{}", String::from_utf8_lossy(&json));
/// ```
pub struct JsonExporter {
    /// Whether to pretty-print the JSON
    pretty: bool,
}

impl JsonExporter {
    /// Create a new JSON exporter
    pub fn new() -> Self {
        Self { pretty: false }
    }

    /// Create a JSON exporter with pretty-printing enabled
    pub fn with_pretty_print() -> Self {
        Self { pretty: true }
    }

    /// Export shaping result to HarfBuzz-compatible JSON
    pub fn export_shaping(&self, shaped: &ShapingResult) -> Result<Vec<u8>> {
        let output = HarfBuzzOutput {
            glyphs: shaped
                .glyphs
                .iter()
                .map(|g| HarfBuzzGlyph {
                    glyph_id: g.id,
                    cluster: g.cluster,
                    x_advance: (g.advance * 64.0) as i32, // Convert to 26.6 fixed point
                    y_advance: 0,
                    x_offset: (g.x * 64.0) as i32,
                    y_offset: (g.y * 64.0) as i32,
                })
                .collect(),
            advance_width: shaped.advance_width,
            advance_height: shaped.advance_height,
            direction: match shaped.direction {
                Direction::LeftToRight => "ltr",
                Direction::RightToLeft => "rtl",
                Direction::TopToBottom => "ttb",
                Direction::BottomToTop => "btt",
            }
            .to_string(),
        };

        let json = if self.pretty {
            serde_json::to_string_pretty(&output)
        } else {
            serde_json::to_string(&output)
        }
        .map_err(|e| ExportError::EncodingFailed(e.to_string()))?;

        Ok(json.into_bytes())
    }
}

impl Default for JsonExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl Exporter for JsonExporter {
    fn name(&self) -> &'static str {
        "json"
    }

    fn export(&self, output: &RenderOutput) -> Result<Vec<u8>> {
        match output {
            RenderOutput::Json(json) => Ok(json.as_bytes().to_vec()),
            _ => Err(ExportError::FormatNotSupported(
                "JSON exporter requires JSON render output".into(),
            )
            .into()),
        }
    }

    fn extension(&self) -> &'static str {
        "json"
    }

    fn mime_type(&self) -> &'static str {
        "application/json"
    }
}

/// HarfBuzz-compatible JSON output structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct HarfBuzzOutput {
    glyphs: Vec<HarfBuzzGlyph>,
    advance_width: f32,
    advance_height: f32,
    direction: String,
}

/// HarfBuzz-compatible glyph information
#[derive(Debug, Clone, Serialize, Deserialize)]
struct HarfBuzzGlyph {
    #[serde(rename = "g")]
    glyph_id: u32,
    #[serde(rename = "cl")]
    cluster: u32,
    #[serde(rename = "ax")]
    x_advance: i32,
    #[serde(rename = "ay")]
    y_advance: i32,
    #[serde(rename = "dx")]
    x_offset: i32,
    #[serde(rename = "dy")]
    y_offset: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use typf_core::types::PositionedGlyph;

    fn create_test_shaping() -> ShapingResult {
        ShapingResult {
            glyphs: vec![
                PositionedGlyph {
                    id: 72, // 'H'
                    x: 0.0,
                    y: 0.0,
                    advance: 10.0,
                    cluster: 0,
                },
                PositionedGlyph {
                    id: 101, // 'e'
                    x: 10.0,
                    y: 0.0,
                    advance: 8.0,
                    cluster: 1,
                },
            ],
            advance_width: 18.0,
            advance_height: 16.0,
            direction: Direction::LeftToRight,
        }
    }

    #[test]
    fn test_json_exporter_creation() {
        let exporter = JsonExporter::new();
        assert!(!exporter.pretty);

        let pretty_exporter = JsonExporter::with_pretty_print();
        assert!(pretty_exporter.pretty);
    }

    #[test]
    fn test_export_shaping_to_json() {
        let exporter = JsonExporter::new();
        let shaped = create_test_shaping();

        let result = exporter.export_shaping(&shaped);
        assert!(result.is_ok());

        let json = String::from_utf8(result.unwrap()).unwrap();
        assert!(json.contains("\"g\":72"));
        assert!(json.contains("\"cl\":0"));
        assert!(json.contains("\"ax\":640")); // 10.0 * 64
    }

    #[test]
    fn test_pretty_print() {
        let exporter = JsonExporter::with_pretty_print();
        let shaped = create_test_shaping();

        let json = String::from_utf8(exporter.export_shaping(&shaped).unwrap()).unwrap();
        // Pretty-printed JSON should have newlines
        assert!(json.contains('\n'));
        assert!(json.contains("  ")); // Indentation
    }

    #[test]
    fn test_direction_encoding() {
        let mut shaped = create_test_shaping();

        // Test all directions
        let exporter = JsonExporter::new();

        shaped.direction = Direction::LeftToRight;
        let json = String::from_utf8(exporter.export_shaping(&shaped).unwrap()).unwrap();
        assert!(json.contains("\"ltr\""));

        shaped.direction = Direction::RightToLeft;
        let json = String::from_utf8(exporter.export_shaping(&shaped).unwrap()).unwrap();
        assert!(json.contains("\"rtl\""));

        shaped.direction = Direction::TopToBottom;
        let json = String::from_utf8(exporter.export_shaping(&shaped).unwrap()).unwrap();
        assert!(json.contains("\"ttb\""));

        shaped.direction = Direction::BottomToTop;
        let json = String::from_utf8(exporter.export_shaping(&shaped).unwrap()).unwrap();
        assert!(json.contains("\"btt\""));
    }

    #[test]
    fn test_fixed_point_conversion() {
        let exporter = JsonExporter::new();
        let shaped = ShapingResult {
            glyphs: vec![PositionedGlyph {
                id: 1,
                x: 1.5,        // Should become 96 (1.5 * 64)
                y: 0.5,        // Should become 32 (0.5 * 64)
                advance: 10.5, // Should become 672 (10.5 * 64)
                cluster: 0,
            }],
            advance_width: 10.5,
            advance_height: 16.0,
            direction: Direction::LeftToRight,
        };

        let json = String::from_utf8(exporter.export_shaping(&shaped).unwrap()).unwrap();
        assert!(json.contains("\"ax\":672"));
        assert!(json.contains("\"dx\":96"));
        assert!(json.contains("\"dy\":32"));
    }

    #[test]
    fn test_exporter_trait() {
        let exporter = JsonExporter::new();
        assert_eq!(exporter.name(), "json");
        assert_eq!(exporter.extension(), "json");
        assert_eq!(exporter.mime_type(), "application/json");
    }
}
