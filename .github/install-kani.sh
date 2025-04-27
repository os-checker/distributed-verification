set -x

COMMIT=5cf80a84afdeceec0d85e790a5cc585ff2e513ed

# Install kani
pushd kani
./scripts/setup/ubuntu/install_deps.sh
cargo build-dev

export PATH=$(pwd)/scripts:$PATH
echo PATH=$PATH >>$GITHUB_ENV
kani --version

popd

pushd tests/dummy-crate
git clone https://github.com/model-checking/verify-rust-std.git --recursive
cd verify-rust-std && git checkout $COMMIT
