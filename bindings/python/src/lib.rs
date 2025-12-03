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
                    if matches!(run.direction, Direction::RightToLeft | Direction::BottomToTop) {
                        Some(run.direction)
                    } else {
                        None
                    }
                })
                .unwrap_or(Direction::LeftToRight)
        }
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

/// Load a font and get information about it
#[pyclass]
struct FontInfo {
    #[pyo3(get)]
    units_per_em: u16,
    #[pyo3(get)]
    path: String,
    #[pyo3(get)]
    face_index: u32,
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
        Ok(Self {
            units_per_em: font_arc.units_per_em(),
            path: path.to_string(),
            face_index,
        })
    }

    /// Get glyph ID for a character
    #[allow(clippy::useless_conversion)]
    fn glyph_id(&self, ch: char) -> PyResult<Option<u32>> {
        let font_arc = load_font(&self.path, self.face_index)?;
        Ok(font_arc.glyph_id(ch))
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

/// Brings Typf's power into the Python ecosystem
///
/// This is the bridge that makes all our Rust magic available to Python.
/// We expose classes, functions, and version info so Python developers
/// can import typf and start rendering beautiful text immediately.
#[pymodule]
fn typf(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Typf>()?;
    m.add_class::<FontInfo>()?;
    #[cfg(feature = "linra")]
    m.add_class::<TypfLinra>()?;
    m.add_function(wrap_pyfunction!(export_image, m)?)?;
    m.add_function(wrap_pyfunction!(render_simple, m)?)?;
    m.add("__version__", VERSION)?;
    m.add("__linra_available__", cfg!(feature = "linra"))?;
    Ok(())
}
