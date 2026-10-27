use crate::qr::{CorrectionLevel, Mode};

/// Number of codewords in a version-1 QR Code (data + error correction).
pub(crate) const VERSION_1_TOTAL_CODEWORDS: usize = 26;

/// Side length of the QR matrix for a given version.
pub(crate) fn version_size(version: u8) -> usize {
    debug_assert!((1..=40).contains(&version));
    17 + 4 * version as usize
}

/// Number of data codewords for a given version and correction level.
pub(crate) fn data_codewords(version: u8, level: CorrectionLevel) -> usize {
    match (version, level) {
        (1, CorrectionLevel::Low) => 19,
        (1, CorrectionLevel::Medium) => 16,
        (1, CorrectionLevel::Quartile) => 13,
        (1, CorrectionLevel::High) => 9,
        _ => unreachable!("only version 1 is implemented"),
    }
}

/// Number of error correction codewords for a given version and level.
pub(crate) fn ecc_codewords(version: u8, level: CorrectionLevel) -> usize {
    VERSION_1_TOTAL_CODEWORDS - data_codewords(version, level)
}

/// Number of bits used by the character count indicator.
pub(crate) fn char_count_bits(version: u8, mode: Mode) -> u8 {
    match (version, mode) {
        (1..=9, Mode::Numeric) => 10,
        (1..=9, Mode::Alphanumeric) => 9,
        (1..=9, Mode::Byte) => 8,
        (1..=9, Mode::Kanji) => 8,
        (10..=26, Mode::Numeric) => 12,
        (10..=26, Mode::Alphanumeric) => 11,
        (10..=26, Mode::Byte) => 16,
        (10..=26, Mode::Kanji) => 10,
        (27..=40, Mode::Numeric) => 14,
        (27..=40, Mode::Alphanumeric) => 13,
        (27..=40, Mode::Byte) => 16,
        (27..=40, Mode::Kanji) => 12,
        _ => unreachable!("invalid version {version} or mode {mode:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_1_capacities_are_correct() {
        assert_eq!(data_codewords(1, CorrectionLevel::Low), 19);
        assert_eq!(data_codewords(1, CorrectionLevel::Medium), 16);
        assert_eq!(data_codewords(1, CorrectionLevel::Quartile), 13);
        assert_eq!(data_codewords(1, CorrectionLevel::High), 9);

        assert_eq!(ecc_codewords(1, CorrectionLevel::Low), 7);
        assert_eq!(ecc_codewords(1, CorrectionLevel::Medium), 10);
        assert_eq!(ecc_codewords(1, CorrectionLevel::Quartile), 13);
        assert_eq!(ecc_codewords(1, CorrectionLevel::High), 17);
    }

    #[test]
    fn version_size_is_21_for_version_1() {
        assert_eq!(version_size(1), 21);
        assert_eq!(version_size(40), 177);
    }

    #[test]
    fn character_count_bits_match_the_specification() {
        assert_eq!(char_count_bits(1, Mode::Numeric), 10);
        assert_eq!(char_count_bits(1, Mode::Alphanumeric), 9);
        assert_eq!(char_count_bits(1, Mode::Byte), 8);
        assert_eq!(char_count_bits(10, Mode::Numeric), 12);
        assert_eq!(char_count_bits(10, Mode::Byte), 16);
        assert_eq!(char_count_bits(27, Mode::Numeric), 14);
        assert_eq!(char_count_bits(27, Mode::Alphanumeric), 13);
    }
}
