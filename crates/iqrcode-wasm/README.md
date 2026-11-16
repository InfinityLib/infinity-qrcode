# iqrcode-wasm

Minimal WebAssembly wrapper around
[`iqrcode-core`](https://crates.io/crates/iqrcode-core), exposing QR Code
generation to JavaScript through `wasm-bindgen`.

Build the JavaScript package with `wasm-pack`, then install it from npm:

```bash
wasm-pack build crates/iqrcode-wasm --target web --scope infinitylib
npm install @infinitylib/iqrcode-wasm
```

Part of the [iqrcode workspace](https://github.com/InfinityLib/infinity-qrcode).

## License

Licensed under either of the Apache License, Version 2.0 or the MIT license,
at your option.
