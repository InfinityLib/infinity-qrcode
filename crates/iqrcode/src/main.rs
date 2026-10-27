use std::io::Write;
use std::process;

use iqrcode::{CorrectionLevel, QrCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Format {
    Ascii,
    Bits,
    Pbm,
    Svg,
    Png,
}

struct Options {
    data: String,
    level: CorrectionLevel,
    version: Option<u8>,
    format: Format,
    scale: usize,
    quiet_zone: usize,
    output: Option<String>,
    info: bool,
    utf8_eci: bool,
}

fn main() {
    if let Err(message) = run() {
        eprintln!("Error: {message}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let options = parse_args()?;

    let qr = match (options.version, options.utf8_eci) {
        (Some(version), true) => QrCode::encode_utf8_version(&options.data, version, options.level),
        (None, true) => QrCode::encode_utf8(&options.data, options.level),
        (Some(version), false) => QrCode::encode_version(&options.data, version, options.level),
        (None, false) => QrCode::encode(&options.data, options.level),
    }
    .map_err(|error| error.to_string())?;

    if options.info {
        eprintln!(
            "Version {}, correction {:?}, mask {}",
            qr.version(),
            qr.correction_level(),
            qr.mask()
        );
    }

    let bytes = match options.format {
        Format::Ascii => qr.to_ascii_with_quiet_zone(options.quiet_zone).into_bytes(),
        Format::Bits => {
            let mut output = String::new();

            for row in qr.modules() {
                for &bit in row {
                    output.push(if bit == 1 { '1' } else { '0' });
                }
                output.push('\n');
            }

            output.into_bytes()
        }
        Format::Pbm => qr.to_pbm(options.quiet_zone).into_bytes(),
        Format::Svg => qr.to_svg(options.scale, options.quiet_zone).into_bytes(),
        Format::Png => {
            #[cfg(feature = "png")]
            {
                qr.to_png(options.scale, options.quiet_zone)
                    .map_err(|error| error.to_string())?
            }

            #[cfg(not(feature = "png"))]
            {
                return Err(
                    "PNG output requires the `png` feature: cargo run -p iqrcode --features png -- ..."
                        .to_string(),
                );
            }
        }
    };

    match options.output {
        Some(path) => std::fs::write(&path, bytes)
            .map_err(|error| format!("cannot write to {path}: {error}"))?,
        None => {
            let mut stdout = std::io::stdout();

            match stdout.write_all(&bytes) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::BrokenPipe => {}
                Err(error) => return Err(format!("cannot write to stdout: {error}")),
            }
        }
    }

    Ok(())
}

fn parse_args() -> Result<Options, String> {
    let mut data = None;
    let mut level = CorrectionLevel::Medium;
    let mut version = None;
    let mut format = Format::Ascii;
    let mut scale = 8;
    let mut quiet_zone = 4;
    let mut output = None;
    let mut info = false;
    let mut utf8_eci = false;

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
            "--format" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| "--format expects a value".to_string())?;
                format = parse_format(&value)?;
            }
            "--scale" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| "--scale expects a value".to_string())?;
                scale = value
                    .parse::<usize>()
                    .map_err(|_| format!("invalid scale: {value}"))?;

                if scale == 0 {
                    return Err("--scale must be at least 1".to_string());
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
            "-o" | "--output" => {
                output = Some(
                    arguments
                        .next()
                        .ok_or_else(|| "--output expects a path".to_string())?,
                );
            }
            "--bits" => format = Format::Bits,
            "--info" => info = true,
            "--utf8-eci" => utf8_eci = true,
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
        format,
        scale,
        quiet_zone,
        output,
        info,
        utf8_eci,
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

fn parse_format(value: &str) -> Result<Format, String> {
    match value.to_ascii_lowercase().as_str() {
        "ascii" | "text" => Ok(Format::Ascii),
        "bits" | "matrix" => Ok(Format::Bits),
        "pbm" => Ok(Format::Pbm),
        "svg" => Ok(Format::Svg),
        "png" => Ok(Format::Png),
        _ => Err(format!("invalid format: {value}")),
    }
}

fn print_help() {
    println!(
        r#"QR Code generator

Usage:
  iqrcode [DATA] [OPTIONS]

Options:
  --level <low|medium|quartile|high>   Correction level (default: medium)
  --version <1..40>                    Force a version (default: automatic)
  --format <ascii|bits|pbm|svg|png>    Output format (default: ascii)
  --scale <n>                          Module size for SVG/PNG (default: 8)
  --quiet-zone <n>                     Quiet zone in modules (default: 4)
  -o, --output <path>                  Write to a file
  --bits                               Shortcut for --format bits
  --info                               Print version, level and mask to stderr
  --utf8-eci                           Add an ECI 26 (UTF-8) designator
  -h, --help                           Show this help

Examples:
  iqrcode "HELLO WORLD"
  iqrcode "HELLO WORLD" --format bits
  iqrcode "HELLO WORLD" --format svg -o hello.svg
  cargo run -p iqrcode --features png -- "HELLO WORLD" --format png -o hello.png

For the educational animation:
  cargo run -p iqrcode-learn -- "HELLO WORLD" --animate
"#
    );
}
