//! WebAssembly bindings for the educational QR Code trace.

#![warn(missing_docs)]

use js_sys::{Uint8Array, Uint32Array};
use wasm_bindgen::prelude::*;

use iqrcode_learn::CorrectionLevel;
use iqrcode_learn::trace::{self, Frame, Trace, TraceOptions};

/// A complete construction trace exposed to JavaScript.
#[wasm_bindgen]
pub struct LearnTrace {
    inner: Trace,
}

#[wasm_bindgen]
impl LearnTrace {
    /// Builds a trace for `data`.
    ///
    /// # Errors
    ///
    /// Throws a JavaScript error if the level is invalid or the data does not fit.
    #[wasm_bindgen(constructor)]
    pub fn new(
        data: &str,
        level: &str,
        version: Option<u8>,
        stride: usize,
        max_frames: usize,
        mask_candidates: bool,
    ) -> Result<LearnTrace, JsValue> {
        let level = parse_level(level)?;
        let trace = trace::trace(
            data,
            level,
            TraceOptions {
                version,
                data_stride: stride,
                show_mask_candidates: mask_candidates,
                max_frames,
            },
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;

        Ok(Self { inner: trace })
    }

    /// Returns the number of construction frames.
    pub fn frame_count(&self) -> usize {
        self.inner.frames.len()
    }

    /// Returns the QR Code version.
    pub fn version(&self) -> u8 {
        self.inner.summary.version
    }

    /// Returns the correction level as a string.
    pub fn level(&self) -> String {
        self.inner.summary.level.to_string()
    }

    /// Returns the selected mask number.
    pub fn mask(&self) -> u8 {
        self.inner.summary.mask
    }

    /// Returns the matrix side length.
    pub fn size(&self) -> usize {
        self.inner.summary.size
    }

    /// Returns the number of modules available for data and ECC.
    pub fn data_modules(&self) -> usize {
        self.inner.summary.data_modules
    }

    /// Returns the number of data codewords.
    pub fn data_codewords(&self) -> usize {
        self.inner.summary.data_codewords
    }

    /// Returns the number of ECC codewords per block.
    pub fn ecc_per_block(&self) -> usize {
        self.inner.summary.ecc_per_block
    }

    /// Returns the number of ECC blocks.
    pub fn blocks(&self) -> usize {
        self.inner.summary.blocks
    }

    /// Returns the total number of codewords.
    pub fn total_codewords(&self) -> usize {
        self.inner.summary.total_codewords
    }

    /// Returns a human-readable segment summary.
    pub fn segments(&self) -> String {
        self.inner
            .summary
            .segments
            .iter()
            .map(|segment| format!("{:?}({})", segment.mode, segment.characters))
            .collect::<Vec<_>>()
            .join(" + ")
    }

    /// Returns the frame at `index`.
    ///
    /// # Errors
    ///
    /// Throws a JavaScript error when `index` is out of range.
    pub fn frame(&self, index: usize) -> Result<LearnFrame, JsValue> {
        let frame = self
            .inner
            .frames
            .get(index)
            .cloned()
            .ok_or_else(|| JsValue::from_str("frame index out of range"))?;

        Ok(LearnFrame { frame })
    }
}

/// One frame exposed to JavaScript.
#[wasm_bindgen]
pub struct LearnFrame {
    frame: Frame,
}

#[wasm_bindgen]
impl LearnFrame {
    /// Returns the stable phase key (`"empty"`, `"data_placement"`, ...).
    pub fn phase(&self) -> String {
        self.frame.phase.key().to_string()
    }

    /// Returns the matrix side length.
    pub fn size(&self) -> usize {
        self.frame.modules.len()
    }

    /// Row-major matrix: 0 = light, 1 = dark.
    pub fn matrix(&self) -> Uint8Array {
        let flat: Vec<u8> = self
            .frame
            .modules
            .iter()
            .flat_map(|row| row.iter().copied())
            .collect();

        Uint8Array::from(&flat[..])
    }

    /// Flattened `(x, y)` pairs of highlighted modules.
    pub fn highlight(&self) -> Uint32Array {
        let flat: Vec<u32> = self
            .frame
            .highlight
            .iter()
            .flat_map(|&(x, y)| [x as u32, y as u32])
            .collect();

        Uint32Array::from(&flat[..])
    }

    /// Returns the index of the placed data bit, when applicable.
    pub fn bit_index(&self) -> Option<usize> {
        self.frame.bit_index
    }

    /// Returns the mask number, when applicable.
    pub fn mask(&self) -> Option<u8> {
        self.frame.mask
    }

    /// Returns the mask penalty score, when applicable.
    pub fn penalty(&self) -> Option<u32> {
        self.frame.penalty
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
