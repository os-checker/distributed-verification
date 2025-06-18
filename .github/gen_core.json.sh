#!/usr/bin/bash
set -ex
set -o pipefail

export WORKSPACE="${WORKSPACE:-$(pwd)}"

export RUST_LOG=off
export OUTPUT_DIR=$WORKSPACE
export KANI_DIR=$WORKSPACE/kani/target/kani
export VERIFY_RUST_STD_LIBRARY=$WORKSPACE/verify-rust-std/library

ls -alh $VERIFY_RUST_STD_LIBRARY

cargo build --bin verify_rust_std
export VERIFY_RUST_STD=$WORKSPACE/target/debug/verify_rust_std

cd tests/dummy-crate
cargo clean
$VERIFY_RUST_STD
