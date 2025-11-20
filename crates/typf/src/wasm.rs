//! TYPF meets the web: Text rendering in your browser
//!
//! JavaScript calls, Rust speed. Perfect for canvas rendering,
//! text measurement, and dynamic typography on the web.

use crate::prelude::*;
use wasm_bindgen::prelude::*;

/// Better panic messages make better debugging experiences
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Your browser's new best friend for text rendering
#[wasm_bindgen]
pub struct WasmRenderer {
    #[cfg(feature = "shaping-none")]
    shaper: Box<dyn Shaper>,
    #[cfg(feature = "render-orge")]
    renderer: Box<dyn Renderer>,
}

#[wasm_bindgen]
impl WasmRenderer {
    /// Ready to render with the minimal, web-friendly backends
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmRenderer, JsValue> {
        #[cfg(not(all(feature = "shaping-none", feature = "render-orge")))]
        return Err(JsValue::from_str("WASM needs shaping-none and render-orge features"));

        #[cfg(all(feature = "shaping-none", feature = "render-orge"))]
        {
            use crate::render_orge::OrgeRenderer;
            use crate::shape_none::NoneShaper;

            Ok(WasmRenderer {
                shaper: Box::new(NoneShaper::new()),
                renderer: Box::new(OrgeRenderer::new()),
            })
        }
    }

    /// Turn text into pixels, right in your browser
    ///
    /// Returns RGBA bytes ready for canvas or image data
    #[wasm_bindgen]
    pub fn render_text(
        &self,
        text: &str,
        font_size: f32,
        _width: Option<u32>,
        _height: Option<u32>,
    ) -> Result<Vec<u8>, JsValue> {
        #[cfg(not(all(feature = "shaping-none", feature = "render-orge")))]
        return Err(JsValue::from_str("Rendering not available in this build"));

        #[cfg(all(feature = "shaping-none", feature = "render-orge"))]
        {
            // TODO: Replace with real font loading
            struct MockFont {
                font_size: f32,
            }
            impl FontRef for MockFont {
                fn data(&self) -> &[u8] {
                    &[]
                }
                fn units_per_em(&self) -> u16 {
                    1000
                }
                fn glyph_id(&self, ch: char) -> Option<u32> {
                    Some(ch as u32)
                }
                fn advance_width(&self, _glyph_id: u32) -> f32 {
                    self.font_size * 0.6
                }
            }

            let font = std::sync::Arc::new(MockFont { font_size });

            // Shape those characters
            let shaping_params = ShapingParams {
                size: font_size,
                ..Default::default()
            };

            let shaped = self
                .shaper
                .shape(text, font.clone(), &shaping_params)
                .map_err(|e| JsValue::from_str(&format!("Shaping error: {}", e)))?;

            // Render to bitmap
            let render_params = RenderParams {
                foreground: Color::rgba(0, 0, 0, 255),
                background: Some(Color::rgba(255, 255, 255, 255)),
                ..Default::default()
            };

            let output = self
                .renderer
                .render(&shaped, font, &render_params)
                .map_err(|e| JsValue::from_str(&format!("Render error: {}", e)))?;

            match output {
                RenderOutput::Bitmap(bitmap) => Ok(bitmap.data),
                _ => Err(JsValue::from_str("Unexpected output format")),
            }
        }
    }

    /// Which version of TYPF are you running?
    #[wasm_bindgen]
    pub fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}

/// Quick text width measurement (approximation for now)
#[wasm_bindgen]
pub fn measure_text(text: &str, font_size: f32) -> f32 {
    // TODO: Use proper text measurement
    text.len() as f32 * font_size * 0.6
}
