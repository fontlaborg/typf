//! Python bindings - TYPF's power, packaged for Python developers
//!
//! This is where Rust meets Python: high-performance text shaping and rendering
//! that feels native in Python code. We handle the complexity of font loading,
//! Unicode processing, and pixel-perfect rendering—all you do is call a function.

#![allow(clippy::useless_conversion)]

use pyo3::exceptions::{PyIOError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};
use std::sync::Arc;
use typf_core::{
    traits::{Exporter, Renderer, Shaper},
    types::{BitmapData, Direction, RenderOutput},
    Color, RenderParams, ShapingParams,
};
#[cfg(feature = "linra")]
use typf_core::linra::{LinraRenderParams, LinraRenderer};
use typf_export::PnmExporter;
use typf_fontdb::Font;

// Note: Skia and Zeno renderers not yet available in workspace

/// The main TYPF interface that Python developers will love
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
    /// Creates your TYPF rendering pipeline
    ///
    /// Choose your weapons:
    /// - Shapers: "none" (debug), "harfbuzz" (professional), "coretext" (macOS), "icu-hb" (Unicode perfect)
    /// - Renderers: "opixa" (pure Rust), "json" (data), "coregraphics" (macOS), "skia" (pro), "zeno" (pure Rust)
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
            _ => {
                return Err(PyValueError::new_err(format!(
                    "Unknown renderer: {}. Available: json, opixa, coregraphics, skia, zeno",
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
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::useless_conversion)]
    #[pyo3(signature = (text, font_path, size=16.0, color=None, background=None, padding=10))]
    fn render_text(
        &self,
        py: Python,
        text: &str,
        font_path: &str,
        size: f32,
        color: Option<(u8, u8, u8, u8)>,
        background: Option<(u8, u8, u8, u8)>,
        padding: u32,
    ) -> PyResult<PyObject> {
        // First, load and validate the font file
        let font = Font::from_file(font_path)
            .map_err(|e| PyIOError::new_err(format!("Failed to load font: {:?}", e)))?;
        let font_arc = Arc::new(font) as Arc<dyn typf_core::traits::FontRef>;

        // Configure how we want to shape the text
        let shaping_params = ShapingParams {
            size,
            direction: Direction::LeftToRight,
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
            RenderOutput::Vector(_) => {
                Err(PyValueError::new_err("Vector output not yet supported in Python bindings"))
            },
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
    #[allow(clippy::useless_conversion)]
    #[pyo3(signature = (text, font_path, size=16.0))]
    fn shape_text(&self, py: Python, text: &str, font_path: &str, size: f32) -> PyResult<PyObject> {
        // Load font
        let font = Font::from_file(font_path)
            .map_err(|e| PyIOError::new_err(format!("Failed to load font: {:?}", e)))?;
        let font_arc = Arc::new(font) as Arc<dyn typf_core::traits::FontRef>;

        // Set up shaping parameters
        let shaping_params = ShapingParams {
            size,
            direction: Direction::LeftToRight,
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
    #[cfg(feature = "export-svg")]
    #[allow(clippy::useless_conversion)]
    #[pyo3(signature = (text, font_path, size=16.0, color=None, padding=10))]
    fn render_to_svg(
        &self,
        text: &str,
        font_path: &str,
        size: f32,
        color: Option<(u8, u8, u8, u8)>,
        padding: u32,
    ) -> PyResult<String> {
        // Load font
        let font = Font::from_file(font_path)
            .map_err(|e| PyIOError::new_err(format!("Failed to load font: {:?}", e)))?;
        let font_arc = Arc::new(font) as Arc<dyn typf_core::traits::FontRef>;

        // Set up shaping parameters
        let shaping_params = ShapingParams {
            size,
            direction: Direction::LeftToRight,
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
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (text, font_path, size=16.0, color=None, background=None, padding=10, features=None, language=None, script=None))]
    fn render_text(
        &self,
        py: Python,
        text: &str,
        font_path: &str,
        size: f32,
        color: Option<(u8, u8, u8, u8)>,
        background: Option<(u8, u8, u8, u8)>,
        padding: u32,
        features: Option<Vec<(String, u32)>>,
        language: Option<String>,
        script: Option<String>,
    ) -> PyResult<PyObject> {
        // Load font
        let font = Font::from_file(font_path)
            .map_err(|e| PyIOError::new_err(format!("Failed to load font: {:?}", e)))?;
        let font_arc = Arc::new(font) as Arc<dyn typf_core::traits::FontRef>;

        // Parse colors
        let foreground = color
            .map(|(r, g, b, a)| Color::rgba(r, g, b, a))
            .unwrap_or(Color::rgba(0, 0, 0, 255));
        let background = background.map(|(r, g, b, a)| Color::rgba(r, g, b, a));

        // Create linra parameters
        let params = LinraRenderParams {
            size,
            direction: Direction::LeftToRight,
            foreground,
            background,
            padding,
            variations: Vec::new(),
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
            }
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
}

#[pymethods]
impl FontInfo {
    /// Load font information
    #[new]
    fn new(path: &str) -> PyResult<Self> {
        let font = Font::from_file(path)
            .map_err(|e| PyIOError::new_err(format!("Failed to load font: {:?}", e)))?;

        let font_ref = Arc::new(font) as Arc<dyn typf_core::traits::FontRef>;
        Ok(Self {
            units_per_em: font_ref.units_per_em(),
            path: path.to_string(),
        })
    }

    /// Get glyph ID for a character
    #[allow(clippy::useless_conversion)]
    fn glyph_id(&self, ch: char) -> PyResult<Option<u32>> {
        let font = Font::from_file(&self.path)
            .map_err(|e| PyIOError::new_err(format!("Failed to load font: {:?}", e)))?;
        let font_ref = Arc::new(font) as Arc<dyn typf_core::traits::FontRef>;
        Ok(font_ref.glyph_id(ch))
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
        return Err(PyValueError::new_err("image_data must be a dict from render_text()"));
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
/// Sometimes you just need to see text on screen. This function uses
/// a built-in stub font for ultra-simple rendering without file I/O.
#[pyfunction]
#[allow(clippy::useless_conversion)]
#[pyo3(signature = (text, size=16.0))]
fn render_simple(py: Python, text: &str, size: f32) -> PyResult<PyObject> {
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

    let shaping_params = ShapingParams {
        size,
        direction: Direction::LeftToRight,
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

/// Brings TYPF's power into the Python ecosystem
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
    m.add("__version__", "2.0.0-dev")?;
    m.add("__linra_available__", cfg!(feature = "linra"))?;
    Ok(())
}
