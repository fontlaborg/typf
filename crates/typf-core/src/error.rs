//! Error types for TYPF

use thiserror::Error;

pub type Result<T> = std::result::Result<T, TypfError>;

/// Main error type for TYPF
#[derive(Debug, Error)]
pub enum TypfError {
    #[error("Feature not implemented: {0}")]
    NotImplemented(String),

    #[error("Feature not compiled: {0}")]
    FeatureNotCompiled(String),

    #[error("Invalid backend combination: shaping={0}, render={1}")]
    UnsupportedBackendCombination(String, String),

    #[error("Font loading failed: {0}")]
    FontLoad(#[from] FontLoadError),

    #[error("Shaping failed: {0}")]
    ShapingFailed(#[from] ShapingError),

    #[error("Rendering failed: {0}")]
    RenderingFailed(#[from] RenderError),

    #[error("Export failed: {0}")]
    ExportFailed(#[from] ExportError),

    #[error("Pipeline error: {0}")]
    Pipeline(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(String),
}

/// Font loading errors
#[derive(Debug, Error)]
pub enum FontLoadError {
    #[error("Font file not found: {0}")]
    FileNotFound(String),

    #[error("Invalid font data")]
    InvalidData,

    #[error("Font not supported: {0}")]
    NotSupported(String),

    #[error("System font not found: {0}")]
    SystemFontNotFound(String),
}

/// Shaping errors
#[derive(Debug, Error)]
pub enum ShapingError {
    #[error("Invalid text input")]
    InvalidText,

    #[error("Script not supported: {0}")]
    ScriptNotSupported(String),

    #[error("Language not supported: {0}")]
    LanguageNotSupported(String),

    #[error("Feature not supported: {0}")]
    FeatureNotSupported(String),

    #[error("Backend error: {0}")]
    BackendError(String),
}

/// Rendering errors
#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Invalid dimensions: {width}x{height}")]
    InvalidDimensions { width: u32, height: u32 },

    #[error("Out of memory")]
    OutOfMemory,

    #[error("Format not supported: {0}")]
    FormatNotSupported(String),

    #[error("Backend error: {0}")]
    BackendError(String),
}

/// Export errors
#[derive(Debug, Error)]
pub enum ExportError {
    #[error("Format not supported: {0}")]
    FormatNotSupported(String),

    #[error("Encoding failed: {0}")]
    EncodingFailed(String),

    #[error("Write failed: {0}")]
    WriteFailed(String),
}
