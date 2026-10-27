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

/// Highest version currently implemented.
pub(crate) const MAX_SUPPORTED_VERSION: u8 = 40;

impl QrCode {
    /// Encodes `data` in the smallest QR Code that can hold it.
    pub fn encode(data: &str, level: CorrectionLevel) -> Result<Self, QrError> {
        let mode = bits::determine_mode(data);
        let mut needed_bits = 0;

        for version in 1..=MAX_SUPPORTED_VERSION {
            needed_bits = bits::required_bits(data, mode, version)?;
            let capacity_bits = tables::data_codewords(version, level) * 8;

            if needed_bits <= capacity_bits {
                return Self::build(data, mode, version, level, None);
            }
        }

        let capacity_bits = tables::data_codewords(MAX_SUPPORTED_VERSION, level) * 8;
        Err(QrError::DataTooLong {
            version: MAX_SUPPORTED_VERSION,
            level,
            needed_bits,
            capacity_bits,
        })
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
        if version == 0 || version > MAX_SUPPORTED_VERSION {
            return Err(QrError::InvalidVersion(version));
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
        let num_blocks = tables::num_blocks(version, level);
        let total_codewords = tables::total_codewords(version);
        let codewords =
            reed_solomon::encode_blocks(&data_codewords, ecc_len, num_blocks, total_codewords);
        debug_assert_eq!(codewords.len(), total_codewords);

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
        self.to_ascii_with_quiet_zone(4)
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

    fn reference_qr(
        data: &str,
        mode: Mode,
        version: u8,
        level: CorrectionLevel,
        mask: u8,
    ) -> ReferenceQrCode {
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
            Version::new(version),
            Version::new(version),
            Some(Mask::new(mask)),
            false,
        )
        .expect("reference encoding failed")
    }

    fn assert_matches_reference_mask(
        data: &str,
        mode: Mode,
        version: u8,
        level: CorrectionLevel,
        mask: u8,
    ) {
        let qr = QrCode::build(data, mode, version, level, Some(mask)).unwrap();
        let reference = reference_qr(data, mode, version, level, mask);

        assert_eq!(
            qr.size(),
            reference.size() as usize,
            "size mismatch for version {version}, {data:?}, {mode:?}, {level:?}, mask {mask}"
        );

        for y in 0..qr.size() {
            for x in 0..qr.size() {
                assert_eq!(
                    qr.modules()[y][x] == 1,
                    reference.get_module(x as i32, y as i32),
                    "module mismatch at ({x}, {y}) for version {version}, {data:?}, \
                     {mode:?}, {level:?}, mask {mask}"
                );
            }
        }
    }

    fn assert_matches_reference(data: &str, mode: Mode, version: u8, level: CorrectionLevel) {
        for mask in 0u8..8 {
            assert_matches_reference_mask(data, mode, version, level, mask);
        }
    }

    fn byte_data_for_version(version: u8) -> String {
        let data_codewords = crate::tables::data_codewords(version, CorrectionLevel::High);
        let count_bits = crate::tables::char_count_bits(version, Mode::Byte) as usize;
        let max_bytes = (data_codewords * 8 - 4 - count_bits) / 8;

        (0..max_bytes)
            .map(|index| char::from(b'a' + (index % 26) as u8))
            .collect()
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
                assert_matches_reference(data, mode, 1, level);
            }
        }
    }

    #[test]
    fn matches_qrcodegen_for_all_versions_in_byte_mode() {
        for version in 1..=MAX_SUPPORTED_VERSION {
            let data = byte_data_for_version(version);
            assert_matches_reference(&data, Mode::Byte, version, CorrectionLevel::High);
        }
    }

    #[test]
    fn matches_qrcodegen_for_all_versions_and_levels_with_mask_0() {
        for version in 1..=MAX_SUPPORTED_VERSION {
            let data = byte_data_for_version(version);

            for level in [
                CorrectionLevel::Low,
                CorrectionLevel::Medium,
                CorrectionLevel::Quartile,
                CorrectionLevel::High,
            ] {
                assert_matches_reference_mask(&data, Mode::Byte, version, level, 0);
            }
        }
    }

    #[test]
    fn matches_qrcodegen_for_representative_versions_in_all_modes_and_levels() {
        let numeric = "1".repeat(150);
        let alphanumeric = "A".repeat(90);

        for version in [7u8, 10, 27, 40] {
            let byte_data = byte_data_for_version(version);
            let cases = [
                (numeric.as_str(), Mode::Numeric),
                (alphanumeric.as_str(), Mode::Alphanumeric),
                (byte_data.as_str(), Mode::Byte),
            ];

            for (data, mode) in cases {
                for level in [
                    CorrectionLevel::Low,
                    CorrectionLevel::Medium,
                    CorrectionLevel::Quartile,
                    CorrectionLevel::High,
                ] {
                    assert_matches_reference(data, mode, version, level);
                }
            }
        }
    }

    #[test]
    fn encode_selects_the_smallest_version_that_fits() {
        let mut cases = vec![(7usize, 1u8), (8, 2), (15, 3), (25, 4), (35, 5), (45, 6)];

        for version in [7u8, 10, 20, 40] {
            cases.push((byte_data_for_version(version).len(), version));
        }

        for (length, expected_version) in cases {
            let data = "a".repeat(length);
            let qr = QrCode::encode(&data, CorrectionLevel::High).unwrap();

            assert_eq!(
                qr.version(),
                expected_version,
                "wrong version for {length} bytes"
            );
        }

        let too_long = "a".repeat(byte_data_for_version(40).len() + 1);
        assert!(matches!(
            QrCode::encode(&too_long, CorrectionLevel::High),
            Err(QrError::DataTooLong { .. })
        ));
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
    fn version_6_matrix_is_41_by_41() {
        let qr = QrCode::encode_version("HELLO WORLD", 6, CorrectionLevel::Low).unwrap();
        let matrix = qr.to_matrix();

        assert_eq!(qr.version(), 6);
        assert_eq!(matrix.len(), 41);
        assert!(matrix.iter().all(|row| row.len() == 41));
        assert!(matrix.iter().flatten().all(|&bit| bit == 0 || bit == 1));
    }

    #[test]
    fn version_40_matrix_is_177_by_177() {
        let data = byte_data_for_version(40);
        let qr = QrCode::encode_version(&data, 40, CorrectionLevel::High).unwrap();
        let matrix = qr.to_matrix();

        assert_eq!(qr.version(), 40);
        assert_eq!(matrix.len(), 177);
        assert!(matrix.iter().all(|row| row.len() == 177));
        assert!(matrix.iter().flatten().all(|&bit| bit == 0 || bit == 1));
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
