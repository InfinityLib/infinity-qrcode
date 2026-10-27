//! Minimal WebAssembly wrapper around `iqrcode-core`.

#![warn(missing_docs)]

use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

use iqrcode_core::{CorrectionLevel, QrCode};

/// A QR Code exposed to JavaScript.
#[wasm_bindgen]
pub struct QrCodeWasm {
    inner: QrCode,
}

#[wasm_bindgen]
impl QrCodeWasm {
    /// Creates a QR Code from text and a correction level (`L`, `M`, `Q` or `H`).
    ///
    /// # Errors
    ///
    /// Throws a JavaScript error if the level is invalid or the data does not fit.
    #[wasm_bindgen(constructor)]
    pub fn new(data: &str, level: &str) -> Result<QrCodeWasm, JsValue> {
        let level = parse_level(level)?;
        let inner =
            QrCode::encode(data, level).map_err(|error| JsValue::from_str(&error.to_string()))?;

        Ok(Self { inner })
    }

    /// Returns the matrix side length.
    pub fn size(&self) -> usize {
        self.inner.size()
    }

    /// Returns the QR Code version.
    pub fn version(&self) -> u8 {
        self.inner.version()
    }

    /// Returns the selected mask number.
    pub fn mask(&self) -> u8 {
        self.inner.mask()
    }

    /// Row-major matrix: 0 = light, 1 = dark.
    pub fn matrix(&self) -> Uint8Array {
        let flat: Vec<u8> = self
            .inner
            .modules()
            .iter()
            .flat_map(|row| row.iter().copied())
            .collect();

        Uint8Array::from(&flat[..])
    }

    /// Returns the QR Code as an SVG string.
    pub fn svg(&self, module_size: usize, quiet_zone: usize) -> String {
        self.inner.to_svg(module_size, quiet_zone)
    }
}

fn parse_level(value: &str) -> Result<CorrectionLevel, JsValue> {
    match value.to_ascii_lowercase().as_str() {
        "l" | "low" => Ok(CorrectionLevel::Low),
        "m" | "medium" => Ok(CorrectionLevel::Medium),
        "q" | "quartile" => Ok(CorrectionLevel::Quartile),
        "h" | "high" => Ok(CorrectionLevel::High),
        _ => Err(JsValue::from_str("invalid correction level")),
    }
}
