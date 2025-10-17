#!/usr/bin/bash
set -ex
set -o pipefail

export WORKSPACE="${WORKSPACE:-$(pwd)}"

export DV_LOG=off
export KANI_DIR=$WORKSPACE/kani/target/kani
export VERIFY_RUST_STD_LIBRARY=$WORKSPACE/verify-rust-std/library

export OUTPUT_DIR=$WORKSPACE/assets/json
mkdir "$OUTPUT_DIR" -p

# test/verify-rust-std needs this, so remember to
# * update runid after verify-rust-std submodule syncs
# * update snapshots after runid changes
rm tmp -rf
gh run download -D tmp -R model-checking/verify-rust-std 18371041107

# Replace kani-list.json:
# * kani-list_verify-rust-std.json can be obtained from local, but here we just override it with the last CI
# mv assets/kani-list_verify-rust-std-CI.json assets/kani-list_verify-rust-std.json
# * override with the latest CI kani-list
# cp tmp/ubuntu-latest-kani-list.json/kani-list.json assets/kani-list_verify-rust-std-CI.json

ls -alh "$VERIFY_RUST_STD_LIBRARY"

# Store data across all crates.
export DB_FILE=$WORKSPACE/tmp/core.sqlite3
# Remove old data: we don't need data history.
rm "$DB_FILE" -f

cargo b --bin distributed-verification
export DISTRIBUTED_VERIFICATION=$WORKSPACE/target/debug/distributed-verification

cargo build --bin verify_rust_std
export VERIFY_RUST_STD=$WORKSPACE/target/debug/verify_rust_std

pushd tests/dummy-crate
cargo clean
$VERIFY_RUST_STD

popd
pushd verify-rust-std
git checkout .

popd
mv "$DB_FILE" assets/core.sqlite3
ls -a assets/core.sqlite3
