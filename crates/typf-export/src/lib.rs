//! Where rendered text leaves the building: export formats for TYPF
//!
//! The final stage of the pipeline. Turns your carefully rendered glyphs
//! into files, streams, or whatever format your application needs.

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

/// Simple bitmap exporter for when you just need to see what happened
pub struct PnmExporter {
    /// Choose your flavor: black-and-white, grayscale, or color
    format: PnmFormat,
}

#[derive(Debug, Clone, Copy)]
pub enum PnmFormat {
    /// PBM - Just black and white pixels
    Pbm,
    /// PGM - 256 shades of gray
    Pgm,
    /// PPM - Full RGB color
    Ppm,
}

impl PnmExporter {
    /// Creates an exporter for your chosen PNM format
    pub fn new(format: PnmFormat) -> Self {
        Self { format }
    }

    /// Quick way to get a color exporter
    pub fn ppm() -> Self {
        Self::new(PnmFormat::Ppm)
    }

    /// Quick way to get a grayscale exporter
    pub fn pgm() -> Self {
        Self::new(PnmFormat::Pgm)
    }

    /// Converts bitmap data into PNM's simple text format
    fn export_bitmap(&self, bitmap: &BitmapData) -> Result<Vec<u8>> {
        let mut output = Vec::new();

        match self.format {
            PnmFormat::Ppm => {
                // PPM needs a simple header first
                writeln!(&mut output, "P3")?; // Magic number for ASCII PPM
                writeln!(&mut output, "{} {}", bitmap.width, bitmap.height)?;
                writeln!(&mut output, "255")?; // Maximum RGB value

                // Transform bitmap data into PPM's text format
                match bitmap.format {
                    BitmapFormat::Rgba8 => {
                        // Strip alpha, keep just RGB
                        for y in 0..bitmap.height {
                            for x in 0..bitmap.width {
                                let idx = ((y * bitmap.width + x) * 4) as usize;
                                write!(
                                    &mut output,
                                    "{} {} {} ",
                                    bitmap.data[idx],     // Red
                                    bitmap.data[idx + 1], // Green
                                    bitmap.data[idx + 2]  // Blue
                                )?;
                            }
                            writeln!(&mut output)?; // New line after each row
                        }
                    },
                    BitmapFormat::Rgb8 => {
                        // Copy RGB values directly
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
                        // Make gray look like color (triplet the value)
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
                        // Expand 1-bit to full RGB
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
                // PGM header (grayscale version of PPM)
                writeln!(&mut output, "P2")?; // Magic number for ASCII PGM
                writeln!(&mut output, "{} {}", bitmap.width, bitmap.height)?;
                writeln!(&mut output, "255")?; // Maximum gray value

                // Flatten everything to grayscale
                match bitmap.format {
                    BitmapFormat::Gray8 => {
                        // Already grayscale, just copy
                        for y in 0..bitmap.height {
                            for x in 0..bitmap.width {
                                let idx = (y * bitmap.width + x) as usize;
                                write!(&mut output, "{} ", bitmap.data[idx])?;
                            }
                            writeln!(&mut output)?;
                        }
                    },
                    BitmapFormat::Rgba8 => {
                        // Convert color to grayscale using luminance
                        for y in 0..bitmap.height {
                            for x in 0..bitmap.width {
                                let idx = ((y * bitmap.width + x) * 4) as usize;
                                let r = bitmap.data[idx] as u32;
                                let g = bitmap.data[idx + 1] as u32;
                                let b = bitmap.data[idx + 2] as u32;
                                // ITU-R BT.709 luminance formula
                                let gray = ((r * 299 + g * 587 + b * 114) / 1000) as u8;
                                write!(&mut output, "{} ", gray)?;
                            }
                            writeln!(&mut output)?;
                        }
                    },
                    BitmapFormat::Rgb8 => {
                        // Color to grayscale conversion
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
                        // Expand 1-bit to 8-bit grayscale
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
                // PBM header (the simplest format - just 0s and 1s)
                writeln!(&mut output, "P1")?; // Magic number for ASCII PBM
                writeln!(&mut output, "{} {}", bitmap.width, bitmap.height)?;

                // Everything becomes black (0) or white (1)
                match bitmap.format {
                    BitmapFormat::Gray1 => {
                        // Already 1-bit, just copy
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
                        // Convert Everything to 1-bit with a simple threshold
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
                                // 127 is a reasonable threshold
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
