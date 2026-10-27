# infinitylib-qrcode-rs

Générateur de QR Code en Rust.

## État actuel

Le projet génère de vrais QR Codes **versions 1 à 40 (21×21 à 177×177)**,
décodables par un lecteur standard.

- Modes Numeric, Alphanumeric, Byte et Kanji (Shift-JIS brut).
- Segmentation multi-modes automatique pour réduire la taille.
- ECI 26 / UTF-8 explicite.
- Niveaux de correction `Low`, `Medium`, `Quartile` et `High`.
- Reed-Solomon, blocs ECC, interleaving, alignment patterns, version
  information, placement des données, masques et format information, sans
  dépendance runtime (hors feature `png`).
- Sélection automatique de la plus petite version qui contient les données.
- Sortie sous forme de matrice `Vec<Vec<u8>>` avec des `0` et des `1`.
- Sorties ASCII, PBM, SVG et PNG (feature `png`).
- Animation terminale pas à pas de la construction.
- Quiet zone configurable.

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

Les autres formats sont disponibles directement sur `QrCode` :

```rust
let pbm = qr.to_pbm(4);
let svg = qr.to_svg(8, 4); // taille d'un module, quiet zone

#[cfg(feature = "png")]
let png: Vec<u8> = qr.to_png(8, 4).unwrap();
```

Pour indiquer explicitement que les données sont de l'UTF-8 :

```rust
let qr = QrCode::encode_utf8("Héllo, wörld !", CorrectionLevel::Medium)?;
```

Pour le mode Kanji, l'API attend des octets Shift-JIS bruts :

```rust
let qr = QrCode::encode_kanji_shift_jis(b"\x93\x5f\xe4\xaa", CorrectionLevel::Low)?;
```

## Démo

```bash
cargo run -- "HELLO WORLD"
cargo run -- "HELLO WORLD" --format bits
cargo run -- "HELLO WORLD" --format pbm -o hello.pbm
cargo run -- "HELLO WORLD" --format svg --scale 8 -o hello.svg
cargo run -- "Héllo" --utf8-eci
cargo run --features png -- "HELLO WORLD" --format png -o hello.png
```

Options principales :

- `--level <low|medium|quartile|high>`
- `--version <1..40>`
- `--format <ascii|bits|pbm|svg|png>`
- `--scale <n>`
- `--quiet-zone <n>`
- `-o, --output <chemin>`
- `--bits`, `--info`, `--utf8-eci`, `--help`

## Animation terminale

L’animation montre la vraie construction du QR Code, étape par étape :

```bash
cargo run -- "HELLO WORLD" --animate
cargo run -- "HELLO WORLD" --animate --version 4 --fps 20
cargo run -- "HELLO WORLD" --animate --no-color --no-mask-candidates
```

Options dédiées :

- `--animate`
- `--fps <n>` : images par seconde (défaut : 20)
- `--stride <n>` : une frame de données tous les `n` modules
- `--max-frames <n>` : limite le nombre de frames de données
- `--no-color`
- `--no-mask-candidates`

La dernière frame est exactement la matrice renvoyée par `to_matrix()`.

## Tests

```bash
cargo test
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
```

Les tests unitaires vérifient les vecteurs connus du standard, comparent les
matrices à `qrcodegen` pour les versions 1 à 40, les 8 masques et les 4 niveaux
de correction, puis décodent les QR générés avec `rqrr`.

## Prochaines étapes

- Conversion UTF-8 vers Shift-JIS pour le mode Kanji.
- Export HTML ou GIF à partir de la même trace.
- Optimisations et support `no_std`.
