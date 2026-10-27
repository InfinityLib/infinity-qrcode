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
        eprintln!("Erreur: {message}");
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
            "--quiet-zone" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| "--quiet-zone attend une valeur".to_string())?;
                quiet_zone = value
                    .parse::<usize>()
                    .map_err(|_| format!("quiet zone invalide: {value}"))?;
            }
            "--no-color" => no_color = true,
            "--no-mask-candidates" => show_mask_candidates = false,
            "--animate" => {}
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
        _ => Err(format!("niveau de correction invalide: {value}")),
    }
}

fn print_help() {
    println!(
        r#"iqrcode-learn — animation pédagogique d'un QR Code

Usage :
  iqrcode-learn [DATA] [OPTIONS]

Options :
  --level <low|medium|quartile|high>   Niveau de correction (défaut : medium)
  --version <1..40>                    Force une version (défaut : automatique)
  --fps <n>                            Images par seconde (défaut : 20)
  --stride <n>                         Une frame de données toutes les n modules (0 = auto)
  --max-frames <n>                     Maximum de frames de données (défaut : 300)
  --quiet-zone <n>                     Quiet zone en modules (défaut : 4)
  --no-color                           Désactive les couleurs ANSI
  --no-mask-candidates                 Ne montre pas les 8 masques candidats
  -h, --help                           Affiche cette aide

Remarque : les explications et les formules sont dans l'UI web
(`crates/iqrcode-learn-wasm/web`). Le terminal montre les phases et les valeurs.

Exemples :
  iqrcode-learn "HELLO WORLD" --animate
  iqrcode-learn "HELLO WORLD" --animate --version 4 --fps 20
"#
    );
}
