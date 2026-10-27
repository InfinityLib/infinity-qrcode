//! Bit-level encoding of QR Code segments.

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

/// One segment of a QR Code data stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Segment<'a> {
    /// An ECI designator. It is not a data segment but is part of the stream.
    Eci(u32),
    /// A data segment encoded in one mode.
    Data { mode: Mode, data: &'a [u8] },
}

/// Chooses the most compact single mode for `data`.
#[cfg(test)]
pub(crate) fn determine_mode(data: &str) -> Mode {
    if data.bytes().all(is_numeric_byte) {
        Mode::Numeric
    } else if data.bytes().all(is_alphanumeric_byte) {
        Mode::Alphanumeric
    } else {
        Mode::Byte
    }
}

pub(crate) fn is_numeric_byte(byte: u8) -> bool {
    byte.is_ascii_digit()
}

pub(crate) fn is_alphanumeric_byte(byte: u8) -> bool {
    alphanumeric_value(byte).is_some()
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

fn validate_numeric(data: &[u8]) -> Result<(), QrError> {
    for &byte in data {
        if !is_numeric_byte(byte) {
            return Err(QrError::InvalidCharacter(char::from(byte)));
        }
    }
    Ok(())
}

fn validate_alphanumeric(data: &[u8]) -> Result<(), QrError> {
    for &byte in data {
        if !is_alphanumeric_byte(byte) {
            return Err(QrError::InvalidCharacter(char::from(byte)));
        }
    }
    Ok(())
}

fn validate_kanji(data: &[u8]) -> Result<(), QrError> {
    if !data.len().is_multiple_of(2) {
        return Err(QrError::InvalidKanjiData);
    }

    for pair in data.as_chunks::<2>().0 {
        let code = (u16::from(pair[0]) << 8) | u16::from(pair[1]);
        let valid = (0x8140..=0x9FFC).contains(&code) || (0xE040..=0xEBBF).contains(&code);

        if !valid {
            return Err(QrError::InvalidKanjiData);
        }
    }

    Ok(())
}

fn validate_segment_data(mode: Mode, data: &[u8]) -> Result<(), QrError> {
    match mode {
        Mode::Numeric => validate_numeric(data),
        Mode::Alphanumeric => validate_alphanumeric(data),
        Mode::Byte => Ok(()),
        Mode::Kanji => validate_kanji(data),
    }
}

fn numeric_bit_count(length: usize) -> usize {
    (length / 3) * 10
        + match length % 3 {
            0 => 0,
            1 => 4,
            2 => 7,
            _ => unreachable!(),
        }
}

fn alphanumeric_bit_count(length: usize) -> usize {
    (length / 2) * 11 + (length % 2) * 6
}

fn segment_char_count(mode: Mode, data_len: usize) -> usize {
    if mode == Mode::Kanji {
        data_len / 2
    } else {
        data_len
    }
}

/// Number of bits for a segment, without validating its contents.
///
/// The caller must ensure that `data_len` is valid for `mode`.
pub(crate) fn segment_bits_unchecked(mode: Mode, data_len: usize, version: u8) -> usize {
    let count_bits = tables::char_count_bits(version, mode) as usize;
    let data_bits = match mode {
        Mode::Numeric => numeric_bit_count(data_len),
        Mode::Alphanumeric => alphanumeric_bit_count(data_len),
        Mode::Byte => data_len * 8,
        Mode::Kanji => (data_len / 2) * 13,
    };

    4 + count_bits + data_bits
}

fn eci_designator_bits(designator: u32) -> Result<usize, QrError> {
    match designator {
        0..=127 => Ok(8),
        128..=16_383 => Ok(16),
        16_384..=999_999 => Ok(24),
        _ => Err(QrError::InvalidEciDesignator(designator)),
    }
}

/// Number of bits for a segment, including validation.
pub(crate) fn segment_bits(segment: &Segment<'_>, version: u8) -> Result<usize, QrError> {
    match segment {
        Segment::Eci(designator) => Ok(4 + eci_designator_bits(*designator)?),
        Segment::Data { mode, data } => {
            validate_segment_data(*mode, data)?;

            let count_bits = tables::char_count_bits(version, *mode) as usize;
            let count = segment_char_count(*mode, data.len());

            if count >= (1usize << count_bits) {
                return Err(QrError::CharacterCountOverflow {
                    mode: *mode,
                    version,
                    count,
                });
            }

            Ok(segment_bits_unchecked(*mode, data.len(), version))
        }
    }
}

/// Total number of bits needed by a sequence of segments.
pub(crate) fn segments_bits(segments: &[Segment<'_>], version: u8) -> Result<usize, QrError> {
    let mut total = 0;

    for segment in segments {
        total += segment_bits(segment, version)?;
    }

    Ok(total)
}

/// Encodes one segment (mode indicator, count and data).
#[cfg(test)]
pub(crate) fn encode_data(
    data: &str,
    mode: Mode,
    version: u8,
    level: CorrectionLevel,
) -> Result<BitBuffer, QrError> {
    encode_segments(
        &[Segment::Data {
            mode,
            data: data.as_bytes(),
        }],
        version,
        level,
    )
}

/// Encodes a sequence of segments, checking that they fit in the selected
/// version and correction level.
pub(crate) fn encode_segments(
    segments: &[Segment<'_>],
    version: u8,
    level: CorrectionLevel,
) -> Result<BitBuffer, QrError> {
    let needed_bits = segments_bits(segments, version)?;
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

    for segment in segments {
        push_segment(segment, &mut bits, version)?;
    }

    debug_assert_eq!(bits.len(), needed_bits);
    Ok(bits)
}

fn push_segment(segment: &Segment<'_>, bits: &mut BitBuffer, version: u8) -> Result<(), QrError> {
    match segment {
        Segment::Eci(designator) => {
            bits.push_bits(0b0111, 4);
            push_eci_designator(*designator, bits)?;
        }
        Segment::Data { mode, data } => {
            validate_segment_data(*mode, data)?;

            let count_bits = tables::char_count_bits(version, *mode) as usize;
            let count = segment_char_count(*mode, data.len());

            bits.push_bits(u32::from(mode.indicator()), 4);
            bits.push_bits(count as u32, count_bits as u8);

            match mode {
                Mode::Numeric => push_numeric(data, bits),
                Mode::Alphanumeric => push_alphanumeric(data, bits),
                Mode::Byte => push_byte(data, bits),
                Mode::Kanji => push_kanji(data, bits),
            }
        }
    }

    Ok(())
}

fn push_eci_designator(designator: u32, bits: &mut BitBuffer) -> Result<(), QrError> {
    match designator {
        0..=127 => bits.push_bits(designator, 8),
        128..=16_383 => {
            bits.push_bits(0b10, 2);
            bits.push_bits(designator, 14);
        }
        16_384..=999_999 => {
            bits.push_bits(0b110, 3);
            bits.push_bits(designator, 21);
        }
        _ => return Err(QrError::InvalidEciDesignator(designator)),
    }

    Ok(())
}

fn push_numeric(data: &[u8], bits: &mut BitBuffer) {
    for chunk in data.chunks(3) {
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

fn push_alphanumeric(data: &[u8], bits: &mut BitBuffer) {
    for chunk in data.chunks(2) {
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

fn push_byte(data: &[u8], bits: &mut BitBuffer) {
    for &byte in data {
        bits.push_bits(u32::from(byte), 8);
    }
}

fn push_kanji(data: &[u8], bits: &mut BitBuffer) {
    for pair in data.as_chunks::<2>().0 {
        let code = (u16::from(pair[0]) << 8) | u16::from(pair[1]);
        let adjusted = if code < 0xE040 {
            code - 0x8140
        } else {
            code - 0xC140
        };
        let value = u32::from(adjusted >> 8) * 0xC0 + u32::from(adjusted & 0xFF);

        bits.push_bits(value, 13);
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
    fn kanji_iso_example() {
        let segments = [Segment::Data {
            mode: Mode::Kanji,
            data: b"\x93\x5f\xe4\xaa",
        }];
        let bits = encode_segments(&segments, 1, CorrectionLevel::Low).unwrap();

        assert_eq!(bits.into_bytes(), vec![0x80, 0x26, 0xCF, 0xEA, 0xA8]);
    }

    #[test]
    fn eci_26_is_encoded_correctly() {
        let segments = [
            Segment::Eci(26),
            Segment::Data {
                mode: Mode::Byte,
                data: b"A",
            },
        ];
        let bits = encode_segments(&segments, 1, CorrectionLevel::Low).unwrap();

        assert_eq!(bits.into_bytes(), vec![0x71, 0xA4, 0x01, 0x41]);
    }

    #[test]
    fn invalid_kanji_data_is_rejected() {
        let odd = [Segment::Data {
            mode: Mode::Kanji,
            data: b"\x93",
        }];
        assert_eq!(
            encode_segments(&odd, 1, CorrectionLevel::Low),
            Err(QrError::InvalidKanjiData)
        );

        let out_of_range = [Segment::Data {
            mode: Mode::Kanji,
            data: b"\x00\x00",
        }];
        assert_eq!(
            encode_segments(&out_of_range, 1, CorrectionLevel::Low),
            Err(QrError::InvalidKanjiData)
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
