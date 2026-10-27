# infinitylib-qrcode-rs

Générateur de QR Code en Rust.

## État actuel

Le projet génère de vrais QR Codes **versions 1 à 40 (21×21 à 177×177)**,
décodables par un lecteur standard.

- Modes Numeric, Alphanumeric et Byte.
- Niveaux de correction `Low`, `Medium`, `Quartile` et `High`.
- Reed-Solomon, blocs ECC, interleaving, alignment patterns, version
  information, placement des données, masques et format information, sans
  dépendance runtime.
- Sélection automatique de la plus petite version qui contient les données.
- Sortie sous forme de matrice `Vec<Vec<u8>>` avec des `0` et des `1`.
- Rendu ASCII avec quiet zone.

## Utilisation

```rust
use infinitylib_qrcode_rs::{CorrectionLevel, QrCode};

let qr = QrCode::encode("HELLO WORLD", CorrectionLevel::Medium).unwrap();
let matrix = qr.to_matrix();

assert_eq!(matrix.len(), 21);
for row in &matrix {
    for &bit in row {
        print!("{bit}");
    }
    println!();
}
```

La matrice renvoyée ne contient pas de quiet zone : `1` représente un module
noir et `0` un module blanc.

Pour un affichage lisible directement dans le terminal :

```rust
println!("{}", qr.to_ascii());
```

## Démo

```bash
cargo run -- "HELLO WORLD"
cargo run -- "HELLO WORLD" --bits
```

## Tests

```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Les tests unitaires vérifient les vecteurs connus du standard, comparent les
matrices à `qrcodegen` pour les versions 1 à 40, les 8 masques et les 4 niveaux
de correction, puis décodent les QR générés avec `rqrr`.

## Prochaines étapes

- Sorties PBM, SVG et PNG.
- Mode Kanji, ECI/UTF-8 et segmentation multi-modes.
- Optimisations et support `no_std`.
