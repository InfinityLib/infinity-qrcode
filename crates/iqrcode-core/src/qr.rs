use core::fmt;

use alloc::string::String;
use alloc::vec::Vec;

use crate::bits::{self, Segment};
use crate::error::QrError;
use crate::matrix::{self, Matrix};
use crate::optimize;
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

/// Version ranges that share the same character-count indicator width.
const VERSION_GROUPS: [(u8, u8); 3] = [(1, 9), (10, 26), (27, 40)];

/// Data codewords, ECC codewords and interleaved codewords for one QR Code.
#[cfg_attr(not(feature = "trace"), allow(dead_code))]
pub(crate) struct EncodedData {
    pub(crate) data_codewords: Vec<u8>,
    pub(crate) ecc_len: usize,
    pub(crate) num_blocks: usize,
    pub(crate) total_codewords: usize,
    pub(crate) codewords: Vec<u8>,
}

impl QrCode {
    /// Encodes `data` in the smallest QR Code that can hold it.
    ///
    /// The input is automatically split into Numeric, Alphanumeric and Byte
    /// segments when that reduces the encoded size.
    pub fn encode(data: &str, level: CorrectionLevel) -> Result<Self, QrError> {
        let (segments, version) = Self::plan(data, level)?;
        Self::build_from_segments(&segments, version, level, None)
    }

    /// Encodes `data` in a specific version.
    ///
    /// The input is automatically split into segments when that reduces the
    /// encoded size.
    pub fn encode_version(
        data: &str,
        version: u8,
        level: CorrectionLevel,
    ) -> Result<Self, QrError> {
        let segments = Self::plan_version(data, version)?;
        Self::build_from_segments(&segments, version, level, None)
    }

    /// Chooses the smallest version and the matching segment layout for `data`.
    pub(crate) fn plan(
        data: &str,
        level: CorrectionLevel,
    ) -> Result<(Vec<Segment<'_>>, u8), QrError> {
        for (group_start, group_end) in VERSION_GROUPS {
            let segments = optimize::optimal_segments(data, group_start);
            let needed_bits = bits::segments_bits(&segments, group_start)?;

            for version in group_start..=group_end {
                let capacity_bits = tables::data_codewords(version, level) * 8;

                if needed_bits <= capacity_bits {
                    return Ok((segments, version));
                }
            }
        }

        let segments = optimize::optimal_segments(data, MAX_SUPPORTED_VERSION);
        let needed_bits = bits::segments_bits(&segments, MAX_SUPPORTED_VERSION)?;
        let capacity_bits = tables::data_codewords(MAX_SUPPORTED_VERSION, level) * 8;

        Err(QrError::DataTooLong {
            version: MAX_SUPPORTED_VERSION,
            level,
            needed_bits,
            capacity_bits,
        })
    }

    /// Chooses the segment layout for an explicit version.
    pub(crate) fn plan_version(data: &str, version: u8) -> Result<Vec<Segment<'_>>, QrError> {
        if version == 0 || version > MAX_SUPPORTED_VERSION {
            return Err(QrError::InvalidVersion(version));
        }

        Ok(optimize::optimal_segments(data, version))
    }

    /// Encodes `data` with an explicit single mode.
    ///
    /// Kanji requires raw Shift-JIS bytes; use [`QrCode::encode_kanji_shift_jis`]
    /// instead.
    pub fn encode_with_mode(
        data: &str,
        mode: Mode,
        version: u8,
        level: CorrectionLevel,
    ) -> Result<Self, QrError> {
        if mode == Mode::Kanji {
            return Err(QrError::KanjiRequiresShiftJis);
        }

        Self::build(data, mode, version, level, None)
    }

    /// Encodes UTF-8 text with an ECI 26 designator followed by a Byte segment.
    ///
    /// This tells decoders explicitly that the byte payload is UTF-8.
    pub fn encode_utf8(data: &str, level: CorrectionLevel) -> Result<Self, QrError> {
        let segments = [
            Segment::Eci(26),
            Segment::Data {
                mode: Mode::Byte,
                data: data.as_bytes(),
            },
        ];

        Self::encode_fixed_segments(&segments, level)
    }

    /// Encodes UTF-8 text with an ECI 26 designator in a specific version.
    pub fn encode_utf8_version(
        data: &str,
        version: u8,
        level: CorrectionLevel,
    ) -> Result<Self, QrError> {
        let segments = [
            Segment::Eci(26),
            Segment::Data {
                mode: Mode::Byte,
                data: data.as_bytes(),
            },
        ];

        Self::build_from_segments(&segments, version, level, None)
    }

    /// Encodes raw Shift-JIS bytes in QR Kanji mode.
    ///
    /// `data` must contain complete two-byte Shift-JIS characters in the ranges
    /// `0x8140..=0x9FFC` or `0xE040..=0xEBBF`.
    pub fn encode_kanji_shift_jis(data: &[u8], level: CorrectionLevel) -> Result<Self, QrError> {
        let segments = [Segment::Data {
            mode: Mode::Kanji,
            data,
        }];

        Self::encode_fixed_segments(&segments, level)
    }

    /// Encodes raw Shift-JIS bytes in QR Kanji mode in a specific version.
    pub fn encode_kanji_shift_jis_version(
        data: &[u8],
        version: u8,
        level: CorrectionLevel,
    ) -> Result<Self, QrError> {
        let segments = [Segment::Data {
            mode: Mode::Kanji,
            data,
        }];

        Self::build_from_segments(&segments, version, level, None)
    }

    fn encode_fixed_segments(
        segments: &[Segment<'_>],
        level: CorrectionLevel,
    ) -> Result<Self, QrError> {
        let mut needed_bits = 0;

        for version in 1..=MAX_SUPPORTED_VERSION {
            needed_bits = bits::segments_bits(segments, version)?;
            let capacity_bits = tables::data_codewords(version, level) * 8;

            if needed_bits <= capacity_bits {
                return Self::build_from_segments(segments, version, level, None);
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

    pub(crate) fn build(
        data: &str,
        mode: Mode,
        version: u8,
        level: CorrectionLevel,
        forced_mask: Option<u8>,
    ) -> Result<Self, QrError> {
        if mode == Mode::Kanji {
            return Err(QrError::KanjiRequiresShiftJis);
        }

        let segments = [Segment::Data {
            mode,
            data: data.as_bytes(),
        }];

        Self::build_from_segments(&segments, version, level, forced_mask)
    }

    pub(crate) fn encode_codewords(
        segments: &[Segment<'_>],
        version: u8,
        level: CorrectionLevel,
    ) -> Result<EncodedData, QrError> {
        let capacity_bits = tables::data_codewords(version, level) * 8;
        let bits = bits::encode_segments(segments, version, level)?;
        let bits = bits::finish(bits, capacity_bits, version, level)?;
        let data_codewords = bits.into_bytes();
        debug_assert_eq!(data_codewords.len(), tables::data_codewords(version, level));

        let ecc_len = tables::ecc_codewords(version, level);
        let num_blocks = tables::num_blocks(version, level);
        let total_codewords = tables::total_codewords(version);
        let codewords =
            reed_solomon::encode_blocks(&data_codewords, ecc_len, num_blocks, total_codewords);
        debug_assert_eq!(codewords.len(), total_codewords);

        Ok(EncodedData {
            data_codewords,
            ecc_len,
            num_blocks,
            total_codewords,
            codewords,
        })
    }

    pub(crate) fn build_from_segments(
        segments: &[Segment<'_>],
        version: u8,
        level: CorrectionLevel,
        forced_mask: Option<u8>,
    ) -> Result<Self, QrError> {
        if version == 0 || version > MAX_SUPPORTED_VERSION {
            return Err(QrError::InvalidVersion(version));
        }
        if let Some(mask) = forced_mask
            && mask > 7
        {
            return Err(QrError::InvalidMask(mask));
        }

        let encoded = Self::encode_codewords(segments, version, level)?;

        let size = tables::version_size(version);
        let mut base = Matrix::new(size);
        base.draw_function_patterns(version);
        base.place_data(&encoded.codewords);

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
    use alloc::format;
    use alloc::string::String;
    use alloc::vec;
    use qrcodegen::{Mask, QrCode as ReferenceQrCode, QrCodeEcc, QrSegment, Version};

    fn reference_ecc(level: CorrectionLevel) -> QrCodeEcc {
        match level {
            CorrectionLevel::Low => QrCodeEcc::Low,
            CorrectionLevel::Medium => QrCodeEcc::Medium,
            CorrectionLevel::Quartile => QrCodeEcc::Quartile,
            CorrectionLevel::High => QrCodeEcc::High,
        }
    }

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

        ReferenceQrCode::encode_segments_advanced(
            &[segment],
            reference_ecc(level),
            Version::new(version),
            Version::new(version),
            Some(Mask::new(mask)),
            false,
        )
        .expect("reference encoding failed")
    }

    fn assert_qr_matches(qr: &QrCode, reference: &ReferenceQrCode, context: &str) {
        assert_eq!(
            qr.size(),
            reference.size() as usize,
            "size mismatch for {context}"
        );

        for y in 0..qr.size() {
            for x in 0..qr.size() {
                assert_eq!(
                    qr.modules()[y][x] == 1,
                    reference.get_module(x as i32, y as i32),
                    "module mismatch at ({x}, {y}) for {context}"
                );
            }
        }
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
        let context = format!("version {version}, {data:?}, {mode:?}, {level:?}, mask {mask}");

        assert_qr_matches(&qr, &reference, &context);
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
    fn matches_qrcodegen_for_utf8_eci() {
        let data = "Héllo";
        let segments = [
            Segment::Eci(26),
            Segment::Data {
                mode: Mode::Byte,
                data: data.as_bytes(),
            },
        ];

        for version in [1u8, 10, 40] {
            for level in [
                CorrectionLevel::Low,
                CorrectionLevel::Medium,
                CorrectionLevel::Quartile,
                CorrectionLevel::High,
            ] {
                for mask in 0u8..8 {
                    let qr =
                        QrCode::build_from_segments(&segments, version, level, Some(mask)).unwrap();
                    let reference = ReferenceQrCode::encode_segments_advanced(
                        &[
                            QrSegment::make_eci(26),
                            QrSegment::make_bytes(data.as_bytes()),
                        ],
                        reference_ecc(level),
                        Version::new(version),
                        Version::new(version),
                        Some(Mask::new(mask)),
                        false,
                    )
                    .expect("reference encoding failed");

                    let context = format!("UTF-8 ECI, version {version}, {level:?}, mask {mask}");
                    assert_qr_matches(&qr, &reference, &context);
                }
            }
        }
    }

    #[test]
    fn matches_qrcodegen_for_kanji_segment() {
        let data = b"\x93\x5f\xe4\xaa";
        let segments = [Segment::Data {
            mode: Mode::Kanji,
            data,
        }];

        // Data bits for the two Shift-JIS characters in the ISO example.
        let mut kanji_data_bits = Vec::new();
        for value in [3487u32, 6826] {
            for shift in (0..13).rev() {
                kanji_data_bits.push(((value >> shift) & 1) != 0);
            }
        }

        let kanji_segment = QrSegment::new(qrcodegen::QrSegmentMode::Kanji, 2, kanji_data_bits);

        for mask in 0u8..8 {
            let qr = QrCode::build_from_segments(&segments, 1, CorrectionLevel::Low, Some(mask))
                .unwrap();
            let reference = ReferenceQrCode::encode_segments_advanced(
                std::slice::from_ref(&kanji_segment),
                reference_ecc(CorrectionLevel::Low),
                Version::new(1),
                Version::new(1),
                Some(Mask::new(mask)),
                false,
            )
            .expect("reference encoding failed");

            let context = format!("Kanji, version 1, Low, mask {mask}");
            assert_qr_matches(&qr, &reference, &context);
        }
    }

    #[test]
    fn kanji_shift_jis_encoding_produces_a_matrix() {
        let data = b"\x93\x5f\xe4\xaa";
        let qr = QrCode::encode_kanji_shift_jis(data, CorrectionLevel::Low).unwrap();

        assert_eq!(qr.version(), 1);
        assert_eq!(qr.size(), 21);

        let qr = QrCode::encode_kanji_shift_jis_version(data, 1, CorrectionLevel::Low).unwrap();
        assert_eq!(qr.size(), 21);
    }

    #[test]
    fn invalid_kanji_data_is_rejected_by_the_public_api() {
        assert_eq!(
            QrCode::encode_kanji_shift_jis(b"\x93", CorrectionLevel::Low),
            Err(QrError::InvalidKanjiData)
        );
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
    fn rejects_kanji_from_utf8() {
        assert_eq!(
            QrCode::encode_with_mode("A", Mode::Kanji, 1, CorrectionLevel::Low),
            Err(QrError::KanjiRequiresShiftJis)
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
