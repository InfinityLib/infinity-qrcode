#!/usr/bin/env bash
# Local CI: the checks that must pass before a push or a merge request.
#
# Run from anywhere in the repository:
#   bash scripts/ci.sh
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

step() {
    printf '\n==> %s\n' "$1"
}

step "Formatting"
cargo fmt --check

step "Linting"
cargo clippy --workspace --all-targets --all-features -- -D warnings

step "Tests"
cargo test --workspace --all-features

step "Minimal core build (no_std)"
cargo check -p iqrcode-core --no-default-features

step "Library without CLI"
cargo check -p iqrcode --no-default-features

step "WebAssembly targets"
cargo check -p iqrcode-learn-wasm --target wasm32-unknown-unknown
cargo check -p iqrcode-wasm --target wasm32-unknown-unknown

if command -v node >/dev/null 2>&1; then
    step "Web JavaScript syntax"
    node --input-type=module --check < crates/iqrcode-learn-wasm/web/app.js
    node --input-type=module --check < crates/iqrcode-learn-wasm/web/i18n/fr.js
    node --input-type=module --check < crates/iqrcode-learn-wasm/web/i18n/en.js
else
    printf '\n  node is not installed: web JavaScript checks skipped.\n' >&2
fi

if command -v ibrain >/dev/null 2>&1; then
    step "ibrain synchronization"
    ibrain sync --check
else
    printf '\n  ibrain is not installed: governance check skipped.\n' >&2
fi

printf '\nAll local CI checks passed.\n'
