//! When things go wrong in the pipeline

use thiserror::Error;

pub type Result<T, E = TypfError> = std::result::Result<T, E>;

/// Every failure in Typf has a story to tell
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

/// When fonts refuse to load
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

/// When shaping goes wrong
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

/// When rendering fails
#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Invalid bitmap dimensions: {width}x{height} (max 65,535 pixels per dimension). For long texts, use smaller font sizes, implement line wrapping, or use SVG export instead of bitmap rendering.")]
    InvalidDimensions { width: u32, height: u32 },

    #[error("Out of memory")]
    OutOfMemory,

    #[error("Format not supported: {0}")]
    FormatNotSupported(String),

    #[error("Backend error: {0}")]
    BackendError(String),

    #[error("Invalid font")]
    InvalidFont,

    #[error("Glyph not found: {0}")]
    GlyphNotFound(u32),

    #[error("Outline extraction failed")]
    OutlineExtractionFailed,

    #[error("Path building failed")]
    PathBuildingFailed,

    #[error("Pixmap creation failed")]
    PixmapCreationFailed,
}

/// When export can't finish
#[derive(Debug, Error)]
pub enum ExportError {
    #[error("Format not supported: {0}")]
    FormatNotSupported(String),

    #[error("Encoding failed: {0}")]
    EncodingFailed(String),

    #[error("Write failed: {0}")]
    WriteFailed(String),
}
