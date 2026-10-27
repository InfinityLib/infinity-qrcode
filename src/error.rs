use core::fmt;

use crate::qr::{CorrectionLevel, Mode};

/// Errors returned while building a QR Code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QrError {
    /// The version is outside the valid range `1..=40`.
    InvalidVersion(u8),
    /// The version is valid, but not implemented yet.
    UnsupportedVersion(u8),
    /// The data does not fit in the selected version and correction level.
    DataTooLong {
        version: u8,
        level: CorrectionLevel,
        needed_bits: usize,
        capacity_bits: usize,
    },
    /// A character cannot be represented in the selected mode.
    InvalidCharacter(char),
    /// The mode is not implemented yet.
    UnsupportedMode(Mode),
    /// The mask number is outside the valid range `0..=7`.
    InvalidMask(u8),
}

impl fmt::Display for QrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidVersion(version) => {
                write!(f, "invalid QR version {version}; expected 1..=40")
            }
            Self::UnsupportedVersion(version) => write!(
                f,
                "QR version {version} is not supported yet; only version 1 is implemented"
            ),
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
            Self::UnsupportedMode(mode) => {
                write!(f, "mode {mode} is not supported yet")
            }
            Self::InvalidMask(mask) => {
                write!(f, "invalid QR mask {mask}; expected 0..=7")
            }
        }
    }
}

impl std::error::Error for QrError {}
