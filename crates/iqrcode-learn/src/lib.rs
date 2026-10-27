//! Terminal construction animation for `iqrcode-core`.
//!
//! This crate is language-neutral: it exposes the trace and displays raw
//! phases/values. Human-readable explanations live in the web UI.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod terminal;

pub use iqrcode_core::trace::{self, Frame, Phase, Trace, TraceOptions};
pub use iqrcode_core::{CorrectionLevel, QrCode, QrError};
