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

mod diff;
mod kani;
mod merge;

fn main() -> Result<()> {
    // arguments passed to rustc
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();

    if args.as_slice() == ["-vv"] {
        // cargo invokes `rustc -vV` first
        run("rustc", &["-vV".to_owned()], &[])
    } else if env::is_wrapper() {
        // then cargo invokes `rustc - --crate-name ___ --print=file-names`
        if args[0] == "-" {
            // `rustc -` is a substitute file name from stdin
            // see https://rust-lang.zulipchat.com/#narrow/channel/182449-t-compiler.2Fhelp/topic/.E2.9C.94.20What.20does.20.60rustc.20-.60do.3F/with/514494493
            args[0] = "src/lib.rs".to_owned();
        }

        if args.iter().any(|arg| is_normal_built(arg)) {
            // build non-core crates
            run("rustc", &args, &[])
        } else {
            let json = serde_json::json!({
                "rustflags": &args,
                "rustc": format!("rustc {}", args.join(" "))
            });
            ENV.write_rustflags_json(&json)?;
            build_core(args)
        }
    } else if let Some(subcmd) = args.first() {
        match subcmd.as_str() {
            "merge" => merge::run(&args),
            "diff" => diff::run(&args),
            "kani-list" => kani::list(&args[1..]),
            "kani-run" => kani::run(&args[1..]),
            "kani-run-no-auto" => kani::run_no_auto(&args[1..]),
            _ => run_cargo(),
        }
    } else {
        run_cargo()
    }
}

fn run_cargo() -> std::result::Result<(), eyre::Error> {
    run(
        "cargo",
        &["build", "-Zbuild-std=core,alloc"].map(String::from),
        &[env::set_rustc_wrapper(), env::set_wrapper()],
    )
}

fn run(cmd: &str, args: &[String], vars: &[(&str, &str)]) -> Result<()> {
    let library = &*ENV.VERIFY_RUST_STD_LIBRARY;
    let rustflags = &*ENV.CARGO_ENCODED_RUSTFLAGS;

    let _span = debug_span!("run", cmd, ?library, ?args, ?vars, rustflags).entered();

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

fn build_core(args: Vec<String>) -> Result<()> {
    let dir = &*ENV.OUTPUT_DIR;
    // `$OUTPUT_DIR/stat` as stat JSON output folder
    let dir_stat = dir.join("stat");
    // Create these folders if not exist.
    _ = std::fs::create_dir_all(&dir_stat);

    let mut new_args = Vec::with_capacity(args.len() + 2);
    new_args.extend(
        [
            "--no-kani-args",
            "--continue-compilation",
            "--json",
            dir.to_str().unwrap(),
            "--stat",
            dir_stat.to_str().unwrap(),
            "--",
        ]
        .map(String::from),
    );
    new_args.extend(args);

    run(&ENV.DISTRIBUTED_VERIFICATION, &new_args, &[])
}

/// Normally build crates such as proc-macros, build scripts, and some common used crates we don't
/// care from verify-rust-std. This can be possible false positive, but it works currently.
fn is_normal_built(arg: &str) -> bool {
    matches!(
        arg,
        "proc-macro"
            | "build_script_build"
            | "syn"
            | "quote"
            | "proc_macro2"
            | "unicode_ident"
            | "version_check"
            | "proc_macro_error"
            | "proc_macro_error_attr"
            | "compiler_builtins"
    )
}

fn read_json<T: serde::de::DeserializeOwned>(path: &str) -> Result<T> {
    let _span = error_span!("read_json", path).entered();
    let file = std::fs::File::open(path)?;
    Ok(serde_json::from_reader(file)?)
}
