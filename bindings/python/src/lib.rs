//! Python bindings - Typf's power, packaged for Python developers
//!
//! This is where Rust meets Python: high-performance text shaping and rendering
//! that feels native in Python code. We handle the complexity of font loading,
//! Unicode processing, and pixel-perfect rendering—all you do is call a function.

#![allow(clippy::useless_conversion)]

use pyo3::exceptions::{PyIOError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};
use std::collections::HashMap;
use std::sync::Arc;
#[cfg(feature = "linra")]
use typf_core::linra::{LinraRenderParams, LinraRenderer};
use typf_core::{
    traits::{Exporter, Renderer, Shaper},
    types::{BitmapData, Direction, RenderOutput},
    Color, RenderParams, ShapingParams,
};
use typf_export::PnmExporter;
use typf_fontdb::TypfFontFace;
use typf_unicode::{UnicodeOptions, UnicodeProcessor};

/// Workspace version (injected at build time)
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Auto-detect text direction using Unicode bidi analysis
fn detect_direction(text: &str, language: Option<&str>) -> Direction {
    let processor = UnicodeProcessor::new();
    let options = UnicodeOptions {
        detect_scripts: true,
        normalize: true,
        bidi_resolve: true,
        language: language.map(|l| l.to_string()),
    };

    match processor.process(text, &options) {
        Ok(runs) => {
            // Find first run with explicit RTL direction
            runs.iter()
                .find_map(|run| {
                    if matches!(
                        run.direction,
                        Direction::RightToLeft | Direction::BottomToTop
                    ) {
                        Some(run.direction)
                    } else {
                        None
                    }
                })
                .unwrap_or(Direction::LeftToRight)
        },
        Err(_) => Direction::LeftToRight,
    }
}

/// Parse direction string to Direction enum
fn parse_direction(dir_str: &str, text: &str, language: Option<&str>) -> PyResult<Direction> {
    match dir_str {
        "auto" => Ok(detect_direction(text, language)),
        "ltr" => Ok(Direction::LeftToRight),
        "rtl" => Ok(Direction::RightToLeft),
        "ttb" => Ok(Direction::TopToBottom),
        "btt" => Ok(Direction::BottomToTop),
        _ => Err(PyValueError::new_err(format!(
            "Invalid direction: {}. Use: auto, ltr, rtl, ttb, btt",
            dir_str
        ))),
    }
}

/// Load font with optional TTC index
fn load_font(font_path: &str, face_index: u32) -> PyResult<Arc<dyn typf_core::traits::FontRef>> {
    let font = if face_index == 0 {
        TypfFontFace::from_file(font_path)
    } else {
        TypfFontFace::from_file_index(font_path, face_index)
    }
    .map_err(|e| PyIOError::new_err(format!("Failed to load font: {:?}", e)))?;

    Ok(Arc::new(font) as Arc<dyn typf_core::traits::FontRef>)
}

// Note: Skia and Zeno renderers not yet available in workspace

/// The main Typf interface that Python developers will love
///
/// This class hides all the Rust complexity behind a simple Python interface.
/// Pick your shaper, pick your renderer, and start rendering beautiful text.
#[pyclass]
struct Typf {
    shaper: Arc<dyn Shaper + Send + Sync>, // How we transform text to glyphs
    renderer: Arc<dyn Renderer + Send + Sync>, // How we turn glyphs into images
}

#[pymethods]
impl Typf {
    /// Creates your Typf rendering pipeline
    ///
    /// Choose your weapons:
    /// - Shapers: "none" (debug), "harfbuzz" (professional), "coretext" (macOS), "icu-hb" (Unicode perfect)
    /// - Renderers: "opixa" (pure Rust), "json" (data), "coregraphics" (macOS), "skia" (pro), "zeno" (pure Rust), "vello-cpu" (high-quality), "vello" (GPU)
    #[new]
    #[pyo3(signature = (shaper="harfbuzz", renderer="opixa"))]
    fn new(shaper: &str, renderer: &str) -> PyResult<Self> {
        // Find and create the requested text shaper
        let shaper: Arc<dyn Shaper + Send + Sync> = match shaper {
            "none" => Arc::new(typf_shape_none::NoneShaper::new()),
            #[cfg(feature = "shaping-hb")]
            "harfbuzz" | "hb" => Arc::new(typf_shape_hb::HarfBuzzShaper::new()),
            #[cfg(feature = "shaping-ct")]
            "coretext" | "ct" | "mac" => Arc::new(typf_shape_ct::CoreTextShaper::new()),
            #[cfg(feature = "shaping-icu-hb")]
            "icu-hb" | "icu-harfbuzz" => Arc::new(typf_shape_icu_hb::IcuHarfBuzzShaper::new()),
            _ => {
                return Err(PyValueError::new_err(format!(
                    "Unknown shaper: {}. Available: none, harfbuzz, coretext, icu-hb",
                    shaper
                )))
            },
        };

        // Find and create the requested pixel renderer
        let renderer: Arc<dyn Renderer + Send + Sync> = match renderer {
            #[cfg(feature = "render-json")]
            "json" => Arc::new(typf_render_json::JsonRenderer::new()),
            "opixa" => Arc::new(typf_render_opixa::OpixaRenderer::new()),
            #[cfg(feature = "render-cg")]
            "coregraphics" | "cg" | "mac" => Arc::new(typf_render_cg::CoreGraphicsRenderer::new()),
            #[cfg(feature = "render-skia")]
            "skia" => Arc::new(typf_render_skia::SkiaRenderer::new()),
            #[cfg(feature = "render-zeno")]
            "zeno" => Arc::new(typf_render_zeno::ZenoRenderer::new()),
            #[cfg(feature = "render-vello-cpu")]
            "vello-cpu" => Arc::new(typf_render_vello_cpu::VelloCpuRenderer::new()),
            #[cfg(feature = "render-vello")]
            "vello" => {
                typf_render_vello::VelloRenderer::new()
                    .map(|r| Arc::new(r) as Arc<dyn Renderer + Send + Sync>)
                    .map_err(|e| PyRuntimeError::new_err(format!("Failed to create GPU renderer: {}", e)))?
            }
            _ => {
                return Err(PyValueError::new_err(format!(
                    "Unknown renderer: {}. Available: json, opixa, coregraphics, skia, zeno, vello-cpu, vello",
                    renderer
                )))
            },
        };

        Ok(Self { shaper, renderer })
    }

    /// Turns your text into beautiful pixels - the main event
    ///
    /// This method does the full pipeline: loads the font, shapes the text,
    /// renders to bitmap, and returns it in a Python-friendly format.
    ///
    /// Args:
    ///     text: The text to render
    ///     font_path: Path to the font file
    ///     size: Font size in pixels (default: 16.0)
    ///     color: Foreground color as (r, g, b, a) tuple (default: black)
    ///     background: Background color as (r, g, b, a) tuple (default: transparent)
    ///     padding: Padding around rendered text in pixels (default: 10)
    ///     variations: Dict of font variation axis settings
    ///     direction: Text direction - "auto", "ltr", "rtl", "ttb", "btt" (default: "auto")
    ///     language: Language hint for direction detection (e.g., "ar", "he")
    ///     face_index: TTC collection face index (default: 0)
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::useless_conversion)]
    #[pyo3(signature = (text, font_path, size=16.0, color=None, background=None, padding=10, variations=None, direction="auto", language=None, face_index=0))]
    fn render_text(
        &self,
        py: Python,
        text: &str,
        font_path: &str,
        size: f32,
        color: Option<(u8, u8, u8, u8)>,
        background: Option<(u8, u8, u8, u8)>,
        padding: u32,
        variations: Option<HashMap<String, f32>>,
        direction: &str,
        language: Option<&str>,
        face_index: u32,
    ) -> PyResult<PyObject> {
        // Load font with optional TTC index
        let font_arc = load_font(font_path, face_index)?;

        let mut variation_vec: Vec<(String, f32)> =
            variations.unwrap_or_default().into_iter().collect();
        variation_vec.sort_by(|a, b| a.0.cmp(&b.0));

        // Auto-detect or parse direction
        let resolved_direction = parse_direction(direction, text, language)?;

        // Configure how we want to shape the text
        let shaping_params = ShapingParams {
            size,
            direction: resolved_direction,
            variations: variation_vec.clone(),
            ..Default::default()
        };

        // Transform text into positioned glyphs
        let shaped = self
            .shaper
            .shape(text, font_arc.clone(), &shaping_params)
            .map_err(|e| PyRuntimeError::new_err(format!("Shaping failed: {:?}", e)))?;

        // Parse colors (default to black on transparent)
        let foreground = color
            .map(|(r, g, b, a)| Color::rgba(r, g, b, a))
            .unwrap_or(Color::rgba(0, 0, 0, 255));
        let background = background.map(|(r, g, b, a)| Color::rgba(r, g, b, a));

        let render_params = RenderParams {
            foreground,
            background,
            padding,
            variations: variation_vec,
            ..Default::default()
        };

        // Render the shaped glyphs into actual pixels
        let rendered = self
            .renderer
            .render(&shaped, font_arc, &render_params)
            .map_err(|e| PyRuntimeError::new_err(format!("Rendering failed: {:?}", e)))?;

        // Package the result for Python consumption
        match rendered {
            RenderOutput::Bitmap(bitmap) => {
                let result = PyDict::new_bound(py);
                result.set_item("width", bitmap.width)?;
                result.set_item("height", bitmap.height)?;
                result.set_item("format", format!("{:?}", bitmap.format))?;
                result.set_item("data", PyBytes::new_bound(py, &bitmap.data))?;
                Ok(result.into()) // Return as a dict with metadata
            },
            RenderOutput::Json(json_str) => {
                // JSON renderers get special handling - return the raw string
                Ok(json_str.into_py(py))
            },
            RenderOutput::Vector(_) => Err(PyValueError::new_err(
                "Vector output not yet supported in Python bindings",
            )),
            RenderOutput::Geometry(_) => Err(PyValueError::new_err(
                "Geometry output not yet supported in Python bindings (use render_text for bitmaps)",
            )),
        }
    }

    /// Get shaper name
    fn get_shaper(&self) -> String {
        self.shaper.name().to_string()
    }

    /// Get renderer name
    fn get_renderer(&self) -> String {
        self.renderer.name().to_string()
    }

    /// Shape text without rendering (for benchmarking and JSON export)
    ///
    /// Args:
    ///     text: The text to shape
    ///     font_path: Path to the font file
    ///     size: Font size in pixels (default: 16.0)
    ///     direction: Text direction - "auto", "ltr", "rtl", "ttb", "btt" (default: "auto")
    ///     language: Language hint for direction detection
    ///     face_index: TTC collection face index (default: 0)
    #[allow(clippy::useless_conversion)]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (text, font_path, size=16.0, direction="auto", language=None, face_index=0))]
    fn shape_text(
        &self,
        py: Python,
        text: &str,
        font_path: &str,
        size: f32,
        direction: &str,
        language: Option<&str>,
        face_index: u32,
    ) -> PyResult<PyObject> {
        // Load font with optional TTC index
        let font_arc = load_font(font_path, face_index)?;

        // Auto-detect or parse direction
        let resolved_direction = parse_direction(direction, text, language)?;

        // Set up shaping parameters
        let shaping_params = ShapingParams {
            size,
            direction: resolved_direction,
            ..Default::default()
        };

        // Shape the text
        let shaped = self
            .shaper
            .shape(text, font_arc.clone(), &shaping_params)
            .map_err(|e| PyRuntimeError::new_err(format!("Shaping failed: {:?}", e)))?;

        // Return basic shaping info as dict
        let result = PyDict::new_bound(py);
        result.set_item("glyph_count", shaped.glyphs.len())?;
        result.set_item("width", shaped.advance_width)?;
        Ok(result.into())
    }

    /// Shape text and return full glyph data for direct rendering
    ///
    /// This is the preferred method for Pycairo/Cairo integration. Returns a
    /// ShapedGlyphs object with zero-copy access to glyph positions.
    ///
    /// Args:
    ///     text: The text to shape
    ///     font_path: Path to the font file
    ///     size: Font size in pixels (default: 16.0)
    ///     direction: Text direction - "auto", "ltr", "rtl", "ttb", "btt" (default: "auto")
    ///     language: Language hint for direction detection
    ///     face_index: TTC collection face index (default: 0)
    ///     variations: Dict of font variation axis settings
    ///
    /// Returns:
    ///     ShapedGlyphs: Object with glyph data accessible via iteration, indexing,
    ///                   or bulk methods like for_cairo(), as_tuples(), etc.
    ///
    /// Example:
    ///     >>> glyphs = typf.shape_glyphs("Hello", "font.ttf", size=24.0)
    ///     >>> for glyph in glyphs:
    ///     ...     print(f"Glyph {glyph.glyph_id} at ({glyph.x}, {glyph.y})")
    ///     >>> # For Cairo:
    ///     >>> cairo_glyphs = glyphs.for_cairo()
    #[allow(clippy::useless_conversion)]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (text, font_path, size=16.0, direction="auto", language=None, face_index=0, variations=None))]
    fn shape_glyphs(
        &self,
        text: &str,
        font_path: &str,
        size: f32,
        direction: &str,
        language: Option<&str>,
        face_index: u32,
        variations: Option<HashMap<String, f32>>,
    ) -> PyResult<ShapedGlyphs> {
        // Load font with optional TTC index
        let font_arc = load_font(font_path, face_index)?;

        let mut variation_vec: Vec<(String, f32)> =
            variations.unwrap_or_default().into_iter().collect();
        variation_vec.sort_by(|a, b| a.0.cmp(&b.0));

        // Auto-detect or parse direction
        let resolved_direction = parse_direction(direction, text, language)?;

        // Set up shaping parameters
        let shaping_params = ShapingParams {
            size,
            direction: resolved_direction,
            variations: variation_vec,
            ..Default::default()
        };

        // Shape the text
        let shaped = self
            .shaper
            .shape(text, font_arc.clone(), &shaping_params)
            .map_err(|e| PyRuntimeError::new_err(format!("Shaping failed: {:?}", e)))?;

        Ok(ShapedGlyphs {
            glyphs: shaped.glyphs,
            advance_width: shaped.advance_width,
            advance_height: shaped.advance_height,
            direction: shaped.direction,
        })
    }

    /// Render text to SVG vector format
    ///
    /// Args:
    ///     text: The text to render
    ///     font_path: Path to the font file
    ///     size: Font size in pixels (default: 16.0)
    ///     color: Foreground color as (r, g, b, a) tuple (default: black)
    ///     padding: Padding around rendered text in pixels (default: 10)
    ///     direction: Text direction - "auto", "ltr", "rtl", "ttb", "btt" (default: "auto")
    ///     language: Language hint for direction detection
    ///     face_index: TTC collection face index (default: 0)
    #[cfg(feature = "export-svg")]
    #[allow(clippy::useless_conversion)]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (text, font_path, size=16.0, color=None, padding=10, direction="auto", language=None, face_index=0))]
    fn render_to_svg(
        &self,
        text: &str,
        font_path: &str,
        size: f32,
        color: Option<(u8, u8, u8, u8)>,
        padding: u32,
        direction: &str,
        language: Option<&str>,
        face_index: u32,
    ) -> PyResult<String> {
        // Load font with optional TTC index
        let font_arc = load_font(font_path, face_index)?;

        // Auto-detect or parse direction
        let resolved_direction = parse_direction(direction, text, language)?;

        // Set up shaping parameters
        let shaping_params = ShapingParams {
            size,
            direction: resolved_direction,
            ..Default::default()
        };

        // Shape the text
        let shaped = self
            .shaper
            .shape(text, font_arc.clone(), &shaping_params)
            .map_err(|e| PyRuntimeError::new_err(format!("Shaping failed: {:?}", e)))?;

        // Set up color
        let foreground = color
            .map(|(r, g, b, a)| Color::rgba(r, g, b, a))
            .unwrap_or(Color::rgba(0, 0, 0, 255));

        // Export to SVG using the proper SVG exporter
        let svg_exporter = typf_export_svg::SvgExporter::new().with_padding(padding as f32);

        let svg_string = svg_exporter
            .export(&shaped, font_arc, foreground)
            .map_err(|e| PyRuntimeError::new_err(format!("SVG export failed: {:?}", e)))?;

        Ok(svg_string)
    }
}

/// Linra text renderer - single-pass shaping AND rendering
///
/// This class provides maximum performance by using platform-native APIs
/// that shape and render text in a single operation. Available on macOS
/// (CoreText) and Windows (DirectWrite).
#[cfg(feature = "linra")]
#[pyclass]
struct TypfLinra {
    renderer: Arc<dyn LinraRenderer>,
}

#[cfg(feature = "linra")]
#[pymethods]
impl TypfLinra {
    /// Creates a new linra renderer
    ///
    /// Available renderers:
    /// - "coretext" / "mac" - CoreText CTLineDraw (macOS only)
    /// - "directwrite" / "win" - DirectWrite DrawTextLayout (Windows only)
    #[new]
    #[pyo3(signature = (renderer="auto"))]
    fn new(renderer: &str) -> PyResult<Self> {
        let renderer: Arc<dyn LinraRenderer> = match renderer {
            #[cfg(feature = "linra-mac")]
            "auto" | "coretext" | "ct" | "mac" | "linra-mac" => {
                Arc::new(typf_os_mac::CoreTextLinraRenderer::new())
            }

            #[cfg(all(feature = "linra-win", target_os = "windows"))]
            "auto" | "directwrite" | "dw" | "win" | "linra-win" => {
                typf_os_win::DirectWriteLinraRenderer::new()
                    .map(|r| Arc::new(r) as Arc<dyn LinraRenderer>)
                    .map_err(|e| PyRuntimeError::new_err(format!("Failed to create DirectWrite renderer: {:?}", e)))?
            }

            _ => {
                return Err(PyValueError::new_err(format!(
                    "Unknown linra renderer: {}. Available: coretext/mac (macOS), directwrite/win (Windows)",
                    renderer
                )))
            }
        };

        Ok(Self { renderer })
    }

    /// Render text using linra (single-pass shaping + rendering)
    ///
    /// This is faster than the traditional shaper+renderer pipeline because
    /// the platform API handles everything in one optimized call.
    ///
    /// Args:
    ///     text: The text to render
    ///     font_path: Path to the font file
    ///     size: Font size in pixels (default: 16.0)
    ///     color: Foreground color as (r, g, b, a) tuple (default: black)
    ///     background: Background color as (r, g, b, a) tuple (default: transparent)
    ///     padding: Padding around rendered text in pixels (default: 10)
    ///     variations: Dict of font variation axis settings
    ///     features: List of OpenType feature tuples (name, value)
    ///     language: Language tag (e.g., "ar", "he", "en")
    ///     script: Script tag (e.g., "Arab", "Hebr", "Latn")
    ///     direction: Text direction - "auto", "ltr", "rtl", "ttb", "btt" (default: "auto")
    ///     face_index: TTC collection face index (default: 0)
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (text, font_path, size=16.0, color=None, background=None, padding=10, variations=None, features=None, language=None, script=None, direction="auto", face_index=0))]
    fn render_text(
        &self,
        py: Python,
        text: &str,
        font_path: &str,
        size: f32,
        color: Option<(u8, u8, u8, u8)>,
        background: Option<(u8, u8, u8, u8)>,
        padding: u32,
        variations: Option<HashMap<String, f32>>,
        features: Option<Vec<(String, u32)>>,
        language: Option<String>,
        script: Option<String>,
        direction: &str,
        face_index: u32,
    ) -> PyResult<PyObject> {
        // Load font with optional TTC index
        let font_arc = load_font(font_path, face_index)?;

        // Parse colors
        let foreground = color
            .map(|(r, g, b, a)| Color::rgba(r, g, b, a))
            .unwrap_or(Color::rgba(0, 0, 0, 255));
        let background = background.map(|(r, g, b, a)| Color::rgba(r, g, b, a));

        let mut variation_vec: Vec<(String, f32)> =
            variations.unwrap_or_default().into_iter().collect();
        variation_vec.sort_by(|a, b| a.0.cmp(&b.0));

        // Auto-detect or parse direction
        let resolved_direction = parse_direction(direction, text, language.as_deref())?;

        // Create linra parameters
        let params = LinraRenderParams {
            size,
            direction: resolved_direction,
            foreground,
            background,
            padding,
            variations: variation_vec,
            features: features.unwrap_or_default(),
            language,
            script,
            antialias: true,
            letter_spacing: 0.0,
            color_palette: 0,
        };

        // Render using linra (single-pass)
        let rendered = self
            .renderer
            .render_text(text, font_arc, &params)
            .map_err(|e| PyRuntimeError::new_err(format!("Linra rendering failed: {:?}", e)))?;

        // Package result
        match rendered {
            RenderOutput::Bitmap(bitmap) => {
                let result = PyDict::new_bound(py);
                result.set_item("width", bitmap.width)?;
                result.set_item("height", bitmap.height)?;
                result.set_item("format", format!("{:?}", bitmap.format))?;
                result.set_item("data", PyBytes::new_bound(py, &bitmap.data))?;
                Ok(result.into())
            },
            _ => Err(PyValueError::new_err("Unexpected render output format")),
        }
    }

    /// Get the renderer name
    fn get_renderer(&self) -> String {
        self.renderer.name().to_string()
    }

    /// Clear internal caches
    fn clear_cache(&self) {
        self.renderer.clear_cache();
    }
}

/// A path operation for vector glyph outlines
///
/// These are the primitive drawing commands that define glyph shapes.
/// Compatible with ReportLab, Cairo, and other vector graphics libraries.
#[pyclass]
#[derive(Clone)]
struct PathOp {
    /// Operation type: "move", "line", "quad", "cubic", "close"
    #[pyo3(get)]
    op: String,
    /// X coordinate (for move, line) or control point X (for curves)
    #[pyo3(get)]
    x: Option<f32>,
    /// Y coordinate (for move, line) or control point Y (for curves)
    #[pyo3(get)]
    y: Option<f32>,
    /// Second control point X (for cubic curves)
    #[pyo3(get)]
    cx: Option<f32>,
    /// Second control point Y (for cubic curves)
    #[pyo3(get)]
    cy: Option<f32>,
    /// Third control point X (for cubic curves only)
    #[pyo3(get)]
    c2x: Option<f32>,
    /// Third control point Y (for cubic curves only)
    #[pyo3(get)]
    c2y: Option<f32>,
}

#[pymethods]
impl PathOp {
    fn __repr__(&self) -> String {
        match self.op.as_str() {
            "move" => format!(
                "PathOp(move, {}, {})",
                self.x.unwrap_or(0.0),
                self.y.unwrap_or(0.0)
            ),
            "line" => format!(
                "PathOp(line, {}, {})",
                self.x.unwrap_or(0.0),
                self.y.unwrap_or(0.0)
            ),
            "quad" => format!(
                "PathOp(quad, cx={}, cy={}, x={}, y={})",
                self.cx.unwrap_or(0.0),
                self.cy.unwrap_or(0.0),
                self.x.unwrap_or(0.0),
                self.y.unwrap_or(0.0)
            ),
            "cubic" => format!(
                "PathOp(cubic, c1=({}, {}), c2=({}, {}), end=({}, {}))",
                self.cx.unwrap_or(0.0),
                self.cy.unwrap_or(0.0),
                self.c2x.unwrap_or(0.0),
                self.c2y.unwrap_or(0.0),
                self.x.unwrap_or(0.0),
                self.y.unwrap_or(0.0)
            ),
            "close" => "PathOp(close)".to_string(),
            _ => format!("PathOp({})", self.op),
        }
    }

    /// Convert to a tuple for easier consumption by graphics libraries
    ///
    /// Returns:
    ///     For "move"/"line": ("op", x, y)
    ///     For "quad": ("quad", cx, cy, x, y)
    ///     For "cubic": ("cubic", c1x, c1y, c2x, c2y, x, y)
    ///     For "close": ("close",)
    fn as_tuple(&self, py: Python) -> PyResult<PyObject> {
        let result = match self.op.as_str() {
            "move" | "line" => (
                self.op.clone(),
                self.x.unwrap_or(0.0),
                self.y.unwrap_or(0.0),
            )
                .into_py(py),
            "quad" => (
                self.op.clone(),
                self.cx.unwrap_or(0.0),
                self.cy.unwrap_or(0.0),
                self.x.unwrap_or(0.0),
                self.y.unwrap_or(0.0),
            )
                .into_py(py),
            "cubic" => (
                self.op.clone(),
                self.cx.unwrap_or(0.0),
                self.cy.unwrap_or(0.0),
                self.c2x.unwrap_or(0.0),
                self.c2y.unwrap_or(0.0),
                self.x.unwrap_or(0.0),
                self.y.unwrap_or(0.0),
            )
                .into_py(py),
            "close" => ("close",).into_py(py),
            _ => (self.op.clone(),).into_py(py),
        };
        Ok(result)
    }
}

/// Convert internal PathOp to Python PathOp
#[allow(dead_code)] // Ready for use when glyph outline extraction is added
fn path_op_to_python(op: &typf_core::types::PathOp) -> PathOp {
    use typf_core::types::PathOp as InternalPathOp;
    match op {
        InternalPathOp::MoveTo { x, y } => PathOp {
            op: "move".to_string(),
            x: Some(*x),
            y: Some(*y),
            cx: None,
            cy: None,
            c2x: None,
            c2y: None,
        },
        InternalPathOp::LineTo { x, y } => PathOp {
            op: "line".to_string(),
            x: Some(*x),
            y: Some(*y),
            cx: None,
            cy: None,
            c2x: None,
            c2y: None,
        },
        InternalPathOp::QuadTo { cx, cy, x, y } => PathOp {
            op: "quad".to_string(),
            x: Some(*x),
            y: Some(*y),
            cx: Some(*cx),
            cy: Some(*cy),
            c2x: None,
            c2y: None,
        },
        InternalPathOp::CubicTo {
            c1x,
            c1y,
            c2x,
            c2y,
            x,
            y,
        } => PathOp {
            op: "cubic".to_string(),
            x: Some(*x),
            y: Some(*y),
            cx: Some(*c1x),
            cy: Some(*c1y),
            c2x: Some(*c2x),
            c2y: Some(*c2y),
        },
        InternalPathOp::Close => PathOp {
            op: "close".to_string(),
            x: None,
            y: None,
            cx: None,
            cy: None,
            c2x: None,
            c2y: None,
        },
    }
}

/// Glyph path data with vector outline
#[pyclass]
#[derive(Clone)]
struct GlyphPath {
    /// Glyph ID in the font
    #[pyo3(get)]
    glyph_id: u32,
    /// X position (in rendered coordinates)
    #[pyo3(get)]
    x: f32,
    /// Y position (in rendered coordinates)
    #[pyo3(get)]
    y: f32,
    /// Path operations defining the outline (in font units)
    #[pyo3(get)]
    ops: Vec<PathOp>,
}

#[pymethods]
impl GlyphPath {
    fn __repr__(&self) -> String {
        format!(
            "GlyphPath(glyph_id={}, pos=({}, {}), {} ops)",
            self.glyph_id,
            self.x,
            self.y,
            self.ops.len()
        )
    }

    /// Get path operations as a list of tuples
    ///
    /// Convenient for passing to graphics libraries like ReportLab or Cairo.
    fn ops_as_tuples(&self, py: Python) -> PyResult<Vec<PyObject>> {
        self.ops.iter().map(|op| op.as_tuple(py)).collect()
    }
}

/// A positioned glyph from shaping results
///
/// This provides access to individual glyph data in a Python-friendly format.
/// For bulk access, use ShapedGlyphs which provides NumPy-compatible views.
#[pyclass]
#[derive(Clone)]
struct PositionedGlyph {
    /// Glyph index in the font
    #[pyo3(get)]
    glyph_id: u32,
    /// Horizontal position in user space
    #[pyo3(get)]
    x: f32,
    /// Vertical position in user space
    #[pyo3(get)]
    y: f32,
    /// Horizontal advance width
    #[pyo3(get)]
    advance: f32,
    /// Cluster index (maps to original text position)
    #[pyo3(get)]
    cluster: u32,
}

#[pymethods]
impl PositionedGlyph {
    fn __repr__(&self) -> String {
        format!(
            "PositionedGlyph(id={}, pos=({:.1}, {:.1}), advance={:.1}, cluster={})",
            self.glyph_id, self.x, self.y, self.advance, self.cluster
        )
    }

    /// Convert to tuple (glyph_id, x, y, advance, cluster)
    ///
    /// Convenient for unpacking or passing to other libraries.
    fn as_tuple(&self, py: Python) -> PyObject {
        (self.glyph_id, self.x, self.y, self.advance, self.cluster).into_py(py)
    }
}

/// Zero-copy access to shaped glyph data
///
/// This class holds shaping results and provides efficient access patterns:
/// - Iteration: for glyph in shaped_glyphs
/// - Indexing: shaped_glyphs[0]
/// - NumPy array (if numpy-interop feature enabled): shaped_glyphs.as_numpy_array()
///
/// The underlying data is stored in a C-compatible format suitable for
/// direct use with graphics libraries like Cairo, Pango, or Pycairo.
#[pyclass]
struct ShapedGlyphs {
    /// The raw glyph data
    glyphs: Vec<typf_core::types::PositionedGlyph>,
    /// Total advance width
    advance_width: f32,
    /// Total advance height
    advance_height: f32,
    /// Text direction
    direction: Direction,
}

#[pymethods]
impl ShapedGlyphs {
    /// Number of glyphs
    fn __len__(&self) -> usize {
        self.glyphs.len()
    }

    /// Get glyph by index
    fn __getitem__(&self, index: isize) -> PyResult<PositionedGlyph> {
        let len = self.glyphs.len() as isize;
        let idx = if index < 0 { len + index } else { index };
        if idx < 0 || idx >= len {
            return Err(pyo3::exceptions::PyIndexError::new_err(
                "glyph index out of range",
            ));
        }
        let g = &self.glyphs[idx as usize];
        Ok(PositionedGlyph {
            glyph_id: g.id,
            x: g.x,
            y: g.y,
            advance: g.advance,
            cluster: g.cluster,
        })
    }

    /// Iterate over glyphs
    fn __iter__(slf: PyRef<'_, Self>) -> ShapedGlyphsIter {
        ShapedGlyphsIter {
            glyphs: slf.glyphs.clone(),
            index: 0,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "ShapedGlyphs({} glyphs, advance=({:.1}, {:.1}), direction={:?})",
            self.glyphs.len(),
            self.advance_width,
            self.advance_height,
            self.direction
        )
    }

    /// Total advance width in pixels
    #[getter]
    fn advance_width(&self) -> f32 {
        self.advance_width
    }

    /// Total advance height in pixels
    #[getter]
    fn advance_height(&self) -> f32 {
        self.advance_height
    }

    /// Text direction as string ("ltr", "rtl", "ttb", "btt")
    #[getter]
    fn direction(&self) -> &'static str {
        match self.direction {
            Direction::LeftToRight => "ltr",
            Direction::RightToLeft => "rtl",
            Direction::TopToBottom => "ttb",
            Direction::BottomToTop => "btt",
        }
    }

    /// Get all glyphs as a list of tuples
    ///
    /// Returns a list of (glyph_id, x, y, advance, cluster) tuples.
    /// Useful for simple iteration or serialization.
    fn as_tuples(&self, py: Python) -> Vec<PyObject> {
        self.glyphs
            .iter()
            .map(|g| (g.id, g.x, g.y, g.advance, g.cluster).into_py(py))
            .collect()
    }

    /// Get glyph IDs as a list
    fn glyph_ids(&self) -> Vec<u32> {
        self.glyphs.iter().map(|g| g.id).collect()
    }

    /// Get X positions as a list
    fn x_positions(&self) -> Vec<f32> {
        self.glyphs.iter().map(|g| g.x).collect()
    }

    /// Get Y positions as a list
    fn y_positions(&self) -> Vec<f32> {
        self.glyphs.iter().map(|g| g.y).collect()
    }

    /// Get advances as a list
    fn advances(&self) -> Vec<f32> {
        self.glyphs.iter().map(|g| g.advance).collect()
    }

    /// Get cluster indices as a list
    fn clusters(&self) -> Vec<u32> {
        self.glyphs.iter().map(|g| g.cluster).collect()
    }

    /// Format for Cairo/Pycairo glyph arrays
    ///
    /// Returns a list of (glyph_id, x, y) tuples in the format expected
    /// by cairo.Context.show_glyphs() and similar APIs.
    fn for_cairo(&self, py: Python) -> Vec<PyObject> {
        self.glyphs
            .iter()
            .map(|g| (g.id as i64, g.x as f64, g.y as f64).into_py(py))
            .collect()
    }
}

/// Iterator for ShapedGlyphs
#[pyclass]
struct ShapedGlyphsIter {
    glyphs: Vec<typf_core::types::PositionedGlyph>,
    index: usize,
}

#[pymethods]
impl ShapedGlyphsIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self) -> Option<PositionedGlyph> {
        if self.index < self.glyphs.len() {
            let g = &self.glyphs[self.index];
            self.index += 1;
            Some(PositionedGlyph {
                glyph_id: g.id,
                x: g.x,
                y: g.y,
                advance: g.advance,
                cluster: g.cluster,
            })
        } else {
            None
        }
    }
}

/// A variable font axis definition
#[pyclass]
#[derive(Clone)]
struct VariationAxisInfo {
    #[pyo3(get)]
    tag: String,
    #[pyo3(get)]
    name: Option<String>,
    #[pyo3(get)]
    min_value: f32,
    #[pyo3(get)]
    default_value: f32,
    #[pyo3(get)]
    max_value: f32,
    #[pyo3(get)]
    hidden: bool,
}

#[pymethods]
impl VariationAxisInfo {
    fn __repr__(&self) -> String {
        format!(
            "VariationAxisInfo(tag='{}', name={:?}, min={}, default={}, max={}, hidden={})",
            self.tag, self.name, self.min_value, self.default_value, self.max_value, self.hidden
        )
    }
}

/// Load a font and get information about it
#[pyclass]
struct FontInfo {
    #[pyo3(get)]
    units_per_em: u16,
    #[pyo3(get)]
    path: String,
    #[pyo3(get)]
    face_index: u32,
    /// Font ascent in font units (positive, above baseline)
    #[pyo3(get)]
    ascent: i16,
    /// Font descent in font units (negative, below baseline)
    #[pyo3(get)]
    descent: i16,
    /// Line gap (leading) in font units
    #[pyo3(get)]
    line_gap: i16,
    /// Whether this is a variable font
    #[pyo3(get)]
    is_variable: bool,
}

#[pymethods]
impl FontInfo {
    /// Load font information
    ///
    /// Args:
    ///     path: Path to the font file
    ///     face_index: TTC collection face index (default: 0)
    #[new]
    #[pyo3(signature = (path, face_index=0))]
    fn new(path: &str, face_index: u32) -> PyResult<Self> {
        let font_arc = load_font(path, face_index)?;

        // Get metrics from the font
        let (ascent, descent, line_gap) = font_arc
            .metrics()
            .map(|m| (m.ascent, m.descent, m.line_gap))
            .unwrap_or((0, 0, 0));

        let is_variable = font_arc.is_variable();

        Ok(Self {
            units_per_em: font_arc.units_per_em(),
            path: path.to_string(),
            face_index,
            ascent,
            descent,
            line_gap,
            is_variable,
        })
    }

    /// Get glyph ID for a character
    #[allow(clippy::useless_conversion)]
    fn glyph_id(&self, ch: char) -> PyResult<Option<u32>> {
        let font_arc = load_font(&self.path, self.face_index)?;
        Ok(font_arc.glyph_id(ch))
    }

    /// Get variation axes for variable fonts
    ///
    /// Returns None for static fonts.
    /// Returns a list of VariationAxisInfo for variable fonts.
    fn variation_axes(&self) -> PyResult<Option<Vec<VariationAxisInfo>>> {
        let font_arc = load_font(&self.path, self.face_index)?;
        Ok(font_arc.variation_axes().map(|axes| {
            axes.into_iter()
                .map(|axis| VariationAxisInfo {
                    tag: axis.tag,
                    name: axis.name,
                    min_value: axis.min_value,
                    default_value: axis.default_value,
                    max_value: axis.max_value,
                    hidden: axis.hidden,
                })
                .collect()
        }))
    }

    /// Calculate line height in font units
    ///
    /// This is the recommended line-to-line distance: ascent - descent + line_gap
    fn line_height(&self) -> i32 {
        self.ascent as i32 - self.descent as i32 + self.line_gap as i32
    }
}

/// Export rendered output to various formats
///
/// Accepts either raw bytes (data, width, height) or a dict from render_text()
#[pyfunction]
#[allow(clippy::useless_conversion)]
#[pyo3(signature = (image_data, format="ppm"))]
fn export_image(py: Python, image_data: PyObject, format: &str) -> PyResult<PyObject> {
    // Handle dict input from render_text() or tuple of (data, width, height)
    let bitmap = if let Ok(dict) = image_data.downcast_bound::<PyDict>(py) {
        // Extract from dictionary
        let width: u32 = dict
            .get_item("width")?
            .ok_or_else(|| PyValueError::new_err("Missing 'width' in image data"))?
            .extract()?;
        let height: u32 = dict
            .get_item("height")?
            .ok_or_else(|| PyValueError::new_err("Missing 'height' in image data"))?
            .extract()?;
        let data_bytes = dict
            .get_item("data")?
            .ok_or_else(|| PyValueError::new_err("Missing 'data' in image data"))?;
        let data: Vec<u8> = data_bytes.extract()?;

        BitmapData {
            width,
            height,
            format: typf_core::types::BitmapFormat::Rgba8,
            data,
        }
    } else {
        return Err(PyValueError::new_err(
            "image_data must be a dict from render_text()",
        ));
    };

    let output = RenderOutput::Bitmap(bitmap);

    let exporter: Box<dyn Exporter> = match format {
        "ppm" => Box::new(PnmExporter::ppm()),
        "pgm" => Box::new(PnmExporter::pgm()),
        "pbm" => Box::new(PnmExporter::new(typf_export::PnmFormat::Pbm)),
        #[cfg(feature = "export-png")]
        "png" => Box::new(typf_export::PngExporter::new()),
        #[cfg(feature = "export-svg")]
        "svg" => Box::new(typf_export::SvgExporter::new()),
        _ => return Err(PyValueError::new_err(format!("Unknown format: {}", format))),
    };

    let exported = exporter
        .export(&output)
        .map_err(|e| PyRuntimeError::new_err(format!("Export failed: {:?}", e)))?;

    Ok(PyBytes::new_bound(py, &exported).into())
}

/// Quick rendering when you don't care about fonts
///
/// .. deprecated:: 2.5.0
///    This function uses a stub font that produces inaccurate metrics.
///    Use `Typf.render_text()` with a real font file instead.
///
/// Sometimes you just need to see text on screen. This function uses
/// a built-in stub font for ultra-simple rendering without file I/O.
///
/// Args:
///     text: The text to render
///     size: Font size in pixels (default: 16.0)
///     direction: Text direction - "auto", "ltr", "rtl" (default: "auto")
#[pyfunction]
#[allow(clippy::useless_conversion)]
#[pyo3(signature = (text, size=16.0, direction="auto"))]
fn render_simple(py: Python, text: &str, size: f32, direction: &str) -> PyResult<PyObject> {
    // Emit deprecation warning
    let warnings = py.import_bound("warnings")?;
    warnings.call_method1(
        "warn",
        (
            "render_simple() uses a stub font with inaccurate metrics. \
             Use Typf.render_text() with a real font file instead.",
            py.get_type_bound::<pyo3::exceptions::PyDeprecationWarning>(),
        ),
    )?;

    // Create a minimal font that works without a font file
    use typf_core::traits::FontRef;

    struct StubFont {
        size: f32,
    }

    impl FontRef for StubFont {
        fn data(&self) -> &[u8] {
            &[] // No actual font data needed
        }
        fn units_per_em(&self) -> u16 {
            1000 // Standard coordinate system
        }
        fn glyph_id(&self, ch: char) -> Option<u32> {
            if ch.is_alphanumeric() || ch.is_whitespace() {
                Some(ch as u32) // Simple character mapping
            } else {
                Some(0) // Fallback glyph
            }
        }
        fn advance_width(&self, _: u32) -> f32 {
            self.size * 0.5 // Rough width approximation
        }
    }

    let typf = Typf::new("none", "opixa")?;
    let font = Arc::new(StubFont { size }) as Arc<dyn FontRef>;

    // Auto-detect or parse direction
    let resolved_direction = parse_direction(direction, text, None)?;

    let shaping_params = ShapingParams {
        size,
        direction: resolved_direction,
        ..Default::default()
    };

    let shaped = typf
        .shaper
        .shape(text, font.clone(), &shaping_params)
        .map_err(|e| PyRuntimeError::new_err(format!("Shaping failed: {:?}", e)))?;

    let render_params = RenderParams {
        foreground: Color::rgba(0, 0, 0, 255),
        background: None,
        padding: 10,
        ..Default::default()
    };

    let rendered = typf
        .renderer
        .render(&shaped, font, &render_params)
        .map_err(|e| PyRuntimeError::new_err(format!("Rendering failed: {:?}", e)))?;

    match rendered {
        RenderOutput::Bitmap(bitmap) => {
            let result = PyDict::new_bound(py);
            result.set_item("width", bitmap.width)?;
            result.set_item("height", bitmap.height)?;
            result.set_item("format", format!("{:?}", bitmap.format))?;
            result.set_item("data", PyBytes::new_bound(py, &bitmap.data))?;
            Ok(result.into())
        },
        _ => Err(PyValueError::new_err("Unexpected render output format")),
    }
}

/// Enable or disable all caching globally
///
/// Caching is disabled by default. Call this with True to enable caching
/// for improved performance when rendering the same text/font combinations
/// repeatedly.
///
/// Args:
///     enabled: True to enable caching, False to disable
///
/// Example:
///     >>> import typf
///     >>> typf.set_caching_enabled(True)   # Enable caching
///     >>> # ... do some rendering ...
///     >>> typf.set_caching_enabled(False)  # Disable caching again
#[pyfunction]
fn set_caching_enabled(enabled: bool) {
    typf_core::cache_config::set_caching_enabled(enabled);
}

/// Check if caching is globally enabled
///
/// Returns:
///     bool: True if caching is enabled, False otherwise (default is False)
///
/// Example:
///     >>> import typf
///     >>> typf.is_caching_enabled()
///     False
#[pyfunction]
fn is_caching_enabled() -> bool {
    typf_core::cache_config::is_caching_enabled()
}

/// Brings Typf's power into the Python ecosystem
///
/// This is the bridge that makes all our Rust magic available to Python.
/// We expose classes, functions, and version info so Python developers
/// can import typf and start rendering beautiful text immediately.
#[pymodule]
fn typf(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Typf>()?;
    m.add_class::<FontInfo>()?;
    m.add_class::<VariationAxisInfo>()?;
    m.add_class::<PathOp>()?;
    m.add_class::<GlyphPath>()?;
    m.add_class::<PositionedGlyph>()?;
    m.add_class::<ShapedGlyphs>()?;
    #[cfg(feature = "linra")]
    m.add_class::<TypfLinra>()?;
    m.add_function(wrap_pyfunction!(export_image, m)?)?;
    m.add_function(wrap_pyfunction!(render_simple, m)?)?;
    m.add_function(wrap_pyfunction!(set_caching_enabled, m)?)?;
    m.add_function(wrap_pyfunction!(is_caching_enabled, m)?)?;
    m.add("__version__", VERSION)?;
    m.add("__linra_available__", cfg!(feature = "linra"))?;
    m.add("__numpy_available__", cfg!(feature = "numpy-interop"))?;
    Ok(())
}
