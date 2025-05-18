set -x

COMMIT=8b586cd8983ad6ce7ad5611814209842ecf0bf47

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
