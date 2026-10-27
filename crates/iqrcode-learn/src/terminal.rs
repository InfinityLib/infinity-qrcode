//! Terminal renderer for the step-by-step animation.

use std::io::{IsTerminal, Write};
use std::time::Duration;

use iqrcode_core::trace::{Frame, Trace};

/// Options for the terminal animation.
#[derive(Debug, Clone)]
pub struct PlayOptions {
    /// Frames per second.
    pub fps: u64,
    /// Quiet zone width in modules.
    pub quiet_zone: usize,
    /// Whether ANSI colors may be used.
    pub use_color: bool,
}

impl Default for PlayOptions {
    fn default() -> Self {
        Self {
            fps: 20,
            quiet_zone: 4,
            use_color: true,
        }
    }
}

/// Plays the animation in the current terminal.
pub fn play(trace: &Trace, options: &PlayOptions) {
    let _cursor = CursorGuard::hide();
    let delay = Duration::from_millis(1000 / options.fps.max(1));
    let total = trace.frames.len();
    let use_color = options.use_color && std::io::stdout().is_terminal();

    for (index, frame) in trace.frames.iter().enumerate() {
        render_frame(
            trace,
            frame,
            index + 1,
            total,
            options.quiet_zone,
            use_color,
        );

        if index + 1 < total {
            std::thread::sleep(delay);
        }
    }

    std::thread::sleep(Duration::from_millis(800));
}

fn render_frame(
    trace: &Trace,
    frame: &Frame,
    index: usize,
    total: usize,
    quiet_zone: usize,
    use_color: bool,
) {
    let summary = &trace.summary;
    print!("\x1b[2J\x1b[H");

    println!("QR Code — construction");
    println!(
        "version {} · level {} · {}×{} · {} data codewords · {} ECC/block · {} block(s)",
        summary.version,
        summary.level,
        summary.size,
        summary.size,
        summary.data_codewords,
        summary.ecc_per_block,
        summary.blocks
    );

    let segments = summary
        .segments
        .iter()
        .map(|segment| format!("{:?}({})", segment.mode, segment.characters))
        .collect::<Vec<_>>()
        .join(" + ");
    println!("segments: {segments}");
    println!();
    println!("[{index}/{total}] phase: {}", frame.phase.key());

    if let Some(bit_index) = frame.bit_index {
        println!("bit: {} / {}", bit_index + 1, summary.data_modules);

        if let Some(&(x, y)) = frame.highlight.first() {
            println!("module: ({x}, {y})");
        }
    }

    if let Some(mask) = frame.mask {
        print!("mask: {mask}");

        if let Some(penalty) = frame.penalty {
            print!(" · penalty: {penalty}");
        }

        println!();
    }

    println!();

    let size = frame.modules.len();
    let highlight = build_highlight_mask(size, &frame.highlight);
    let width = (size + 2 * quiet_zone) * 2;
    let mut output = String::new();

    for _ in 0..quiet_zone {
        push_quiet_line(&mut output, width, use_color);
    }

    for (y, row) in frame.modules.iter().enumerate() {
        push_rendered_row(&mut output, row, &highlight[y], quiet_zone, use_color);
    }

    for _ in 0..quiet_zone {
        push_quiet_line(&mut output, width, use_color);
    }

    print!("{output}");
    let _ = std::io::stdout().flush();
}

fn build_highlight_mask(size: usize, positions: &[(usize, usize)]) -> Vec<Vec<bool>> {
    let mut mask = vec![vec![false; size]; size];

    for &(x, y) in positions {
        if x < size && y < size {
            mask[y][x] = true;
        }
    }

    mask
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CellColor {
    Dark,
    Light,
    Highlight,
}

impl CellColor {
    fn ansi(self) -> &'static str {
        match self {
            Self::Dark => "\x1b[40m",
            Self::Light => "\x1b[47m",
            Self::Highlight => "\x1b[41m",
        }
    }
}

fn push_rendered_row(
    output: &mut String,
    row: &[u8],
    highlight: &[bool],
    quiet_zone: usize,
    use_color: bool,
) {
    let module_count = row.len() + 2 * quiet_zone;

    if use_color {
        let mut current = None;

        for index in 0..module_count {
            let color = if index < quiet_zone || index >= quiet_zone + row.len() {
                CellColor::Light
            } else {
                let x = index - quiet_zone;

                if highlight[x] {
                    CellColor::Highlight
                } else if row[x] == 1 {
                    CellColor::Dark
                } else {
                    CellColor::Light
                }
            };

            if current != Some(color) {
                output.push_str(color.ansi());
                current = Some(color);
            }

            // Two terminal cells per module keep the QR Code roughly square.
            output.push_str("  ");
        }

        output.push_str("\x1b[0m\n");
    } else {
        output.push_str(&"  ".repeat(quiet_zone));

        for (x, &module) in row.iter().enumerate() {
            output.push_str(if highlight[x] {
                "▓▓"
            } else if module == 1 {
                "██"
            } else {
                "  "
            });
        }

        output.push_str(&"  ".repeat(quiet_zone));
        output.push('\n');
    }
}

fn push_quiet_line(output: &mut String, width: usize, use_color: bool) {
    if use_color {
        output.push_str("\x1b[47m");
        output.push_str(&" ".repeat(width));
        output.push_str("\x1b[0m\n");
    } else {
        output.push_str(&" ".repeat(width));
        output.push('\n');
    }
}

struct CursorGuard;

impl CursorGuard {
    fn hide() -> Self {
        print!("\x1b[?25l");
        let _ = std::io::stdout().flush();
        Self
    }
}

impl Drop for CursorGuard {
    fn drop(&mut self) {
        print!("\x1b[0m\x1b[?25h");
        let _ = std::io::stdout().flush();
    }
}
