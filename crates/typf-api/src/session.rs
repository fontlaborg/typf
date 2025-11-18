// this_file: crates/typf-api/src/session.rs

use crate::backend::{create_backend, create_default_backend, Backend, BackendFeatures, DynBackend, FontMetrics};
use typf_core::{Bitmap, Font, RenderOptions, TypfError, Result}; // Import Result as well
use typf_core::types::ShapingResult;

/// Main entry point for the TYPF rendering engine.
///
/// A `Session` manages the active rendering backend and provides
/// high-level methods for text shaping and rendering.
pub struct Session {
    current_backend: Box<dyn DynBackend>,
    active_font: Font,
}

impl Session {
    /// Creates a new `SessionBuilder` for configuring a `Session`.
    pub fn builder(active_font: Font) -> SessionBuilder {
        SessionBuilder::new(active_font)
    }

    /// Sets the active rendering backend for this session.
    ///
    /// This allows switching between different rendering engines at runtime.
    pub fn set_backend(&mut self, backend: Backend) {
        self.current_backend = create_backend(backend);
    }

    /// Shapes the given text using the active font and backend.
    pub fn shape_text(&self, text: &str) -> ShapingResult {
        self.current_backend.shape_text(text, &self.active_font)
    }

    /// Renders a single glyph to a bitmap using the active backend.
    pub fn render_glyph(&self, glyph_id: u32, options: RenderOptions) -> Option<Bitmap> {
        self.current_backend
            .render_glyph(&self.active_font, glyph_id, options)
    }

    /// Renders shaped text to a bitmap using the active backend.
    pub fn render_shaped_text(&self, shaped_text: &ShapingResult, options: RenderOptions) -> Option<Bitmap> {
        self.current_backend.render_shaped_text(shaped_text, options)
    }

    /// Calculates the font metrics for the active font using the active backend.
    pub fn font_metrics(&self) -> FontMetrics {
        self.current_backend.font_metrics(&self.active_font)
    }

    /// Returns the features supported by the currently active backend.
    pub fn supported_features(&self) -> BackendFeatures {
        self.current_backend.supported_features()
    }

    /// Renders a given text string to a bitmap.
    ///
    /// This is a convenience method that combines shaping and rendering.
    pub fn render(
        &self,
        text: &str,
        font_size: f32,
        _color: Option<u32>, // TODO: Implement color handling
        grayscale: bool,
    ) -> Result<Bitmap> {
        let shaped_text = self.shape_text(text);
        let options = RenderOptions {
            font_size,
            grayscale,
            // TODO: Pass color from _color
            ..Default::default()
        };
        self.current_backend
            .render_shaped_text(&shaped_text, options)
            .ok_or_else(|| TypfError::render("Failed to render shaped text".to_string()))
    }
}

/// A builder for configuring and creating a `Session`.
pub struct SessionBuilder {
    active_font: Font,
    initial_backend: Option<Backend>,
}

impl SessionBuilder {
    /// Creates a new `SessionBuilder` with the given active font.
    pub fn new(active_font: Font) -> Self {
        Self {
            active_font,
            initial_backend: None,
        }
    }

    /// Specifies the initial backend to use for the session.
    /// If not set, a default backend will be chosen based on platform and features.
    pub fn with_backend(mut self, backend: Backend) -> Self {
        self.initial_backend = Some(backend);
        self
    }

    /// Builds the `Session` instance.
    pub fn build(self) -> Session {
        let current_backend = match self.initial_backend {
            Some(backend) => create_backend(backend),
            None => create_default_backend(),
        };

        Session {
            current_backend,
            active_font: self.active_font,
        }
    }
}