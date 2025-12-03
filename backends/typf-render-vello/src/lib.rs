//! Vello GPU renderer backend for typf.
//!
//! This crate provides GPU-accelerated text rendering using the Vello hybrid renderer.
//! It uses wgpu for GPU access and provides high-performance rendering by leveraging
//! GPU compute for path rasterization.
//!
//! # Example
//!
//! ```ignore
//! use typf_render_vello::VelloRenderer;
//! use typf_core::traits::Renderer;
//!
//! let renderer = VelloRenderer::new()?;
//! let output = renderer.render(&shaped_result, font, &params)?;
//! ```

use std::sync::Arc;

use thiserror::Error;
use typf_core::{
    error::{RenderError, Result},
    traits::{FontRef, Renderer},
    types::{BitmapData, BitmapFormat, RenderOutput, ShapingResult},
    Color, RenderParams,
};
use vello_common::{
    glyph::Glyph,
    kurbo::Affine,
    peniko::{color::AlphaColor, FontData},
};
use vello_hybrid::{RenderSize, RenderTargetConfig, Scene};
use wgpu::{Device, Queue};

/// Errors specific to the Vello GPU renderer.
#[derive(Error, Debug)]
pub enum VelloError {
    /// Failed to initialize wgpu adapter.
    #[error("Failed to find a suitable GPU adapter")]
    NoAdapter,
    /// Failed to create wgpu device.
    #[error("Failed to create GPU device: {0}")]
    DeviceCreation(String),
    /// Rendering failed.
    #[error("Rendering failed: {0}")]
    RenderFailed(String),
}

impl From<VelloError> for RenderError {
    fn from(e: VelloError) -> Self {
        RenderError::BackendError(e.to_string())
    }
}

impl From<VelloError> for typf_core::TypfError {
    fn from(e: VelloError) -> Self {
        typf_core::TypfError::RenderingFailed(RenderError::BackendError(e.to_string()))
    }
}

/// Configuration for the Vello GPU renderer.
#[derive(Debug, Clone)]
pub struct VelloConfig {
    /// Whether to use CPU shaders as fallback (slower but more compatible).
    pub use_cpu_fallback: bool,
    /// Power preference for GPU adapter selection.
    pub power_preference: wgpu::PowerPreference,
}

impl Default for VelloConfig {
    fn default() -> Self {
        Self {
            use_cpu_fallback: false,
            power_preference: wgpu::PowerPreference::HighPerformance,
        }
    }
}

/// GPU context holding wgpu device and queue.
struct GpuContext {
    device: Device,
    queue: Queue,
}

impl GpuContext {
    fn new(config: &VelloConfig) -> Result<Self, VelloError> {
        pollster::block_on(Self::new_async(config))
    }

    async fn new_async(config: &VelloConfig) -> Result<Self, VelloError> {
        let instance = wgpu::Instance::default();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: config.power_preference,
                force_fallback_adapter: config.use_cpu_fallback,
                compatible_surface: None,
            })
            .await
            .map_err(|_| VelloError::NoAdapter)?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("typf-vello"),
                required_features: wgpu::Features::empty(),
                ..Default::default()
            })
            .await
            .map_err(|e| VelloError::DeviceCreation(e.to_string()))?;

        Ok(Self { device, queue })
    }
}

/// Vello GPU renderer for high-performance text rendering.
///
/// This renderer uses the Vello hybrid CPU/GPU rendering engine with wgpu
/// for GPU acceleration. It provides excellent performance for rendering
/// large amounts of text or text at large sizes.
pub struct VelloRenderer {
    config: VelloConfig,
    gpu: GpuContext,
}

impl std::fmt::Debug for VelloRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VelloRenderer")
            .field("config", &self.config)
            .field("gpu", &"<GpuContext>")
            .finish()
    }
}

impl VelloRenderer {
    /// Creates a new Vello GPU renderer with default configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if GPU initialization fails (no suitable adapter or device creation fails).
    pub fn new() -> Result<Self, VelloError> {
        Self::with_config(VelloConfig::default())
    }

    /// Creates a new Vello GPU renderer with custom configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if GPU initialization fails.
    pub fn with_config(config: VelloConfig) -> Result<Self, VelloError> {
        let gpu = GpuContext::new(&config)?;
        Ok(Self { config, gpu })
    }

    /// Convert typf color to Vello AlphaColor.
    fn to_vello_color(color: Color) -> AlphaColor<vello_common::peniko::color::Srgb> {
        AlphaColor::from_rgba8(color.r, color.g, color.b, color.a)
    }

    /// Convert typf ShapingResult glyphs to Vello Glyph format.
    fn convert_glyphs(shaped: &ShapingResult) -> Vec<Glyph> {
        shaped
            .glyphs
            .iter()
            .map(|g| Glyph {
                id: g.id,
                x: g.x,
                y: g.y,
            })
            .collect()
    }

    /// Render to a GPU texture and read back to CPU.
    fn render_to_bitmap(
        &self,
        scene: &Scene,
        width: u32,
        height: u32,
    ) -> Result<Vec<u8>, VelloError> {
        let device = &self.gpu.device;
        let queue = &self.gpu.queue;

        // Create render target texture
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("typf-vello-target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create Vello renderer
        let mut renderer = vello_hybrid::Renderer::new(
            device,
            &RenderTargetConfig {
                format: wgpu::TextureFormat::Rgba8Unorm,
                width,
                height,
            },
        );

        let render_size = RenderSize { width, height };

        // Create command encoder and render
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("typf-vello-render"),
        });

        renderer
            .render(scene, device, queue, &mut encoder, &render_size, &texture_view)
            .map_err(|e| VelloError::RenderFailed(format!("{:?}", e)))?;

        // Create buffer for readback (with row padding to 256 bytes)
        let bytes_per_row = (width * 4).next_multiple_of(256);
        let buffer_size = (bytes_per_row as u64) * (height as u64);
        let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("typf-vello-readback"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // Copy texture to buffer
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: None,
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        // Submit and wait
        queue.submit([encoder.finish()]);

        // Map buffer for reading
        let buffer_slice = readback_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, |_| {});
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("GPU poll failed");

        // Read data and remove row padding
        let mapped = buffer_slice.get_mapped_range();
        let mut result = Vec::with_capacity((width * height * 4) as usize);
        for row in mapped.chunks_exact(bytes_per_row as usize) {
            result.extend_from_slice(&row[0..(width * 4) as usize]);
        }
        drop(mapped);
        readback_buffer.unmap();

        Ok(result)
    }
}

impl Renderer for VelloRenderer {
    fn name(&self) -> &'static str {
        "vello"
    }

    fn render(
        &self,
        shaped: &ShapingResult,
        font: Arc<dyn FontRef>,
        params: &RenderParams,
    ) -> Result<RenderOutput> {
        log::debug!(
            "VelloRenderer: Rendering {} glyphs",
            shaped.glyphs.len()
        );

        let padding = params.padding as f32;
        let font_size = shaped.advance_height;

        // Calculate canvas dimensions
        let width = (shaped.advance_width + padding * 2.0).ceil() as u32;
        let height = (font_size * 1.5 + padding * 2.0).ceil() as u32;

        // Sanity check dimensions
        if width == 0 || height == 0 {
            return Err(RenderError::ZeroDimensions { width, height }.into());
        }

        // vello_hybrid uses u16 for dimensions internally
        let width_u16 = width.min(u16::MAX as u32) as u16;
        let height_u16 = height.min(u16::MAX as u32) as u16;

        // Create scene
        let mut scene = Scene::new(width_u16, height_u16);

        // Set background color if specified
        if let Some(bg) = params.background {
            scene.set_paint(Self::to_vello_color(bg));
            scene.fill_rect(&vello_common::kurbo::Rect::new(
                0.0,
                0.0,
                width as f64,
                height as f64,
            ));
        }

        // Set foreground color
        scene.set_paint(Self::to_vello_color(params.foreground));

        // Create font data for Vello
        let font_bytes = font.data().to_vec();
        let font_data = FontData::new(font_bytes.into(), 0);

        // Convert glyphs
        let glyphs = Self::convert_glyphs(shaped);

        if !glyphs.is_empty() {
            // Calculate baseline position
            let baseline_y = padding + font_size * 0.8; // Approximate baseline

            // Set transform for positioning
            scene.set_transform(Affine::translate((padding as f64, baseline_y as f64)));

            // Render glyphs using glyph_run builder
            scene
                .glyph_run(&font_data)
                .font_size(font_size)
                .fill_glyphs(glyphs.into_iter());
        }

        // Render scene to bitmap
        let rgba_data = self.render_to_bitmap(&scene, width, height)?;

        Ok(RenderOutput::Bitmap(BitmapData {
            width,
            height,
            format: BitmapFormat::Rgba8,
            data: rgba_data,
        }))
    }

    fn supports_format(&self, format: &str) -> bool {
        matches!(format, "bitmap" | "rgba" | "rgb")
    }

    fn clear_cache(&self) {
        // GPU resources are recreated per render, no persistent cache
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = VelloConfig::default();
        assert!(!config.use_cpu_fallback);
        assert!(matches!(
            config.power_preference,
            wgpu::PowerPreference::HighPerformance
        ));
    }

    #[test]
    fn test_color_conversion() {
        let color = Color::rgba(255, 128, 64, 200);
        let vello_color = VelloRenderer::to_vello_color(color);
        // AlphaColor stores components as f32 in [0, 1] range
        assert!((vello_color.components[0] - 1.0).abs() < 0.01); // R
        assert!((vello_color.components[1] - 0.5).abs() < 0.05); // G
        assert!((vello_color.components[2] - 0.25).abs() < 0.05); // B
        assert!((vello_color.components[3] - 0.78).abs() < 0.05); // A
    }

    #[test]
    fn test_glyph_conversion() {
        use typf_core::types::{Direction, PositionedGlyph};

        let shaped = ShapingResult {
            glyphs: vec![
                PositionedGlyph {
                    id: 65,
                    cluster: 0,
                    advance: 10.0,
                    x: 0.0,
                    y: 0.0,
                },
                PositionedGlyph {
                    id: 66,
                    cluster: 1,
                    advance: 12.0,
                    x: 1.0,
                    y: 2.0,
                },
            ],
            advance_width: 22.0,
            advance_height: 16.0,
            direction: Direction::LeftToRight,
        };

        let glyphs = VelloRenderer::convert_glyphs(&shaped);
        assert_eq!(glyphs.len(), 2);
        assert_eq!(glyphs[0].id, 65);
        assert_eq!(glyphs[0].x, 0.0);
        assert_eq!(glyphs[0].y, 0.0);
        assert_eq!(glyphs[1].id, 66);
        assert_eq!(glyphs[1].x, 1.0);
        assert_eq!(glyphs[1].y, 2.0);
    }
}
