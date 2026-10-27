//! Bit-level encoding of QR Code data.

use crate::error::QrError;
use crate::qr::{CorrectionLevel, Mode};
use crate::tables;

/// A growable sequence of bits, each stored as `0` or `1`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct BitBuffer {
    bits: Vec<u8>,
}

impl BitBuffer {
    pub(crate) fn new() -> Self {
        Self { bits: Vec::new() }
    }

    pub(crate) fn len(&self) -> usize {
        self.bits.len()
    }

    /// Appends the `count` least significant bits of `value`, most significant
    /// bit first.
    pub(crate) fn push_bits(&mut self, value: u32, count: u8) {
        debug_assert!(count <= 32);
        debug_assert!(count == 32 || value < (1u32 << count));

        for shift in (0..count).rev() {
            self.bits.push(((value >> shift) & 1) as u8);
        }
    }

    /// Converts the buffer to bytes, padding the last byte with zeroes.
    pub(crate) fn into_bytes(self) -> Vec<u8> {
        let mut bytes = vec![0u8; self.bits.len().div_ceil(8)];

        for (index, &bit) in self.bits.iter().enumerate() {
            if bit != 0 {
                bytes[index / 8] |= 1 << (7 - index % 8);
            }
        }

        bytes
    }
}

/// Chooses the most compact single mode for `data`.
pub(crate) fn determine_mode(data: &str) -> Mode {
    if data.chars().all(|character| character.is_ascii_digit()) {
        Mode::Numeric
    } else if data
        .chars()
        .all(|character| character.is_ascii() && alphanumeric_value(character as u8).is_some())
    {
        Mode::Alphanumeric
    } else {
        Mode::Byte
    }
}

/// Maps a character to its QR alphanumeric value.
fn alphanumeric_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'A'..=b'Z' => Some(byte - b'A' + 10),
        b' ' => Some(36),
        b'$' => Some(37),
        b'%' => Some(38),
        b'*' => Some(39),
        b'+' => Some(40),
        b'-' => Some(41),
        b'.' => Some(42),
        b'/' => Some(43),
        b':' => Some(44),
        _ => None,
    }
}

fn validate_numeric(data: &str) -> Result<(), QrError> {
    if let Some(character) = data.chars().find(|character| !character.is_ascii_digit()) {
        return Err(QrError::InvalidCharacter(character));
    }
    Ok(())
}

fn validate_alphanumeric(data: &str) -> Result<(), QrError> {
    for character in data.chars() {
        if !character.is_ascii() || alphanumeric_value(character as u8).is_none() {
            return Err(QrError::InvalidCharacter(character));
        }
    }
    Ok(())
}

fn data_bit_count(data: &str, mode: Mode) -> usize {
    let count = data.len();

    match mode {
        Mode::Numeric => {
            (count / 3) * 10
                + match count % 3 {
                    0 => 0,
                    1 => 4,
                    2 => 7,
                    _ => unreachable!(),
                }
        }
        Mode::Alphanumeric => (count / 2) * 11 + (count % 2) * 6,
        Mode::Byte => count * 8,
        Mode::Kanji => unreachable!("Kanji mode is rejected before counting"),
    }
}

/// Encodes `data` with one mode, including the mode indicator and character
/// count indicator.
pub(crate) fn encode_data(
    data: &str,
    mode: Mode,
    version: u8,
    level: CorrectionLevel,
) -> Result<BitBuffer, QrError> {
    match mode {
        Mode::Numeric => validate_numeric(data)?,
        Mode::Alphanumeric => validate_alphanumeric(data)?,
        Mode::Byte => {}
        Mode::Kanji => return Err(QrError::UnsupportedMode(mode)),
    }

    let count_bits = tables::char_count_bits(version, mode) as usize;
    let needed_bits = 4 + count_bits + data_bit_count(data, mode);
    let capacity_bits = tables::data_codewords(version, level) * 8;

    if needed_bits > capacity_bits {
        return Err(QrError::DataTooLong {
            version,
            level,
            needed_bits,
            capacity_bits,
        });
    }

    let mut bits = BitBuffer::new();
    bits.push_bits(u32::from(mode.indicator()), 4);
    bits.push_bits(data.len() as u32, count_bits as u8);

    match mode {
        Mode::Numeric => encode_numeric(data, &mut bits),
        Mode::Alphanumeric => encode_alphanumeric(data, &mut bits),
        Mode::Byte => encode_byte(data, &mut bits),
        Mode::Kanji => unreachable!(),
    }

    debug_assert_eq!(bits.len(), needed_bits);
    Ok(bits)
}

fn encode_numeric(data: &str, bits: &mut BitBuffer) {
    for chunk in data.as_bytes().chunks(3) {
        let mut value = 0u32;
        for &digit in chunk {
            value = value * 10 + u32::from(digit - b'0');
        }

        let bit_count = match chunk.len() {
            1 => 4,
            2 => 7,
            3 => 10,
            _ => unreachable!(),
        };
        bits.push_bits(value, bit_count);
    }
}

fn encode_alphanumeric(data: &str, bits: &mut BitBuffer) {
    for chunk in data.as_bytes().chunks(2) {
        let first =
            u32::from(alphanumeric_value(chunk[0]).expect("alphanumeric input was validated"));

        if chunk.len() == 2 {
            let second =
                u32::from(alphanumeric_value(chunk[1]).expect("alphanumeric input was validated"));
            bits.push_bits(first * 45 + second, 11);
        } else {
            bits.push_bits(first, 6);
        }
    }
}

fn encode_byte(data: &str, bits: &mut BitBuffer) {
    for &byte in data.as_bytes() {
        bits.push_bits(u32::from(byte), 8);
    }
}

/// Appends the terminator, pads to a byte boundary, then alternates the QR
/// padding bytes `0xEC` and `0x11` until the data capacity is full.
pub(crate) fn finish(
    mut bits: BitBuffer,
    capacity_bits: usize,
    version: u8,
    level: CorrectionLevel,
) -> Result<BitBuffer, QrError> {
    if bits.len() > capacity_bits {
        return Err(QrError::DataTooLong {
            version,
            level,
            needed_bits: bits.len(),
            capacity_bits,
        });
    }

    let terminator_bits = (capacity_bits - bits.len()).min(4);
    bits.push_bits(0, terminator_bits as u8);

    let padding_to_byte = (8 - bits.len() % 8) % 8;
    bits.push_bits(0, padding_to_byte as u8);

    let pad_bytes = [0xECu8, 0x11u8];
    let mut pad_index = 0;
    while bits.len() < capacity_bits {
        bits.push_bits(u32::from(pad_bytes[pad_index]), 8);
        pad_index ^= 1;
    }

    Ok(bits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_iso_example() {
        let bits = encode_data("01234567", Mode::Numeric, 1, CorrectionLevel::Medium).unwrap();

        assert_eq!(bits.into_bytes(), vec![0x10, 0x20, 0x0C, 0x56, 0x61, 0x80]);
    }

    #[test]
    fn alphanumeric_hello_world_is_padded_like_the_reference() {
        let bits = encode_data(
            "HELLO WORLD",
            Mode::Alphanumeric,
            1,
            CorrectionLevel::Quartile,
        )
        .unwrap();
        let capacity_bits = tables::data_codewords(1, CorrectionLevel::Quartile) * 8;
        let bits = finish(bits, capacity_bits, 1, CorrectionLevel::Quartile).unwrap();

        assert_eq!(
            bits.into_bytes(),
            vec![
                0x20, 0x5B, 0x0B, 0x78, 0xD1, 0x72, 0xDC, 0x4D, 0x43, 0x40, 0xEC, 0x11, 0xEC,
            ]
        );
    }

    #[test]
    fn modes_are_detected_correctly() {
        assert_eq!(determine_mode("012345"), Mode::Numeric);
        assert_eq!(determine_mode("HELLO 123"), Mode::Alphanumeric);
        assert_eq!(determine_mode("HELLO$%*+-./:"), Mode::Alphanumeric);
        assert_eq!(determine_mode("Hello"), Mode::Byte);
        assert_eq!(determine_mode("é"), Mode::Byte);
    }

    #[test]
    fn numeric_mode_rejects_non_digits() {
        assert_eq!(
            encode_data("12A4", Mode::Numeric, 1, CorrectionLevel::Low),
            Err(QrError::InvalidCharacter('A'))
        );
    }

    #[test]
    fn alphanumeric_mode_rejects_lowercase() {
        assert_eq!(
            encode_data("Hello", Mode::Alphanumeric, 1, CorrectionLevel::Low),
            Err(QrError::InvalidCharacter('e'))
        );
    }

    #[test]
    fn data_too_long_is_reported_before_encoding() {
        let result = encode_data(
            &"A".repeat(100),
            Mode::Alphanumeric,
            1,
            CorrectionLevel::High,
        );

        assert!(matches!(
            result,
            Err(QrError::DataTooLong {
                version: 1,
                level: CorrectionLevel::High,
                ..
            })
        ));
    }

    #[test]
    fn finish_adds_terminator_and_padding() {
        let bits = encode_data("1", Mode::Numeric, 1, CorrectionLevel::Low).unwrap();
        let capacity_bits = tables::data_codewords(1, CorrectionLevel::Low) * 8;
        let bits = finish(bits, capacity_bits, 1, CorrectionLevel::Low).unwrap();

        assert_eq!(bits.len(), capacity_bits);
        assert_eq!(bits.into_bytes().len(), 19);
    }
}
