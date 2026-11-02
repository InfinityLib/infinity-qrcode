# iqrcode

QR Code generator with a command-line interface and optional PNG output,
built on [`iqrcode-core`](https://crates.io/crates/iqrcode-core).

- `cli` feature (default): the `iqrcode` binary.
- `png` feature: PNG output.
- `full`: both.

```bash
cargo install iqrcode
iqrcode "HELLO WORLD"
```

Or as a library:

```rust
use iqrcode::{CorrectionLevel, QrCode};

let qr = QrCode::encode("HELLO WORLD", CorrectionLevel::Medium).unwrap();
println!("{}", qr.to_ascii());
```

Part of the [iqrcode workspace](https://github.com/InfinityLib/infinity-qrcode).

## License

Licensed under either of the Apache License, Version 2.0 or the MIT license,
at your option.
