#!/usr/bin/bash
set -exo pipefail

# NOTE: verify-rust-std pins its kani commit, and
# distributed-verification pins another.
# So this means for verify-rust-std jobs,
# distributed-verification may be broken to compile.
git submodule update --init --recursive kani

cd kani

# Temporary patch to https://github.com/model-checking/kani/pull/4312
git apply ../.github/patch/kani_contract_mode.patch

./scripts/setup/ubuntu/install_deps.sh
cargo build-dev -- --release

export PATH=$(pwd)/scripts:$PATH
echo PATH="$PATH" >>"$GITHUB_ENV"
kani --version
