//! `VERIFY_RUST_STD_LIBRARY=path/to/verify-rust-std/library` and
//! `KANI_DIR=path/to/kani` should be set beforehand.

use eyre::{Context, Result};
use std::process::{Command, Stdio};

#[macro_use]
extern crate tracing;
#[macro_use]
extern crate eyre;

mod env;
use env::ENV;

use crate::env::set_wrapper;

fn main() -> Result<()> {
    let mut args = std::env::args().collect::<Vec<_>>();
    rustc_flags();

    if args.len() == 2 && args[1].as_str() == "-vV" {
        // cargo invokes `rustc -vV` first
        run("rustc", &["-vV".to_owned()], &[])
    } else if env::is_wrapper() {
        // then cargo invokes `rustc - --crate-name ___ --print=file-names`
        if args[1] == "-" {
            // `rustc -` is a substitute file name from stdin
            // see https://rust-lang.zulipchat.com/#narrow/channel/182449-t-compiler.2Fhelp/topic/.E2.9C.94.20What.20does.20.60rustc.20-.60do.3F/with/514494493
            args[1] = "src/lib.rs".to_owned();
        }

        let rustc_args = &args[1..];
        if args.iter().any(|arg| arg == "core") {
            let json = serde_json::json!({
                "rustflags": &rustc_args,
                "rustc": format!("rustc {}", rustc_args.join(" "))
            });
            ENV.write_rustflags_json(&json)?;
            build_core(args.split_off(1))
        } else {
            // build non-core crates
            run("rustc", rustc_args, &[])
        }
    } else {
        run(
            "cargo",
            &["build", "-Zbuild-std=core"].map(String::from),
            &[env::set_rustc_wrapper(), set_wrapper()],
        )
    }
}

fn run(cmd: &str, args: &[String], vars: &[(&str, &str)]) -> Result<()> {
    let library = &*ENV.VERIFY_RUST_STD_LIBRARY;
    // CARGO_ENCODED_RUSTFLAGS takes a string that separte arguments by 0x1f
    let rustc_flags = rustc_flags();

    let _span = debug_span!("run", cmd, ?library, ?args, ?vars, ?rustc_flags).entered();

    let rustflags = rustc_flags.join("\u{1f}");
    let status = Command::new(cmd)
        .args(args)
        .env("__CARGO_TESTS_ONLY_SRC_ROOT", library)
        .env("CARGO_ENCODED_RUSTFLAGS", rustflags)
        .envs(vars.iter().copied())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| "Failed to spawn a cmd process")?
        .wait()
        .with_context(|| "Failed to wait a cmd process")?;
    ensure!(status.success(), "Process aborts.");
    Ok(())
}

const KANI_ARGS: &[&str] = &[
    "-C",
    "overflow-checks=on",
    "-Z",
    "unstable-options",
    "-Z",
    "trim-diagnostic-paths=no",
    "-Z",
    "human_readable_cgu_names",
    "-Z",
    "always-encode-mir",
    "--cfg=kani",
    "-Z",
    "crate-attr=feature(register_tool)",
    "-Z",
    "crate-attr=register_tool(kanitool)",
    // "-L",
    // "/home/zjp/rust/distributed-verification/kani/target/kani/no_core/lib",
    // "--extern",
    // "kani_core",
    "-C",
    "panic=abort",
    "-C",
    "symbol-mangling-version=v0",
    "-Z",
    "panic_abort_tests=yes",
    "-Z",
    "mir-enable-passes=-RemoveStorageMarkers",
    "--check-cfg=cfg(kani)",
];

fn rustc_flags() -> Vec<String> {
    // inject kani_core dependency to recognize kani module in core
    let kani_dir = &*ENV.KANI_DIR;
    // -Lpath must be an absolute path
    let kani_lib = kani_dir.join("no_core").join("lib");
    let kani_lib = kani_lib.canonicalize().unwrap_or_else(|err| panic!("{kani_lib:?}: {err}"));
    assert!(std::fs::exists(&kani_lib).unwrap());
    let kani_core = ["-L", kani_lib.to_str().unwrap(), "--extern=kani_core"];

    KANI_ARGS.iter().copied().chain(kani_core).map(|arg| arg.to_owned()).collect()
}

#[test]
fn test_rustc_flags() {
    dbg!(rustc_flags());
}

fn build_core(args: Vec<String>) -> Result<()> {
    let mut new_args = Vec::with_capacity(args.len() + 2);
    let core_json = ENV.core_json();
    new_args.extend(
        [
            "--no-kani-args",
            "--simplify-json",
            "--continue-compilation",
            "--json",
            core_json.to_str().unwrap(),
            "--",
        ]
        .map(String::from),
    );
    new_args.extend(args);
    run(&ENV.DISTRIBUTED_VERIFICATION, &new_args, &[])
}
