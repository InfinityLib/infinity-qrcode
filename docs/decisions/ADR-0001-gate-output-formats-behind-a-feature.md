# ADR-0001 : Gate output formats behind a `format` feature

- **Status** : `proposed`
- **Date** : 2026-11-16

## Context

`iqrcode-core` carried the ASCII, PBM, SVG and PNG serializers in every build.
The only consumer of `iqrcode-wasm` uses `matrix()`, `size()`, `version()` and
`mask()`; it renders SVG and PNG itself, with custom colours and canvas export,
and never calls the wrapper's `svg()`. Keeping the serializers unconditional
therefore shipped a second, redundant renderer — with fixed black and white
output — into the WebAssembly package, and into every minimal `no_std` build.

Constraints:

- the formats stay in `iqrcode-core`, where the `QrCode` inherent methods live;
- the default core build and the WASM package must not carry them;
- the `iqrcode` facade and the CLI keep every current format, including PNG;
- `iqrcode-learn` and `iqrcode-learn-wasm` keep building unchanged;
- no new dependency: `png` stays the only optional one.

## Decision

- The output formats stay in `crates/iqrcode-core/src/output.rs`, and the module
  is compiled only with the `format` feature:
  `#[cfg(feature = "format")] mod output;`.
- `format` does not require `std`, so the serializers remain usable in a
  `no_std + alloc` build. `png` implies `format` and `std`:
  `png = ["format", "std", "dep:png"]`.
- ASCII (including `to_ascii` and `impl Display for QrCode`), PBM and SVG
  require `format`; `to_png` requires `format` and `png`.
- The feature is named `format` because the CLI already exposes these as
  `--format <ascii|pbm|svg|png>` and the core module is `output`.
- `iqrcode-wasm` drops `svg()` and exposes only the constructor, `matrix()`,
  `size()`, `version()` and `mask()`.
- The `iqrcode` facade enables `std` and `format`, so the CLI keeps every
  format; `png` remains behind its own feature.

## Consequences

- **Positive**: the default core build and the WASM package no longer carry the
  serializers. Measured with `wasm-pack`, dropping the export takes the package
  from 58 320 bytes (25 665 gzipped) to 56 498 bytes (24 776 gzipped). The WASM
  API is clearer: one renderer, owned by the frontend.
- **Negative**: a consumer of `to_ascii`, `to_ascii_with_quiet_zone`, `to_pbm`,
  `to_svg`, `to_png` or `Display` on a default-feature build must enable
  `format`. This is a source-breaking change, acceptable before 0.2.0.
- **Trade-offs**: the feature adds a second way for a build to fail (a missing
  `format`), in exchange for an explicit contract about what the core emits.

## Rejected alternatives

- **A separate `iqrcode-format` crate**: rejected — one consumer only, inherent
  implementations cannot leave the crate that defines `QrCode`, and it would
  move code without removing a line of it.
- **Keeping `svg()` in the WASM wrapper**: rejected — fixed black and white
  output, a second renderer duplicating the frontend's, and 889 gzipped bytes
  for an unused export.
- **Naming the feature `render`**: rejected — the crate already uses
  "renderer" for the terminal animation in `iqrcode-learn`, and the formats are
  serializers, not a render pipeline.
