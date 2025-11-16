// this_file: backends/typf-core/src/surface.rs

//! Shared helpers for converting backend-specific buffers into [`RenderOutput`]s.

use crate::{
    types::{Bitmap, RenderFormat, RenderOutput},
    TypfError, Result,
};

/// Raw pixel format for a render surface.
#[derive(Debug, Clone, Copy)]
pub enum SurfaceFormat {
    /// RGBA ordering.
    Rgba,
    /// BGRA ordering.
    Bgra,
    /// Grayscale alpha-less mask.
    Gray,
}

/// Render surface produced by a backend prior to format conversion/encoding.
#[derive(Debug)]
pub struct RenderSurface {
    width: u32,
    height: u32,
    format: SurfaceFormat,
    premultiplied: bool,
    data: Vec<u8>,
}

impl RenderSurface {
    /// Create a new RGBA surface.
    pub fn from_rgba(width: u32, height: u32, data: Vec<u8>, premultiplied: bool) -> Self {
        Self {
            width,
            height,
            format: SurfaceFormat::Rgba,
            premultiplied,
            data,
        }
    }

    /// Create a new BGRA surface.
    pub fn from_bgra(width: u32, height: u32, data: Vec<u8>, premultiplied: bool) -> Self {
        Self {
            width,
            height,
            format: SurfaceFormat::Bgra,
            premultiplied,
            data,
        }
    }

    /// Create a grayscale surface (used for alpha-only glyph caches).
    pub fn from_gray(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            width,
            height,
            format: SurfaceFormat::Gray,
            premultiplied: false,
            data,
        }
    }

    /// Convert the surface into a [`RenderOutput`].
    pub fn into_render_output(self, format: RenderFormat) -> Result<RenderOutput> {
        let width = self.width;
        let height = self.height;
        match format {
            RenderFormat::Svg => Err(TypfError::render(
                "RenderSurface cannot be converted to SVG output",
            )),
            RenderFormat::Raw => {
                let rgba = self.into_rgba_data()?;
                Ok(RenderOutput::Bitmap(Bitmap {
                    width,
                    height,
                    data: rgba,
                }))
            }
            RenderFormat::Png => {
                let rgba = self.into_rgba_data()?;
                let png_data = encode_png(width, height, &rgba)?;
                Ok(RenderOutput::Png(png_data))
            }
        }
    }

    fn into_rgba_data(mut self) -> Result<Vec<u8>> {
        match self.format {
            SurfaceFormat::Gray => Ok(expand_gray(&self.data)),
            SurfaceFormat::Rgba => {
                if self.premultiplied {
                    unpremultiply(&mut self.data);
                }
                Ok(std::mem::take(&mut self.data))
            }
            SurfaceFormat::Bgra => {
                bgra_to_rgba(&mut self.data);
                if self.premultiplied {
                    unpremultiply(&mut self.data);
                }
                Ok(std::mem::take(&mut self.data))
            }
        }
    }
}

/// Expand grayscale to RGBA (optimized for auto-vectorization).
fn expand_gray(data: &[u8]) -> Vec<u8> {
    let len = data.len();
    let mut rgba = vec![0u8; len * 4];

    // Process 4 pixels at a time for better cache locality and vectorization
    let chunks = len / 4;
    let remainder = len % 4;

    for i in 0..chunks {
        let src_idx = i * 4;
        let dst_idx = i * 16;

        // Unrolled loop - compiler can vectorize this
        rgba[dst_idx] = data[src_idx];
        rgba[dst_idx + 1] = data[src_idx];
        rgba[dst_idx + 2] = data[src_idx];
        rgba[dst_idx + 3] = 255;

        rgba[dst_idx + 4] = data[src_idx + 1];
        rgba[dst_idx + 5] = data[src_idx + 1];
        rgba[dst_idx + 6] = data[src_idx + 1];
        rgba[dst_idx + 7] = 255;

        rgba[dst_idx + 8] = data[src_idx + 2];
        rgba[dst_idx + 9] = data[src_idx + 2];
        rgba[dst_idx + 10] = data[src_idx + 2];
        rgba[dst_idx + 11] = 255;

        rgba[dst_idx + 12] = data[src_idx + 3];
        rgba[dst_idx + 13] = data[src_idx + 3];
        rgba[dst_idx + 14] = data[src_idx + 3];
        rgba[dst_idx + 15] = 255;
    }

    // Handle remainder
    for i in 0..remainder {
        let src_idx = chunks * 4 + i;
        let dst_idx = chunks * 16 + i * 4;
        rgba[dst_idx] = data[src_idx];
        rgba[dst_idx + 1] = data[src_idx];
        rgba[dst_idx + 2] = data[src_idx];
        rgba[dst_idx + 3] = 255;
    }

    rgba
}

/// Swap R and B channels (BGRA ↔ RGBA). Optimized for auto-vectorization.
#[inline]
fn bgra_to_rgba(data: &mut [u8]) {
    // Process in chunks for better vectorization
    for chunk in data.chunks_exact_mut(4) {
        // Single swap is efficient and vectorizes well
        chunk.swap(0, 2);
    }
}

/// Unpremultiply alpha (optimized with integer math and lookup table).
///
/// Uses fixed-point arithmetic to avoid expensive float divisions.
/// For alpha in [1..254], we compute: channel_out = (channel_in * 255) / alpha
fn unpremultiply(data: &mut [u8]) {
    // Build a lookup table for 255/alpha to avoid division in tight loop
    // unpremul_table[a] = ceil(255.0 * 256.0 / a) for a in 1..=254
    // We use 256x scaling factor for fixed-point math
    static UNPREMUL_TABLE: [u16; 256] = {
        let mut table = [0u16; 256];
        let mut i = 1;
        while i < 255 {
            // (255 * 256) / i, rounded up
            table[i] = ((255 * 256 + i - 1) / i) as u16;
            i += 1;
        }
        table[0] = 0;     // Avoid division by zero
        table[255] = 256; // alpha=255 means already unpremultiplied
        table
    };

    for chunk in data.chunks_exact_mut(4) {
        let alpha = chunk[3] as usize;
        if alpha == 0 || alpha == 255 {
            continue;
        }

        let factor = UNPREMUL_TABLE[alpha];

        // Fixed-point multiply: (channel * factor) >> 8
        // This is much faster than floating-point division
        chunk[0] = ((chunk[0] as u16 * factor) >> 8).min(255) as u8;
        chunk[1] = ((chunk[1] as u16 * factor) >> 8).min(255) as u8;
        chunk[2] = ((chunk[2] as u16 * factor) >> 8).min(255) as u8;
    }
}

fn encode_png(width: u32, height: u32, data: &[u8]) -> Result<Vec<u8>> {
    let mut png_data = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png_data, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(|err| TypfError::render(format!("PNG encoder error: {err}")))?;
        writer
            .write_image_data(data)
            .map_err(|err| TypfError::render(format!("PNG write error: {err}")))?;
    } // writer and encoder are dropped here
    Ok(png_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{RenderFormat, RenderOutput};

    fn bitmap_data(output: RenderOutput) -> Vec<u8> {
        match output {
            RenderOutput::Bitmap(bitmap) => bitmap.data,
            other => panic!("expected bitmap output, got {other:?}"),
        }
    }

    #[test]
    fn bgra_surface_converts_and_unpremultiplies() {
        let surface = RenderSurface::from_bgra(1, 1, vec![16, 32, 64, 128], true);
        let data = bitmap_data(surface.into_render_output(RenderFormat::Raw).unwrap());
        assert_eq!(data, vec![127, 63, 31, 128]);
    }

    #[test]
    fn gray_surface_expands_to_rgba() {
        let surface = RenderSurface::from_gray(3, 1, vec![0, 128, 255]);
        let data = bitmap_data(surface.into_render_output(RenderFormat::Raw).unwrap());
        assert_eq!(
            data,
            vec![0, 0, 0, 255, 128, 128, 128, 255, 255, 255, 255, 255,]
        );
    }

    #[test]
    fn rgba_surface_respects_premultiplication_flag() {
        let surface = RenderSurface::from_rgba(1, 1, vec![10, 20, 30, 40], false);
        let data = bitmap_data(surface.into_render_output(RenderFormat::Raw).unwrap());
        assert_eq!(data, vec![10, 20, 30, 40]);
    }

    #[test]
    fn png_encoding_round_trips_pixels() {
        let surface = RenderSurface::from_rgba(1, 1, vec![5, 6, 7, 8], false);
        let output = surface.into_render_output(RenderFormat::Png).unwrap();
        let png_bytes = match output {
            RenderOutput::Png(bytes) => bytes,
            other => panic!("expected png output, got {other:?}"),
        };

        let decoder = png::Decoder::new(png_bytes.as_slice());
        let mut reader = decoder.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).unwrap();
        assert_eq!(info.width, 1);
        assert_eq!(info.height, 1);
        assert_eq!(&buf[..4], &[5, 6, 7, 8]);
    }

    #[test]
    fn svg_conversion_returns_error() {
        let surface = RenderSurface::from_rgba(1, 1, vec![0, 0, 0, 0], false);
        let err = surface
            .into_render_output(RenderFormat::Svg)
            .expect_err("SVG conversion should fail for bitmap surfaces");
        assert!(err
            .to_string()
            .contains("RenderSurface cannot be converted to SVG output"));
    }
}
