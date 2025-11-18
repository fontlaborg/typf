//! Export module for TYPF
//!
//! This module provides exporters for various output formats.

use std::io::Write;
use typf_core::{
    error::{ExportError, Result},
    traits::Exporter,
    types::{BitmapData, BitmapFormat, RenderOutput},
};

pub mod json;
pub mod png;
pub mod svg;

pub use json::JsonExporter;
pub use png::PngExporter;
pub use svg::SvgExporter;

/// PNM (Portable Any Map) exporter for minimal bitmap output
pub struct PnmExporter {
    /// Which PNM format to use
    format: PnmFormat,
}

#[derive(Debug, Clone, Copy)]
pub enum PnmFormat {
    /// PBM - Portable Bitmap (black and white)
    Pbm,
    /// PGM - Portable Graymap
    Pgm,
    /// PPM - Portable Pixmap (color)
    Ppm,
}

impl PnmExporter {
    /// Create a new PNM exporter
    pub fn new(format: PnmFormat) -> Self {
        Self { format }
    }

    /// Create a PPM (color) exporter
    pub fn ppm() -> Self {
        Self::new(PnmFormat::Ppm)
    }

    /// Create a PGM (grayscale) exporter
    pub fn pgm() -> Self {
        Self::new(PnmFormat::Pgm)
    }

    fn export_bitmap(&self, bitmap: &BitmapData) -> Result<Vec<u8>> {
        let mut output = Vec::new();

        match self.format {
            PnmFormat::Ppm => {
                // PPM header
                writeln!(&mut output, "P3")?; // ASCII format
                writeln!(&mut output, "{} {}", bitmap.width, bitmap.height)?;
                writeln!(&mut output, "255")?; // Max color value

                // Convert bitmap data to PPM format
                match bitmap.format {
                    BitmapFormat::Rgba8 => {
                        // Write RGB values, ignoring alpha
                        for y in 0..bitmap.height {
                            for x in 0..bitmap.width {
                                let idx = ((y * bitmap.width + x) * 4) as usize;
                                write!(
                                    &mut output,
                                    "{} {} {} ",
                                    bitmap.data[idx],     // R
                                    bitmap.data[idx + 1], // G
                                    bitmap.data[idx + 2]  // B
                                )?;
                            }
                            writeln!(&mut output)?;
                        }
                    },
                    BitmapFormat::Rgb8 => {
                        // Direct RGB copy
                        for y in 0..bitmap.height {
                            for x in 0..bitmap.width {
                                let idx = ((y * bitmap.width + x) * 3) as usize;
                                write!(
                                    &mut output,
                                    "{} {} {} ",
                                    bitmap.data[idx],
                                    bitmap.data[idx + 1],
                                    bitmap.data[idx + 2]
                                )?;
                            }
                            writeln!(&mut output)?;
                        }
                    },
                    BitmapFormat::Gray8 => {
                        // Convert grayscale to RGB
                        for y in 0..bitmap.height {
                            for x in 0..bitmap.width {
                                let idx = (y * bitmap.width + x) as usize;
                                let gray = bitmap.data[idx];
                                write!(&mut output, "{} {} {} ", gray, gray, gray)?;
                            }
                            writeln!(&mut output)?;
                        }
                    },
                    BitmapFormat::Gray1 => {
                        // Convert 1-bit to RGB
                        for y in 0..bitmap.height {
                            for x in 0..bitmap.width {
                                let byte_idx = ((y * bitmap.width + x) / 8) as usize;
                                let bit_idx = ((y * bitmap.width + x) % 8) as usize;
                                let bit = (bitmap.data[byte_idx] >> (7 - bit_idx)) & 1;
                                let value = if bit == 1 { 255 } else { 0 };
                                write!(&mut output, "{} {} {} ", value, value, value)?;
                            }
                            writeln!(&mut output)?;
                        }
                    },
                }
            },
            PnmFormat::Pgm => {
                // PGM header
                writeln!(&mut output, "P2")?; // ASCII format
                writeln!(&mut output, "{} {}", bitmap.width, bitmap.height)?;
                writeln!(&mut output, "255")?; // Max gray value

                // Convert to grayscale
                match bitmap.format {
                    BitmapFormat::Gray8 => {
                        for y in 0..bitmap.height {
                            for x in 0..bitmap.width {
                                let idx = (y * bitmap.width + x) as usize;
                                write!(&mut output, "{} ", bitmap.data[idx])?;
                            }
                            writeln!(&mut output)?;
                        }
                    },
                    BitmapFormat::Rgba8 => {
                        // Convert RGBA to grayscale using luminance
                        for y in 0..bitmap.height {
                            for x in 0..bitmap.width {
                                let idx = ((y * bitmap.width + x) * 4) as usize;
                                let r = bitmap.data[idx] as u32;
                                let g = bitmap.data[idx + 1] as u32;
                                let b = bitmap.data[idx + 2] as u32;
                                // Use standard luminance formula
                                let gray = ((r * 299 + g * 587 + b * 114) / 1000) as u8;
                                write!(&mut output, "{} ", gray)?;
                            }
                            writeln!(&mut output)?;
                        }
                    },
                    BitmapFormat::Rgb8 => {
                        // Convert RGB to grayscale
                        for y in 0..bitmap.height {
                            for x in 0..bitmap.width {
                                let idx = ((y * bitmap.width + x) * 3) as usize;
                                let r = bitmap.data[idx] as u32;
                                let g = bitmap.data[idx + 1] as u32;
                                let b = bitmap.data[idx + 2] as u32;
                                let gray = ((r * 299 + g * 587 + b * 114) / 1000) as u8;
                                write!(&mut output, "{} ", gray)?;
                            }
                            writeln!(&mut output)?;
                        }
                    },
                    BitmapFormat::Gray1 => {
                        // Convert 1-bit to 8-bit grayscale
                        for y in 0..bitmap.height {
                            for x in 0..bitmap.width {
                                let byte_idx = ((y * bitmap.width + x) / 8) as usize;
                                let bit_idx = ((y * bitmap.width + x) % 8) as usize;
                                let bit = (bitmap.data[byte_idx] >> (7 - bit_idx)) & 1;
                                let value = if bit == 1 { 255 } else { 0 };
                                write!(&mut output, "{} ", value)?;
                            }
                            writeln!(&mut output)?;
                        }
                    },
                }
            },
            PnmFormat::Pbm => {
                // PBM header
                writeln!(&mut output, "P1")?; // ASCII format
                writeln!(&mut output, "{} {}", bitmap.width, bitmap.height)?;

                // Convert to 1-bit
                match bitmap.format {
                    BitmapFormat::Gray1 => {
                        for y in 0..bitmap.height {
                            for x in 0..bitmap.width {
                                let byte_idx = ((y * bitmap.width + x) / 8) as usize;
                                let bit_idx = ((y * bitmap.width + x) % 8) as usize;
                                let bit = (bitmap.data[byte_idx] >> (7 - bit_idx)) & 1;
                                write!(&mut output, "{} ", bit)?;
                            }
                            writeln!(&mut output)?;
                        }
                    },
                    _ => {
                        // Convert other formats to 1-bit using threshold
                        for y in 0..bitmap.height {
                            for x in 0..bitmap.width {
                                let gray = match bitmap.format {
                                    BitmapFormat::Gray8 => {
                                        bitmap.data[(y * bitmap.width + x) as usize]
                                    },
                                    BitmapFormat::Rgba8 => {
                                        let idx = ((y * bitmap.width + x) * 4) as usize;
                                        let r = bitmap.data[idx] as u32;
                                        let g = bitmap.data[idx + 1] as u32;
                                        let b = bitmap.data[idx + 2] as u32;
                                        ((r * 299 + g * 587 + b * 114) / 1000) as u8
                                    },
                                    BitmapFormat::Rgb8 => {
                                        let idx = ((y * bitmap.width + x) * 3) as usize;
                                        let r = bitmap.data[idx] as u32;
                                        let g = bitmap.data[idx + 1] as u32;
                                        let b = bitmap.data[idx + 2] as u32;
                                        ((r * 299 + g * 587 + b * 114) / 1000) as u8
                                    },
                                    _ => 0,
                                };
                                let bit = if gray > 127 { 1 } else { 0 };
                                write!(&mut output, "{} ", bit)?;
                            }
                            writeln!(&mut output)?;
                        }
                    },
                }
            },
        }

        Ok(output)
    }
}

impl Exporter for PnmExporter {
    fn name(&self) -> &'static str {
        match self.format {
            PnmFormat::Pbm => "pbm",
            PnmFormat::Pgm => "pgm",
            PnmFormat::Ppm => "ppm",
        }
    }

    fn export(&self, output: &RenderOutput) -> Result<Vec<u8>> {
        match output {
            RenderOutput::Bitmap(bitmap) => self.export_bitmap(bitmap),
            _ => Err(ExportError::FormatNotSupported(
                "PNM exporter only supports bitmap output".into(),
            )
            .into()),
        }
    }

    fn extension(&self) -> &'static str {
        match self.format {
            PnmFormat::Pbm => "pbm",
            PnmFormat::Pgm => "pgm",
            PnmFormat::Ppm => "ppm",
        }
    }

    fn mime_type(&self) -> &'static str {
        match self.format {
            PnmFormat::Pbm => "image/x-portable-bitmap",
            PnmFormat::Pgm => "image/x-portable-graymap",
            PnmFormat::Ppm => "image/x-portable-pixmap",
        }
    }
}

impl Default for PnmExporter {
    fn default() -> Self {
        Self::ppm() // Default to color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ppm_export() {
        let exporter = PnmExporter::ppm();

        // Create a small test bitmap
        let bitmap = BitmapData {
            width: 2,
            height: 2,
            format: BitmapFormat::Rgba8,
            data: vec![
                255, 0, 0, 255, // Red pixel
                0, 255, 0, 255, // Green pixel
                0, 0, 255, 255, // Blue pixel
                255, 255, 255, 255, // White pixel
            ],
        };

        let output = RenderOutput::Bitmap(bitmap);
        let exported = exporter.export(&output).unwrap();

        let text = String::from_utf8(exported).unwrap();
        assert!(text.starts_with("P3"));
        assert!(text.contains("2 2")); // Dimensions
        assert!(text.contains("255")); // Max value
    }

    #[test]
    fn test_pgm_export() {
        let exporter = PnmExporter::pgm();

        let bitmap = BitmapData {
            width: 2,
            height: 1,
            format: BitmapFormat::Gray8,
            data: vec![128, 255],
        };

        let output = RenderOutput::Bitmap(bitmap);
        let exported = exporter.export(&output).unwrap();

        let text = String::from_utf8(exported).unwrap();
        assert!(text.starts_with("P2"));
        assert!(text.contains("2 1"));
        assert!(text.contains("128"));
        assert!(text.contains("255"));
    }

    #[test]
    fn test_extension_and_mime() {
        let ppm = PnmExporter::ppm();
        assert_eq!(ppm.extension(), "ppm");
        assert_eq!(ppm.mime_type(), "image/x-portable-pixmap");

        let pgm = PnmExporter::pgm();
        assert_eq!(pgm.extension(), "pgm");
        assert_eq!(pgm.mime_type(), "image/x-portable-graymap");
    }
}
