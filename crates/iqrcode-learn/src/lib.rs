//! Educational layer for `iqrcode-core`.
//!
//! It exposes the construction trace, written explanations and a terminal
//! animation. The web version lives in `iqrcode-learn-wasm`.

pub mod lesson;
pub mod terminal;

pub use iqrcode_core::trace::{self, Frame, Phase, Trace, TraceOptions};
pub use iqrcode_core::{CorrectionLevel, Mode, QrCode, QrError};
