# iqrcode-learn-wasm

WebAssembly bindings for the educational QR Code construction trace, exposing
every construction step of [`iqrcode-core`](https://crates.io/crates/iqrcode-core)
to JavaScript through `wasm-bindgen`.

Build the JavaScript package with `wasm-pack`, then install it from npm:

```bash
wasm-pack build crates/iqrcode-learn-wasm --target web --scope infinitylib
npm install @infinitylib/iqrcode-learn-wasm
```

Part of the [iqrcode workspace](https://github.com/InfinityLib/infinity-qrcode).

## License

Licensed under either of the Apache License, Version 2.0 or the MIT license,
at your option.
