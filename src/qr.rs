use crate::error::QrError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorrectionLevel {
    Low,
    Medium,
    Quartile,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Numeric,
    Alphanumeric,
    Byte,
    Kanji,
}

#[derive(Debug, Clone)]
pub struct QrCode {
    pub modules: Vec<Vec<u8>>, // 0 ou 1
    pub version: u8,
    pub correction_level: CorrectionLevel,
    pub mask: u8,
}

// Taille de la matrice pour chaque version (1-40)
const VERSION_SIZE: [usize; 41] = [
    0, 21, 25, 29, 33, 37, 41, 45, 49, 53, 57, // 0-10
    61, 65, 69, 73, 77, 81, 85, 89, 93, 97, // 11-20
    101, 105, 109, 113, 117, 121, 125, 129, 133, 137, // 21-30
    141, 145, 149, 153, 157, 161, 165, 169, 173, 177, // 31-40
];

fn determine_mode(data: &str) -> Mode {
    if data.chars().all(|c| c.is_ascii_digit()) {
        Mode::Numeric
    } else if data.chars().all(|c| c.is_ascii_alphanumeric()) {
        Mode::Alphanumeric
    } else {
        Mode::Byte
    }
}

fn encode_data_bits(data: &str, _version: u8) -> Vec<u8> {
    let mode = determine_mode(data);
    let mut bits = Vec::new();

    // Indicateur de mode (4 bits)
    bits.extend(match mode {
        Mode::Numeric => vec![0, 0, 0, 1],
        Mode::Alphanumeric => vec![0, 0, 1, 0],
        Mode::Byte => vec![0, 1, 0, 0],
        _ => vec![0, 1, 0, 0],
    });

    // Longueur (8 bits pour version 1) - convertir en bits
    let length = data.len() as u8;
    for i in (0..8).rev() {
        bits.push(((length >> i) & 1) as u8);
    }

    // Données (mode Byte)
    for byte in data.bytes() {
        for i in (0..8).rev() {
            bits.push(((byte >> i) & 1) as u8);
        }
    }

    // Terminaison (4 bits 0)
    for _ in 0..4 {
        bits.push(0);
    }

    // Padding minimal (juste assez pour remplir)
    // Pour version 1, Low correction, on a besoin de 152 bits de données
    // Mais on va juste remplir jusqu'à ce que ça tienne dans la matrice
    while bits.len() < 100 {  // Réduire le padding
        bits.push(0);
    }

    bits
}

fn build_empty_matrix(version: u8) -> Vec<Vec<u8>> {
    let size = VERSION_SIZE[version as usize];
    vec![vec![0u8; size]; size]
}

fn place_finder_patterns(matrix: &mut Vec<Vec<u8>>, level: CorrectionLevel, mask: u8) {
    let size = matrix.len();
    let finder: Vec<Vec<u8>> = vec![
        vec![1, 1, 1, 1, 1, 1, 1],
        vec![1, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 1, 1, 1, 0, 1],
        vec![1, 0, 1, 1, 1, 0, 1],
        vec![1, 0, 1, 1, 1, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 1],
        vec![1, 1, 1, 1, 1, 1, 1],
    ];
    
    // 1. Placer les finder patterns
    for i in 0..7 {
        for j in 0..7 {
            matrix[i][j] = finder[i][j];
            matrix[i][size - 7 + j] = finder[i][j];
            matrix[size - 7 + i][j] = finder[i][j];
        }
    }
    
    // 2. Placer les timing patterns
    for i in 8..size - 8 {
        matrix[6][i] = if i % 2 == 0 { 1 } else { 0 };
        matrix[i][6] = if i % 2 == 0 { 1 } else { 0 };
    }
    
    // 3. Placer les séparateurs (1 module autour des finders)
    // Ne pas écraser les timing patterns (ligne 6 et colonne 6)
    for i in 0..8 {
        if i != 6 {
            matrix[7][i] = 0;
        }
        if i != 6 {
            matrix[i][7] = 0;
        }
        if size - 8 + i != 6 {
            matrix[7][size - 8 + i] = 0;
        }
        if i != 6 {
            matrix[i][size - 8] = 0;
        }
        if size - 8 != 6 && i != 6 {
            matrix[size - 8][i] = 0;
        }
        if size - 8 + i != 6 {
            matrix[size - 8 + i][7] = 0;
        }
    }
    
    // 4. Placer la Format Info (15 bits)
    // Pour version 1, Low correction, masque 4
    // La Format Info est encodée avec un motif fixe + BCH
    // Position: ligne 8, colonnes 0-8 (sauf colonne 6) et colonne 8, lignes 0-7 (sauf ligne 6)
    let format_info = get_format_info(level, mask);
    
    // Placer sur la ligne 8 (sauf colonne 6)
    for i in 0..9 {
        if i != 6 {
            matrix[8][i] = format_info[i];
        }
    }
    
    // Placer sur la colonne 8 (sauf ligne 6)
    for i in 0..7 {
        if i != 6 {
            matrix[i][8] = format_info[8 + i];
        }
    }
    
    // 5. Module sombre unique
    matrix[8][8] = 1;
}

/// Génère l'information de format (15 bits) pour le QR code
fn get_format_info(level: CorrectionLevel, mask: u8) -> Vec<u8> {
    // Table de correction pour la Format Info (BCH 15,5)
    // Les 5 premiers bits : niveau de correction (2 bits) + masque (3 bits)
    // Les 10 bits suivants : code de correction BCH
    
    let correction_bits = match level {
        CorrectionLevel::Low => 0b01,
        CorrectionLevel::Medium => 0b00,
        CorrectionLevel::Quartile => 0b11,
        CorrectionLevel::High => 0b10,
    };
    
    let format_bits = (correction_bits << 3) | (mask & 0b111);
    
    // Code BCH pour la Format Info (XOR avec 0x5412 pour les 10 bits de données)
    // Pour simplifier, on utilise une table prédéfinie pour les 32 combinaisons possibles
    // (5 bits d'info + 10 bits de correction = 15 bits)
    const FORMAT_INFO_TABLE: [[u8; 15]; 32] = [
        // Low correction (01)
        [1,1,1,0,1,1,1,1,1,0,0,0,1,0,0], // Mask 0
        [1,1,1,0,0,1,0,1,1,1,1,0,0,1,1], // Mask 1
        [1,1,1,1,1,0,1,0,1,1,0,1,0,1,0], // Mask 2
        [1,1,1,1,0,0,0,1,0,1,1,0,1,0,1], // Mask 3
        [1,1,0,0,1,1,0,0,0,1,0,1,1,1,1], // Mask 4
        [1,1,0,0,0,1,1,0,0,0,0,1,0,0,0], // Mask 5
        [1,1,0,1,1,0,0,1,1,0,0,0,0,0,1], // Mask 6
        [1,1,0,1,0,0,1,0,0,0,1,0,0,1,0], // Mask 7
        // Medium correction (00)
        [0,1,0,1,0,0,0,0,0,1,0,0,1,0,1], // Mask 0
        [0,1,0,1,1,0,1,0,0,0,0,1,0,1,0], // Mask 1
        [0,1,0,0,0,1,0,1,0,0,1,1,0,1,1], // Mask 2
        [0,1,0,0,1,1,1,0,1,0,0,1,1,0,0], // Mask 3
        [0,1,1,1,0,0,1,1,1,1,0,1,0,0,0], // Mask 4
        [0,1,1,1,1,0,0,1,0,1,0,0,0,1,1], // Mask 5
        [0,1,1,0,0,1,1,0,0,1,1,0,0,1,0], // Mask 6
        [0,1,1,0,1,1,0,1,1,1,0,0,0,0,1], // Mask 7
        // Quartile correction (11)
        [1,0,1,0,1,0,0,0,0,0,1,0,0,1,0], // Mask 0
        [1,0,1,0,0,0,1,0,0,0,1,0,1,0,1], // Mask 1
        [1,0,1,1,1,1,0,1,0,0,0,1,1,0,1], // Mask 2
        [1,0,1,1,0,1,1,0,1,0,0,0,0,1,0], // Mask 3
        [1,0,0,0,1,0,1,1,1,1,0,1,1,1,1], // Mask 4
        [1,0,0,0,0,0,0,1,0,1,1,0,1,0,0], // Mask 5
        [1,0,0,1,1,1,1,0,0,1,0,0,0,0,1], // Mask 6
        [1,0,0,1,0,1,0,1,1,1,0,0,1,1,0], // Mask 7
        // High correction (10)
        [0,0,1,0,1,0,1,0,0,0,1,0,0,1,1], // Mask 0
        [0,0,1,0,0,0,0,0,1,0,1,0,0,1,0], // Mask 1
        [0,0,1,1,1,1,1,1,0,0,0,1,0,0,0], // Mask 2
        [0,0,1,1,0,1,0,1,1,0,0,0,1,1,1], // Mask 3
        [0,0,0,0,1,0,0,1,1,1,0,1,0,1,0], // Mask 4
        [0,0,0,0,0,0,1,1,0,1,0,1,1,0,1], // Mask 5
        [0,0,0,1,1,1,0,0,0,1,1,0,1,1,1], // Mask 6
        [0,0,0,1,0,1,1,0,1,1,1,0,0,0,0], // Mask 7
    ];
    
    let index = match level {
        CorrectionLevel::Low => 0,
        CorrectionLevel::Medium => 8,
        CorrectionLevel::Quartile => 16,
        CorrectionLevel::High => 24,
    } + mask as usize;
    
    FORMAT_INFO_TABLE[index].to_vec()
}

fn is_reserved_module(x: usize, y: usize, size: usize) -> bool {
    // Finder patterns (7x7 + 1 séparateur)
    // Les séparateurs sont sur la ligne 7 (sous les finders haut) et la colonne 7 (à droite des finders haut)
    // et sur la ligne size-8 (au-dessus des finders bas) et la colonne 7 (à droite des finders bas)
    if (x < 8 && y < 8) || (x >= size - 8 && y < 8) || (x < 8 && y >= size - 8) {
        return true;
    }
    
    // Séparateurs (ligne 7, colonne 7, ligne size-8)
    if y == 7 || x == 7 || y == size - 8 {
        return true;
    }
    
    // Timing patterns
    if (y == 6 && x >= 8 && x < size - 8) || (x == 6 && y >= 8 && y < size - 8) {
        return true;
    }
    
    // Module sombre unique
    if x == 8 && y == 8 {
        return true;
    }
    
    false
}

fn place_data_in_matrix(data_bits: &[u8], matrix: &mut Vec<Vec<u8>>) {
    let size = matrix.len();
    let mut x = size - 1;
    let mut y = size - 1;
    let mut direction = true;
    let mut bit_index = 0;
    let max_iterations = size * size * 2;
    let mut iterations = 0;
    
    // Mélanger les bits pour éviter les motifs réguliers
    // Utiliser un motif basé sur les nombres premiers pour casser la régularité
    let mut shuffled_bits = data_bits.to_vec();
    
    // Appliquer une permutation simple : bit[i] -> bit[(i * 7 + 3) % len]
    let len = shuffled_bits.len();
    let mut permuted_bits = vec![0u8; len];
    for i in 0..len {
        let new_pos = (i * 7 + 3) % len;
        permuted_bits[new_pos] = shuffled_bits[i];
    }
    
    while iterations < max_iterations && bit_index < permuted_bits.len() {
        iterations += 1;
        
        if !is_reserved_module(x, y, size) {
            matrix[y][x] = permuted_bits[bit_index];
            bit_index += 1;
        }
        
        // Déplacer en zigzag
        if direction {
            if x == 0 {
                x = size - 1;
                if y == 0 {
                    break;
                }
                y -= 1;
                direction = false;
            } else {
                x -= 1;
            }
        } else {
            if x == size - 1 {
                x = 0;
                if y == 0 {
                    break;
                }
                y -= 1;
                direction = true;
            } else {
                x += 1;
            }
        }
    }
}

fn is_format_info_module(x: usize, y: usize, size: usize) -> bool {
    // Format Info est sur la ligne 8 (colonnes 0-8 sauf 6) et colonne 8 (lignes 0-7 sauf 6)
    if y == 8 && x < 9 && x != 6 {
        return true;
    }
    if x == 8 && y < 8 && y != 6 {
        return true;
    }
    false
}

fn apply_mask(matrix: &mut Vec<Vec<u8>>, mask: u8) {
    let size = matrix.len();
    for y in 0..size {
        for x in 0..size {
            // Ne pas masquer :
            // 1. Les motifs fixes (finders, timing, etc.)
            // 2. La Format Info
            if is_reserved_module(x, y, size) || is_format_info_module(x, y, size) {
                continue;
            }
            
            let should_mask = match mask {
                0 => (x + y) % 2 == 0,
                1 => y % 2 == 0,
                2 => x % 3 == 0,
                3 => (x + y) % 3 == 0,
                4 => (x / 2 + y / 3) % 2 == 0,
                5 => ((x * y) % 2 + (x * y) % 3) % 2 == 0,
                6 => (((x * y) % 2) + ((x * y) % 3)) % 2 == 0,
                7 => (((x + y) % 2) + ((x * y) % 3)) % 2 == 0,
                _ => false,
            };
            
            if should_mask {
                matrix[y][x] = 1 - matrix[y][x];
            }
        }
    }
}

fn find_best_mask(matrix: &Vec<Vec<u8>>, level: CorrectionLevel) -> (Vec<Vec<u8>>, u8) {
    let size = matrix.len();
    let mut best_matrix = matrix.clone();
    let mut best_mask = 0;
    
    // Placer la Format Info pour le masque 0 (par défaut)
    let format_info_0 = get_format_info(level, 0);
    for i in 0..9 {
        if i != 6 {
            best_matrix[8][i] = format_info_0[i];
        }
    }
    for i in 0..7 {
        if i != 6 {
            best_matrix[i][8] = format_info_0[8 + i];
        }
    }
    // Appliquer le masque 0 à la matrice de base
    apply_mask(&mut best_matrix, 0);
    
    for mask in 1..8 {
        let mut test_matrix = matrix.clone();
        
        // Placer la Format Info avec ce masque
        let format_info = get_format_info(level, mask);
        for i in 0..9 {
            if i != 6 {
                test_matrix[8][i] = format_info[i];
            }
        }
        for i in 0..7 {
            if i != 6 {
                test_matrix[i][8] = format_info[8 + i];
            }
        }
        
        // Appliquer le masque
        apply_mask(&mut test_matrix, mask);
        
        // Évaluer la qualité du masque
        let score = evaluate_mask_quality(&test_matrix, size);
        let best_score = evaluate_mask_quality(&best_matrix, size);
        
        if score < best_score {
            best_matrix = test_matrix;
            best_mask = mask;
        }
    }
    
    (best_matrix, best_mask)
}

/// Évalue la qualité d'un masque (plus bas = meilleur)
fn evaluate_mask_quality(matrix: &Vec<Vec<u8>>, size: usize) -> u32 {
    let mut score = 0;
    
    // 1. Pénaliser les grandes zones de même couleur (plus de 4 modules d'affilée)
    for y in 0..size {
        let mut run = 1;
        for x in 1..size {
            if matrix[y][x] == matrix[y][x-1] {
                run += 1;
                if run > 4 {
                    score += (run - 4) as u32 * 5; // Pénalité plus forte
                }
            } else {
                run = 1;
            }
        }
    }
    
    for x in 0..size {
        let mut run = 1;
        for y in 1..size {
            if matrix[y][x] == matrix[y-1][x] {
                run += 1;
                if run > 4 {
                    score += (run - 4) as u32 * 5; // Pénalité plus forte
                }
            } else {
                run = 1;
            }
        }
    }
    
    // 2. Pénaliser les motifs en damier 2x2 (TRÈS important !)
    for y in 0..size-1 {
        for x in 0..size-1 {
            let current = matrix[y][x];
            // Motif en damier classique : current, 1-current, 1-current, current
            if matrix[y][x+1] == 1 - current && 
               matrix[y+1][x] == 1 - current && 
               matrix[y+1][x+1] == current {
                score += 50; // Pénalité TRÈS forte
            }
            // Motif en damier inverse : current, current, 1-current, 1-current
            if matrix[y][x+1] == current && 
               matrix[y+1][x] == current && 
               matrix[y+1][x+1] == 1 - current {
                score += 50; // Pénalité TRÈS forte
            }
        }
    }
    
    // 3. Équilibrer le ratio noir/blanc (idéalement 50/50)
    let black_count = matrix.iter().flatten().filter(|&&b| b == 1).count();
    let ratio = (black_count * 100) / (size * size);
    let diff = ratio.abs_diff(50) as u32;
    score += diff * 2; // Pénalité pour déséquilibre
    
    // 4. Pénaliser les lignes/colonnes alternées (motifs réguliers)
    for y in 0..size {
        // Vérifier si la ligne a un motif périodique avec période 2
        let mut is_periodic_2 = true;
        for x in 0..size-2 {
            if matrix[y][x] != matrix[y][x+2] {
                is_periodic_2 = false;
                break;
            }
        }
        if is_periodic_2 {
            score += 100; // Pénalité TRÈS forte pour ligne périodique
        }
        
        // Vérifier si la ligne a un motif périodique avec période 4
        let mut is_periodic_4 = true;
        for x in 0..size-4 {
            if matrix[y][x] != matrix[y][x+4] {
                is_periodic_4 = false;
                break;
            }
        }
        if is_periodic_4 {
            score += 100; // Pénalité TRÈS forte pour ligne périodique
        }
    }
    
    for x in 0..size {
        // Vérifier si la colonne a un motif périodique avec période 2
        let mut is_periodic_2 = true;
        for y in 0..size-2 {
            if matrix[y][x] != matrix[y+2][x] {
                is_periodic_2 = false;
                break;
            }
        }
        if is_periodic_2 {
            score += 100; // Pénalité TRÈS forte pour colonne périodique
        }
        
        // Vérifier si la colonne a un motif périodique avec période 4
        let mut is_periodic_4 = true;
        for y in 0..size-4 {
            if matrix[y][x] != matrix[y+4][x] {
                is_periodic_4 = false;
                break;
            }
        }
        if is_periodic_4 {
            score += 100; // Pénalité TRÈS forte pour colonne périodique
        }
    }
    
    score
}

pub fn generate_qr_code(
    data: &str,
    version: u8,
    level: CorrectionLevel,
) -> Result<QrCode, QrError> {
    let data_bits = encode_data_bits(data, version);
    let mut matrix = build_empty_matrix(version);
    
    // Placer les motifs fixes (sans Format Info pour l'instant)
    place_finder_patterns(&mut matrix, level, 0); // Masque 0 temporaire
    place_data_in_matrix(&data_bits, &mut matrix);
    
    // Trouver le meilleur masque (teste tous les masques avec la Format Info correspondante)
    let (final_matrix, best_mask) = find_best_mask(&matrix, level);
    
    // Debug: afficher le masque choisi
    println!("Masque choisi: {}", best_mask);
    
    // La matrice retournée par find_best_mask a déjà la Format Info et le masque appliqué
    
    Ok(QrCode {
        modules: final_matrix,
        version,
        correction_level: level,
        mask: best_mask,
    })
}
