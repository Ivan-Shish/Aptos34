#!/bin/bash -eux
# Copyright © Aptos Foundation
# Parts of the project are originally copyright © Meta Platforms, Inc.
# SPDX-License-Identifier: Apache-2.0

# This script must run in the `aptos-core` repo root directory

set -e

export RUSTFLAGS="$RUSTFLAGS --cfg tokio_unstable"
export ROOTDIR="$(pwd)"
export OUT="${OUT:-${ROOTDIR}/out}"

cd testsuite/aptos-fuzzer/

# Function to build a fuzzer
build_fuzzer() {
    local $fuzzer_name=$1
    cd fuzz
    SINGLE_FUZZ_TARGET=$fuzzer_name cargo +nightly fuzz build -O -a
    cp -r $ROOTDIR/target/x86_64-unknown-linux-gnu/release/fuzz_builder $OUT/$fuzzer_name
    rm $ROOTDIR/target/x86_64-unknown-linux-gnu/release/fuzz_builder
    cd ..
}

if [ -z "$1" ]; then
    # If no argument is provided, build all fuzzers
    fuzzers=$(cargo run --bin aptos-fuzzer list --no-desc > fuzzer_list)
    for fuzzer_name in $fuzzers; do
    do
        build_fuzzer $fuzzer_name
    done
else
    # If an argument is provided, use it as fuzzer_name
    local $fuzzer_name=$1
    build_fuzzer $fuzzer_name
fi