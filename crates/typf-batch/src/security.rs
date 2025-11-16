// this_file: crates/typf-batch/src/security.rs

//! Security and validation utilities (ported from haforu).
//!
//! Provides path sanitization, input size limits, and basic validation.

use camino::{Utf8Path, Utf8PathBuf};
use std::time::Duration;

/// Maximum allowed JSON input size (10MB)
pub const MAX_JSON_SIZE: usize = 10 * 1024 * 1024;
/// Maximum allowed number of jobs per spec
pub const MAX_JOBS_PER_SPEC: usize = 1000;
/// Maximum allowed text length
pub const MAX_TEXT_LENGTH: usize = 10_000;
/// Maximum allowed font file size (50MB)
pub const MAX_FONT_SIZE: u64 = 50 * 1024 * 1024;
/// Default per-job timeout
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Validation errors.
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Path not found: {0}")]
    PathNotFound(String),

    #[error("Path outside base directory")]
    PathOutsideBase,

    #[error("Font file too large: {size} bytes (max: {max})")]
    FontTooLarge { size: u64, max: u64 },

    #[error("Text too long: {len} characters (max: {max})")]
    TextTooLong { len: usize, max: usize },
}

/// Validate and sanitize a font path.
///
/// Returns canonical absolute path if valid.
pub fn sanitize_path(
    path: &Utf8Path,
    base_dir: Option<&Utf8Path>,
) -> Result<Utf8PathBuf, SecurityError> {
    let path_str = path.as_str();

    // Check for invalid components
    if path_str.contains("..") || path_str.contains('~') {
        return Err(SecurityError::InvalidPath(
            "Path contains invalid components (.. or ~)".to_string(),
        ));
    }

    // Resolve to absolute path
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else if let Some(base) = base_dir {
        base.join(path)
    } else {
        let cwd = std::env::current_dir()
            .map_err(|e| SecurityError::InvalidPath(format!("Failed to get current dir: {}", e)))?;
        let cwd = Utf8PathBuf::from_path_buf(cwd)
            .map_err(|_| SecurityError::InvalidPath("Non-UTF8 current directory".to_string()))?;
        cwd.join(path)
    };

    // Canonicalize
    let canonical_std = std::fs::canonicalize(abs.as_std_path()).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            SecurityError::PathNotFound(abs.to_string())
        } else {
            SecurityError::InvalidPath(format!("Cannot resolve path {}: {}", abs, e))
        }
    })?;

    let canonical = Utf8PathBuf::from_path_buf(canonical_std)
        .map_err(|_| SecurityError::InvalidPath("Canonical path is not valid UTF-8".to_string()))?;

    // Check base directory restriction
    if let Some(base) = base_dir {
        let base_canon_std = std::fs::canonicalize(base.as_std_path()).map_err(|e| {
            SecurityError::InvalidPath(format!("Cannot resolve base path {}: {}", base, e))
        })?;

        let base_canon = Utf8PathBuf::from_path_buf(base_canon_std)
            .map_err(|_| SecurityError::InvalidPath("Base path is not valid UTF-8".to_string()))?;

        if !canonical.starts_with(&base_canon) {
            return Err(SecurityError::PathOutsideBase);
        }
    }

    Ok(canonical)
}

/// Validate font file size.
pub fn validate_font_size(size: u64) -> Result<(), SecurityError> {
    if size > MAX_FONT_SIZE {
        return Err(SecurityError::FontTooLarge {
            size,
            max: MAX_FONT_SIZE,
        });
    }
    Ok(())
}

/// Validate text input length.
pub fn validate_text_input(text: &str) -> Result<(), SecurityError> {
    if text.len() > MAX_TEXT_LENGTH {
        return Err(SecurityError::TextTooLong {
            len: text.len(),
            max: MAX_TEXT_LENGTH,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_font_size_ok() {
        assert!(validate_font_size(1000).is_ok());
        assert!(validate_font_size(MAX_FONT_SIZE).is_ok());
    }

    #[test]
    fn test_validate_font_size_too_large() {
        assert!(validate_font_size(MAX_FONT_SIZE + 1).is_err());
    }

    #[test]
    fn test_validate_text_input_ok() {
        assert!(validate_text_input("Hello").is_ok());
        assert!(validate_text_input(&"a".repeat(MAX_TEXT_LENGTH)).is_ok());
    }

    #[test]
    fn test_validate_text_input_too_long() {
        assert!(validate_text_input(&"a".repeat(MAX_TEXT_LENGTH + 1)).is_err());
    }

    #[test]
    fn test_sanitize_path_rejects_dotdot() {
        let path = Utf8Path::new("../etc/passwd");
        assert!(sanitize_path(path, None).is_err());
    }

    #[test]
    fn test_sanitize_path_rejects_tilde() {
        let path = Utf8Path::new("~/test");
        assert!(sanitize_path(path, None).is_err());
    }
}
