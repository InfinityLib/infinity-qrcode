# iqrcode

[English](README.md) | **Français**

Générateur de QR Code en Rust, avec un cœur `no_std`, une CLI, une animation
terminale pédagogique et une version WebAssembly.

## Architecture

```text
crates/
  iqrcode-core/       cœur QR : encodage, ECC, matrice, trace optionnelle
  iqrcode/            façade std + CLI + PNG
  iqrcode-learn/      animation terminale neutre de la construction
  iqrcode-learn-wasm/ API WebAssembly pédagogique + UI web i18n FR/EN
  iqrcode-wasm/       wrapper WASM minimal pour générer un QR
```

Le cœur ne dépend pas de `std` par défaut : il utilise `alloc`.

## Version minimale

```bash
cargo build -p iqrcode-core --no-default-features
```

Le cœur expose :

- l'encodage Numeric / Alphanumeric / Byte / Kanji,
- la segmentation multi-modes,
- Reed-Solomon et l'entrelacement,
- les versions 1 à 40,
- les sorties ASCII, PBM et SVG.

Sans feature, il n'y a ni CLI, ni PNG, ni animation.

## Version complète

```bash
cargo build -p iqrcode --features full
```

Features :

- `iqrcode-core` : `std`, `trace`, `png` (PNG implique `std`).
- `iqrcode` : `cli` (activée par défaut), `png`, `full`.

Le build par défaut de `iqrcode` active `cli`.

## Utilisation

```rust
use iqrcode::{CorrectionLevel, QrCode};

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

Autres sorties :

```rust
let pbm = qr.to_pbm(4);
let svg = qr.to_svg(8, 4);

#[cfg(feature = "png")]
let png = qr.to_png(8, 4).unwrap();
```

## CLI

```bash
cargo run -p iqrcode -- "HELLO WORLD"
cargo run -p iqrcode -- "HELLO WORLD" --format bits
cargo run -p iqrcode -- "HELLO WORLD" --format pbm -o hello.pbm
cargo run -p iqrcode -- "HELLO WORLD" --format svg --scale 8 -o hello.svg
cargo run -p iqrcode --features png -- "HELLO WORLD" --format png -o hello.png
cargo run -p iqrcode -- "Héllo" --utf8-eci
```

Options principales :

- `--level <low|medium|quartile|high>`
- `--version <1..40>`
- `--format <ascii|bits|pbm|svg|png>`
- `--scale <n>`
- `--quiet-zone <n>`
- `-o, --output <chemin>`
- `--bits`, `--info`, `--utf8-eci`

Les messages et l'aide de la CLI sont en anglais.

## Animation pédagogique

```bash
cargo run -p iqrcode-learn -- "HELLO WORLD" --animate
cargo run -p iqrcode-learn -- "HELLO WORLD" --animate --version 4 --fps 20
cargo run -p iqrcode-learn -- "HELLO WORLD" --no-color
```

L'animation terminale montre les phases et les valeurs calculées :

- matrice vide,
- finder patterns,
- timing patterns,
- alignment patterns,
- version information,
- format information réservée,
- placement des données,
- masques candidats et pénalités,
- masque retenu,
- format information finale,
- QR Code complet.

Le terminal est volontairement technique et neutre. Les explications, formules
et traductions sont dans l'UI web (`crates/iqrcode-learn-wasm/web`).

Options :

- `--fps <n>` (défaut : 20)
- `--stride <n>` (0 = auto)
- `--max-frames <n>`
- `--quiet-zone <n>`
- `--no-color`
- `--no-mask-candidates`

## WASM

Version pédagogique complète avec UI web :

```bash
cd crates/iqrcode-learn-wasm
wasm-pack build --target web --out-dir web/pkg
cd web
python3 -m http.server 8080
```

Ouvrir <http://localhost:8080>.

L'UI web gère elle-même les traductions :

```text
crates/iqrcode-learn-wasm/web/i18n/fr.js
crates/iqrcode-learn-wasm/web/i18n/en.js
```

Un sélecteur de langue est disponible dans la page. Le WASM expose uniquement
les données (phase, matrice, highlight, bit, masque, pénalité), pas les textes.

Wrapper WASM minimal :

```bash
wasm-pack build crates/iqrcode-wasm --target web
```

## Tests

```bash
cargo test --workspace --all-features
cargo test -p iqrcode-core --features trace
cargo check -p iqrcode-core --no-default-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --check
```

Les tests comparent les matrices à `qrcodegen` pour les versions 1 à 40, les
8 masques et les 4 niveaux de correction, puis décodent les QR générés avec
`rqrr`.

## Serde

Serde n'est pas utilisé pour l'instant. Il ne sera ajouté que si on veut
exporter les frames en JSON, par exemple pour un autre lecteur web. Le WASM
actuel expose directement les données via `wasm-bindgen`.

## Prochaines étapes

- Conversion UTF-8 vers Shift-JIS pour le mode Kanji.
- Peaufinage de l'UI web et export HTML/GIF.
- Validation `no_std` sur une cible embarquée.
- Optimisations et benchmarks.

## Licence

Distribué sous licence Apache 2.0 ou MIT, au choix.

- [LICENSE-APACHE](LICENSE-APACHE)
- [LICENSE-MIT](LICENSE-MIT)
