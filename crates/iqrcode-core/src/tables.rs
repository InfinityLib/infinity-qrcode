use crate::qr::{CorrectionLevel, Mode};

/// Number of ECC codewords per block, indexed by correction level and version.
///
/// The first column is padding because QR versions start at 1.
/// Values come from ISO/IEC 18004, Table 9.
const ECC_CODEWORDS_PER_BLOCK: [[i8; 41]; 4] = [
    [
        -1, 7, 10, 15, 20, 26, 18, 20, 24, 30, 18, 20, 24, 26, 30, 22, 24, 28, 30, 28, 28, 28, 28,
        30, 30, 26, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
    ],
    [
        -1, 10, 16, 26, 18, 24, 16, 18, 22, 22, 26, 30, 22, 22, 24, 24, 28, 28, 26, 26, 26, 26, 28,
        28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28,
    ],
    [
        -1, 13, 22, 18, 26, 18, 24, 18, 22, 20, 24, 28, 26, 24, 20, 30, 24, 28, 28, 26, 30, 28, 30,
        30, 30, 30, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
    ],
    [
        -1, 17, 28, 22, 16, 22, 28, 26, 26, 24, 28, 24, 28, 22, 24, 24, 30, 28, 28, 26, 28, 30, 24,
        30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
    ],
];

/// Number of error correction blocks, indexed by correction level and version.
///
/// The first column is padding because QR versions start at 1.
/// Values come from ISO/IEC 18004, Table 9.
const NUM_ERROR_CORRECTION_BLOCKS: [[i8; 41]; 4] = [
    [
        -1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 4, 4, 4, 4, 4, 6, 6, 6, 6, 7, 8, 8, 9, 9, 10, 12, 12, 12,
        13, 14, 15, 16, 17, 18, 19, 19, 20, 21, 22, 24, 25,
    ],
    [
        -1, 1, 1, 1, 2, 2, 4, 4, 4, 5, 5, 5, 8, 9, 9, 10, 10, 11, 13, 14, 16, 17, 17, 18, 20, 21,
        23, 25, 26, 28, 29, 31, 33, 35, 37, 38, 40, 43, 45, 47, 49,
    ],
    [
        -1, 1, 1, 2, 2, 4, 4, 6, 6, 8, 8, 8, 10, 12, 16, 12, 17, 16, 18, 21, 20, 23, 23, 25, 27,
        29, 34, 34, 35, 38, 40, 43, 45, 48, 51, 53, 56, 59, 62, 65, 68,
    ],
    [
        -1, 1, 1, 2, 4, 4, 4, 5, 6, 8, 8, 11, 11, 16, 16, 18, 16, 19, 21, 25, 25, 25, 34, 30, 32,
        35, 37, 40, 42, 45, 48, 51, 54, 57, 60, 63, 66, 70, 74, 77, 81,
    ],
];

fn level_index(level: CorrectionLevel) -> usize {
    match level {
        CorrectionLevel::Low => 0,
        CorrectionLevel::Medium => 1,
        CorrectionLevel::Quartile => 2,
        CorrectionLevel::High => 3,
    }
}

/// Side length of the QR matrix for a given version.
pub(crate) fn version_size(version: u8) -> usize {
    debug_assert!((1..=40).contains(&version));
    17 + 4 * version as usize
}

/// Number of modules available for data and ECC codewords, remainder bits
/// included.
pub(crate) fn raw_data_modules(version: u8) -> usize {
    let version = version as usize;
    let mut result = (16 * version + 128) * version + 64;

    if version >= 2 {
        let num_align = version / 7 + 2;
        result -= (25 * num_align - 10) * num_align - 55;

        if version >= 7 {
            result -= 36;
        }
    }

    result
}

/// Number of codewords (data + ECC) in a version.
pub(crate) fn total_codewords(version: u8) -> usize {
    raw_data_modules(version) / 8
}

/// Number of ECC codewords in each block.
pub(crate) fn ecc_codewords(version: u8, level: CorrectionLevel) -> usize {
    ECC_CODEWORDS_PER_BLOCK[level_index(level)][version as usize] as usize
}

/// Number of error correction blocks.
pub(crate) fn num_blocks(version: u8, level: CorrectionLevel) -> usize {
    NUM_ERROR_CORRECTION_BLOCKS[level_index(level)][version as usize] as usize
}

/// Number of data codewords for a given version and correction level.
pub(crate) fn data_codewords(version: u8, level: CorrectionLevel) -> usize {
    total_codewords(version) - ecc_codewords(version, level) * num_blocks(version, level)
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
    fn versions_2_to_6_capacities_are_correct() {
        let expected = [
            // version, L, M, Q, H
            (2, 34, 28, 22, 16),
            (3, 55, 44, 34, 26),
            (4, 80, 64, 48, 36),
            (5, 108, 86, 62, 46),
            (6, 136, 108, 76, 60),
        ];

        for (version, low, medium, quartile, high) in expected {
            assert_eq!(
                total_codewords(version),
                match version {
                    2 => 44,
                    3 => 70,
                    4 => 100,
                    5 => 134,
                    6 => 172,
                    _ => unreachable!(),
                }
            );
            assert_eq!(data_codewords(version, CorrectionLevel::Low), low);
            assert_eq!(data_codewords(version, CorrectionLevel::Medium), medium);
            assert_eq!(data_codewords(version, CorrectionLevel::Quartile), quartile);
            assert_eq!(data_codewords(version, CorrectionLevel::High), high);
        }
    }

    #[test]
    fn raw_data_module_counts_match_the_specification() {
        let expected = [
            (1, 208),
            (2, 359),
            (3, 567),
            (4, 807),
            (5, 1079),
            (6, 1383),
            (7, 1568),
            (40, 29648),
        ];

        for (version, modules) in expected {
            assert_eq!(raw_data_modules(version), modules);
        }
    }

    #[test]
    fn version_40_capacities_are_correct() {
        assert_eq!(total_codewords(40), 3706);
        assert_eq!(data_codewords(40, CorrectionLevel::Low), 2956);
        assert_eq!(data_codewords(40, CorrectionLevel::High), 1276);
        assert_eq!(num_blocks(40, CorrectionLevel::High), 81);
    }

    #[test]
    fn version_size_is_correct() {
        assert_eq!(version_size(1), 21);
        assert_eq!(version_size(2), 25);
        assert_eq!(version_size(6), 41);
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
