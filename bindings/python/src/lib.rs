//! Python bindings for TYPF text rendering pipeline

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
use typf_export::PnmExporter;
use typf_fontdb::Font;

// Note: Skia and Zeno renderers not yet available in workspace

/// Python-facing TYPF pipeline
#[pyclass]
struct Typf {
    shaper: Arc<dyn Shaper + Send + Sync>,
    renderer: Arc<dyn Renderer + Send + Sync>,
}

#[pymethods]
impl Typf {
    /// Create a new TYPF instance
    #[new]
    #[pyo3(signature = (shaper="harfbuzz", renderer="orge"))]
    fn new(shaper: &str, renderer: &str) -> PyResult<Self> {
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

        let renderer: Arc<dyn Renderer + Send + Sync> = match renderer {
            #[cfg(feature = "render-json")]
            "json" => Arc::new(typf_render_json::JsonRenderer::new()),
            "orge" => Arc::new(typf_render_orge::OrgeRenderer::new()),
            #[cfg(feature = "render-cg")]
            "coregraphics" | "cg" | "mac" => Arc::new(typf_render_cg::CoreGraphicsRenderer::new()),
            #[cfg(feature = "render-skia")]
            "skia" => Arc::new(typf_render_skia::SkiaRenderer::new()),
            #[cfg(feature = "render-zeno")]
            "zeno" => Arc::new(typf_render_zeno::ZenoRenderer::new()),
            _ => {
                return Err(PyValueError::new_err(format!(
                    "Unknown renderer: {}. Available: json, orge, coregraphics, skia, zeno",
                    renderer
                )))
            },
        };

        Ok(Self { shaper, renderer })
    }

    /// Render text to an image
    #[allow(clippy::too_many_arguments)]
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

        // Set up render parameters
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

        // Render to bitmap
        let rendered = self
            .renderer
            .render(&shaped, font_arc, &render_params)
            .map_err(|e| PyRuntimeError::new_err(format!("Rendering failed: {:?}", e)))?;

        // Convert to Python object based on output type
        match rendered {
            RenderOutput::Bitmap(bitmap) => {
                let result = PyDict::new_bound(py);
                result.set_item("width", bitmap.width)?;
                result.set_item("height", bitmap.height)?;
                result.set_item("format", format!("{:?}", bitmap.format))?;
                result.set_item("data", PyBytes::new_bound(py, &bitmap.data))?;
                Ok(result.into())
            },
            RenderOutput::Json(json_str) => {
                // Return JSON string directly
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

/// Simple convenience function for rendering text
#[pyfunction]
#[pyo3(signature = (text, size=16.0))]
fn render_simple(py: Python, text: &str, size: f32) -> PyResult<PyObject> {
    // Create stub font for simple rendering
    use typf_core::traits::FontRef;

    struct StubFont {
        size: f32,
    }

    impl FontRef for StubFont {
        fn data(&self) -> &[u8] {
            &[]
        }
        fn units_per_em(&self) -> u16 {
            1000
        }
        fn glyph_id(&self, ch: char) -> Option<u32> {
            if ch.is_alphanumeric() || ch.is_whitespace() {
                Some(ch as u32)
            } else {
                Some(0)
            }
        }
        fn advance_width(&self, _: u32) -> f32 {
            self.size * 0.5
        }
    }

    let typf = Typf::new("none", "orge")?;
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

/// Python module definition
#[pymodule]
fn typf(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Typf>()?;
    m.add_class::<FontInfo>()?;
    m.add_function(wrap_pyfunction!(export_image, m)?)?;
    m.add_function(wrap_pyfunction!(render_simple, m)?)?;
    m.add("__version__", "2.0.0-dev")?;
    Ok(())
}
