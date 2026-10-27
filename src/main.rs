use std::io::Write;
use std::process;

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
}

fn main() {
    if let Err(message) = run() {
        eprintln!("Erreur: {message}");
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
  -h, --help                           Affiche cette aide

Exemples :
  infinitylib-qrcode-rs "HELLO WORLD"
  infinitylib-qrcode-rs "HELLO WORLD" --format bits
  infinitylib-qrcode-rs "HELLO WORLD" --format svg -o hello.svg
  cargo run --features png -- "HELLO WORLD" --format png -o hello.png
"#
    );
}
