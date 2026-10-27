#!/usr/bin/env bash
# Point git at the hooks versioned in .githooks/, once per clone.
# A versioned hook cannot be configured by git alone: every clone installs it.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

if [ ! -d .githooks ]; then
    echo "no .githooks directory here" >&2
    exit 1
fi

chmod +x .githooks/* 2>/dev/null || true
git config core.hooksPath .githooks
echo "hooks installed: core.hooksPath=$(git config --get core.hooksPath)"
