use std::io::{IsTerminal, Write};
use std::process;
use std::time::Duration;

use infinitylib_qrcode_rs::trace::{self, Frame, TraceOptions};
use infinitylib_qrcode_rs::{CorrectionLevel, QrCode};

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
    animate: bool,
    fps: u64,
    stride: usize,
    max_frames: usize,
    no_color: bool,
    show_mask_candidates: bool,
}

fn main() {
    if let Err(message) = run() {
        eprintln!("Erreur: {message}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let options = parse_args()?;

    if options.animate {
        if options.output.is_some() {
            return Err("--animate ne peut pas être utilisé avec --output".to_string());
        }
        if options.utf8_eci {
            return Err("--animate ne supporte pas encore --utf8-eci".to_string());
        }
        if options.format != Format::Ascii {
            return Err("--animate utilise toujours le rendu terminal".to_string());
        }

        return animate(&options);
    }

    let qr = match (options.version, options.utf8_eci) {
        (Some(version), true) => QrCode::encode_utf8_version(&options.data, version, options.level),
        (None, true) => QrCode::encode_utf8(&options.data, options.level),
        (Some(version), false) => QrCode::encode_version(&options.data, version, options.level),
        (None, false) => QrCode::encode(&options.data, options.level),
    }
    .map_err(|error| error.to_string())?;

    if options.info {
        eprintln!(
            "Version {}, correction {:?}, masque {}",
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
                    "la sortie PNG nécessite la feature `png` : cargo run --features png -- ..."
                        .to_string(),
                );
            }
        }
    };

    match options.output {
        Some(path) => std::fs::write(&path, bytes)
            .map_err(|error| format!("impossible d'écrire dans {path}: {error}"))?,
        None => {
            let mut stdout = std::io::stdout();

            match stdout.write_all(&bytes) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::BrokenPipe => {}
                Err(error) => return Err(format!("impossible d'écrire sur stdout: {error}")),
            }
        }
    }

    Ok(())
}

fn animate(options: &Options) -> Result<(), String> {
    let trace_options = TraceOptions {
        version: options.version,
        data_stride: options.stride,
        show_mask_candidates: options.show_mask_candidates,
        max_frames: options.max_frames,
    };

    let trace = trace::trace(&options.data, options.level, trace_options)
        .map_err(|error| error.to_string())?;
    let _cursor = CursorGuard::hide();

    let delay = Duration::from_millis(1000 / options.fps.max(1));
    let total = trace.frames.len();
    let use_color = !options.no_color && std::io::stdout().is_terminal();

    for (index, frame) in trace.frames.iter().enumerate() {
        render_frame(
            &trace,
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
    Ok(())
}

fn render_frame(
    trace: &trace::Trace,
    frame: &Frame,
    index: usize,
    total: usize,
    quiet_zone: usize,
    use_color: bool,
) {
    let summary = &trace.summary;
    print!("\x1b[2J\x1b[H");

    println!("QR Code — construction pas à pas");
    println!(
        "Version {} · niveau {} · {}×{} · {} data codewords · {} ECC/bloc · {} bloc(s)",
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
    println!("Segments : {segments}");
    println!();
    println!("[{index}/{total}] {}", frame.title);
    println!("{}", frame.description);

    if let Some(mask) = frame.mask {
        print!("Masque {mask}");

        if let Some(penalty) = frame.penalty {
            print!(" · pénalité {penalty}");
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
    let mut animate = false;
    let mut fps = 20;
    let mut stride = 0;
    let mut max_frames = 300;
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
                    .ok_or_else(|| "--level attend une valeur".to_string())?;
                level = parse_level(&value)?;
            }
            "--version" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| "--version attend une valeur".to_string())?;
                let parsed = value
                    .parse::<u8>()
                    .map_err(|_| format!("version invalide: {value}"))?;

                if !(1..=40).contains(&parsed) {
                    return Err(format!("version invalide: {parsed} (attendu: 1..=40)"));
                }

                version = Some(parsed);
            }
            "--format" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| "--format attend une valeur".to_string())?;
                format = parse_format(&value)?;
            }
            "--scale" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| "--scale attend une valeur".to_string())?;
                scale = value
                    .parse::<usize>()
                    .map_err(|_| format!("échelle invalide: {value}"))?;

                if scale == 0 {
                    return Err("--scale doit être supérieur ou égal à 1".to_string());
                }
            }
            "--quiet-zone" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| "--quiet-zone attend une valeur".to_string())?;
                quiet_zone = value
                    .parse::<usize>()
                    .map_err(|_| format!("quiet zone invalide: {value}"))?;
            }
            "-o" | "--output" => {
                output = Some(
                    arguments
                        .next()
                        .ok_or_else(|| "--output attend un chemin".to_string())?,
                );
            }
            "--bits" => format = Format::Bits,
            "--info" => info = true,
            "--utf8-eci" => utf8_eci = true,
            "--animate" => animate = true,
            "--fps" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| "--fps attend une valeur".to_string())?;
                fps = value
                    .parse::<u64>()
                    .map_err(|_| format!("fps invalide: {value}"))?;

                if fps == 0 {
                    return Err("--fps doit être supérieur ou égal à 1".to_string());
                }
            }
            "--stride" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| "--stride attend une valeur".to_string())?;
                stride = value
                    .parse::<usize>()
                    .map_err(|_| format!("stride invalide: {value}"))?;
            }
            "--max-frames" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| "--max-frames attend une valeur".to_string())?;
                max_frames = value
                    .parse::<usize>()
                    .map_err(|_| format!("max-frames invalide: {value}"))?;

                if max_frames == 0 {
                    return Err("--max-frames doit être supérieur ou égal à 1".to_string());
                }
            }
            "--no-color" => no_color = true,
            "--no-mask-candidates" => show_mask_candidates = false,
            _ if argument.starts_with('-') => {
                return Err(format!("option inconnue: {argument}"));
            }
            _ => {
                if data.is_some() {
                    return Err("un seul argument de données est autorisé".to_string());
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
        animate,
        fps,
        stride,
        max_frames,
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
        _ => Err(format!("niveau de correction invalide: {value}")),
    }
}

fn parse_format(value: &str) -> Result<Format, String> {
    match value.to_ascii_lowercase().as_str() {
        "ascii" | "text" => Ok(Format::Ascii),
        "bits" | "matrix" => Ok(Format::Bits),
        "pbm" => Ok(Format::Pbm),
        "svg" => Ok(Format::Svg),
        "png" => Ok(Format::Png),
        _ => Err(format!("format invalide: {value}")),
    }
}

fn print_help() {
    println!(
        r#"Générateur de QR Code

Usage :
  infinitylib-qrcode-rs [DATA] [OPTIONS]

Options :
  --level <low|medium|quartile|high>   Niveau de correction (défaut : medium)
  --version <1..40>                    Force une version (défaut : automatique)
  --format <ascii|bits|pbm|svg|png>    Format de sortie (défaut : ascii)
  --scale <n>                          Taille d'un module pour SVG/PNG (défaut : 8)
  --quiet-zone <n>                     Quiet zone en modules (défaut : 4)
  -o, --output <chemin>                Écrit dans un fichier
  --bits                               Raccourci pour --format bits
  --info                               Affiche la version, le niveau et le masque sur stderr
  --utf8-eci                           Ajoute un designateur ECI 26 (UTF-8)

Animation terminal :
  --animate                            Montre la construction pas à pas
  --fps <n>                            Images par seconde (défaut : 20)
  --stride <n>                         Une frame de données toutes les n modules (0 = auto)
  --max-frames <n>                     Maximum de frames de données (défaut : 300)
  --no-color                           Désactive les couleurs ANSI
  --no-mask-candidates                 Ne montre pas les 8 masques candidats

  -h, --help                           Affiche cette aide

Exemples :
  infinitylib-qrcode-rs "HELLO WORLD"
  infinitylib-qrcode-rs "HELLO WORLD" --format bits
  infinitylib-qrcode-rs "HELLO WORLD" --format svg -o hello.svg
  infinitylib-qrcode-rs "HELLO WORLD" --animate
  infinitylib-qrcode-rs "HELLO WORLD" --animate --fps 20 --version 4
  cargo run --features png -- "HELLO WORLD" --format png -o hello.png
"#
    );
}
