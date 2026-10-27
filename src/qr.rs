use core::fmt;

use crate::bits;
use crate::error::QrError;
use crate::matrix::{self, Matrix};
use crate::reed_solomon;
use crate::tables;

/// Error correction level of a QR Code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorrectionLevel {
    Low,
    Medium,
    Quartile,
    High,
}

impl CorrectionLevel {
    /// Two-bit value used in the format information.
    pub(crate) const fn format_bits(self) -> u8 {
        match self {
            Self::Low => 0b01,
            Self::Medium => 0b00,
            Self::Quartile => 0b11,
            Self::High => 0b10,
        }
    }
}

impl fmt::Display for CorrectionLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::Quartile => "Quartile",
            Self::High => "High",
        };
        f.write_str(name)
    }
}

/// Data encoding mode of a QR Code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Numeric,
    Alphanumeric,
    Byte,
    Kanji,
}

impl Mode {
    /// Four-bit mode indicator.
    pub(crate) const fn indicator(self) -> u8 {
        match self {
            Self::Numeric => 0b0001,
            Self::Alphanumeric => 0b0010,
            Self::Byte => 0b0100,
            Self::Kanji => 0b1000,
        }
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Numeric => "Numeric",
            Self::Alphanumeric => "Alphanumeric",
            Self::Byte => "Byte",
            Self::Kanji => "Kanji",
        };
        f.write_str(name)
    }
}

/// A generated QR Code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QrCode {
    modules: Vec<Vec<u8>>,
    version: u8,
    correction_level: CorrectionLevel,
    mask: u8,
}

impl QrCode {
    /// Encodes `data` in a version-1 QR Code with the given correction level.
    ///
    /// Version 1 is the only version implemented for now.
    pub fn encode(data: &str, level: CorrectionLevel) -> Result<Self, QrError> {
        Self::encode_version(data, 1, level)
    }

    /// Encodes `data` in a specific version.
    pub fn encode_version(
        data: &str,
        version: u8,
        level: CorrectionLevel,
    ) -> Result<Self, QrError> {
        let mode = bits::determine_mode(data);
        Self::encode_with_mode(data, mode, version, level)
    }

    /// Encodes `data` with an explicit mode.
    pub fn encode_with_mode(
        data: &str,
        mode: Mode,
        version: u8,
        level: CorrectionLevel,
    ) -> Result<Self, QrError> {
        Self::build(data, mode, version, level, None)
    }

    pub(crate) fn build(
        data: &str,
        mode: Mode,
        version: u8,
        level: CorrectionLevel,
        forced_mask: Option<u8>,
    ) -> Result<Self, QrError> {
        if version == 0 || version > 40 {
            return Err(QrError::InvalidVersion(version));
        }
        if version != 1 {
            return Err(QrError::UnsupportedVersion(version));
        }
        if mode == Mode::Kanji {
            return Err(QrError::UnsupportedMode(mode));
        }
        if let Some(mask) = forced_mask
            && mask > 7
        {
            return Err(QrError::InvalidMask(mask));
        }

        let capacity_bits = tables::data_codewords(version, level) * 8;
        let bits = bits::encode_data(data, mode, version, level)?;
        let bits = bits::finish(bits, capacity_bits, version, level)?;
        let data_codewords = bits.into_bytes();
        debug_assert_eq!(data_codewords.len(), tables::data_codewords(version, level));

        let ecc_len = tables::ecc_codewords(version, level);
        let ecc_codewords = reed_solomon::encode(&data_codewords, ecc_len);

        let mut codewords = data_codewords;
        codewords.extend_from_slice(&ecc_codewords);
        debug_assert_eq!(codewords.len(), tables::VERSION_1_TOTAL_CODEWORDS);

        let size = tables::version_size(version);
        let mut base = Matrix::new(size);
        base.draw_function_patterns(version);
        base.place_data(&codewords);

        let (modules, mask) = matrix::choose_mask(&base, level, forced_mask)?;

        Ok(Self {
            modules,
            version,
            correction_level: level,
            mask,
        })
    }

    /// Returns a copy of the module matrix, without any quiet zone.
    ///
    /// `1` represents a dark module and `0` a light module.
    pub fn to_matrix(&self) -> Vec<Vec<u8>> {
        self.modules.clone()
    }

    /// Returns the module matrix as a borrowed slice.
    pub fn modules(&self) -> &[Vec<u8>] {
        &self.modules
    }

    /// Side length of the module matrix.
    pub fn size(&self) -> usize {
        self.modules.len()
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn correction_level(&self) -> CorrectionLevel {
        self.correction_level
    }

    pub fn mask(&self) -> u8 {
        self.mask
    }

    /// Renders the QR Code as text, with a four-module quiet zone.
    pub fn to_ascii(&self) -> String {
        const QUIET_ZONE: usize = 4;

        let width = self.size() + 2 * QUIET_ZONE;
        let mut output = String::with_capacity((width * 2 + 1) * (width + 2 * QUIET_ZONE));

        for _ in 0..QUIET_ZONE {
            for _ in 0..width {
                output.push_str("  ");
            }
            output.push('\n');
        }

        for row in &self.modules {
            for _ in 0..QUIET_ZONE {
                output.push_str("  ");
            }

            for &module in row {
                output.push_str(if module == 1 { "██" } else { "  " });
            }

            for _ in 0..QUIET_ZONE {
                output.push_str("  ");
            }
            output.push('\n');
        }

        for _ in 0..QUIET_ZONE {
            for _ in 0..width {
                output.push_str("  ");
            }
            output.push('\n');
        }

        output
    }
}

impl fmt::Display for QrCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_ascii())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qrcodegen::{Mask, QrCode as ReferenceQrCode, QrCodeEcc, QrSegment, Version};

    fn reference_qr(data: &str, mode: Mode, level: CorrectionLevel, mask: u8) -> ReferenceQrCode {
        let segment = match mode {
            Mode::Numeric => QrSegment::make_numeric(data),
            Mode::Alphanumeric => QrSegment::make_alphanumeric(data),
            Mode::Byte => QrSegment::make_bytes(data.as_bytes()),
            Mode::Kanji => unreachable!(),
        };

        let ecc = match level {
            CorrectionLevel::Low => QrCodeEcc::Low,
            CorrectionLevel::Medium => QrCodeEcc::Medium,
            CorrectionLevel::Quartile => QrCodeEcc::Quartile,
            CorrectionLevel::High => QrCodeEcc::High,
        };

        ReferenceQrCode::encode_segments_advanced(
            &[segment],
            ecc,
            Version::new(1),
            Version::new(1),
            Some(Mask::new(mask)),
            false,
        )
        .expect("reference encoding failed")
    }

    fn assert_matches_reference(data: &str, mode: Mode, level: CorrectionLevel) {
        for mask in 0u8..8 {
            let qr = QrCode::build(data, mode, 1, level, Some(mask)).unwrap();
            let reference = reference_qr(data, mode, level, mask);

            assert_eq!(
                qr.size(),
                reference.size() as usize,
                "size mismatch for {data:?}, {mode:?}, {level:?}, mask {mask}"
            );

            for y in 0..qr.size() {
                for x in 0..qr.size() {
                    assert_eq!(
                        qr.modules()[y][x] == 1,
                        reference.get_module(x as i32, y as i32),
                        "module mismatch at ({x}, {y}) for {data:?}, {mode:?}, {level:?}, mask {mask}"
                    );
                }
            }
        }
    }

    #[test]
    fn matches_qrcodegen_for_all_v1_modes_levels_and_masks() {
        let cases = [
            ("01234567", Mode::Numeric),
            ("HELLO", Mode::Alphanumeric),
            ("Hi!", Mode::Byte),
        ];

        for level in [
            CorrectionLevel::Low,
            CorrectionLevel::Medium,
            CorrectionLevel::Quartile,
            CorrectionLevel::High,
        ] {
            for (data, mode) in cases {
                assert_matches_reference(data, mode, level);
            }
        }
    }

    #[test]
    fn matrix_is_21_by_21_and_only_contains_bits() {
        let qr = QrCode::encode_version("HELLO WORLD", 1, CorrectionLevel::Quartile).unwrap();
        let matrix = qr.to_matrix();

        assert_eq!(matrix.len(), 21);
        assert!(matrix.iter().all(|row| row.len() == 21));
        assert!(matrix.iter().flatten().all(|&bit| bit == 0 || bit == 1));
        assert_eq!(matrix, qr.modules());
    }

    #[test]
    fn rejects_out_of_range_versions() {
        assert_eq!(
            QrCode::encode_version("A", 0, CorrectionLevel::Low),
            Err(QrError::InvalidVersion(0))
        );
        assert_eq!(
            QrCode::encode_version("A", 41, CorrectionLevel::Low),
            Err(QrError::InvalidVersion(41))
        );
    }

    #[test]
    fn rejects_valid_but_unimplemented_versions() {
        assert_eq!(
            QrCode::encode_version("A", 2, CorrectionLevel::Low),
            Err(QrError::UnsupportedVersion(2))
        );
    }

    #[test]
    fn rejects_data_that_does_not_fit() {
        let data = "A".repeat(100);

        assert!(matches!(
            QrCode::encode_version(&data, 1, CorrectionLevel::High),
            Err(QrError::DataTooLong {
                version: 1,
                level: CorrectionLevel::High,
                ..
            })
        ));
    }

    #[test]
    fn rejects_invalid_characters_in_explicit_mode() {
        assert_eq!(
            QrCode::encode_with_mode("abc", Mode::Numeric, 1, CorrectionLevel::Low),
            Err(QrError::InvalidCharacter('a'))
        );
    }

    #[test]
    fn rejects_kanji_mode_for_now() {
        assert_eq!(
            QrCode::encode_with_mode("A", Mode::Kanji, 1, CorrectionLevel::Low),
            Err(QrError::UnsupportedMode(Mode::Kanji))
        );
    }

    #[test]
    fn rejects_invalid_forced_masks() {
        assert_eq!(
            QrCode::build("A", Mode::Alphanumeric, 1, CorrectionLevel::Low, Some(8)),
            Err(QrError::InvalidMask(8))
        );
    }
}
