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
}
