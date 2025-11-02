**Distributed and resource-efficient verification for verify-rust-std**

Context: [Distributed and resource-efficient verification][distributed], GSoC Rust 2025

[distributed]: https://github.com/rust-lang/google-summer-of-code/tree/45141d74c28d91e114cf621d2d56aea6c3f82547?tab=readme-ov-file#distributed-and-resource-efficient-verification

## Initialization

### Submodules

There are three submodules, while only verify-rust-std and kani are needed.

`verify-rust-std` requires a specific Rust toolchain and verification tool
versions. Therefore, the commit SHA of kani is pinned, and `rust-toolchain.toml`
is a symlink to that in kani submodule.

```bash
# Only recursively initialize verify-rust-std and kani submodules.
git submodule update --init --recursive verify-rust-std kani
```

`ui/verify-rust-std_data` takes longer time to download, and stores backing data
for [WebUI](https://os-checker.github.io/distributed-verification), which is 
unnecessary to initialize in most cases.

### Install kani

```bash
cd kani

# Temporary patch to https://github.com/model-checking/kani/pull/4312
git apply ../.github/patch/kani_contract_mode.patch

./scripts/setup/ubuntu/install_deps.sh
cargo build-dev -- --release

export PATH=$(pwd)/scripts:$PATH
```

There is a bug in kani as referenced above, so we have to patch to make dv work.

### Install dv and vrs

```bash
cargo install --path . --locked
```

The project has two CLIs, `distributed-verification` and `verify_rust_std`.

`distributed-verification` (dv for short) is a rustc wrapper, mainly collecting
kani proofs and info based on the function call graph. 

`verify_rust_std` (vrs for short) acts as both a cargo subcommand
`RUSTC=distributed-verification cargo build` and an independent binary to
execute the verification of kani proofs.


