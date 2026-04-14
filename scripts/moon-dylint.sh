#!/usr/bin/env bash
# Build the Dylint cdylib, then run `cargo dylint` on the root workspace.
# Expects `EFFECT_DYLINT_MOON_PATH` (devenv enterShell) so `crates/effect-rs-dylint-rules` uses the Fenix nightly toolchain.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export PATH="${EFFECT_DYLINT_MOON_PATH:-$PATH}"

DYLINT_RULES_DIR="$ROOT/crates/effect-rs-dylint-rules"
(
    cd "$DYLINT_RULES_DIR"
    cargo build --release
)

export DYLINT_LIBRARY_PATH="$DYLINT_RULES_DIR/target/release"
cd "$ROOT"

cmd="${1:?usage: $0 workspace|package <crate-name>}"
case "$cmd" in
    workspace)
        cargo dylint --all --workspace
        ;;
    package)
        pkg="${2:?package name required}"
        cargo dylint --all --package "$pkg" --workspace
        ;;
    *)
        echo "usage: $0 workspace|package <crate-name>" >&2
        exit 1
        ;;
esac
