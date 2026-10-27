//! QR Code matrix construction: function patterns, data placement and masks.

use crate::error::QrError;
use crate::qr::CorrectionLevel;

/// A square QR matrix.
///
/// `modules[y][x]` is `1` for a dark module and `0` for a light module.
#[derive(Debug, Clone)]
pub(crate) struct Matrix {
    size: usize,
    modules: Vec<Vec<u8>>,
    function: Vec<Vec<bool>>,
}

impl Matrix {
    pub(crate) fn new(size: usize) -> Self {
        Self {
            size,
            modules: vec![vec![0; size]; size],
            function: vec![vec![false; size]; size],
        }
    }

    /// Draws the fixed patterns shared by every mask candidate.
    pub(crate) fn draw_function_patterns(&mut self, version: u8) {
        debug_assert_eq!(self.size, 17 + 4 * version as usize);

        self.draw_finder(0, 0);
        self.draw_finder(self.size - 7, 0);
        self.draw_finder(0, self.size - 7);
        self.draw_timing_patterns();
        self.draw_alignment_patterns(version);

        if version >= 7 {
            self.draw_version_info(version);
        }

        self.reserve_format_info();
    }

    /// Draws one finder pattern and its light separator.
    fn draw_finder(&mut self, origin_x: usize, origin_y: usize) {
        let origin_x = origin_x as isize;
        let origin_y = origin_y as isize;
        let size = self.size as isize;

        for dy in -1isize..=7 {
            for dx in -1isize..=7 {
                let x = origin_x + dx;
                let y = origin_y + dy;

                if x < 0 || y < 0 || x >= size || y >= size {
                    continue;
                }

                let inside_finder = (0..7).contains(&dx) && (0..7).contains(&dy);
                let dark = inside_finder
                    && (dx == 0
                        || dx == 6
                        || dy == 0
                        || dy == 6
                        || ((2..=4).contains(&dx) && (2..=4).contains(&dy)));

                self.set_function(x as usize, y as usize, u8::from(dark));
            }
        }
    }

    fn draw_timing_patterns(&mut self) {
        for coordinate in 8..self.size - 8 {
            let value = u8::from(coordinate % 2 == 0);
            self.set_function(coordinate, 6, value);
            self.set_function(6, coordinate, value);
        }
    }

    /// Draws every alignment pattern except the three that would overlap a
    /// finder pattern.
    fn draw_alignment_patterns(&mut self, version: u8) {
        let positions = alignment_positions(version);
        let count = positions.len();

        for i in 0..count {
            for j in 0..count {
                let overlaps_finder =
                    (i == 0 && j == 0) || (i == 0 && j == count - 1) || (i == count - 1 && j == 0);

                if !overlaps_finder {
                    self.draw_alignment_pattern(positions[i], positions[j]);
                }
            }
        }
    }

    fn draw_alignment_pattern(&mut self, center_x: usize, center_y: usize) {
        for dy in -2isize..=2 {
            for dx in -2isize..=2 {
                let x = center_x as isize + dx;
                let y = center_y as isize + dy;

                if x < 0 || y < 0 || x >= self.size as isize || y >= self.size as isize {
                    continue;
                }

                // 5x5 pattern: dark border, light ring, dark centre.
                let dark = dx.abs().max(dy.abs()) != 1;
                self.set_function(x as usize, y as usize, u8::from(dark));
            }
        }
    }

    /// Draws the two copies of the 18-bit version information.
    fn draw_version_info(&mut self, version: u8) {
        let bits = version_info_bits(version);

        for index in 0..18 {
            let bit = ((bits >> index) & 1) as u8;
            let a = self.size - 11 + index % 3;
            let b = index / 3;

            self.set_function(a, b, bit);
            self.set_function(b, a, bit);
        }
    }

    /// Reserves both copies of the format information and the dark module.
    fn reserve_format_info(&mut self) {
        for i in 0..=5 {
            self.set_function(8, i, 0);
        }
        self.set_function(8, 7, 0);
        self.set_function(8, 8, 0);
        self.set_function(7, 8, 0);
        for i in 9..=14 {
            self.set_function(14 - i, 8, 0);
        }

        for i in 0..=7 {
            self.set_function(self.size - 1 - i, 8, 0);
        }
        for i in 8..=14 {
            self.set_function(8, self.size - 15 + i, 0);
        }

        // The dark module is always dark and is not part of the format bits.
        self.set_function(8, self.size - 8, 1);
    }

    fn draw_format_info(&mut self, level: CorrectionLevel, mask: u8) {
        let bits = format_info_bits(level, mask);
        let bit = |index: usize| ((bits >> index) & 1) as u8;

        for i in 0..=5 {
            self.set_function(8, i, bit(i));
        }
        self.set_function(8, 7, bit(6));
        self.set_function(8, 8, bit(7));
        self.set_function(7, 8, bit(8));
        for i in 9..=14 {
            self.set_function(14 - i, 8, bit(i));
        }

        for i in 0..=7 {
            self.set_function(self.size - 1 - i, 8, bit(i));
        }
        for i in 8..=14 {
            self.set_function(8, self.size - 15 + i, bit(i));
        }

        self.set_function(8, self.size - 8, 1);
    }

    /// Fills all non-function modules with the interleaved codewords.
    ///
    /// Bits are placed in two-module-wide columns, from right to left, in an
    /// alternating upward/downward zigzag. Column 6 (the vertical timing
    /// pattern) is skipped.
    pub(crate) fn place_data(&mut self, codewords: &[u8]) {
        let total_bits = codewords.len() * 8;
        let mut bit_index = 0usize;
        let mut right = self.size as isize - 1;

        while right >= 1 {
            if right == 6 {
                right = 5;
            }

            for vertical in 0..self.size {
                for offset in 0..2 {
                    let x = (right - offset) as usize;
                    let upward = ((right + 1) & 2) == 0;
                    let y = if upward {
                        self.size - 1 - vertical
                    } else {
                        vertical
                    };

                    if !self.is_function(x, y) {
                        let bit = if bit_index < total_bits {
                            (codewords[bit_index / 8] >> (7 - bit_index % 8)) & 1
                        } else {
                            0
                        };

                        self.modules[y][x] = bit;
                        bit_index += 1;
                    }
                }
            }

            right -= 2;
        }

        let function_modules = self
            .function
            .iter()
            .flatten()
            .filter(|&&is_function| is_function)
            .count();
        debug_assert_eq!(bit_index, self.size * self.size - function_modules);
    }

    fn apply_mask(&mut self, mask: u8) {
        for y in 0..self.size {
            for x in 0..self.size {
                if !self.is_function(x, y) && mask_bit(mask, x, y) {
                    self.modules[y][x] ^= 1;
                }
            }
        }
    }

    /// Standard QR mask penalty score (lower is better).
    fn penalty(&self) -> u32 {
        let size = self.size;
        let mut score = 0u32;

        // Rule 1: runs of five or more same-coloured modules.
        for y in 0..size {
            let mut run_length = 1;
            let mut run_color = self.modules[y][0];

            for x in 1..size {
                if self.modules[y][x] == run_color {
                    run_length += 1;
                } else {
                    if run_length >= 5 {
                        score += 3 + (run_length - 5) as u32;
                    }
                    run_length = 1;
                    run_color = self.modules[y][x];
                }
            }

            if run_length >= 5 {
                score += 3 + (run_length - 5) as u32;
            }
        }

        for x in 0..size {
            let mut run_length = 1;
            let mut run_color = self.modules[0][x];

            for y in 1..size {
                if self.modules[y][x] == run_color {
                    run_length += 1;
                } else {
                    if run_length >= 5 {
                        score += 3 + (run_length - 5) as u32;
                    }
                    run_length = 1;
                    run_color = self.modules[y][x];
                }
            }

            if run_length >= 5 {
                score += 3 + (run_length - 5) as u32;
            }
        }

        // Rule 2: 2x2 blocks of the same colour.
        for y in 0..size - 1 {
            for x in 0..size - 1 {
                let color = self.modules[y][x];
                if self.modules[y][x + 1] == color
                    && self.modules[y + 1][x] == color
                    && self.modules[y + 1][x + 1] == color
                {
                    score += 3;
                }
            }
        }

        // Rule 3: finder-like patterns. The four light modules of the quiet
        // zone are included by scanning a border of zeroes around the matrix.
        const PATTERN_A: u32 = 0b10111010000;
        const PATTERN_B: u32 = 0b00001011101;

        for y in 0..size {
            let mut window = 0u32;

            for x in 0..size + 8 {
                let bit = if (4..size + 4).contains(&x) {
                    u32::from(self.modules[y][x - 4])
                } else {
                    0
                };

                window = ((window << 1) | bit) & 0x7FF;
                if x >= 10 && (window == PATTERN_A || window == PATTERN_B) {
                    score += 40;
                }
            }
        }

        for x in 0..size {
            let mut window = 0u32;

            for y in 0..size + 8 {
                let bit = if (4..size + 4).contains(&y) {
                    u32::from(self.modules[y - 4][x])
                } else {
                    0
                };

                window = ((window << 1) | bit) & 0x7FF;
                if y >= 10 && (window == PATTERN_A || window == PATTERN_B) {
                    score += 40;
                }
            }
        }

        // Rule 4: dark/light balance. Ten points per 5% deviation from 50%.
        let dark_modules = self
            .modules
            .iter()
            .flatten()
            .copied()
            .filter(|&module| module == 1)
            .count() as u32;
        let total_modules = (size * size) as u32;
        let deviation = (dark_modules * 20).abs_diff(total_modules * 10) / total_modules;
        score += deviation * 10;

        score
    }

    fn set_function(&mut self, x: usize, y: usize, value: u8) {
        if x < self.size && y < self.size {
            self.modules[y][x] = value & 1;
            self.function[y][x] = true;
        }
    }

    fn is_function(&self, x: usize, y: usize) -> bool {
        self.function[y][x]
    }
}

/// Returns the alignment pattern centre coordinates for a version.
///
/// Alignment patterns use the same coordinates on both axes. Version 1 has no
/// alignment patterns.
pub(crate) fn alignment_positions(version: u8) -> Vec<usize> {
    if version == 1 {
        return Vec::new();
    }

    let version = version as usize;
    let num_align = version / 7 + 2;
    let step = (version * 8 + num_align * 3 + 5) / (num_align * 4 - 4) * 2;
    let size = 17 + 4 * version;

    let mut positions: Vec<usize> = (0..num_align - 1)
        .map(|index| size - 7 - index * step)
        .collect();
    positions.push(6);
    positions.reverse();
    positions
}

/// Applies one of the eight QR data mask formulas.
fn mask_bit(mask: u8, x: usize, y: usize) -> bool {
    match mask {
        0 => (x + y).is_multiple_of(2),
        1 => y.is_multiple_of(2),
        2 => x.is_multiple_of(3),
        3 => (x + y).is_multiple_of(3),
        4 => (y / 2 + x / 3).is_multiple_of(2),
        5 => ((x * y) % 2) + ((x * y) % 3) == 0,
        6 => (((x * y) % 2) + ((x * y) % 3)).is_multiple_of(2),
        7 => (((x + y) % 2) + ((x * y) % 3)).is_multiple_of(2),
        _ => unreachable!("mask must be in 0..=7"),
    }
}

/// Computes the 18-bit version information with BCH(18,6) error correction.
///
/// The returned value is indexed least-significant bit first, matching the
/// placement order defined by the standard.
pub(crate) fn version_info_bits(version: u8) -> u32 {
    debug_assert!((7..=40).contains(&version));

    let version = u32::from(version);
    let mut remainder = version << 12;

    for bit in (12..18).rev() {
        if remainder & (1u32 << bit) != 0 {
            remainder ^= 0x1F25u32 << (bit - 12);
        }
    }

    (version << 12) | (remainder & 0xFFF)
}

/// Computes the 15-bit format information with BCH(15,5) error correction.
///
/// The returned value is indexed least-significant bit first, matching the
/// placement order defined by the standard.
pub(crate) fn format_info_bits(level: CorrectionLevel, mask: u8) -> u16 {
    debug_assert!(mask <= 7);

    let data = (u16::from(level.format_bits()) << 3) | u16::from(mask);
    let mut remainder = data << 10;

    for bit in (10..15).rev() {
        if remainder & (1u16 << bit) != 0 {
            remainder ^= 0x537u16 << (bit - 10);
        }
    }

    ((data << 10) | (remainder & 0x3FF)) ^ 0x5412
}

/// Tries every mask (or uses `forced_mask` in tests) and keeps the matrix with
/// the lowest penalty score.
pub(crate) fn choose_mask(
    base: &Matrix,
    level: CorrectionLevel,
    forced_mask: Option<u8>,
) -> Result<(Vec<Vec<u8>>, u8), QrError> {
    if let Some(mask) = forced_mask {
        if mask > 7 {
            return Err(QrError::InvalidMask(mask));
        }

        let mut matrix = base.clone();
        matrix.apply_mask(mask);
        matrix.draw_format_info(level, mask);
        return Ok((matrix.modules, mask));
    }

    let mut best: Option<(u32, u8, Matrix)> = None;

    for mask in 0u8..8 {
        let mut matrix = base.clone();
        matrix.apply_mask(mask);
        matrix.draw_format_info(level, mask);
        let score = matrix.penalty();

        let is_better = match &best {
            Some((best_score, _, _)) => score < *best_score,
            None => true,
        };

        if is_better {
            best = Some((score, mask, matrix));
        }
    }

    let (_, mask, matrix) = best.expect("there are eight masks");
    Ok((matrix.modules, mask))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_info_matches_reference_values() {
        let expected = [
            (
                CorrectionLevel::Low,
                [
                    0x77C4, 0x72F3, 0x7DAA, 0x789D, 0x662F, 0x6318, 0x6C41, 0x6976,
                ],
            ),
            (
                CorrectionLevel::Medium,
                [
                    0x5412, 0x5125, 0x5E7C, 0x5B4B, 0x45F9, 0x40CE, 0x4F97, 0x4AA0,
                ],
            ),
            (
                CorrectionLevel::Quartile,
                [
                    0x355F, 0x3068, 0x3F31, 0x3A06, 0x24B4, 0x2183, 0x2EDA, 0x2BED,
                ],
            ),
            (
                CorrectionLevel::High,
                [
                    0x1689, 0x13BE, 0x1CE7, 0x19D0, 0x0762, 0x0255, 0x0D0C, 0x083B,
                ],
            ),
        ];

        for (level, values) in expected {
            for (mask, expected_value) in values.into_iter().enumerate() {
                assert_eq!(format_info_bits(level, mask as u8), expected_value);
            }
        }
    }

    #[test]
    fn finder_timing_and_dark_module_are_placed() {
        let mut matrix = Matrix::new(21);
        matrix.draw_function_patterns(1);

        // Finder pattern.
        assert_eq!(matrix.modules[0][0], 1);
        assert_eq!(matrix.modules[0][6], 1);
        assert_eq!(matrix.modules[1][1], 0);
        assert_eq!(matrix.modules[3][3], 1);

        // Separator below the top-left finder.
        assert_eq!(matrix.modules[7][0], 0);

        // Timing patterns.
        assert_eq!(matrix.modules[6][8], 1);
        assert_eq!(matrix.modules[6][9], 0);
        assert_eq!(matrix.modules[8][6], 1);
        assert_eq!(matrix.modules[9][6], 0);

        // Dark module.
        assert_eq!(matrix.modules[13][8], 1);
        assert!(matrix.function[13][8]);
    }

    #[test]
    fn function_modules_match_the_raw_data_capacity() {
        for version in 1..=40 {
            let size = 17 + 4 * version as usize;
            let mut matrix = Matrix::new(size);
            matrix.draw_function_patterns(version);

            let function_modules = matrix
                .function
                .iter()
                .flatten()
                .filter(|&&is_function| is_function)
                .count();

            assert_eq!(
                size * size - function_modules,
                crate::tables::raw_data_modules(version),
                "wrong number of data modules for version {version}"
            );
        }
    }

    #[test]
    fn version_info_matches_reference_values() {
        let expected = [
            (7, 0x07C94),
            (8, 0x085BC),
            (9, 0x09A99),
            (10, 0x0A4D3),
            (20, 0x149A6),
            (40, 0x28C69),
        ];

        for (version, bits) in expected {
            assert_eq!(version_info_bits(version), bits);
        }
    }

    #[test]
    fn version_info_is_placed_on_both_sides() {
        let mut matrix = Matrix::new(45);
        matrix.draw_function_patterns(7);
        let bits = version_info_bits(7);

        for index in 0..18 {
            let bit = ((bits >> index) & 1) as u8;
            let a = 45 - 11 + index % 3;
            let b = index / 3;

            assert_eq!(matrix.modules[b][a], bit);
            assert_eq!(matrix.modules[a][b], bit);
            assert!(matrix.function[b][a]);
            assert!(matrix.function[a][b]);
        }
    }

    #[test]
    fn alignment_positions_are_correct() {
        assert_eq!(alignment_positions(1), Vec::<usize>::new());
        assert_eq!(alignment_positions(2), vec![6, 18]);
        assert_eq!(alignment_positions(3), vec![6, 22]);
        assert_eq!(alignment_positions(6), vec![6, 34]);
        assert_eq!(alignment_positions(7), vec![6, 22, 38]);
        assert_eq!(alignment_positions(32), vec![6, 34, 60, 86, 112, 138]);
        assert_eq!(alignment_positions(40), vec![6, 30, 58, 86, 114, 142, 170]);
    }

    #[test]
    fn alignment_pattern_is_drawn_on_version_2() {
        let mut matrix = Matrix::new(25);
        matrix.draw_function_patterns(2);

        // Centre and rings of the alignment pattern at (18, 18).
        assert_eq!(matrix.modules[18][18], 1);
        assert_eq!(matrix.modules[18][17], 0);
        assert_eq!(matrix.modules[18][16], 1);
        assert_eq!(matrix.modules[17][17], 0);
        assert_eq!(matrix.modules[16][16], 1);

        assert!(matrix.function[18][18]);
        assert!(matrix.function[16][16]);
    }

    #[test]
    fn mask_formulas_match_the_specification() {
        assert!(mask_bit(0, 0, 0));
        assert!(!mask_bit(0, 1, 0));

        assert!(mask_bit(1, 0, 0));
        assert!(!mask_bit(1, 0, 1));

        assert!(mask_bit(2, 0, 0));
        assert!(!mask_bit(2, 1, 0));
        assert!(mask_bit(2, 3, 0));

        assert!(mask_bit(3, 0, 0));
        assert!(!mask_bit(3, 1, 0));
        assert!(mask_bit(3, 1, 2));

        assert!(mask_bit(4, 0, 0));
        assert!(!mask_bit(4, 3, 0));
        assert!(mask_bit(4, 3, 2));

        assert!(mask_bit(5, 0, 0));
        assert!(!mask_bit(5, 1, 1));
        assert!(mask_bit(5, 2, 3));

        assert!(mask_bit(6, 0, 0));
        assert!(mask_bit(6, 1, 1));
        assert!(!mask_bit(6, 2, 2));

        assert!(mask_bit(7, 0, 0));
        assert!(mask_bit(7, 2, 0));
        assert!(!mask_bit(7, 1, 1));
    }
}
