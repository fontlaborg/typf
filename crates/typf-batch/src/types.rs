// this_file: crates/typf-batch/src/types.rs

//! Batch job types and structures (ported from haforu).
//!
//! Defines job specifications, results, and related data structures
//! for batch processing of font rendering jobs.

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete batch job specification (top-level JSON input).
#[derive(Debug, Clone, Deserialize)]
pub struct JobSpec {
    /// API version (must be "1.0")
    pub version: String,
    /// List of rendering jobs to process
    pub jobs: Vec<Job>,
}

/// Single rendering job.
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

/// Font configuration for a job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    /// Absolute path to font file
    pub path: Utf8PathBuf,
    /// Font size in points (typically 1000 for FontSimi)
    pub size: u32,
    /// Variable font coordinates (axis tag → value)
    #[serde(default)]
    pub variations: HashMap<String, f32>,
}

/// Text configuration for a job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextConfig {
    /// Text content to render (single glyph or string)
    pub content: String,
    /// Optional script hint (e.g., "Latn", "Cyrl")
    #[serde(default)]
    pub script: Option<String>,
    /// Requested text direction (ltr, rtl, ttb, btt)
    #[serde(default)]
    pub direction: Option<String>,
    /// Requested language hint (e.g., "en", "ar")
    #[serde(default)]
    pub language: Option<String>,
    /// OpenType feature toggles (e.g., ["liga=0", "kern"])
    #[serde(default)]
    pub features: Vec<String>,
}

/// Rendering parameters for a job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderingConfig {
    /// Output format ("pgm", "png", or "metrics")
    pub format: String,
    /// Encoding ("binary" for PGM, "base64" for JSONL)
    pub encoding: String,
    /// Canvas width in pixels
    pub width: u32,
    /// Canvas height in pixels
    pub height: u32,
}

/// Job result (JSONL output line).
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
    /// Sanitized font metadata (path + applied variations)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font: Option<FontResult>,
    /// Timing information
    pub timing: TimingInfo,
    /// Memory statistics (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<MemoryInfo>,
}

/// Rendering output data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderingOutput {
    /// Output format ("pgm" or "png")
    pub format: String,
    /// Encoding ("base64")
    pub encoding: String,
    /// Base64-encoded image data
    pub data: String,
    /// Image width in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
    /// Actual bounding box of rendered content (x, y, w, h)
    pub actual_bbox: (u32, u32, u32, u32),
}

/// Metrics output data for metrics-only jobs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsOutput {
    /// Normalized pixel density [0.0, 1.0]
    pub density: f64,
    /// Longest contiguous non-zero run relative to canvas size [0.0, 1.0]
    pub beam: f64,
}

/// Timing statistics for a job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingInfo {
    /// Time spent shaping text (milliseconds)
    pub shape_ms: f64,
    /// Time spent rasterizing glyphs (milliseconds)
    pub render_ms: f64,
    /// Total time for job (milliseconds)
    pub total_ms: f64,
}

/// Memory usage statistics (optional).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    /// Font cache size (megabytes)
    pub font_cache_mb: f64,
    /// Total memory usage (megabytes)
    pub total_mb: f64,
}

/// Font metadata emitted with each job result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontResult {
    /// Absolute path used after sanitization
    pub path: String,
    /// Applied variation coordinates (after clamping/dropping)
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
    /// Create error result for a failed job.
    pub fn error(id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            status: "error".to_string(),
            rendering: None,
            metrics: None,
            error: Some(message.into()),
            font: None,
            timing: TimingInfo::default(),
            memory: None,
        }
    }

    /// Create success result with rendering output.
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
            memory: None,
        }
    }

    /// Create success result with metrics output.
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
            memory: None,
        }
    }
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
    fn test_timing_info_default() {
        let timing = TimingInfo::default();
        assert_eq!(timing.shape_ms, 0.0);
        assert_eq!(timing.render_ms, 0.0);
        assert_eq!(timing.total_ms, 0.0);
    }
}
