//! Dynamic-programming segmentation for mixed-mode QR Codes.

use crate::bits::{Segment, is_alphanumeric_byte, is_numeric_byte, segment_bits_unchecked};
use crate::qr::Mode;
use crate::tables;

/// Returns a segmentation of `data` that minimises the number of encoded bits
/// for the given version.
///
/// Kanji is not considered automatically because it requires Shift-JIS input.
pub(crate) fn optimal_segments<'a>(data: &'a str, version: u8) -> Vec<Segment<'a>> {
    let bytes = data.as_bytes();
    let length = bytes.len();

    if length == 0 {
        return vec![Segment::Data {
            mode: Mode::Byte,
            data: bytes,
        }];
    }

    let mut costs = vec![usize::MAX; length + 1];
    let mut choices: Vec<Option<(usize, Mode)>> = vec![None; length + 1];
    costs[0] = 0;

    let max_numeric = max_count(version, Mode::Numeric);
    let max_alphanumeric = max_count(version, Mode::Alphanumeric);
    let max_byte = max_count(version, Mode::Byte);

    for end in 1..=length {
        if !data.is_char_boundary(end) {
            continue;
        }

        // Byte segments can start at any character boundary.
        for start in (0..end).rev() {
            let segment_len = end - start;

            if segment_len > max_byte {
                break;
            }
            if costs[start] == usize::MAX || !data.is_char_boundary(start) {
                continue;
            }

            let cost = costs[start] + segment_bits_unchecked(Mode::Byte, segment_len, version);
            update(&mut costs, &mut choices, end, start, Mode::Byte, cost);
        }

        // Numeric segments extend over a contiguous run of digits.
        let mut start = end;
        let mut segment_len = 0;

        while start > 0 && is_numeric_byte(bytes[start - 1]) {
            if segment_len == max_numeric {
                break;
            }

            start -= 1;
            segment_len += 1;

            if costs[start] == usize::MAX {
                continue;
            }

            let cost = costs[start] + segment_bits_unchecked(Mode::Numeric, segment_len, version);
            update(&mut costs, &mut choices, end, start, Mode::Numeric, cost);
        }

        // Alphanumeric segments extend over a contiguous QR-alphanumeric run.
        let mut start = end;
        let mut segment_len = 0;

        while start > 0 && is_alphanumeric_byte(bytes[start - 1]) {
            if segment_len == max_alphanumeric {
                break;
            }

            start -= 1;
            segment_len += 1;

            if costs[start] == usize::MAX {
                continue;
            }

            let cost =
                costs[start] + segment_bits_unchecked(Mode::Alphanumeric, segment_len, version);
            update(
                &mut costs,
                &mut choices,
                end,
                start,
                Mode::Alphanumeric,
                cost,
            );
        }
    }

    let mut segments = Vec::new();
    let mut position = length;

    while position > 0 {
        let (start, mode) = choices[position].expect("data should be encodable in Byte mode");
        segments.push(Segment::Data {
            mode,
            data: &bytes[start..position],
        });
        position = start;
    }

    segments.reverse();
    segments
}

fn max_count(version: u8, mode: Mode) -> usize {
    (1usize << tables::char_count_bits(version, mode)) - 1
}

fn update(
    costs: &mut [usize],
    choices: &mut [Option<(usize, Mode)>],
    end: usize,
    start: usize,
    mode: Mode,
    cost: usize,
) {
    if cost < costs[end] {
        costs[end] = cost;
        choices[end] = Some((start, mode));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bits::segments_bits;

    fn modes(segments: &[Segment<'_>]) -> Vec<Mode> {
        segments
            .iter()
            .map(|segment| match segment {
                Segment::Data { mode, .. } => *mode,
                Segment::Eci(_) => unreachable!("the optimiser never emits ECI segments"),
            })
            .collect()
    }

    fn joined(segments: &[Segment<'_>]) -> Vec<u8> {
        let mut result = Vec::new();

        for segment in segments {
            if let Segment::Data { data, .. } = segment {
                result.extend_from_slice(data);
            }
        }

        result
    }

    #[test]
    fn keeps_homogeneous_data_in_one_segment() {
        assert_eq!(
            modes(&optimal_segments("0123456789", 1)),
            vec![Mode::Numeric]
        );
        assert_eq!(
            modes(&optimal_segments("HELLO WORLD", 1)),
            vec![Mode::Alphanumeric]
        );
        assert_eq!(modes(&optimal_segments("hello", 1)), vec![Mode::Byte]);
    }

    #[test]
    fn splits_mixed_data_into_multiple_segments() {
        let data = "HELLO12345678901234567890world";
        let segments = optimal_segments(data, 1);

        assert!(segments.len() > 1);
        assert_eq!(joined(&segments), data.as_bytes());
    }

    #[test]
    fn never_uses_more_bits_than_a_single_byte_segment() {
        let data = "ABC12345678901234567890xyz";
        let optimized = segments_bits(&optimal_segments(data, 1), 1).unwrap();
        let byte_only = segment_bits_unchecked(Mode::Byte, data.len(), 1);

        assert!(optimized <= byte_only);
    }

    #[test]
    fn splits_utf8_at_character_boundaries() {
        let data = "héllo wörld";
        let segments = optimal_segments(data, 1);

        assert_eq!(joined(&segments), data.as_bytes());

        for segment in &segments {
            if let Segment::Data { data, .. } = segment {
                assert!(std::str::from_utf8(data).is_ok());
            }
        }
    }
}
