//! Reed-Solomon encoding over GF(256), as used by QR Codes.

use crate::gf256;

/// Builds the generator polynomial `∏(x - α^i)` for `i in 0..ecc_len`.
///
/// Coefficients are stored from the highest degree to the lowest degree.
pub(crate) fn generator_polynomial(ecc_len: usize) -> Vec<u8> {
    let mut generator = vec![1u8];

    for i in 0..ecc_len {
        let factor = gf256::pow(i);
        let mut next = vec![0u8; generator.len() + 1];

        for (degree, &coefficient) in generator.iter().enumerate() {
            // x * generator
            next[degree] ^= coefficient;
            // α^i * generator
            next[degree + 1] ^= gf256::mul(coefficient, factor);
        }

        generator = next;
    }

    generator
}

/// Returns the `ecc_len` error correction codewords for `data`.
pub(crate) fn encode(data: &[u8], ecc_len: usize) -> Vec<u8> {
    let generator = generator_polynomial(ecc_len);

    // Multiply the message by x^ecc_len, then divide by the generator.
    let mut message = Vec::with_capacity(data.len() + ecc_len);
    message.extend_from_slice(data);
    message.resize(data.len() + ecc_len, 0);

    for i in 0..data.len() {
        let coefficient = message[i];
        if coefficient == 0 {
            continue;
        }

        for (offset, &generator_coefficient) in generator.iter().enumerate() {
            message[i + offset] ^= gf256::mul(generator_coefficient, coefficient);
        }
    }

    message[data.len()..].to_vec()
}

/// Splits the data codewords into QR blocks, computes the ECC codewords for
/// each block, then interleaves data and ECC codewords as required by the
/// standard.
///
/// Short blocks come first. The number of long blocks is
/// `total_codewords % num_blocks`.
pub(crate) fn encode_blocks(
    data: &[u8],
    ecc_len: usize,
    num_blocks: usize,
    total_codewords: usize,
) -> Vec<u8> {
    debug_assert!(num_blocks > 0);
    debug_assert_eq!(data.len() + ecc_len * num_blocks, total_codewords);

    let num_long_blocks = total_codewords % num_blocks;
    let num_short_blocks = num_blocks - num_long_blocks;
    let short_block_len = total_codewords / num_blocks;
    let short_data_len = short_block_len - ecc_len;

    let mut blocks: Vec<(&[u8], Vec<u8>)> = Vec::with_capacity(num_blocks);
    let mut offset = 0;

    for block_index in 0..num_blocks {
        let extra_codeword = if block_index >= num_short_blocks {
            1
        } else {
            0
        };
        let data_len = short_data_len + extra_codeword;
        let block_data = &data[offset..offset + data_len];
        let block_ecc = encode(block_data, ecc_len);

        blocks.push((block_data, block_ecc));
        offset += data_len;
    }

    debug_assert_eq!(offset, data.len());

    let max_data_len = short_data_len + 1;
    let mut result = Vec::with_capacity(total_codewords);

    // Interleave data codewords.
    for index in 0..max_data_len {
        for (block_data, _) in &blocks {
            if let Some(&byte) = block_data.get(index) {
                result.push(byte);
            }
        }
    }

    // Interleave ECC codewords.
    for index in 0..ecc_len {
        for (_, block_ecc) in &blocks {
            result.push(block_ecc[index]);
        }
    }

    debug_assert_eq!(result.len(), total_codewords);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evaluate(polynomial: &[u8], x: u8) -> u8 {
        let mut result = 0u8;
        for &coefficient in polynomial {
            result = gf256::mul(result, x) ^ coefficient;
        }
        result
    }

    #[test]
    fn generator_polynomial_for_7_ecc_codewords() {
        let generator = generator_polynomial(7);
        let expected_exponents = [0usize, 87, 229, 146, 149, 238, 102, 21];

        assert_eq!(generator.len(), expected_exponents.len());
        for (&coefficient, exponent) in generator.iter().zip(expected_exponents) {
            assert_eq!(coefficient, gf256::pow(exponent));
        }
    }

    #[test]
    fn known_hello_world_version_1_q_vector() {
        let data = [32, 91, 11, 120, 209, 114, 220, 77, 67, 64, 236, 17, 236];
        let expected_ecc = [168, 72, 22, 82, 217, 54, 156, 0, 46, 15, 180, 122, 16];

        assert_eq!(encode(&data, 13), expected_ecc);
    }

    #[test]
    fn every_codeword_is_divisible_by_the_generator() {
        let data = b"HELLO WORLD";
        let ecc_len = 13;
        let ecc = encode(data, ecc_len);

        let mut codeword = data.to_vec();
        codeword.extend_from_slice(&ecc);

        for i in 0..ecc_len {
            assert_eq!(evaluate(&codeword, gf256::pow(i)), 0);
        }
    }

    #[test]
    fn block_encoding_produces_the_expected_number_of_codewords() {
        use crate::qr::CorrectionLevel;
        use crate::tables;

        let cases = [
            (1, CorrectionLevel::Low),
            (2, CorrectionLevel::Low),
            (3, CorrectionLevel::Quartile),
            (5, CorrectionLevel::Quartile),
            (6, CorrectionLevel::High),
        ];

        for (version, level) in cases {
            let data_len = tables::data_codewords(version, level);
            let ecc_len = tables::ecc_codewords(version, level);
            let num_blocks = tables::num_blocks(version, level);
            let total = tables::total_codewords(version);
            let data = vec![0u8; data_len];

            let codewords = encode_blocks(&data, ecc_len, num_blocks, total);

            assert_eq!(codewords.len(), total);
        }
    }
}
