#!/bin/bash
# Copyright (c) Aptos
# SPDX-License-Identifier: Apache-2.0

###########################################
# Build and package a release for the CLI #
###########################################

# Note: This must be run from the root of the aptos-core repository

set -e

NAME='aptos-cli'
CRATE_NAME='aptos'
CARGO_PATH="crates/$CRATE_NAME/Cargo.toml"
TARGET=$1
VERSION=$(grep "^\w*version =" < "$CARGO_PATH" | sed 's/^.*=[ ]*"//g' | sed 's/".*$//g')

# TODO: Make the target names more friendly
# Build with the CLI size optimization over speed
echo "Building release $VERSION of $NAME for $TARGET"
cargo build -p "$CRATE_NAME" --profile cli --target "$TARGET"

cd "target/$TARGET/cli/"

# Compress the CLI
ZIP_NAME="$NAME-$VERSION-$TARGET.zip"

echo "Zipping release: $ZIP_NAME"
zip "$ZIP_NAME" $CRATE_NAME
mv "$ZIP_NAME" ../../..

# TODO: Add installation instructions?
