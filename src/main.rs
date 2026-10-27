mod qr;
mod error;

use qr::{CorrectionLevel, generate_qr_code};

fn main() {
    let data = "HELLO";
    let version = 1;
    let level = CorrectionLevel::Low;

    match generate_qr_code(data, version, level) {
        Ok(qr_code) => {
            let size = qr_code.modules.len();
            println!("QR Code Version {} (Correction: {:?}, Mask: {})", qr_code.version, qr_code.correction_level, qr_code.mask);
            println!("Size: {}x{}\n", size, size);
            
            // Affichage 1: Mode compact (0/1 avec espaces)
            println!("=== Mode Compact (avec espaces) ===");
            for row in &qr_code.modules {
                for &bit in row {
                    print!("{}", bit);
                }
                println!();
            }
            
            // Affichage 2: Mode lisible (avec espaces entre chaque bit)
            println!("\n=== Mode Lisible (espaces entre bits) ===");
            for row in &qr_code.modules {
                for &bit in row {
                    print!("{} ", bit);
                }
                println!();
            }
            
            // Affichage 3: Mode bloc large (1=noir, 0=blanc) avec marges
            println!("\n=== Mode Bloc Large (1=noir, 0=blanc) avec marges ===");
            
            // Marge supérieure (2 lignes vides)
            println!();
            println!();
            
            for row in &qr_code.modules {
                // Marge gauche (4 espaces)
                print!("    ");
                // Ligne supérieure du bloc (2x2 par bit)
                for &bit in row {
                    if bit == 1 {
                        print!("████"); // Noir pour 1
                    } else {
                        print!("    "); // Blanc pour 0
                    }
                }
                // Marge droite (4 espaces)
                println!("    ");
                // Ligne inférieure du bloc
                print!("    ");
                for &bit in row {
                    if bit == 1 {
                        print!("████"); // Noir pour 1
                    } else {
                        print!("    "); // Blanc pour 0
                    }
                }
                // Marge droite (4 espaces)
                println!("    ");
            }
            
            // Marge inférieure (2 lignes vides)
            println!();
            println!();
            
            // Affichage 4: Avec légendes
            println!("\n=== Avec Légendes ===");
            println!("Légende: F=Finder, T=Timing, D=Dark Module, *=Données");
            for (y, row) in qr_code.modules.iter().enumerate() {
                for (x, &bit) in row.iter().enumerate() {
                    // Détecter les motifs
                    if (x < 7 && y < 7) || (x >= size-7 && y < 7) || (x < 7 && y >= size-7) {
                        print!("F"); // Finder
                    } else if (x == 6 && y >= 8 && y < size-8) || (y == 6 && x >= 8 && x < size-8) {
                        print!("T"); // Timing
                    } else if x == 8 && y == 8 {
                        print!("D"); // Dark Module
                    } else if (x < 8 && y == 7) || (y < 8 && x == 7) || 
                              (x >= size-8 && y == 7) || (y < 8 && x == size-8) ||
                              (x < 8 && y == size-8) || (y >= size-8 && x == 7) {
                        print!("S"); // Séparateur
                    } else {
                        print!("*"); // Données
                    }
                }
                println!();
            }
        }
        Err(e) => eprintln!("Erreur: {:?}", e),
    }
}
