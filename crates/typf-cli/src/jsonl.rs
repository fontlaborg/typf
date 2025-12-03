//! Structured batch processing: JSON in, JSON out
//!
//! Perfect for automation, testing frameworks, and integration with
//! other tools. Each line is a complete job specification.

#![allow(dead_code)] // Legacy JSONL batch processing - retained for future v2.1 batch command

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// A batch of rendering jobs to process
#[derive(Debug, Clone, Deserialize)]
pub struct JobSpec {
    /// API version compatibility (defaults to "2.0")
    #[serde(default = "default_version")]
    pub _version: String,
    /// All the jobs we need to render
    pub jobs: Vec<Job>,
}

fn default_version() -> String {
    "2.0".to_string()
}

/// One complete rendering request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// How to identify this job in the results
    pub id: String,
    /// Which font to use and how
    pub font: TypfFontRenderableConfig,
    /// What text to render
    pub text: TextConfig,
    /// How the output should look
    pub rendering: RenderingConfig,
}

/// Where the font data comes from (file path + optional face index)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypfFontSourceConfig {
    /// Where to find the font file
    pub path: PathBuf,
    /// Which face to pick inside a collection (defaults to 0)
    #[serde(default)]
    pub face_index: Option<u32>,
}

/// Variable font coordinates (instance-level)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TypfFontInstanceConfig {
    /// Variable font axis settings (weight, width, etc.)
    #[serde(default)]
    pub variations: HashMap<String, f32>,
}

/// Render-ready font settings (source + instance + size)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypfFontRenderableConfig {
    /// Font container (path + face index)
    pub source: TypfFontSourceConfig,
    /// Selected variation coordinates (instance)
    #[serde(default)]
    pub instance: TypfFontInstanceConfig,
    /// Point size for rendering (renderable)
    pub size: f32,
}

/// Text content and language settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextConfig {
    /// What we're actually rendering
    pub content: String,
    /// Script hint for better shaping (Latn, Arab, etc.)
    #[serde(default)]
    pub script: Option<String>,
    /// Which way the text flows
    #[serde(default)]
    pub direction: Option<String>,
    /// Language for locale-specific rules
    #[serde(default)]
    pub language: Option<String>,
    /// OpenType features to enable/disable
    #[serde(default)]
    pub features: Vec<String>,
}

/// Output format and rendering settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderingConfig {
    /// What we're outputting (ppm, pgm, pbm, png, metrics)
    pub format: String,
    /// How to encode the data (base64 for JSONL)
    pub encoding: String,
    /// How wide the canvas should be
    pub width: u32,
    /// How tall the canvas should be
    pub height: u32,
}

/// What came out of processing a job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    /// Matches the input job ID
    pub id: String,
    /// Did we succeed or fail?
    pub status: String,
    /// The rendered image (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rendering: Option<RenderingOutput>,
    /// Text metrics (if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<MetricsOutput>,
    /// What went wrong (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Info about the font we used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font: Option<TypfFontRenderableResult>,
    /// How long everything took
    pub timing: TimingInfo,
}

/// The actual rendered image data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderingOutput {
    /// What format we produced
    pub format: String,
    /// How we encoded it
    pub encoding: String,
    /// Base64-encoded pixel data
    pub data: String,
    /// Image dimensions
    pub width: u32,
    pub height: u32,
}

/// Text measurement data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsOutput {
    /// How many glyphs we shaped
    pub glyph_count: usize,
    /// How wide the text runs
    pub advance_width: f32,
    /// Bounding box coordinates
    pub bbox: (f32, f32, f32, f32),
}

/// Performance timing breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingInfo {
    /// Time spent turning characters into glyphs
    pub shape_ms: f64,
    /// Time spent turning glyphs into pixels
    pub render_ms: f64,
    /// Total time from start to finish
    pub total_ms: f64,
}

/// Information about the font we actually used
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontResult {
    /// Renderable point size that was requested
    pub size: f32,
}

/// Font source + instance returned in results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypfFontRenderableResult {
    /// Source container describing where the font came from
    pub source: TypfFontSourceConfig,
    /// Instance coordinates that were applied
    #[serde(skip_serializing_if = "instance_is_empty", default)]
    pub instance: TypfFontInstanceConfig,
    /// Renderable parameters used
    pub render: FontResult,
}

fn instance_is_empty(instance: &TypfFontInstanceConfig) -> bool {
    instance.variations.is_empty()
}

impl Default for TimingInfo {
    fn default() -> Self {
        Self {
            shape_ms: 0.0,
            render_ms: 0.0,
            total_ms: 0.0,
        }
    }
}

impl JobResult {
    /// Create error result for a failed job
    pub fn error(id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            status: "error".to_string(),
            rendering: None,
            metrics: None,
            error: Some(message.into()),
            font: None,
            timing: TimingInfo::default(),
        }
    }

    /// Create success result with rendering output
    pub fn success_render(
        id: String,
        rendering: RenderingOutput,
        font: TypfFontRenderableResult,
        timing: TimingInfo,
    ) -> Self {
        Self {
            id,
            status: "success".to_string(),
            rendering: Some(rendering),
            metrics: None,
            error: None,
            font: Some(font),
            timing,
        }
    }

    /// Create success result with metrics output
    pub fn success_metrics(
        id: String,
        metrics: MetricsOutput,
        font: TypfFontRenderableResult,
        timing: TimingInfo,
    ) -> Self {
        Self {
            id,
            status: "success".to_string(),
            rendering: None,
            metrics: Some(metrics),
            error: None,
            font: Some(font),
            timing,
        }
    }
}

/// Process a complete batch of jobs from JSON input
pub fn run_batch() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{stdin, stdout, Write};
    use std::time::Instant;

    let start = Instant::now();

    // Read the entire job specification
    let mut input = String::new();
    std::io::Read::read_to_string(&mut stdin().lock(), &mut input)?;

    // Parse what we need to do
    let spec: JobSpec = serde_json::from_str(&input)?;

    eprintln!("Processing {} jobs...", spec.jobs.len());

    // Process each job (TODO: parallelize with rayon for speed)
    let results: Vec<JobResult> = spec.jobs.iter().map(process_job).collect();

    // Write out results, one JSON per line
    let mut out = stdout().lock();
    for result in results {
        serde_json::to_writer(&mut out, &result)?;
        writeln!(&mut out)?;
    }

    eprintln!(
        "Completed {} jobs in {:.2}s",
        spec.jobs.len(),
        start.elapsed().as_secs_f64()
    );

    Ok(())
}

/// Stream jobs one by one, perfect for pipelines
pub fn run_stream() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{stdin, stdout, BufRead, Write};

    let mut out = stdout().lock();

    // Read, process, and output one job at a time
    for line in stdin().lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        // Try to parse the job
        let job: Job = match serde_json::from_str(&line) {
            Ok(job) => job,
            Err(e) => {
                let error_result = JobResult::error("parse_error", format!("Invalid JSON: {}", e));
                serde_json::to_writer(&mut out, &error_result)?;
                writeln!(&mut out)?;
                out.flush()?;
                continue;
            },
        };

        // Do the actual rendering work
        let result = process_job(&job);

        // Send the result back immediately
        serde_json::to_writer(&mut out, &result)?;
        writeln!(&mut out)?;
        out.flush()?;
    }

    Ok(())
}

/// Turn one job spec into one rendered result
fn process_job(job: &Job) -> JobResult {
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
    use std::fs;
    use std::sync::Arc;
    use std::time::Instant;
    use typf_core::{
        traits::{Exporter, FontRef, Renderer, Shaper},
        types::{Direction, RenderOutput},
        Color, RenderParams, ShapingParams,
    };
    use typf_export::PnmExporter;
    use typf_render_opixa::OpixaRenderer;
    use typf_shape_none::NoneShaper;

    let start = Instant::now();

    // Load font
    let font_path = &job.font.source.path;
    let font_data = match fs::read(font_path) {
        Ok(data) => data,
        Err(e) => {
            return JobResult::error(&job.id, format!("Failed to load font: {}", e));
        },
    };

    // Create simple font wrapper
    struct SimpleFont {
        data: Vec<u8>,
    }

    impl FontRef for SimpleFont {
        fn data(&self) -> &[u8] {
            &self.data
        }

        fn units_per_em(&self) -> u16 {
            1000
        }

        fn glyph_id(&self, ch: char) -> Option<u32> {
            if ch.is_ascii() {
                Some(ch as u32)
            } else {
                Some(0)
            }
        }

        fn advance_width(&self, _glyph_id: u32) -> f32 {
            600.0
        }
    }

    let font = Arc::new(SimpleFont { data: font_data });

    // Parse direction
    let direction = match job.text.direction.as_deref() {
        Some("rtl") => Direction::RightToLeft,
        Some("ttb") => Direction::TopToBottom,
        Some("btt") => Direction::BottomToTop,
        _ => Direction::LeftToRight,
    };

    // Create shaping parameters
    let shaping_params = ShapingParams {
        size: job.font.size,
        direction,
        language: job.text.language.clone(),
        script: job.text.script.clone(),
        features: Vec::new(), // TODO: parse job.text.features
        variations: job
            .font
            .instance
            .variations
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect(),
        letter_spacing: 0.0,
    };

    let shape_start = Instant::now();

    // Shape the text
    let shaper = Arc::new(NoneShaper::new());
    let shaped = match shaper.shape(&job.text.content, font.clone(), &shaping_params) {
        Ok(shaped) => shaped,
        Err(e) => {
            return JobResult::error(&job.id, format!("Shaping failed: {}", e));
        },
    };

    let shape_ms = shape_start.elapsed().as_secs_f64() * 1000.0;
    let render_start = Instant::now();

    // Handle metrics-only output
    if job.rendering.format == "metrics" {
        let metrics = MetricsOutput {
            glyph_count: shaped.glyphs.len(),
            advance_width: shaped.advance_width,
            bbox: (0.0, 0.0, shaped.advance_width, shaped.advance_height),
        };

        let total_ms = start.elapsed().as_secs_f64() * 1000.0;

        return JobResult::success_metrics(
            job.id.clone(),
            metrics,
            TypfFontRenderableResult {
                source: job.font.source.clone(),
                instance: job.font.instance.clone(),
                render: FontResult {
                    size: job.font.size,
                },
            },
            TimingInfo {
                shape_ms,
                render_ms: 0.0,
                total_ms,
            },
        );
    }

    // Render the text
    let render_params = RenderParams {
        foreground: Color::black(),
        background: Some(Color::white()),
        padding: 10,
        antialias: true,
        ..Default::default()
    };

    let renderer = Arc::new(OpixaRenderer::new());
    let rendered = match renderer.render(&shaped, font.clone(), &render_params) {
        Ok(rendered) => rendered,
        Err(e) => {
            return JobResult::error(&job.id, format!("Rendering failed: {}", e));
        },
    };

    let render_ms = render_start.elapsed().as_secs_f64() * 1000.0;

    // Export to requested format
    let exporter: Arc<dyn Exporter> = match job.rendering.format.as_str() {
        "ppm" => Arc::new(PnmExporter::ppm()),
        "pgm" => Arc::new(PnmExporter::pgm()),
        "pbm" => Arc::new(PnmExporter::new(typf_export::PnmFormat::Pbm)),
        _ => {
            return JobResult::error(
                &job.id,
                format!("Unsupported format: {}", job.rendering.format),
            );
        },
    };

    let exported = match exporter.export(&rendered) {
        Ok(data) => data,
        Err(e) => {
            return JobResult::error(&job.id, format!("Export failed: {}", e));
        },
    };

    // Get dimensions from rendered output
    let (width, height) = match &rendered {
        RenderOutput::Bitmap(bitmap) => (bitmap.width, bitmap.height),
        _ => (0, 0),
    };

    // Base64 encode if requested
    let data = if job.rendering.encoding == "base64" {
        BASE64.encode(&exported)
    } else {
        String::from_utf8_lossy(&exported).to_string()
    };

    let total_ms = start.elapsed().as_secs_f64() * 1000.0;

    JobResult::success_render(
        job.id.clone(),
        RenderingOutput {
            format: job.rendering.format.clone(),
            encoding: job.rendering.encoding.clone(),
            data,
            width,
            height,
        },
        TypfFontRenderableResult {
            source: job.font.source.clone(),
            instance: job.font.instance.clone(),
            render: FontResult {
                size: job.font.size,
            },
        },
        TimingInfo {
            shape_ms,
            render_ms,
            total_ms,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_result_error() {
        let result = JobResult::error("job1", "test error");
        assert_eq!(result.id, "job1");
        assert_eq!(result.status, "error");
        assert_eq!(result.error, Some("test error".to_string()));
        assert!(result.rendering.is_none());
    }

    #[test]
    fn test_job_deserialization() {
        let json = r#"{
            "id": "test1",
            "font": {"source": {"path": "/fonts/arial.ttf"}, "instance": {"variations": {"wght": 400}}, "size": 24},
            "text": {"content": "Hello"},
            "rendering": {"format": "ppm", "encoding": "base64", "width": 800, "height": 600}
        }"#;

        let job: Job = serde_json::from_str(json).unwrap();
        assert_eq!(job.id, "test1");
        assert_eq!(job.font.size, 24.0);
        assert_eq!(job.font.source.path, PathBuf::from("/fonts/arial.ttf"));
        assert_eq!(job.text.content, "Hello");
    }

    #[test]
    fn test_job_spec_deserialization() {
        let json = r#"{
            "version": "2.0",
            "jobs": [
                {
                    "id": "job1",
                    "font": {"source": {"path": "/fonts/arial.ttf"}, "size": 24},
                    "text": {"content": "Test"},
                    "rendering": {"format": "ppm", "encoding": "base64", "width": 800, "height": 600}
                }
            ]
        }"#;

        let spec: JobSpec = serde_json::from_str(json).unwrap();
        assert_eq!(spec._version, "2.0");
        assert_eq!(spec.jobs.len(), 1);
    }
}
