//! JSONL batch job types for TYPF v2
//!
//! Defines job specifications and results for batch processing,
//! compatible with old-typf for migration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Complete batch job specification (JSON input)
#[derive(Debug, Clone, Deserialize)]
pub struct JobSpec {
    /// API version (should be "2.0") - validated during deserialization
    #[serde(default = "default_version")]
    pub _version: String,
    /// List of rendering jobs to process
    pub jobs: Vec<Job>,
}

fn default_version() -> String {
    "2.0".to_string()
}

/// Single rendering job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Unique job identifier for correlation with results
    pub id: String,
    /// Font configuration
    pub font: FontConfig,
    /// Text to render
    pub text: TextConfig,
    /// Rendering parameters
    pub rendering: RenderingConfig,
}

/// Font configuration for a job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    /// Path to font file
    pub path: PathBuf,
    /// Font size in points
    pub size: f32,
    /// Variable font coordinates (axis tag → value)
    #[serde(default)]
    pub variations: HashMap<String, f32>,
}

/// Text configuration for a job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextConfig {
    /// Text content to render
    pub content: String,
    /// Optional script hint (e.g., "Latn", "Arab")
    #[serde(default)]
    pub script: Option<String>,
    /// Text direction (ltr, rtl)
    #[serde(default)]
    pub direction: Option<String>,
    /// Language hint (e.g., "en", "ar")
    #[serde(default)]
    pub language: Option<String>,
    /// OpenType feature toggles (e.g., ["liga", "kern=0"])
    #[serde(default)]
    pub features: Vec<String>,
}

/// Rendering parameters for a job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderingConfig {
    /// Output format ("ppm", "pgm", "pbm", "png", "metrics")
    pub format: String,
    /// Encoding ("base64" for JSONL, "binary" for files)
    pub encoding: String,
    /// Canvas width in pixels
    pub width: u32,
    /// Canvas height in pixels
    pub height: u32,
}

/// Job result (JSONL output line)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    /// Job ID (matches input)
    pub id: String,
    /// Status: "success" or "error"
    pub status: String,
    /// Rendering output (only present on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rendering: Option<RenderingOutput>,
    /// Metrics output (present when format == "metrics")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<MetricsOutput>,
    /// Error message (only present on error)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Font metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font: Option<FontResult>,
    /// Timing information
    pub timing: TimingInfo,
}

/// Rendering output data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderingOutput {
    /// Output format ("ppm", "pgm", "pbm", "png")
    pub format: String,
    /// Encoding ("base64")
    pub encoding: String,
    /// Base64-encoded image data
    pub data: String,
    /// Image width in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
}

/// Metrics output data for metrics-only jobs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsOutput {
    /// Number of glyphs rendered
    pub glyph_count: usize,
    /// Total advance width
    pub advance_width: f32,
    /// Bounding box (x, y, width, height)
    pub bbox: (f32, f32, f32, f32),
}

/// Timing statistics for a job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingInfo {
    /// Time spent shaping text (milliseconds)
    pub shape_ms: f64,
    /// Time spent rendering (milliseconds)
    pub render_ms: f64,
    /// Total time for job (milliseconds)
    pub total_ms: f64,
}

/// Font metadata emitted with each job result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontResult {
    /// Font path used
    pub path: String,
    /// Applied variation coordinates
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub variations: HashMap<String, f32>,
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
        font: FontResult,
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
        font: FontResult,
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

/// Run batch mode: read JobSpec from stdin, process jobs, output JSONL results
pub fn run_batch() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{stdin, stdout, Write};
    use std::time::Instant;

    let start = Instant::now();

    // Read entire stdin as JSON
    let mut input = String::new();
    std::io::Read::read_to_string(&mut stdin().lock(), &mut input)?;

    // Parse job specification
    let spec: JobSpec = serde_json::from_str(&input)?;

    eprintln!("Processing {} jobs...", spec.jobs.len());

    // Process each job sequentially (could parallelize with rayon)
    let results: Vec<JobResult> = spec.jobs.iter().map(process_job).collect();

    // Output JSONL results to stdout
    let mut out = stdout().lock();
    for result in results {
        serde_json::to_writer(&mut out, &result)?;
        writeln!(&mut out)?;
    }

    eprintln!("Completed {} jobs in {:.2}s", spec.jobs.len(), start.elapsed().as_secs_f64());

    Ok(())
}

/// Run streaming mode: read JSONL from stdin, process each job, output JSONL results
pub fn run_stream() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{stdin, stdout, BufRead, Write};

    let mut out = stdout().lock();

    for line in stdin().lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        // Parse job from JSONL
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

        // Process job
        let result = process_job(&job);

        // Output result immediately (streaming)
        serde_json::to_writer(&mut out, &result)?;
        writeln!(&mut out)?;
        out.flush()?;
    }

    Ok(())
}

/// Process a single rendering job
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
    use typf_render_orge::OrgeRenderer;
    use typf_shape_none::NoneShaper;

    let start = Instant::now();

    // Load font
    let font_data = match fs::read(&job.font.path) {
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
            FontResult {
                path: job.font.path.display().to_string(),
                variations: job.font.variations.clone(),
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
    };

    let renderer = Arc::new(OrgeRenderer::new());
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
        FontResult {
            path: job.font.path.display().to_string(),
            variations: job.font.variations.clone(),
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
            "font": {"path": "/fonts/arial.ttf", "size": 24},
            "text": {"content": "Hello"},
            "rendering": {"format": "ppm", "encoding": "base64", "width": 800, "height": 600}
        }"#;

        let job: Job = serde_json::from_str(json).unwrap();
        assert_eq!(job.id, "test1");
        assert_eq!(job.font.size, 24.0);
        assert_eq!(job.text.content, "Hello");
    }

    #[test]
    fn test_job_spec_deserialization() {
        let json = r#"{
            "version": "2.0",
            "jobs": [
                {
                    "id": "job1",
                    "font": {"path": "/fonts/arial.ttf", "size": 24},
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
