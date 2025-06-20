set -ex

# NOTE: verify-rust-std pins its kani commit, and
# distributed-verification pins another.
# So this means for verify-rust-std jobs,
# distributed-verification may be broken to compile.
git submodule update --init kani

# Install kani
pushd kani
git submodule update --init charon

./scripts/setup/ubuntu/install_deps.sh
cargo build-dev -- --release

export PATH=$(pwd)/scripts:$PATH
echo PATH=$PATH >>$GITHUB_ENV
kani --version

popd
