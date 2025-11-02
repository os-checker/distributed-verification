**Distributed and resource-efficient verification for verify-rust-std**

Context: [Distributed and resource-efficient verification][distributed], GSoC Rust 2025

[distributed]: https://github.com/rust-lang/google-summer-of-code/tree/45141d74c28d91e114cf621d2d56aea6c3f82547?tab=readme-ov-file#distributed-and-resource-efficient-verification

## Initialization

### Submodules

There are three submodules, while only verify-rust-std and kani are needed.

`verify-rust-std` requires a specific Rust toolchain and verification tool versions.
Therefore, the commit SHA of kani is pinned, and `rust-toolchain.toml` is a symlink to
that in kani submodule.

```bash
# Only recursively initialize verify-rust-std and kani submodules.
git submodule update --init --recursive verify-rust-std kani
```

`ui/verify-rust-std_data` takes longer time to download, and stores backing data for
[WebUI](https://os-checker.github.io/distributed-verification), which is unnecessary to
initialize in most cases.

### Install kani

```bash
cd kani

# Temporary patch to https://github.com/model-checking/kani/pull/4312
git apply ../.github/patch/kani_contract_mode.patch

./scripts/setup/ubuntu/install_deps.sh
cargo build-dev -- --release

export PATH=$(pwd)/scripts:$PATH
```

There is a bug in kani as referenced above, so a patch is made to let dv work.

### Install dv and vrs

```bash
cargo install --path . --locked
```

The project has two CLIs, `distributed-verification` and `verify_rust_std`.

`distributed-verification` (dv for short) is a rustc wrapper for verify-rust-std, mainly
collecting kani proofs and info based on the function call graph. 

`verify_rust_std` (vrs for short) acts as both a cargo subcommand
`RUSTC=distributed-verification cargo build` and an independent binary to execute the
verification of kani proofs.

Details of the two tools are exhibited in the next section.

# Usage and Explanations

## distributed-verification (dv)

The dv basically invokes `rustc` with extra compiler flags to have kani work on the
standard library, such as
* registering tool attribute `kanitool` that `#[kani]` expands to,
* injecting dependency crates `kani` and `libstd.rlib` that are not passed through normal
  Cargo.toml,
* other arguments regarding kani sysroot, conditional compilation, and mir options.

These flags are essential to make the standard library compile for kani's needs.

<details>

<summary>
It's not recommended to use dv directly except for testing because of the tool design. But
here are commands to see its arguments as per the CLI help:
</summary>

```bash
$ export LD_LIBRARY_PATH=$(rustc --print sysroot)/lib
$ distributed-verification --help
Usage: distributed-verification [OPTIONS] [RUSTC_ARGS]...

Arguments:
  [RUSTC_ARGS]...
          Args for rustc. Usage: `distributed-verification -- [rustc_args]`
          
          No need to pass rustc as the first argument.

Options:
      --json [<JSON>]
          Possible one of these values:
          * `--json false`: skip serializing to json
          * `--json folder`: serlialize into a file under this folder with name being `{crate_name}.json`
          * `--json` or `--json stdout`: print to stdout
          
          NOTE: the default value is stdout, so if no other output is specified,
          the bahavior is `--json`. To skip all analyses, set `--json false`.
          
          [default: stdout]

      --no-kani-args
          Rustc args for kani. Default to true, especially auto emitting kani args for rustc on single rs file

      --check-kani-list <CHECK_KANI_LIST>
          Run `kani list` after the analysis, and compare proofs to ensure analyzed proofs are identical to ones from kani list

      --continue-compilation
          Continue compilation. Default to false, meaning compilation stops once proofs are analyzed

      --stat [<STAT>]
          Emit statistics for proofs as an alternative JSON. No normal JSON is emitted.
          
          The option value is just like `--json`. If both `--json` and `--stat` are set, prefer this.
          
          [default: false]
```


</details>

The outputs of dv include
* a sqlite3 file containing all the reached functions in compilation, which is stored in
  a path that the environment variable `DB_FILE` specifies;
* a bunch of JSON files containing the proof names and hashed from each compiled crate,
  which is stored in a folder that the argument `--json` specifies.

<details>

<summary>Sqlite3 scheme and proof JSONs</summary>

```sql
CREATE TABLE IF NOT EXISTS db (
  file TEXT NOT NULL,
  name TEXT NOT NULL,
  hash TEXT NOT NULL PRIMARY KEY,
  hash_direct TEXT NOT NULL,
  inst_kind TEXT,
  proof_kind TEXT,
  attrs TEXT,
  src TEXT,
  macro_backtrace_len INTEGER,
  macro_backtrace TEXT,
  callees_len INTEGER,
  callees TEXT,
  crate TEXT,
  path TEXT
) STRICT;
```

```json
{
  "hash": "326596390904428285310370185513349855953",
  "name": "alloc::layout::verify::check_align_to",
  "file": "core/src/alloc/layout.rs",
  "proof_kind": "Contract"
},
```

There are `core.sqlite3` and hash `json` outputs under `assets` folder.

</details>

The most significant information for a kani proof are a hash value that is computed
through reachability analysis on the called functions along the whole call graph. The same
analysis code is stolen from kani to keep traversal logics identical.

There will be two hash vaules for each function, becuase dv first computes the hash from 
direct calls individually and globally, then computes the desired hash by tracing down the
call graph and aggregating all the direct hashes.

It only takes 2 minutes to extract proofs and dump function info, while kani needs 20
minutes to generate `kani-list.json`.

## verify_rust_std (vrs)

The vrs can be used as follows
* `verify_rust_std` applies dv to the standard library as `cargo build` wrapper. The cargo
  build system helps pass correct compiler flags via cargo flags like `-Zbuild-std` and
  `__CARGO_TESTS_ONLY_SRC_ROOT` to compile the standard library through dv. Some edge
  cases are also handled, for instance, running `rust -vV` and skipping build scripts. The
  vrs command asks for some environment variables to be present to store outputs of dv.

```bash
export VERIFY_RUST_STD_LIBRARY=$PWD/verify-rust-std/library
export KANI_DIR=$PWD/kani/target/kani
export OUTPUT_DIR=$PWD/assets
verify_rust_std
```

* `verify_rust_std merge --hash-json ... --kani-list ... > merge.json` merges the hash
  value into `kani-list.json`, and stores the output in `merge.json`.
* `verify_rust_std diff --old merge-old.json --new merge-new.json > diff.json` removes
  proofs from new `merge.json` that have existed in old `merged.json`, and stores the
  output in `diff.json` which should be verified as wel as ideally be small in minor
  source code changes.
* `verify_rust_std kani-list` is a `kani autoharness --list` wrapper to generate
  `kani-list.json` regarding standard, contract, and automated harnesses. The dv doesn't 
  implement autoharness filtering logics, thus `kani` cmd is used.
* `verify_rust_std kani-run proof1 ...` is a `kani autoharness` wrapper to run
  verifications. Each proof name passed in must be the exact function name and passed to
  kani with extra `--include-pattern` and `--harness`.
* `verify_rust_std kani-run-no-auto proof1 ...` is a `kani verify-std` wrapper to only 
  verify standard and contract harnesses. The proof name must be exact with extra
  `--harness` passed.

These kani command wrappers pass extra kani arguments too, but are not so useful in
practice (see [#143] and [#4438]).

[#143]: https://github.com/os-checker/distributed-verification/issues/143
[#4438]: https://github.com/model-checking/kani/issues/4438

