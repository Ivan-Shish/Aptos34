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

# Check if we're in the right directory
if [ ! -f "$CARGO_PATH" ]; then
  echo "Please ensure you run this script from the aptos-core main directory"
  exit -3
fi

VERSION=`cat "$CARGO_PATH" | grep "^\w*version =" | sed 's/^.*=[ ]*"//g' | sed 's/".*$//g'`
TARGET="$1"

# Let's make friendlier names for all the builds
if [ "$TARGET" == "i686-apple-darwin" ]; then
  PLATFORM="macOS-x86_32"
elif [ "$TARGET" == "x86_64-apple-darwin" ]; then
  PLATFORM="macOS-x86_64"
elif [ "$TARGET" == "aarch64-apple-darwin" ]; then
  PLATFORM="macOS-arm64"
elif [ "$TARGET" == "i686-unknown-linux-gnu" ]; then
  PLATFORM="linux-x86_32-gnu"
elif [ "$TARGET" == "x86_64-unknown-linux-gnu" ]; then
  PLATFORM="linux-x86_64-gnu"
elif [ "$TARGET" == "aarch64-unknown-linux-gnu" ]; then
  PLATFORM="linux-arm64-gnu"
elif [ "$TARGET" == "i686-unknown-linux-musl" ]; then
  PLATFORM="linux-x86_32-musl"
elif [ "$TARGET" == "x86_64-unknown-linux-musl" ]; then
  PLATFORM="linux-x86_64-musl"
elif [ "$TARGET" == "aarch64-unknown-linux-musl" ]; then
  PLATFORM="linux-arm64-musl"
elif [ "$TARGET" == "i686-pc-windows-msvc" ]; then
  PLATFORM="windows-x86_32-gnu"
elif [ "$TARGET" == "x86_64-pc-windows-msvc" ]; then
  PLATFORM="windows-x86_64-gnu"
elif [ -z "$TARGET" ]; then
  echo "Must provide a build target architecture\n"
  echo "Please checkout https://doc.rust-lang.org/nightly/rustc/platform-support.html for possible targets"
  exit -2
else
  echo "$TARGET currently not supported by this script"
  echo "Please checkout https://doc.rust-lang.org/nightly/rustc/platform-support.html for possible targets"
  exit -1
fi

# Ensure we have the libraries to build it
rustup target add $TARGET

# Build the actual crate
echo "Building release $VERSION of $NAME for $TARGET"
cargo build -p $CRATE_NAME --profile cli --target $TARGET

# Compress CLI
cd target/$TARGET/cli/

ZIP_NAME="$NAME-$VERSION-$TARGET.zip"

echo "Zipping release: $ZIP_NAME"
zip $ZIP_NAME $CRATE_NAME
mv $ZIP_NAME ../..

