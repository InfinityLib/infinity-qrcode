//! A small QR Code generator written in Rust.
//!
//! The current implementation supports QR Code versions 1 to 40 (21×21 to
//! 177×177 modules), with the Numeric, Alphanumeric and Byte modes, all four
//! error correction levels, automatic version selection, block interleaving,
//! alignment patterns and version information.
//!
//! ```
//! use infinitylib_qrcode_rs::{CorrectionLevel, QrCode};
//!
//! let qr = QrCode::encode("HELLO WORLD", CorrectionLevel::Medium).unwrap();
//! let matrix = qr.to_matrix();
//!
//! assert_eq!(matrix.len(), 21);
//! assert!(matrix.iter().flatten().all(|&bit| bit == 0 || bit == 1));
//! ```

#![forbid(unsafe_code)]

mod bits;
mod gf256;
mod matrix;
mod reed_solomon;
mod tables;

pub mod error;
pub mod qr;

pub use error::QrError;
pub use qr::{CorrectionLevel, Mode, QrCode};
