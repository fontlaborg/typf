// this_file: crates/typf-batch/src/lib.rs

#![deny(missing_docs)]

//! Batch job processing infrastructure for typf.
//!
//! Provides job specifications, validation, and processing utilities
//! for batch rendering workflows.

pub mod security;
pub mod types;

pub use security::{sanitize_path, validate_font_size, validate_text_input};
pub use security::{SecurityError, MAX_FONT_SIZE, MAX_JOBS_PER_SPEC, MAX_TEXT_LENGTH};
pub use types::*;

use camino::Utf8Path;

impl JobSpec {
    /// Validate job specification structure and parameters.
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.validate_header()?;

        for job in &self.jobs {
            job.validate()?;
        }

        Ok(())
    }

    /// Validate header-level constraints (version + job counts).
    pub fn validate_header(&self) -> Result<(), ValidationError> {
        // Check version
        if self.version != "1.0" {
            return Err(ValidationError::UnsupportedVersion(self.version.clone()));
        }

        // Check jobs array is non-empty
        if self.jobs.is_empty() {
            return Err(ValidationError::EmptyJobList);
        }

        // Check limit on number of jobs
        if self.jobs.len() > MAX_JOBS_PER_SPEC {
            return Err(ValidationError::TooManyJobs {
                count: self.jobs.len(),
                max: MAX_JOBS_PER_SPEC,
            });
        }

        Ok(())
    }
}

impl Job {
    /// Validate individual job parameters.
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Check ID is non-empty
        if self.id.is_empty() {
            return Err(ValidationError::EmptyJobId);
        }

        // Validate font size
        if self.font.size == 0 || self.font.size > 10000 {
            return Err(ValidationError::InvalidFontSize(self.font.size));
        }

        // Validate text content
        validate_text_input(&self.text.content)
            .map_err(|e| ValidationError::Security(format!("{}", e)))?;

        // Validate canvas dimensions
        if self.rendering.width == 0 || self.rendering.height == 0 {
            return Err(ValidationError::InvalidCanvasDimensions {
                width: self.rendering.width,
                height: self.rendering.height,
            });
        }

        // Validate format
        let valid_formats = ["pgm", "png", "svg", "metrics"];
        if !valid_formats.contains(&self.rendering.format.as_str()) {
            return Err(ValidationError::InvalidFormat(
                self.rendering.format.clone(),
            ));
        }

        // Validate encoding
        // Note: "json" is valid for metrics format, though deprecated in favor of base64
        let valid_encodings = ["binary", "base64", "json"];
        if !valid_encodings.contains(&self.rendering.encoding.as_str()) {
            return Err(ValidationError::InvalidEncoding(
                self.rendering.encoding.clone(),
            ));
        }

        Ok(())
    }

    /// Validate and sanitize font path.
    pub fn sanitize_font_path(
        &self,
        base_dir: Option<&Utf8Path>,
    ) -> Result<camino::Utf8PathBuf, SecurityError> {
        sanitize_path(&self.font.path, base_dir)
    }
}

/// Validation errors for job specifications.
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    /// Unsupported API version (expected "1.0")
    #[error("Unsupported API version: {0}, expected '1.0'")]
    UnsupportedVersion(String),

    /// Jobs array is empty (must contain at least one job)
    #[error("Jobs array is empty")]
    EmptyJobList,

    /// Too many jobs in specification (exceeds MAX_JOBS_PER_SPEC)
    #[error("Too many jobs: {count} (max: {max})")]
    TooManyJobs {
        /// Number of jobs in specification
        count: usize,
        /// Maximum allowed jobs
        max: usize,
    },

    /// Job ID is empty (must be non-empty string)
    #[error("Job ID is empty")]
    EmptyJobId,

    /// Invalid font size (must be 1-10000)
    #[error("Invalid font size: {0} (must be 1-10000)")]
    InvalidFontSize(u32),

    /// Invalid canvas dimensions (must be non-zero)
    #[error("Invalid canvas dimensions: {width}x{height}")]
    InvalidCanvasDimensions {
        /// Canvas width in pixels
        width: u32,
        /// Canvas height in pixels
        height: u32,
    },

    /// Invalid output format (must be pgm, png, svg, or metrics)
    #[error("Invalid output format: {0} (must be pgm, png, svg, or metrics)")]
    InvalidFormat(String),

    /// Invalid encoding (must be binary, base64, or json)
    #[error("Invalid encoding: {0} (must be binary, base64, or json)")]
    InvalidEncoding(String),

    /// Security validation failed
    #[error("Security validation failed: {0}")]
    Security(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_valid_job() -> Job {
        Job {
            id: "test1".to_string(),
            font: FontConfig {
                path: "test.ttf".into(),
                size: 1000,
                variations: HashMap::new(),
            },
            text: TextConfig {
                content: "A".to_string(),
                script: None,
                direction: None,
                language: None,
                features: vec![],
            },
            rendering: RenderingConfig {
                format: "pgm".to_string(),
                encoding: "base64".to_string(),
                width: 100,
                height: 100,
            },
        }
    }

    #[test]
    fn test_valid_job_spec() {
        let spec = JobSpec {
            version: "1.0".to_string(),
            jobs: vec![create_valid_job()],
        };
        assert!(spec.validate().is_ok());
    }

    #[test]
    fn test_invalid_version() {
        let spec = JobSpec {
            version: "2.0".to_string(),
            jobs: vec![create_valid_job()],
        };
        assert!(spec.validate().is_err());
    }

    #[test]
    fn test_empty_jobs() {
        let spec = JobSpec {
            version: "1.0".to_string(),
            jobs: vec![],
        };
        assert!(spec.validate().is_err());
    }

    #[test]
    fn test_invalid_font_size() {
        let mut job = create_valid_job();
        job.font.size = 0;
        assert!(job.validate().is_err());

        job.font.size = 10001;
        assert!(job.validate().is_err());
    }

    #[test]
    fn test_invalid_dimensions() {
        let mut job = create_valid_job();
        job.rendering.width = 0;
        assert!(job.validate().is_err());

        let mut job = create_valid_job();
        job.rendering.height = 0;
        assert!(job.validate().is_err());
    }

    #[test]
    fn test_invalid_format() {
        let mut job = create_valid_job();
        job.rendering.format = "invalid".to_string();
        assert!(job.validate().is_err());
    }
}
