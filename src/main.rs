use infinitylib_qrcode_rs::{CorrectionLevel, QrCode};

fn main() {
    let mut arguments = std::env::args().skip(1);
    let data = arguments
        .next()
        .unwrap_or_else(|| "HELLO WORLD".to_string());
    let bits_only = arguments.any(|argument| argument == "--bits");

    match QrCode::encode(&data, CorrectionLevel::Medium) {
        Ok(qr) => {
            if bits_only {
                for row in qr.modules() {
                    for &bit in row {
                        print!("{bit}");
                    }
                    println!();
                }
            } else {
                println!(
                    "Version {}, correction {:?}, masque {}",
                    qr.version(),
                    qr.correction_level(),
                    qr.mask()
                );
                println!("{qr}");
            }
        }
        Err(error) => {
            eprintln!("Erreur: {error}");
            std::process::exit(1);
        }
    }
}
