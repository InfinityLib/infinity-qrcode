//! Core QR Code generator.
//!
//! The current implementation supports QR Code versions 1 to 40 (21×21 to
//! 177×177 modules), with the Numeric, Alphanumeric, Byte and Kanji modes, all
//! four error correction levels, automatic version selection, block
//! interleaving, alignment patterns and version information.
//!
//! [`QrCode::encode`] automatically splits the input into mixed-mode segments.
//! UTF-8 text can be marked with an ECI 26 designator using
//! [`QrCode::encode_utf8`], and Kanji mode accepts raw Shift-JIS bytes through
//! [`QrCode::encode_kanji_shift_jis`].
//!
//! Outputs are available as a `Vec<Vec<u8>>` matrix, ASCII, PBM and SVG, plus
//! PNG when the `png` feature is enabled.
//!
//! The `trace` feature enables the [`trace`] module, which records every
//! construction step for educational tools and animations.
//!
//! ```
//! use iqrcode_core::{CorrectionLevel, QrCode};
//!
//! let qr = QrCode::encode("HELLO WORLD", CorrectionLevel::Medium).unwrap();
//! let matrix = qr.to_matrix();
//!
//! assert_eq!(matrix.len(), 21);
//! assert!(matrix.iter().flatten().all(|&bit| bit == 0 || bit == 1));
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

extern crate alloc;

#[cfg(test)]
extern crate std;

mod bits;
mod gf256;
mod matrix;
mod optimize;
mod output;
mod reed_solomon;
mod tables;

/// Error types returned by the encoder.
pub mod error;
/// QR Code types and encoding entry points.
pub mod qr;

/// Construction trace, enabled by the `trace` feature.
#[cfg(feature = "trace")]
pub mod trace;

pub use error::QrError;
pub use qr::{CorrectionLevel, Mode, QrCode};
