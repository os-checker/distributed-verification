#!/usr/bin/bash
set -ex
set -o pipefail

export WORKSPACE="${WORKSPACE:-$(pwd)}"

export RUST_LOG=off
export OUTPUT_DIR=$WORKSPACE/assets
export KANI_DIR=$WORKSPACE/kani/target/kani
export VERIFY_RUST_STD_LIBRARY=$WORKSPACE/verify-rust-std/library

# test/verify-rust-std needs this, so remember to
# * update runid after verify-rust-std submodule syncs
# * update snapshots after runid changes
# rm tmp -rf
# gh run download -D tmp -R model-checking/verify-rust-std 16777123952

ls -alh $VERIFY_RUST_STD_LIBRARY

cargo b --bin distributed-verification
export DISTRIBUTED_VERIFICATION=$WORKSPACE/target/debug/distributed-verification

cargo build --bin verify_rust_std
export VERIFY_RUST_STD=$WORKSPACE/target/debug/verify_rust_std

pushd tests/dummy-crate
cargo clean
$VERIFY_RUST_STD

popd
pushd verify-rust-std
ls -a library/core/db.sqlite3
mv library/core/db.sqlite3 ../assets/core.sqlite3
git checkout .
