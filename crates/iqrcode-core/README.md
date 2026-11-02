# iqrcode-core

Core QR Code encoder in Rust: encoding, Reed-Solomon error correction, matrix
construction and optional construction trace.

- QR Code versions 1 to 40, all four error correction levels.
- Numeric, Alphanumeric, Byte and Kanji modes, with automatic mixed-mode
  segmentation.
- `no_std` compatible by default (requires `alloc`).
- Output as a matrix, ASCII, PBM or SVG; PNG behind the `png` feature.
- Optional `trace` feature recording every construction step.

```rust
use iqrcode_core::{CorrectionLevel, QrCode};

let qr = QrCode::encode("HELLO WORLD", CorrectionLevel::Medium).unwrap();
let matrix = qr.to_matrix();
```

Part of the [iqrcode workspace](https://github.com/InfinityLib/infinity-qrcode).

## License

Licensed under either of the Apache License, Version 2.0 or the MIT license,
at your option.
