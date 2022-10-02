#!/bin/bash
# Copyright (c) Aptos
# SPDX-License-Identifier: Apache-2.0

CRATE_NAME="$1"
TARGET="$2"

if [ -z "$CRATE_NAME" ]; then
  printf "No crate name given\n"
  printf "scripts/cross-build-binary.sh [crate_name] [target]\n"
  exit 1
elif [ -z "$TARGET" ]; then
  printf "No target name given\n"
  printf "scripts/cross-build-binary.sh [crate_name] [target]\n"
  exit 1
fi

rustup target add $TARGET

# Build the actual crate
echo "Building release of $CRATE_NAME for $TARGET"
cargo build -p $CRATE_NAME --release --target $TARGET
