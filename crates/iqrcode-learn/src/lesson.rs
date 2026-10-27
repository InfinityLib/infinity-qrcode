//! Educational explanations for each construction phase.

use iqrcode_core::trace::Phase;

/// One pedagogical explanation.
#[derive(Debug, Clone, Copy)]
pub struct Lesson {
    pub title: &'static str,
    pub explanation: &'static str,
    pub formula: Option<&'static str>,
}

/// Lessons for the data pipeline, before the matrix is built.
pub const PIPELINE_LESSONS: [Lesson; 5] = [
    Lesson {
        title: "Segmentation",
        explanation: "Le texte est découpé en segments Numeric, Alphanumeric ou Byte. \
                      Chaque segment a un en-tête et un compteur, mais un mode plus compact \
                      peut faire économiser beaucoup de bits.",
        formula: Some("coût = 4 bits de mode + bits du compteur + bits des données"),
    },
    Lesson {
        title: "Flux de bits",
        explanation: "On ajoute un terminator, on aligne sur l'octet, puis on remplit la \
                      capacité avec les octets de padding 0xEC et 0x11 en alternance.",
        formula: Some("capacité = 8 × data_codewords"),
    },
    Lesson {
        title: "Codewords",
        explanation: "Les bits sont regroupés en octets, puis répartis en blocs selon la \
                      version et le niveau de correction.",
        formula: Some("data_codewords = total_codewords − ecc_par_bloc × blocs"),
    },
    Lesson {
        title: "Reed-Solomon",
        explanation: "Chaque bloc reçoit des codewords de correction. Le polynôme générateur \
                      a pour racines α^0 … α^(t−1), donc le codeword complet est divisible \
                      par ce polynôme.",
        formula: Some("g(x) = ∏(x − α^i),  c(x) = m(x)·x^t + r(x)"),
    },
    Lesson {
        title: "Entrelacement",
        explanation: "Les codewords de données puis d'ECC sont entrelacés entre les blocs. \
                      Une rayure sur le QR est ainsi répartie sur plusieurs blocs.",
        formula: Some("data[0][0], data[1][0], … puis ecc[0][0], ecc[1][0], …"),
    },
];

/// Returns the lesson associated with a matrix-construction phase.
pub fn for_phase(phase: Phase) -> Lesson {
    match phase {
        Phase::Empty => Lesson {
            title: "Matrice vide",
            explanation: "La matrice a 17 + 4v modules de côté. Tous les modules sont \
                          initialement clairs.",
            formula: Some("taille = 17 + 4v"),
        },
        Phase::Finders => Lesson {
            title: "Finder patterns",
            explanation: "Trois carrés 7×7 permettent au scanner de repérer le QR et son \
                          orientation. Chaque finder est entouré d'un séparateur clair.",
            formula: None,
        },
        Phase::Timing => Lesson {
            title: "Timing patterns",
            explanation: "La ligne 6 et la colonne 6 alternent noir et blanc. Elles donnent \
                          une référence de taille au décodeur.",
            formula: Some("module(i, 6) = i mod 2"),
        },
        Phase::Alignment => Lesson {
            title: "Alignment patterns",
            explanation: "Des motifs 5×5 sont placés sur une grille. Ils aident le scanner à \
                          corriger la perspective et les déformations.",
            formula: Some("n = v/7 + 2 ; step = (8v + 3n + 5)/(4n − 4) × 2"),
        },
        Phase::VersionInfo => Lesson {
            title: "Version information",
            explanation: "À partir de la version 7, 18 bits indiquent la version. Ils sont \
                          protégés par un code BCH(18,6) et placés en deux copies.",
            formula: Some("version_info = (version << 12) | BCH_12(version), générateur 0x1F25"),
        },
        Phase::FormatReserved => Lesson {
            title: "Format information réservée",
            explanation: "Deux copies de 15 bits accueilleront le niveau de correction et le \
                          masque. Le dark module est toujours noir.",
            formula: None,
        },
        Phase::DataPlacement => Lesson {
            title: "Placement des données",
            explanation: "Les bits sont placés par paires de colonnes, de droite à gauche, en \
                          zigzag. La colonne 6 est sautée car elle contient le timing pattern.",
            formula: Some("paires de colonnes : droite/gauche, montée/descente alternées"),
        },
        Phase::MaskCandidate => Lesson {
            title: "Candidat de masque",
            explanation: "On teste les 8 masques. Chaque masque est évalué avec quatre règles \
                          de pénalité : suites, blocs 2×2, motifs type finder et équilibre \
                          noir/blanc.",
            formula: Some("N1=3, N2=3, N3=40, N4=10 par tranche de 5 %"),
        },
        Phase::MaskChosen => Lesson {
            title: "Masque retenu",
            explanation: "Le masque de plus faible pénalité est appliqué aux modules de \
                          données uniquement. Les motifs fixes ne sont pas masqués.",
            formula: Some("masque = argmin(score)"),
        },
        Phase::FormatInfo => Lesson {
            title: "Format information",
            explanation: "Les 5 bits utiles (2 pour le niveau, 3 pour le masque) sont protégés \
                          par un BCH(15,5), puis XORés avec 0x5412.",
            formula: Some("format = ((data << 10) | BCH_10(data)) XOR 0x5412, générateur 0x537"),
        },
        Phase::Final => Lesson {
            title: "QR Code final",
            explanation: "La matrice est complète. Il ne reste plus qu'à ajouter la quiet \
                          zone et à l'afficher ou l'exporter.",
            formula: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_phase_has_a_lesson() {
        let phases = [
            Phase::Empty,
            Phase::Finders,
            Phase::Timing,
            Phase::Alignment,
            Phase::VersionInfo,
            Phase::FormatReserved,
            Phase::DataPlacement,
            Phase::MaskCandidate,
            Phase::MaskChosen,
            Phase::FormatInfo,
            Phase::Final,
        ];

        for phase in phases {
            let lesson = for_phase(phase);
            assert!(!lesson.title.is_empty());
            assert!(!lesson.explanation.is_empty());
        }
    }

    #[test]
    fn pipeline_lessons_are_present() {
        assert_eq!(PIPELINE_LESSONS.len(), 5);
        assert!(
            PIPELINE_LESSONS
                .iter()
                .all(|lesson| !lesson.explanation.is_empty())
        );
    }
}
