//! Arithmetic in the finite field GF(256) used by QR Code Reed-Solomon codes.
//!
//! QR Codes use the primitive polynomial `x^8 + x^4 + x^3 + x^2 + 1`
//! (`0x11D`) and the primitive element `α = 2`.

const PRIMITIVE_POLYNOMIAL: u16 = 0x11D;

const fn build_exp_log_tables() -> ([u8; 512], [u8; 256]) {
    let mut exp = [0u8; 512];
    let mut log = [0u8; 256];

    let mut value: u16 = 1;
    let mut exponent = 0;
    while exponent < 255 {
        exp[exponent] = value as u8;
        log[value as usize] = exponent as u8;

        value <<= 1;
        if value & 0x100 != 0 {
            value ^= PRIMITIVE_POLYNOMIAL;
        }
        exponent += 1;
    }

    // Duplicate the cycle so that exponent sums up to 508 fit in the table.
    let mut i = 255;
    while i < 512 {
        exp[i] = exp[i - 255];
        i += 1;
    }

    (exp, log)
}

const TABLES: ([u8; 512], [u8; 256]) = build_exp_log_tables();

/// Multiplies two field elements.
pub(crate) fn mul(a: u8, b: u8) -> u8 {
    if a == 0 || b == 0 {
        0
    } else {
        let log_a = TABLES.1[a as usize] as usize;
        let log_b = TABLES.1[b as usize] as usize;
        TABLES.0[log_a + log_b]
    }
}

/// Returns `α^exponent`.
pub(crate) fn pow(exponent: usize) -> u8 {
    TABLES.0[exponent % 255]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiplication_identities_hold() {
        for value in 0..=255u8 {
            assert_eq!(mul(0, value), 0);
            assert_eq!(mul(value, 0), 0);
            assert_eq!(mul(1, value), value);
            assert_eq!(mul(value, 1), value);
        }
    }

    #[test]
    fn powers_follow_the_primitive_polynomial() {
        assert_eq!(pow(0), 1);
        assert_eq!(pow(1), 2);
        assert_eq!(pow(7), 128);
        assert_eq!(pow(8), 0x1D);
        assert_eq!(pow(255), 1);
        assert_eq!(pow(256), 2);
    }

    #[test]
    fn multiplication_reduces_modulo_the_primitive_polynomial() {
        assert_eq!(mul(2, 0x80), 0x1D);
        assert_eq!(mul(3, 5), 15);
    }
}
