#!/usr/bin/bash
set -ex
set -o pipefail

export WORKSPACE="${WORKSPACE:-$(pwd)}"

export RUST_LOG=off
export OUTPUT_DIR=$WORKSPACE
export KANI_DIR=$WORKSPACE/kani/target/kani
export VERIFY_RUST_STD_LIBRARY=$WORKSPACE/verify-rust-std/library

ls -alh $VERIFY_RUST_STD_LIBRARY

pushd tests/dummy-crate
cargo r --example verify_rust_std
