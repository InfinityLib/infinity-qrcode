//! QR Code generator with standard-library conveniences.
//!
//! This crate re-exports the [`iqrcode_core`] API and adds:
//!
//! - the `cli` feature and the `iqrcode` binary,
//! - the `png` feature for PNG output.
//!
//! ```
//! use iqrcode::{CorrectionLevel, QrCode};
//!
//! let qr = QrCode::encode("HELLO WORLD", CorrectionLevel::Medium).unwrap();
//! assert_eq!(qr.to_matrix().len(), 21);
//! ```

#![forbid(unsafe_code)]

pub use iqrcode_core::{CorrectionLevel, Mode, QrCode, QrError, error, qr};
