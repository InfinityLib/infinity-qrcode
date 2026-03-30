mod qr;
mod error;

use qr::{CorrectionLevel, generate_qr_code};

fn main() {
    let data = "HELLO";
    let version = 1;
    let level = CorrectionLevel::Low;

    match generate_qr_code(data, version, level) {
        Ok(qr_code) => {
            for row in qr_code.modules {
                println!("{:?}", row);
            }
        }
        Err(e) => eprintln!("Erreur: {:?}", e),
    }
}