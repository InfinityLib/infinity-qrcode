# iqrcode

**English** | [Français](README.fr.md)

QR Code generator in Rust with a `no_std` core, a CLI, an educational terminal
animation and a WebAssembly version.

## Architecture

```text
crates/
  iqrcode-core/       QR core: encoding, ECC, matrix, optional trace
  iqrcode/            std facade + CLI + PNG
  iqrcode-learn/      neutral terminal construction animation
  iqrcode-learn-wasm/ WebAssembly educational API + web UI with FR/EN i18n
  iqrcode-wasm/       minimal WASM wrapper for QR generation
```

The core does not require `std` by default: it uses `alloc`.

## Minimal build

```bash
cargo build -p iqrcode-core --no-default-features
```

The core provides:

- Numeric, Alphanumeric, Byte and Kanji encoding,
- mixed-mode segmentation,
- Reed-Solomon and block interleaving,
- versions 1 to 40,
- ASCII, PBM and SVG output.

Without features there is no CLI, no PNG and no animation.

## Full build

```bash
cargo build -p iqrcode --features full
```

Features:

- `iqrcode-core`: `std`, `trace`, `png` (PNG implies `std`).
- `iqrcode`: `cli` (enabled by default), `png`, `full`.

The default `iqrcode` build enables `cli`.

## Usage

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

Other outputs:

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

Main options:

- `--level <low|medium|quartile|high>`
- `--version <1..40>`
- `--format <ascii|bits|pbm|svg|png>`
- `--scale <n>`
- `--quiet-zone <n>`
- `-o, --output <path>`
- `--bits`, `--info`, `--utf8-eci`

The CLI messages and help text are in English.

## Educational animation

```bash
cargo run -p iqrcode-learn -- "HELLO WORLD" --animate
cargo run -p iqrcode-learn -- "HELLO WORLD" --animate --version 4 --fps 20
cargo run -p iqrcode-learn -- "HELLO WORLD" --no-color
```

The terminal animation shows the phases and computed values:

- empty matrix,
- finder patterns,
- timing patterns,
- alignment patterns,
- version information,
- reserved format information,
- data placement,
- mask candidates and penalties,
- chosen mask,
- final format information,
- complete QR Code.

The terminal is intentionally technical and language-neutral. Explanations,
formulas and translations live in the web UI
(`crates/iqrcode-learn-wasm/web`).

Options:

- `--fps <n>` (default: 20)
- `--stride <n>` (0 = auto)
- `--max-frames <n>`
- `--quiet-zone <n>`
- `--no-color`
- `--no-mask-candidates`

## WASM

Full educational version with a web UI:

```bash
cd crates/iqrcode-learn-wasm
wasm-pack build --target web --out-dir web/pkg
cd web
python3 -m http.server 8080
```

Open <http://localhost:8080>.

The web UI owns its translations:

```text
crates/iqrcode-learn-wasm/web/i18n/fr.js
crates/iqrcode-learn-wasm/web/i18n/en.js
```

A language selector is available on the page. The WASM layer exposes only data
(phase, matrix, highlight, bit, mask, penalty), not text.

Minimal WASM wrapper:

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

Tests compare matrices against `qrcodegen` for versions 1 to 40, all 8 masks
and all 4 correction levels, then decode generated QR Codes with `rqrr`.

## Serde

Serde is not used for now. It would only be added if we need to export frames
as JSON, for example for another web player. The current WASM layer exposes
data directly through `wasm-bindgen`.

## Next steps

- UTF-8 to Shift-JIS conversion for Kanji mode.
- Web UI polish and HTML/GIF export.
- `no_std` validation on an embedded target.
- Optimizations and benchmarks.
