#!/usr/bin/env bash

# set -x
set -e

# shellcheck disable=SC1091
. xx-cargo --setup-target-triple
RUST_TARGET_TRIPLE="$(xx-cargo --print-target-triple)"

if ! xx-info is-cross; then
    # remove these clang declarations to avoid linking errors
    unset "CARGO_TARGET_$(echo "$RUST_TARGET_TRIPLE" | tr '[:lower:]' '[:upper:]' | tr - _)_LINKER";
    unset "CC_$(echo "$RUST_TARGET_TRIPLE" | tr - _)"
fi

cargo build --release --bin aero2solver --target="$RUST_TARGET_TRIPLE"
xx-verify "./target/$RUST_TARGET_TRIPLE/release/aero2solver"
cp "./target/$RUST_TARGET_TRIPLE/release/aero2solver" "./target/aero2solver"