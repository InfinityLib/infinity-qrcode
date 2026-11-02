use std::io::IsTerminal;
use std::process;

use iqrcode_learn::CorrectionLevel;
use iqrcode_learn::terminal::{self, PlayOptions};
use iqrcode_learn::trace::{self, TraceOptions};

struct Options {
    data: String,
    level: CorrectionLevel,
    version: Option<u8>,
    fps: u64,
    stride: usize,
    max_frames: usize,
    quiet_zone: usize,
    no_color: bool,
    show_mask_candidates: bool,
}

fn main() {
    if let Err(message) = run() {
        eprintln!("Error: {message}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let options = parse_args()?;

    let trace_options = TraceOptions {
        version: options.version,
        data_stride: options.stride,
        show_mask_candidates: options.show_mask_candidates,
        max_frames: options.max_frames,
    };

    let trace = trace::trace(&options.data, options.level, trace_options)
        .map_err(|error| error.to_string())?;

    let play_options = PlayOptions {
        fps: options.fps,
        quiet_zone: options.quiet_zone,
        use_color: !options.no_color && std::io::stdout().is_terminal(),
    };

    terminal::play(&trace, &play_options);
    Ok(())
}

fn parse_args() -> Result<Options, String> {
    let mut data = None;
    let mut level = CorrectionLevel::Medium;
    let mut version = None;
    let mut fps = 20;
    let mut stride = 0;
    let mut max_frames = 300;
    let mut quiet_zone = 4;
    let mut no_color = false;
    let mut show_mask_candidates = true;

    let mut arguments = std::env::args().skip(1);

    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "-h" | "--help" => {
                print_help();
                process::exit(0);
            }
            "--level" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| "--level expects a value".to_string())?;
                level = parse_level(&value)?;
            }
            "--version" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| "--version expects a value".to_string())?;
                let parsed = value
                    .parse::<u8>()
                    .map_err(|_| format!("invalid version: {value}"))?;

                if !(1..=40).contains(&parsed) {
                    return Err(format!("invalid version: {parsed} (expected 1..=40)"));
                }

                version = Some(parsed);
            }
            "--fps" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| "--fps expects a value".to_string())?;
                fps = value
                    .parse::<u64>()
                    .map_err(|_| format!("invalid fps: {value}"))?;

                if fps == 0 {
                    return Err("--fps must be at least 1".to_string());
                }
            }
            "--stride" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| "--stride expects a value".to_string())?;
                stride = value
                    .parse::<usize>()
                    .map_err(|_| format!("invalid stride: {value}"))?;
            }
            "--max-frames" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| "--max-frames expects a value".to_string())?;
                max_frames = value
                    .parse::<usize>()
                    .map_err(|_| format!("invalid max-frames: {value}"))?;

                if max_frames == 0 {
                    return Err("--max-frames must be at least 1".to_string());
                }
            }
            "--quiet-zone" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| "--quiet-zone expects a value".to_string())?;
                quiet_zone = value
                    .parse::<usize>()
                    .map_err(|_| format!("invalid quiet zone: {value}"))?;
            }
            "--no-color" => no_color = true,
            "--no-mask-candidates" => show_mask_candidates = false,
            "--animate" => {}
            _ if argument.starts_with('-') => {
                return Err(format!("unknown option: {argument}"));
            }
            _ => {
                if data.is_some() {
                    return Err("only one data argument is allowed".to_string());
                }
                data = Some(argument);
            }
        }
    }

    Ok(Options {
        data: data.unwrap_or_else(|| "HELLO WORLD".to_string()),
        level,
        version,
        fps,
        stride,
        max_frames,
        quiet_zone,
        no_color,
        show_mask_candidates,
    })
}

fn parse_level(value: &str) -> Result<CorrectionLevel, String> {
    match value.to_ascii_lowercase().as_str() {
        "l" | "low" => Ok(CorrectionLevel::Low),
        "m" | "medium" => Ok(CorrectionLevel::Medium),
        "q" | "quartile" => Ok(CorrectionLevel::Quartile),
        "h" | "high" => Ok(CorrectionLevel::High),
        _ => Err(format!("invalid correction level: {value}")),
    }
}

fn print_help() {
    println!(
        r#"iqrcode-learn — step-by-step QR Code animation

Usage:
  iqrcode-learn [DATA] [OPTIONS]

Options:
  --level <low|medium|quartile|high>   Correction level (default: medium)
  --version <1..40>                    Force a version (default: automatic)
  --fps <n>                            Frames per second (default: 20)
  --stride <n>                         One data frame every n modules (0 = auto)
  --max-frames <n>                     Maximum number of data frames (default: 300)
  --quiet-zone <n>                     Quiet zone in modules (default: 4)
  --no-color                           Disable ANSI colors
  --no-mask-candidates                 Hide the eight mask candidates
  -h, --help                           Show this help

Note: the terminal is language-neutral and shows phases and values. A separate
web frontend can provide explanations and translations from the
`iqrcode-learn-wasm` trace.

Examples:
  iqrcode-learn "HELLO WORLD" --animate
  iqrcode-learn "HELLO WORLD" --animate --version 4 --fps 20
"#
    );
}
