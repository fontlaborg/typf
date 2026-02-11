//! Structured batch processing: JSON in, JSON out
//!
//! Perfect for automation, testing frameworks, and integration with
//! other tools. Each line is a complete job specification.
// this_file: crates/typf-cli/src/jsonl.rs

#![allow(dead_code)] // Legacy JSONL batch processing - retained for future v2.1 batch command

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// A batch of rendering jobs to process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSpec {
    /// API version compatibility (defaults to "2.0")
    #[serde(default = "default_version", rename = "version", alias = "_version")]
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
    validate_spec_version(&spec._version)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    eprintln!("Processing {} jobs...", spec.jobs.len());

    // Process jobs in parallel while preserving output order.
    let results = process_jobs(&spec.jobs);

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

fn process_jobs(jobs: &[Job]) -> Vec<JobResult> {
    // Slice parallel iterators are indexed; collect() preserves input order.
    jobs.par_iter().map(process_job).collect()
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
        types::RenderOutput,
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
    let direction = match parse_text_direction(job.text.direction.as_deref()) {
        Ok(direction) => direction,
        Err(e) => return JobResult::error(&job.id, format!("Invalid text.direction: {}", e)),
    };

    let features = match parse_text_features(&job.text.features) {
        Ok(features) => features,
        Err(e) => return JobResult::error(&job.id, format!("Invalid OpenType feature: {}", e)),
    };

    let variations = match parse_instance_variations(&job.font.instance.variations) {
        Ok(variations) => variations,
        Err(e) => {
            return JobResult::error(&job.id, format!("Invalid font.instance.variations: {}", e))
        },
    };

    let encoding = match parse_rendering_encoding(&job.rendering.encoding) {
        Ok(encoding) => encoding,
        Err(e) => return JobResult::error(&job.id, format!("Invalid rendering.encoding: {}", e)),
    };

    let format = match parse_rendering_format(&job.rendering.format) {
        Ok(format) => format,
        Err(e) => return JobResult::error(&job.id, format!("Invalid rendering.format: {}", e)),
    };

    // Create shaping parameters
    let shaping_params = ShapingParams {
        size: job.font.size,
        direction,
        language: job.text.language.clone(),
        script: job.text.script.clone(),
        features,
        variations,
        letter_spacing: 0.0,
    };
    if !shaping_params.size.is_finite() {
        return JobResult::error(&job.id, "Invalid font.size: value must be finite");
    }
    if let Err(error) = shaping_params.validate() {
        return JobResult::error(&job.id, format!("Invalid font.size: {}", error));
    }

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
    if matches!(format, RenderingFormat::Metrics) {
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
    let exporter: Arc<dyn Exporter> = match format {
        RenderingFormat::Ppm => Arc::new(PnmExporter::ppm()),
        RenderingFormat::Pgm => Arc::new(PnmExporter::pgm()),
        RenderingFormat::Pbm => Arc::new(PnmExporter::new(typf_export::PnmFormat::Pbm)),
        RenderingFormat::Metrics => unreachable!("metrics format is handled before export"),
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

    // Encode output payload based on requested transport encoding.
    let data = match encoding {
        RenderingEncoding::Base64 => BASE64.encode(&exported),
        RenderingEncoding::Plain => String::from_utf8_lossy(&exported).to_string(),
    };

    let total_ms = start.elapsed().as_secs_f64() * 1000.0;

    JobResult::success_render(
        job.id.clone(),
        RenderingOutput {
            format: format.as_str().to_string(),
            encoding: encoding.as_str().to_string(),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RenderingFormat {
    Ppm,
    Pgm,
    Pbm,
    Metrics,
}

impl RenderingFormat {
    fn as_str(self) -> &'static str {
        match self {
            Self::Ppm => "ppm",
            Self::Pgm => "pgm",
            Self::Pbm => "pbm",
            Self::Metrics => "metrics",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RenderingEncoding {
    Base64,
    Plain,
}

impl RenderingEncoding {
    fn as_str(self) -> &'static str {
        match self {
            Self::Base64 => "base64",
            Self::Plain => "plain",
        }
    }
}

fn parse_rendering_encoding(raw: &str) -> Result<RenderingEncoding, String> {
    let normalized = raw.trim();
    if normalized.eq_ignore_ascii_case("base64") {
        Ok(RenderingEncoding::Base64)
    } else if normalized.eq_ignore_ascii_case("plain") {
        Ok(RenderingEncoding::Plain)
    } else {
        Err(format!(
            "'{}' is not supported; expected one of: base64, plain",
            raw
        ))
    }
}

fn parse_rendering_format(raw: &str) -> Result<RenderingFormat, String> {
    let normalized = raw.trim();
    if normalized.eq_ignore_ascii_case("ppm") {
        Ok(RenderingFormat::Ppm)
    } else if normalized.eq_ignore_ascii_case("pgm") {
        Ok(RenderingFormat::Pgm)
    } else if normalized.eq_ignore_ascii_case("pbm") {
        Ok(RenderingFormat::Pbm)
    } else if normalized.eq_ignore_ascii_case("metrics") {
        Ok(RenderingFormat::Metrics)
    } else {
        Err(format!(
            "'{}' is not supported; expected one of: ppm, pgm, pbm, metrics",
            raw
        ))
    }
}

fn parse_instance_variations(raw: &HashMap<String, f32>) -> Result<Vec<(String, f32)>, String> {
    let mut parsed: Vec<(String, f32)> = Vec::with_capacity(raw.len());

    // HashMap iteration order is arbitrary; sort tags first so validation
    // diagnostics are deterministic across runs.
    let mut entries: Vec<(&String, &f32)> = raw.iter().collect();
    entries.sort_by(|left, right| left.0.cmp(right.0));

    for (tag, value) in entries {
        if !tag
            .as_bytes()
            .iter()
            .all(|byte| (0x20..=0x7E).contains(byte))
        {
            return Err(format!("axis '{}' must use ASCII bytes in 0x20..0x7E", tag));
        }

        if tag.len() != 4 {
            return Err(format!(
                "axis '{}' has invalid tag length {}; expected 4 characters",
                tag,
                tag.len()
            ));
        }

        if !value.is_finite() {
            return Err(format!("axis '{}' has non-finite value {}", tag, value));
        }

        parsed.push((tag.clone(), *value));
    }

    parsed.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(parsed)
}

fn validate_spec_version(version: &str) -> Result<(), String> {
    let normalized = version.trim();
    if normalized.is_empty() {
        return Err("version is empty".to_string());
    }

    let parts: Vec<&str> = normalized.split('.').collect();
    if parts.len() > 2 {
        return Err(format!(
            "version '{}' has too many segments; expected '2' or '2.<minor>'",
            normalized
        ));
    }

    let major = parts[0].parse::<u32>().map_err(|_| {
        format!(
            "version '{}' must start with a numeric major version",
            normalized
        )
    })?;

    if parts.len() == 2 && (parts[1].is_empty() || parts[1].parse::<u32>().is_err()) {
        return Err(format!(
            "version '{}' must use a numeric minor version when provided",
            normalized
        ));
    }

    if major == 2 {
        Ok(())
    } else {
        Err(format!(
            "unsupported JSONL version '{}'; expected major version 2.x",
            normalized
        ))
    }
}

fn parse_text_direction(raw: Option<&str>) -> Result<typf_core::types::Direction, String> {
    use typf_core::types::Direction;

    let normalized = raw.map(str::trim).filter(|value| !value.is_empty());

    match normalized {
        None => Ok(Direction::LeftToRight),
        Some(value) if value.eq_ignore_ascii_case("ltr") => Ok(Direction::LeftToRight),
        Some(value) if value.eq_ignore_ascii_case("rtl") => Ok(Direction::RightToLeft),
        Some(value) if value.eq_ignore_ascii_case("ttb") => Ok(Direction::TopToBottom),
        Some(value) if value.eq_ignore_ascii_case("btt") => Ok(Direction::BottomToTop),
        Some(value) => Err(format!(
            "'{}' is not supported; expected one of: ltr, rtl, ttb, btt",
            value
        )),
    }
}

fn parse_text_features(feature_specs: &[String]) -> Result<Vec<(String, u32)>, String> {
    let mut parsed = Vec::new();

    for spec in feature_specs {
        for token in spec.split([',', ' ', '\t', '\n', '\r']) {
            let token = token.trim();
            if token.is_empty() {
                continue;
            }
            let (tag, value) = parse_feature_token(token)?;
            if let Some(existing) = parsed.iter_mut().find(|(existing, _)| existing == &tag) {
                // Keep stable output ordering while making duplicate tags deterministic.
                existing.1 = value;
            } else {
                parsed.push((tag, value));
            }
        }
    }

    Ok(parsed)
}

fn parse_feature_token(token: &str) -> Result<(String, u32), String> {
    let (tag, value) = if let Some(stripped) = token.strip_prefix('+') {
        (stripped, 1)
    } else if let Some(stripped) = token.strip_prefix('-') {
        (stripped, 0)
    } else if let Some(eq_pos) = token.find('=') {
        let tag = &token[..eq_pos];
        let value_str = &token[eq_pos + 1..];
        let value = value_str
            .parse::<u32>()
            .map_err(|_| format!("feature '{}' has invalid value '{}'", tag, value_str))?;
        (tag, value)
    } else {
        (token, 1)
    };

    if !tag
        .as_bytes()
        .iter()
        .all(|byte| (0x20..=0x7E).contains(byte))
    {
        return Err(format!(
            "feature '{}' must use ASCII bytes in 0x20..0x7E",
            tag
        ));
    }

    if tag.len() != 4 {
        return Err(format!(
            "feature '{}' has invalid tag length {}; expected 4 characters",
            tag,
            tag.len()
        ));
    }

    Ok((tag.to_string(), value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use typf_core::types::Direction;

    fn test_job(id: &str) -> Job {
        Job {
            id: id.to_string(),
            font: TypfFontRenderableConfig {
                source: TypfFontSourceConfig {
                    path: PathBuf::from("/definitely/missing/font.ttf"),
                    face_index: None,
                },
                instance: TypfFontInstanceConfig::default(),
                size: 24.0,
            },
            text: TextConfig {
                content: "Hello".to_string(),
                script: None,
                direction: Some("ltr".to_string()),
                language: None,
                features: Vec::new(),
            },
            rendering: RenderingConfig {
                format: "ppm".to_string(),
                encoding: "base64".to_string(),
                width: 800,
                height: 600,
            },
        }
    }

    fn workspace_test_font() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.pop(); // crates
        path.pop(); // workspace root
        path.push("test-fonts");
        path.push("NotoSans-Regular.ttf");
        path
    }

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
            "version": "2.1",
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
        assert_eq!(spec._version, "2.1");
        assert_eq!(spec.jobs.len(), 1);
    }

    #[test]
    fn test_job_spec_deserialization_supports_legacy_underscore_version() {
        let json = r#"{
            "_version": "2.2",
            "jobs": []
        }"#;

        let spec: JobSpec = serde_json::from_str(json).unwrap();
        assert_eq!(spec._version, "2.2");
    }

    #[test]
    fn test_job_spec_serialization_uses_version_field() {
        let spec = JobSpec {
            _version: "2.3".to_string(),
            jobs: Vec::new(),
        };

        let value = serde_json::to_value(spec).unwrap();
        assert_eq!(value["version"], json!("2.3"));
        assert!(
            value.get("_version").is_none(),
            "serialized JobSpec must use canonical 'version' key"
        );
    }

    #[test]
    fn test_validate_spec_version_accepts_v2_series() {
        validate_spec_version("2").expect("major-only 2 should be accepted");
        validate_spec_version("2.0").expect("2.0 should be accepted");
        validate_spec_version("2.99").expect("2.99 should be accepted");
    }

    #[test]
    fn test_validate_spec_version_accepts_surrounding_whitespace() {
        validate_spec_version(" 2.0 ").expect("whitespace-trimmed 2.0 should be accepted");
    }

    #[test]
    fn test_validate_spec_version_rejects_other_majors() {
        let err = validate_spec_version("3.0").expect_err("3.x should be rejected");
        assert!(
            err.contains("expected major version 2.x"),
            "expected unsupported-version error, got: {}",
            err
        );
    }

    #[test]
    fn test_validate_spec_version_rejects_non_numeric_major() {
        let err = validate_spec_version("v2").expect_err("non-numeric major should error");
        assert!(
            err.contains("numeric major version"),
            "expected numeric-version error, got: {}",
            err
        );
    }

    #[test]
    fn test_validate_spec_version_rejects_empty_version() {
        let err = validate_spec_version(" \t ").expect_err("blank version should error");
        assert!(
            err.contains("version is empty"),
            "expected empty-version error, got: {}",
            err
        );
    }

    #[test]
    fn test_validate_spec_version_rejects_non_numeric_minor() {
        let err = validate_spec_version("2.beta").expect_err("non-numeric minor should error");
        assert!(
            err.contains("numeric minor"),
            "expected numeric-minor error, got: {}",
            err
        );
    }

    #[test]
    fn test_validate_spec_version_rejects_extra_segments() {
        let err = validate_spec_version("2.0.1").expect_err("extra segments should error");
        assert!(
            err.contains("too many segments"),
            "expected too-many-segments error, got: {}",
            err
        );
    }

    #[test]
    fn test_parse_text_direction_defaults_to_ltr_when_missing() {
        let direction = parse_text_direction(None).expect("missing direction should default");
        assert_eq!(direction, Direction::LeftToRight);
    }

    #[test]
    fn test_parse_text_direction_accepts_valid_values_case_insensitively() {
        assert_eq!(
            parse_text_direction(Some("rtl")).expect("rtl should parse"),
            Direction::RightToLeft
        );
        assert_eq!(
            parse_text_direction(Some("TTB")).expect("TTB should parse"),
            Direction::TopToBottom
        );
        assert_eq!(
            parse_text_direction(Some("btt")).expect("btt should parse"),
            Direction::BottomToTop
        );
    }

    #[test]
    fn test_parse_text_direction_accepts_surrounding_whitespace() {
        assert_eq!(
            parse_text_direction(Some("  RTL\t")).expect("trimmed RTL should parse"),
            Direction::RightToLeft
        );
    }

    #[test]
    fn test_parse_text_direction_when_empty_then_defaults_to_ltr() {
        assert_eq!(
            parse_text_direction(Some(" \n\t ")).expect("blank direction should default"),
            Direction::LeftToRight
        );
    }

    #[test]
    fn test_parse_text_direction_rejects_unknown_value() {
        let err = parse_text_direction(Some("sideways")).expect_err("invalid direction must fail");
        assert!(
            err.contains("expected one of"),
            "expected list of supported directions, got: {}",
            err
        );
    }

    #[test]
    fn test_parse_text_features_when_mixed_syntax_then_parsed_values() {
        let parsed = parse_text_features(&[
            "+liga".to_string(),
            "kern=0".to_string(),
            "smcp".to_string(),
            "cv01=2".to_string(),
        ])
        .unwrap();

        assert_eq!(
            parsed,
            vec![
                ("liga".to_string(), 1),
                ("kern".to_string(), 0),
                ("smcp".to_string(), 1),
                ("cv01".to_string(), 2),
            ]
        );
    }

    #[test]
    fn test_parse_text_features_when_duplicate_tags_then_last_value_wins() {
        let parsed = parse_text_features(&[
            "+liga".to_string(),
            "kern=0".to_string(),
            "liga=0".to_string(),
            "cv01=1".to_string(),
            "cv01=3".to_string(),
        ])
        .expect("duplicate tags should parse deterministically");

        assert_eq!(
            parsed,
            vec![
                ("liga".to_string(), 0),
                ("kern".to_string(), 0),
                ("cv01".to_string(), 3),
            ]
        );
    }

    #[test]
    fn test_parse_text_features_when_bad_value_then_error() {
        let error = parse_text_features(&["liga=on".to_string()]).unwrap_err();
        assert!(
            error.contains("invalid value"),
            "expected invalid value error, got: {}",
            error
        );
    }

    #[test]
    fn test_parse_text_features_when_bad_tag_length_then_error() {
        let error = parse_text_features(&["ligature=1".to_string()]).unwrap_err();
        assert!(
            error.contains("expected 4 characters"),
            "expected tag-length validation error, got: {}",
            error
        );
    }

    #[test]
    fn test_parse_text_features_when_non_printable_ascii_then_error() {
        let error = parse_text_features(&["\u{7f}abc=1".to_string()]).unwrap_err();
        assert!(
            error.contains("0x20..0x7E"),
            "expected ASCII-range validation error, got: {}",
            error
        );
    }

    #[test]
    fn test_parse_text_features_when_non_ascii_multibyte_then_error() {
        let error = parse_text_features(&["éght=1".to_string()]).unwrap_err();
        assert!(
            error.contains("0x20..0x7E"),
            "expected ASCII-range validation error, got: {}",
            error
        );
    }

    #[test]
    fn test_parse_text_features_when_tab_newline_separated_then_parsed_values() {
        let parsed = parse_text_features(&["+liga,\tkern=0\nsmcp".to_string()])
            .expect("tab/newline-delimited feature list should parse");
        assert_eq!(
            parsed,
            vec![
                ("liga".to_string(), 1),
                ("kern".to_string(), 0),
                ("smcp".to_string(), 1),
            ]
        );
    }

    #[test]
    fn test_parse_rendering_encoding_accepts_case_insensitive_values() {
        assert_eq!(
            parse_rendering_encoding("base64").expect("base64 should parse"),
            RenderingEncoding::Base64
        );
        assert_eq!(
            parse_rendering_encoding("PLAIN").expect("plain should parse"),
            RenderingEncoding::Plain
        );
    }

    #[test]
    fn test_parse_rendering_encoding_accepts_surrounding_whitespace() {
        assert_eq!(
            parse_rendering_encoding("  base64\t").expect("trimmed base64 should parse"),
            RenderingEncoding::Base64
        );
        assert_eq!(
            parse_rendering_encoding("\nplain ").expect("trimmed plain should parse"),
            RenderingEncoding::Plain
        );
    }

    #[test]
    fn test_parse_rendering_encoding_rejects_unknown_value() {
        let error = parse_rendering_encoding("hex").expect_err("unknown encoding should fail");
        assert!(
            error.contains("expected one of: base64, plain"),
            "expected supported-encoding guidance, got: {}",
            error
        );
    }

    #[test]
    fn test_parse_rendering_format_accepts_case_insensitive_values_with_whitespace() {
        assert_eq!(
            parse_rendering_format("  PPM\t").expect("trimmed PPM should parse"),
            RenderingFormat::Ppm
        );
        assert_eq!(
            parse_rendering_format("metrics").expect("metrics should parse"),
            RenderingFormat::Metrics
        );
    }

    #[test]
    fn test_parse_rendering_format_rejects_unknown_value() {
        let error = parse_rendering_format("png").expect_err("unsupported format should fail");
        assert!(
            error.contains("expected one of: ppm, pgm, pbm, metrics"),
            "expected supported-format guidance, got: {}",
            error
        );
    }

    #[test]
    fn test_parse_instance_variations_sorts_by_axis_tag() {
        let mut raw = HashMap::new();
        raw.insert("wght".to_string(), 700.0);
        raw.insert("opsz".to_string(), 12.0);
        raw.insert("wdth".to_string(), 110.0);

        let parsed = parse_instance_variations(&raw).expect("variations should parse");
        assert_eq!(
            parsed,
            vec![
                ("opsz".to_string(), 12.0),
                ("wdth".to_string(), 110.0),
                ("wght".to_string(), 700.0),
            ]
        );
    }

    #[test]
    fn test_parse_instance_variations_rejects_invalid_tag_length() {
        let mut raw = HashMap::new();
        raw.insert("weight".to_string(), 700.0);

        let error = parse_instance_variations(&raw).expect_err("invalid tag length must fail");
        assert!(
            error.contains("expected 4 characters"),
            "expected tag-length validation error, got: {}",
            error
        );
    }

    #[test]
    fn test_parse_instance_variations_rejects_non_printable_ascii_tag() {
        let mut raw = HashMap::new();
        raw.insert("\u{7f}ght".to_string(), 700.0);

        let error = parse_instance_variations(&raw).expect_err("non-printable ascii must fail");
        assert!(
            error.contains("0x20..0x7E"),
            "expected ASCII-range validation error, got: {}",
            error
        );
    }

    #[test]
    fn test_parse_instance_variations_rejects_non_ascii_multibyte_tag() {
        let mut raw = HashMap::new();
        raw.insert("éght".to_string(), 700.0);

        let error = parse_instance_variations(&raw).expect_err("non-ascii tags must fail");
        assert!(
            error.contains("0x20..0x7E"),
            "expected ASCII-range validation error, got: {}",
            error
        );
    }

    #[test]
    fn test_parse_instance_variations_rejects_non_finite_value() {
        let mut raw = HashMap::new();
        raw.insert("wght".to_string(), f32::INFINITY);

        let error = parse_instance_variations(&raw).expect_err("non-finite values must fail");
        assert!(
            error.contains("non-finite value"),
            "expected finite-value validation error, got: {}",
            error
        );
    }

    #[test]
    fn test_parse_instance_variations_when_multiple_invalid_then_error_is_deterministic() {
        let mut raw = HashMap::new();
        raw.insert("éght".to_string(), 700.0);
        raw.insert("weight".to_string(), 700.0);

        let error = parse_instance_variations(&raw)
            .expect_err("validation should report deterministic first error");
        assert!(
            error.contains("axis 'weight' has invalid tag length"),
            "expected deterministic tag-length-first error, got: {}",
            error
        );
    }

    #[test]
    fn test_process_job_when_font_size_is_non_finite_then_error() {
        let mut job = test_job("job-non-finite-size");
        job.font.source.path = workspace_test_font();
        job.font.size = f32::NAN;

        let result = process_job(&job);
        assert_eq!(result.status, "error", "invalid font size must fail fast");
        let error = result.error.unwrap_or_default();
        assert!(
            error.contains("Invalid font.size"),
            "expected font.size validation context, got: {}",
            error
        );
        assert!(
            error.contains("finite"),
            "expected finite-value guidance, got: {}",
            error
        );
    }

    #[test]
    fn test_process_job_when_font_size_is_non_positive_then_error() {
        let mut job = test_job("job-zero-size");
        job.font.source.path = workspace_test_font();
        job.font.size = 0.0;

        let result = process_job(&job);
        assert_eq!(result.status, "error", "non-positive font size must fail");
        let error = result.error.unwrap_or_default();
        assert!(
            error.contains("Invalid font.size"),
            "expected font.size validation context, got: {}",
            error
        );
        assert!(
            error.contains("positive"),
            "expected positive-size guidance, got: {}",
            error
        );
    }

    #[test]
    fn test_process_job_when_format_has_surrounding_whitespace_then_succeeds() {
        let mut job = test_job("job-format-trimmed");
        job.font.source.path = workspace_test_font();
        job.rendering.format = "  PPM\t".to_string();

        let result = process_job(&job);
        assert_eq!(result.status, "success", "trimmed format should succeed");
        let rendering = result
            .rendering
            .expect("successful render should include rendering payload");
        assert_eq!(
            rendering.format, "ppm",
            "successful output should use canonical lowercase format"
        );
    }

    #[test]
    fn test_process_jobs_when_parallel_then_preserves_input_order() {
        let jobs = vec![test_job("job-a"), test_job("job-b"), test_job("job-c")];
        let results = process_jobs(&jobs);

        let ids: Vec<&str> = results.iter().map(|result| result.id.as_str()).collect();
        assert_eq!(ids, vec!["job-a", "job-b", "job-c"]);
        assert!(
            results.iter().all(|result| result.status == "error"),
            "missing font should fail consistently in deterministic order"
        );
    }

    #[test]
    fn test_process_jobs_when_parallel_many_jobs_then_preserves_input_order() {
        let jobs: Vec<Job> = (0..128)
            .map(|idx| test_job(&format!("job-{:03}", idx)))
            .collect();
        let results = process_jobs(&jobs);

        let ids: Vec<&str> = results.iter().map(|result| result.id.as_str()).collect();
        let expected_ids: Vec<String> = (0..128).map(|idx| format!("job-{:03}", idx)).collect();

        assert_eq!(
            ids,
            expected_ids.iter().map(String::as_str).collect::<Vec<_>>()
        );
    }
}
