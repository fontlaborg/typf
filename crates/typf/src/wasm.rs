//! WebAssembly bindings for TYPF
//!
//! Provides JavaScript-friendly API for using TYPF in web browsers.

#![cfg(feature = "wasm")]

use crate::prelude::*;
use wasm_bindgen::prelude::*;

/// Initialize panic hook for better error messages in WASM
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// WASM-friendly text renderer
#[wasm_bindgen]
pub struct WasmRenderer {
    #[cfg(feature = "shaping-none")]
    shaper: Box<dyn Shaper>,
    #[cfg(feature = "render-orge")]
    renderer: Box<dyn Renderer>,
}

#[wasm_bindgen]
impl WasmRenderer {
    /// Create a new WASM renderer with default backends
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmRenderer, JsValue> {
        #[cfg(not(all(feature = "shaping-none", feature = "render-orge")))]
        return Err(JsValue::from_str("WASM build requires shaping-none and render-orge features"));

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

    /// Render text to RGBA bitmap
    ///
    /// Returns a Uint8Array containing RGBA pixel data
    #[wasm_bindgen]
    pub fn render_text(
        &self,
        text: &str,
        font_size: f32,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Result<Vec<u8>, JsValue> {
        #[cfg(not(all(feature = "shaping-none", feature = "render-orge")))]
        return Err(JsValue::from_str("Rendering not available in this build"));

        #[cfg(all(feature = "shaping-none", feature = "render-orge"))]
        {
            // Create mock font for now
            struct MockFont;
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
                    font_size * 0.6
                }
            }

            let font = std::sync::Arc::new(MockFont);

            // Shape the text
            let shaping_params = ShapingParams {
                font_size,
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

    /// Get version string
    #[wasm_bindgen]
    pub fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}

/// Simple text measurement function
#[wasm_bindgen]
pub fn measure_text(text: &str, font_size: f32) -> f32 {
    // Simple approximation for now
    text.len() as f32 * font_size * 0.6
}
