set -ex

# NOTE: verify-rust-std pins its kani commit, and
# distributed-verification pins another.
# So this means for verify-rust-std jobs,
# distributed-verification may be broken to compile.
COMMIT_VERIFY_RUST_STD=0840b22
# COMMIT_KANI=d6853436382d876e574fb8a3fdef5b798a6e7d0d

# Install kani
pushd kani
# git pull origin main --rebase
# git log -1
# git checkout $COMMIT_KANI
./scripts/setup/ubuntu/install_deps.sh
cargo build-dev

export PATH=$(pwd)/scripts:$PATH
echo PATH=$PATH >>$GITHUB_ENV
kani --version

popd

pushd tests/dummy-crate
git clone https://github.com/model-checking/verify-rust-std.git --recursive
cd verify-rust-std && git checkout $COMMIT_VERIFY_RUST_STD
