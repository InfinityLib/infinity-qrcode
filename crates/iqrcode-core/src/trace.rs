//! Step-by-step construction trace, used by the terminal animation.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::bits::{self, Segment};
use crate::error::QrError;
use crate::matrix::{self, Matrix};
use crate::qr::{CorrectionLevel, Mode, QrCode};
use crate::tables;

/// Stage of the QR Code construction shown by a frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Empty,
    Finders,
    Timing,
    Alignment,
    VersionInfo,
    FormatReserved,
    DataPlacement,
    MaskCandidate,
    MaskChosen,
    FormatInfo,
    Final,
}

/// One visual step of the construction.
#[derive(Debug, Clone)]
pub struct Frame {
    pub phase: Phase,
    pub title: &'static str,
    pub description: String,
    pub modules: Vec<Vec<u8>>,
    pub highlight: Vec<(usize, usize)>,
    pub bit_index: Option<usize>,
    pub mask: Option<u8>,
    pub penalty: Option<u32>,
}

/// Summary of the data that produced the traced QR Code.
#[derive(Debug, Clone)]
pub struct SegmentSummary {
    pub mode: Mode,
    pub characters: usize,
    pub bits: usize,
}

/// Summary of the encoded QR Code.
#[derive(Debug, Clone)]
pub struct TraceSummary {
    pub version: u8,
    pub level: CorrectionLevel,
    pub mask: u8,
    pub size: usize,
    pub segments: Vec<SegmentSummary>,
    pub data_codewords: usize,
    pub ecc_per_block: usize,
    pub blocks: usize,
    pub total_codewords: usize,
}

/// A complete trace: summary plus every visual frame.
#[derive(Debug, Clone)]
pub struct Trace {
    pub summary: TraceSummary,
    pub frames: Vec<Frame>,
}

/// Options controlling the number of frames and whether mask candidates are
/// included.
#[derive(Debug, Clone)]
pub struct TraceOptions {
    /// Force a version, or `None` to choose the smallest one.
    pub version: Option<u8>,
    /// Show one data frame every `data_stride` modules. `0` means automatic.
    pub data_stride: usize,
    /// Include the eight mask candidates and their penalty scores.
    pub show_mask_candidates: bool,
    /// Approximate maximum number of data-placement frames.
    pub max_frames: usize,
}

impl Default for TraceOptions {
    fn default() -> Self {
        Self {
            version: None,
            data_stride: 1,
            show_mask_candidates: true,
            max_frames: 300,
        }
    }
}

/// Builds a QR Code and records every construction step.
pub fn trace(data: &str, level: CorrectionLevel, options: TraceOptions) -> Result<Trace, QrError> {
    let (segments, version) = match options.version {
        Some(version) => (QrCode::plan_version(data, version)?, version),
        None => QrCode::plan(data, level)?,
    };

    let encoded = QrCode::encode_codewords(&segments, version, level)?;
    let size = tables::version_size(version);
    let total_data_modules = tables::raw_data_modules(version);
    let max_frames = options.max_frames.max(10);
    let stride = options
        .data_stride
        .max(1)
        .max(total_data_modules.div_ceil(max_frames));

    let mut frames = Vec::new();
    let mut matrix = Matrix::new(size);

    let empty = matrix.snapshot();
    frames.push(frame(
        Phase::Empty,
        "Matrice vide",
        format!("Version {version} : matrice de {size}×{size} modules"),
        empty,
        Vec::new(),
    ));

    let before = matrix.snapshot();
    matrix.draw_finders();
    let after = matrix.snapshot();
    frames.push(frame(
        Phase::Finders,
        "Finder patterns",
        "Les trois carrés de repérage et leurs séparateurs".to_string(),
        after.clone(),
        diff(&before, &after),
    ));

    let before = matrix.snapshot();
    matrix.draw_timing_patterns();
    let after = matrix.snapshot();
    frames.push(frame(
        Phase::Timing,
        "Timing patterns",
        "Les lignes et colonnes 6 alternent noir et blanc".to_string(),
        after.clone(),
        diff(&before, &after),
    ));

    if version >= 2 {
        let before = matrix.snapshot();
        matrix.draw_alignment_patterns(version);
        let after = matrix.snapshot();
        frames.push(frame(
            Phase::Alignment,
            "Alignment patterns",
            "Les motifs 5×5 aident le scanner à corriger la perspective".to_string(),
            after.clone(),
            diff(&before, &after),
        ));
    }

    if version >= 7 {
        let before = matrix.snapshot();
        matrix.draw_version_info(version);
        let after = matrix.snapshot();
        frames.push(frame(
            Phase::VersionInfo,
            "Version information",
            format!("18 bits BCH(18,6) pour la version {version}"),
            after.clone(),
            diff(&before, &after),
        ));
    }

    let before = matrix.snapshot();
    matrix.reserve_format_info();
    let after = matrix.snapshot();
    frames.push(frame(
        Phase::FormatReserved,
        "Format information réservée",
        "Les deux copies de la format information et le dark module".to_string(),
        after.clone(),
        diff(&before, &after),
    ));

    matrix.place_data_with(&encoded.codewords, |matrix, x, y, bit_index| {
        let is_last = bit_index + 1 == total_data_modules;

        if bit_index.is_multiple_of(stride) || is_last {
            frames.push(
                frame(
                    Phase::DataPlacement,
                    "Placement des données",
                    format!(
                        "Bit {} / {} — module ({x}, {y})",
                        bit_index + 1,
                        total_data_modules
                    ),
                    matrix.snapshot(),
                    vec![(x, y)],
                )
                .with_bit_index(bit_index),
            );
        }
    });

    let base = matrix;
    let (chosen_modules, chosen_mask) =
        matrix::choose_mask_with(&base, level, None, |candidate, mask, score| {
            if options.show_mask_candidates {
                frames.push(
                    frame(
                        Phase::MaskCandidate,
                        "Candidat de masque",
                        format!("Masque {mask} — pénalité {score}"),
                        candidate.snapshot(),
                        Vec::new(),
                    )
                    .with_mask(mask)
                    .with_penalty(score),
                );
            }
        })?;

    let mut chosen_matrix = base.clone();
    chosen_matrix.apply_mask(chosen_mask);
    frames.push(
        frame(
            Phase::MaskChosen,
            "Masque retenu",
            format!("Le masque {chosen_mask} est appliqué aux modules de données"),
            chosen_matrix.snapshot(),
            Vec::new(),
        )
        .with_mask(chosen_mask),
    );

    chosen_matrix.draw_format_info(level, chosen_mask);
    frames.push(
        frame(
            Phase::FormatInfo,
            "Format information",
            format!("Niveau {level}, masque {chosen_mask}"),
            chosen_matrix.snapshot(),
            matrix::format_info_positions(size),
        )
        .with_mask(chosen_mask),
    );

    debug_assert_eq!(chosen_matrix.snapshot(), chosen_modules);

    frames.push(
        frame(
            Phase::Final,
            "QR Code final",
            format!("Version {version}, niveau {level}, masque {chosen_mask}"),
            chosen_modules,
            Vec::new(),
        )
        .with_mask(chosen_mask),
    );

    let segments = segments
        .iter()
        .map(|segment| match *segment {
            Segment::Data { mode, data } => SegmentSummary {
                mode,
                characters: if mode == Mode::Kanji {
                    data.len() / 2
                } else {
                    data.len()
                },
                bits: bits::segment_bits(segment, version)
                    .expect("segments were already validated"),
            },
            Segment::Eci(_) => unreachable!("trace planning never emits ECI segments"),
        })
        .collect();

    Ok(Trace {
        summary: TraceSummary {
            version,
            level,
            mask: chosen_mask,
            size,
            segments,
            data_codewords: encoded.data_codewords.len(),
            ecc_per_block: encoded.ecc_len,
            blocks: encoded.num_blocks,
            total_codewords: encoded.total_codewords,
        },
        frames,
    })
}

impl Frame {
    fn with_bit_index(mut self, bit_index: usize) -> Self {
        self.bit_index = Some(bit_index);
        self
    }

    fn with_mask(mut self, mask: u8) -> Self {
        self.mask = Some(mask);
        self
    }

    fn with_penalty(mut self, penalty: u32) -> Self {
        self.penalty = Some(penalty);
        self
    }
}

fn frame(
    phase: Phase,
    title: &'static str,
    description: String,
    modules: Vec<Vec<u8>>,
    highlight: Vec<(usize, usize)>,
) -> Frame {
    Frame {
        phase,
        title,
        description,
        modules,
        highlight,
        bit_index: None,
        mask: None,
        penalty: None,
    }
}

fn diff(before: &[Vec<u8>], after: &[Vec<u8>]) -> Vec<(usize, usize)> {
    let mut result = Vec::new();

    for (y, (before_row, after_row)) in before.iter().zip(after).enumerate() {
        for (x, (&before_value, &after_value)) in before_row.iter().zip(after_row).enumerate() {
            if before_value != after_value {
                result.push((x, y));
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn final_frame_matches_the_real_qr() {
        let trace = trace(
            "HELLO WORLD",
            CorrectionLevel::Medium,
            TraceOptions::default(),
        )
        .unwrap();
        let qr = QrCode::encode("HELLO WORLD", CorrectionLevel::Medium).unwrap();

        assert_eq!(trace.summary.version, qr.version());
        assert_eq!(trace.frames.last().unwrap().modules, qr.to_matrix());
    }

    #[test]
    fn final_frame_matches_an_explicit_version() {
        let trace = trace(
            "HELLO WORLD",
            CorrectionLevel::Quartile,
            TraceOptions {
                version: Some(7),
                ..TraceOptions::default()
            },
        )
        .unwrap();
        let qr = QrCode::encode_version("HELLO WORLD", 7, CorrectionLevel::Quartile).unwrap();

        assert_eq!(trace.summary.version, 7);
        assert!(
            trace
                .frames
                .iter()
                .any(|frame| frame.phase == Phase::VersionInfo)
        );
        assert_eq!(trace.frames.last().unwrap().modules, qr.to_matrix());
    }

    #[test]
    fn mask_candidates_can_be_included_or_skipped() {
        let with_masks = trace("HELLO", CorrectionLevel::Medium, TraceOptions::default()).unwrap();
        let without_masks = trace(
            "HELLO",
            CorrectionLevel::Medium,
            TraceOptions {
                show_mask_candidates: false,
                ..TraceOptions::default()
            },
        )
        .unwrap();

        let with_count = with_masks
            .frames
            .iter()
            .filter(|frame| frame.phase == Phase::MaskCandidate)
            .count();
        let without_count = without_masks
            .frames
            .iter()
            .filter(|frame| frame.phase == Phase::MaskCandidate)
            .count();

        assert_eq!(with_count, 8);
        assert_eq!(without_count, 0);
    }

    #[test]
    fn stride_limits_the_number_of_data_frames() {
        let trace = trace(
            "HELLO WORLD",
            CorrectionLevel::Medium,
            TraceOptions {
                data_stride: 10,
                ..TraceOptions::default()
            },
        )
        .unwrap();

        let data_frames = trace
            .frames
            .iter()
            .filter(|frame| frame.phase == Phase::DataPlacement)
            .count();

        assert!(data_frames <= 22);
    }
}
