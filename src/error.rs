use core::fmt;

use crate::qr::{CorrectionLevel, Mode};

/// Errors returned while building a QR Code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QrError {
    /// The version is outside the valid range `1..=40`.
    InvalidVersion(u8),
    /// The data does not fit in the selected version and correction level.
    DataTooLong {
        version: u8,
        level: CorrectionLevel,
        needed_bits: usize,
        capacity_bits: usize,
    },
    /// A character cannot be represented in the selected mode.
    InvalidCharacter(char),
    /// Kanji mode needs raw Shift-JIS bytes.
    KanjiRequiresShiftJis,
    /// The Shift-JIS byte sequence is not valid for QR Kanji mode.
    InvalidKanjiData,
    /// The ECI designator is outside the supported range `0..=999999`.
    InvalidEciDesignator(u32),
    /// A segment exceeds the capacity of its character count indicator.
    CharacterCountOverflow {
        mode: Mode,
        version: u8,
        count: usize,
    },
    /// The mask number is outside the valid range `0..=7`.
    InvalidMask(u8),
    /// PNG encoding failed.
    PngEncoding(String),
}

impl fmt::Display for QrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidVersion(version) => {
                write!(f, "invalid QR version {version}; expected 1..=40")
            }
            Self::DataTooLong {
                version,
                level,
                needed_bits,
                capacity_bits,
            } => write!(
                f,
                "data too long for version {version} with correction level {level}: \
                 {needed_bits} bits needed, {capacity_bits} available"
            ),
            Self::InvalidCharacter(character) => {
                write!(
                    f,
                    "character {character:?} cannot be encoded in the selected mode"
                )
            }
            Self::KanjiRequiresShiftJis => write!(
                f,
                "Kanji mode requires raw Shift-JIS bytes; use encode_kanji_shift_jis"
            ),
            Self::InvalidKanjiData => write!(
                f,
                "invalid Shift-JIS data for QR Kanji mode; expected two-byte characters in \
                 0x8140..=0x9FFC or 0xE040..=0xEBBF"
            ),
            Self::InvalidEciDesignator(designator) => write!(
                f,
                "invalid ECI designator {designator}; expected 0..=999999"
            ),
            Self::CharacterCountOverflow {
                mode,
                version,
                count,
            } => write!(
                f,
                "segment has {count} characters in mode {mode}, which does not fit the count \
                 indicator for version {version}"
            ),
            Self::InvalidMask(mask) => {
                write!(f, "invalid QR mask {mask}; expected 0..=7")
            }
            Self::PngEncoding(message) => {
                write!(f, "PNG encoding failed: {message}")
            }
        }
    }
}

impl std::error::Error for QrError {}
