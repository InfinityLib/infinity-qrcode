//! Output formats for a generated QR Code.

use std::fmt::Write as _;

use crate::qr::QrCode;

#[cfg(feature = "png")]
use crate::error::QrError;

impl QrCode {
    /// Renders the QR Code as text with a custom quiet zone.
    pub fn to_ascii_with_quiet_zone(&self, quiet_zone: usize) -> String {
        let width = self.size() + 2 * quiet_zone;
        let mut output = String::with_capacity((width * 2 + 1) * (width + 2 * quiet_zone));

        for _ in 0..quiet_zone {
            for _ in 0..width {
                output.push_str("  ");
            }
            output.push('\n');
        }

        for row in self.modules() {
            for _ in 0..quiet_zone {
                output.push_str("  ");
            }

            for &module in row {
                output.push_str(if module == 1 { "██" } else { "  " });
            }

            for _ in 0..quiet_zone {
                output.push_str("  ");
            }
            output.push('\n');
        }

        for _ in 0..quiet_zone {
            for _ in 0..width {
                output.push_str("  ");
            }
            output.push('\n');
        }

        output
    }

    /// Renders the QR Code as a plain-text PBM (P1) image.
    ///
    /// `quiet_zone` is expressed in modules.
    pub fn to_pbm(&self, quiet_zone: usize) -> String {
        let size = self.size();
        let width = size + 2 * quiet_zone;
        let mut output = String::new();

        output.push_str("P1\n");
        writeln!(output, "{width} {width}").expect("writing to a String cannot fail");

        for y in 0..width {
            for x in 0..width {
                if x > 0 {
                    output.push(' ');
                }

                let module = if x < quiet_zone
                    || y < quiet_zone
                    || x >= quiet_zone + size
                    || y >= quiet_zone + size
                {
                    0
                } else {
                    self.modules()[y - quiet_zone][x - quiet_zone]
                };

                output.push(if module == 1 { '1' } else { '0' });
            }

            output.push('\n');
        }

        output
    }

    /// Renders the QR Code as an SVG image.
    ///
    /// `module_size` is the size of one module in pixels, and `quiet_zone` is
    /// the width of the light border in modules.
    pub fn to_svg(&self, module_size: usize, quiet_zone: usize) -> String {
        let module_size = module_size.max(1);
        let size = self.size();
        let width = (size + 2 * quiet_zone) * module_size;
        let mut output = String::new();

        writeln!(
            output,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{width}\" viewBox=\"0 0 {width} {width}\" shape-rendering=\"crispEdges\">"
        )
        .expect("writing to a String cannot fail");

        writeln!(
            output,
            "<rect width=\"{width}\" height=\"{width}\" fill=\"#ffffff\"/>"
        )
        .expect("writing to a String cannot fail");

        output.push_str("<path fill=\"#000000\" d=\"");

        for (y, row) in self.modules().iter().enumerate() {
            for (x, &module) in row.iter().enumerate() {
                if module == 1 {
                    let x = (x + quiet_zone) * module_size;
                    let y = (y + quiet_zone) * module_size;

                    write!(
                        output,
                        "M{x} {y}h{module_size}v{module_size}h-{module_size}z"
                    )
                    .expect("writing to a String cannot fail");
                }
            }
        }

        output.push_str("\"/>\n");
        output.push_str("</svg>\n");
        output
    }

    /// Renders the QR Code as an 8-bit greyscale PNG.
    ///
    /// This method is available with the `png` feature.
    #[cfg(feature = "png")]
    pub fn to_png(&self, module_size: usize, quiet_zone: usize) -> Result<Vec<u8>, QrError> {
        let module_size = module_size.max(1);
        let size = self.size();
        let dimension = (size + 2 * quiet_zone)
            .checked_mul(module_size)
            .ok_or_else(|| QrError::PngEncoding("image dimensions overflow".to_string()))?;

        let width = u32::try_from(dimension)
            .map_err(|_| QrError::PngEncoding("image is too large for PNG".to_string()))?;

        let pixel_count = dimension
            .checked_mul(dimension)
            .ok_or_else(|| QrError::PngEncoding("image dimensions overflow".to_string()))?;

        let mut pixels = vec![255u8; pixel_count];

        for (y, row) in self.modules().iter().enumerate() {
            for (x, &module) in row.iter().enumerate() {
                if module != 1 {
                    continue;
                }

                let start_x = (x + quiet_zone) * module_size;
                let start_y = (y + quiet_zone) * module_size;

                for offset_y in 0..module_size {
                    let start = (start_y + offset_y) * dimension + start_x;
                    pixels[start..start + module_size].fill(0);
                }
            }
        }

        let mut output = Vec::new();

        {
            let mut encoder = png::Encoder::new(&mut output, width, width);
            encoder.set_color(png::ColorType::Grayscale);
            encoder.set_depth(png::BitDepth::Eight);

            let mut writer = encoder
                .write_header()
                .map_err(|error| QrError::PngEncoding(error.to_string()))?;

            writer
                .write_image_data(&pixels)
                .map_err(|error| QrError::PngEncoding(error.to_string()))?;

            writer
                .finish()
                .map_err(|error| QrError::PngEncoding(error.to_string()))?;
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qr::CorrectionLevel;

    fn sample_qr() -> QrCode {
        QrCode::encode_version("HELLO WORLD", 1, CorrectionLevel::Medium).unwrap()
    }

    #[test]
    fn ascii_quiet_zone_changes_line_count() {
        let qr = sample_qr();

        assert_eq!(qr.to_ascii_with_quiet_zone(0).lines().count(), 21);
        assert_eq!(qr.to_ascii().lines().count(), 29);
    }

    #[test]
    fn pbm_has_a_header_and_the_expected_dimensions() {
        let qr = sample_qr();
        let pbm = qr.to_pbm(2);
        let mut lines = pbm.lines();

        assert_eq!(lines.next(), Some("P1"));
        assert_eq!(lines.next(), Some("25 25"));
        assert_eq!(lines.count(), 25);
    }

    #[test]
    fn svg_has_the_expected_dimensions() {
        let qr = sample_qr();
        let svg = qr.to_svg(4, 2);

        assert!(svg.contains(r#"width="100" height="100" viewBox="0 0 100 100""#));
        assert!(svg.contains("fill=\"#000000\""));
    }

    #[cfg(feature = "png")]
    #[test]
    fn png_has_the_expected_dimensions_and_quiet_zone() {
        let qr = sample_qr();
        let png = qr.to_png(4, 2).unwrap();

        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");

        let decoder = png::Decoder::new(std::io::Cursor::new(png));
        let mut reader = decoder.read_info().unwrap();
        let mut pixels = vec![
            0;
            reader
                .output_buffer_size()
                .expect("PNG output buffer size should be known")
        ];
        let info = reader.next_frame(&mut pixels).unwrap();

        let expected = ((21 + 4) * 4) as u32;
        assert_eq!(info.width, expected);
        assert_eq!(info.height, expected);

        // The quiet zone is white.
        assert_eq!(pixels[0], 255);

        // The top-left finder module starts at (quiet_zone, quiet_zone).
        let index = (2 * 4) * expected as usize + (2 * 4);
        assert_eq!(pixels[index], 0);
    }
}
