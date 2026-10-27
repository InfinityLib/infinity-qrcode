//! WebAssembly bindings for the educational QR Code trace.

use js_sys::{Uint8Array, Uint32Array};
use wasm_bindgen::prelude::*;

use iqrcode_learn::CorrectionLevel;
use iqrcode_learn::lesson::{self, Lesson, PIPELINE_LESSONS};
use iqrcode_learn::trace::{self, Frame, Trace, TraceOptions};

/// A complete construction trace exposed to JavaScript.
#[wasm_bindgen]
pub struct LearnTrace {
    inner: Trace,
}

#[wasm_bindgen]
impl LearnTrace {
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

    pub fn frame_count(&self) -> usize {
        self.inner.frames.len()
    }

    pub fn version(&self) -> u8 {
        self.inner.summary.version
    }

    pub fn level(&self) -> String {
        self.inner.summary.level.to_string()
    }

    pub fn mask(&self) -> u8 {
        self.inner.summary.mask
    }

    pub fn size(&self) -> usize {
        self.inner.summary.size
    }

    pub fn data_codewords(&self) -> usize {
        self.inner.summary.data_codewords
    }

    pub fn ecc_per_block(&self) -> usize {
        self.inner.summary.ecc_per_block
    }

    pub fn blocks(&self) -> usize {
        self.inner.summary.blocks
    }

    pub fn total_codewords(&self) -> usize {
        self.inner.summary.total_codewords
    }

    pub fn segments(&self) -> String {
        self.inner
            .summary
            .segments
            .iter()
            .map(|segment| format!("{:?}({})", segment.mode, segment.characters))
            .collect::<Vec<_>>()
            .join(" + ")
    }

    pub fn frame(&self, index: usize) -> Result<LearnFrame, JsValue> {
        let frame = self
            .inner
            .frames
            .get(index)
            .cloned()
            .ok_or_else(|| JsValue::from_str("frame index out of range"))?;
        let lesson = lesson::for_phase(frame.phase);

        Ok(LearnFrame { frame, lesson })
    }

    pub fn pipeline_lesson_count(&self) -> usize {
        PIPELINE_LESSONS.len()
    }

    pub fn pipeline_lesson_title(&self, index: usize) -> Option<String> {
        PIPELINE_LESSONS
            .get(index)
            .map(|lesson| lesson.title.to_string())
    }

    pub fn pipeline_lesson_explanation(&self, index: usize) -> Option<String> {
        PIPELINE_LESSONS
            .get(index)
            .map(|lesson| lesson.explanation.to_string())
    }

    pub fn pipeline_lesson_formula(&self, index: usize) -> Option<String> {
        PIPELINE_LESSONS
            .get(index)
            .and_then(|lesson| lesson.formula.map(str::to_string))
    }
}

/// One frame exposed to JavaScript.
#[wasm_bindgen]
pub struct LearnFrame {
    frame: Frame,
    lesson: Lesson,
}

#[wasm_bindgen]
impl LearnFrame {
    pub fn phase(&self) -> String {
        format!("{:?}", self.frame.phase)
    }

    pub fn title(&self) -> String {
        self.frame.title.to_string()
    }

    pub fn description(&self) -> String {
        self.frame.description.clone()
    }

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

    pub fn bit_index(&self) -> Option<usize> {
        self.frame.bit_index
    }

    pub fn mask(&self) -> Option<u8> {
        self.frame.mask
    }

    pub fn penalty(&self) -> Option<u32> {
        self.frame.penalty
    }

    pub fn lesson_title(&self) -> String {
        self.lesson.title.to_string()
    }

    pub fn lesson_explanation(&self) -> String {
        self.lesson.explanation.to_string()
    }

    pub fn lesson_formula(&self) -> Option<String> {
        self.lesson.formula.map(str::to_string)
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
