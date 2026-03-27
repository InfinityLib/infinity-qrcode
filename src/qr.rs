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
    pub modules: Vec<Vec<bool>>,
    pub version: u8,
    pub correction_level: CorrectionLevel,
    pub mask: u8,
}

fn determine_mode(data: &str) -> Mode {
    if data.chars().all(|c| c.is_ascii_digit()) {
        Mode::Numeric
    } else if data.chars().all(|c| c.is_alphanumeric()) {
        Mode::Alphanumeric
    } else {
        Mode::Byte
    }
}

fn encode_numeric(data: &str) -> Vec<u8> {
    // TODO: Regrouper par 3 chiffres et convertir en bits
    vec![]
}

fn encode_alphanumeric(data: &str) -> Vec<u8> {
    // TODO: Convertir chaque caractère en 6 bits
    vec![]
}

fn encode_byte(data: &str) -> Vec<u8> {
    // TODO: Convertir chaque octet en 8 bits (UTF-8)
    data.bytes().flat_map(|b| {
        (0..8).rev().map(move |i| ((b >> i) & 1) as u8)
    }).collect()
}

/// Encodage des données selon le mode choisi.
/// Retourne un vecteur de bits (0 ou 1) ou une erreur.
pub fn encode_data(data: &str, mode: Mode) -> Result<Vec<u8>, QrError> {
    let mut bits = Vec::new();

    bits.extend(match mode {
        Mode::Numeric => vec![0, 0, 0, 1],
        Mode::Alphanumeric => vec![0, 0, 1, 0],
        Mode::Byte => vec![0, 1, 0, 0],
        _ => return Err(QrError::InvalidMode),
    });

    let length = data.len() as u8;
    bits.extend(((length >> 4) as u8 ).to_be_bytes());
    bits.extend((length as u8 ).to_be_bytes());

    // TODO 4: Convertir les données en bits selon le mode
    // - Pour chaque caractère/paquet de données :
    //   - Mode Numeric : regrouper par 3 chiffres → 10 bits.
    //   - Mode Alphanumeric : convertir chaque caractère en 6 bits (tableau prédéfini).
    //   - Mode Byte : convertir chaque octet en 8 bits (UTF-8).

    // TODO 5: Retourner le vecteur de bits
    // Exemple de retour : vec![0, 0, 0, 1, 0, 0, 0, 0, ...]

    // Pour l'instant, retourne un résultat vide (à implémenter)
    Ok(vec![])
}

pub fn add_error_correction(
    data: &[u8],
    version: u8,
    level: CorrectionLevel,
) -> Result<Vec<u8>, QrError> {
    // TODO: Implémenter
    use reed_solomon::ReedSolomon;
    let rs = ReedSolomon::new(10, 17); // (taille_correction, taille_données)
    let mut corrected = rs.encode(data)?;
}

fn build_empty_matrix(version: u8) -> Vec<Vec<bool>> {
// TODO: Implémenter
}

fn place_data_in_matrix(data: &[u8], matrix: &mut Vec<Vec<bool>>, version: u8) {
    // TODO: Implémenter
}

fn apply_mask(matrix: &mut Vec<Vec<bool>>, mask: u8) -> u32 {
    // TODO: Implémenter
}

pub fn generate_qr_code(
    data: &str,
    version: u8,
    level: CorrectionLevel,
) -> Result<QrCode, QrError> {
    // TODO: Implémenter
}