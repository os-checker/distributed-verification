//! `VERIFY_RUST_STD_LIBRARY=path/to/verify-rust-std/library` and
//! `KANI_DIR=path/to/kani` should be set beforehand.
use std::{
    env::var,
    path::PathBuf,
    process::{Command, Stdio},
};

fn main() {
    let mut args = std::env::args().collect::<Vec<_>>();
    rustc_flags();
    // dbg!(&args);

    if args.len() == 2 && args[1].as_str() == "-vV" {
        // cargo invokes `rustc -vV` first
        run("rustc", &["-vV".to_owned()], &[]);
    } else if std::env::var("WRAPPER").as_deref() == Ok("1") {
        // then cargo invokes `rustc - --crate-name ___ --print=file-names`
        let mut rustc_args = args.split_off(1);
        drop(args);
        if rustc_args[0] == "-" {
            // `rustc -` is a substitute file name from stdin
            // see https://rust-lang.zulipchat.com/#narrow/channel/182449-t-compiler.2Fhelp/topic/.E2.9C.94.20What.20does.20.60rustc.20-.60do.3F/with/514494493
            rustc_args[0] = "src/lib.rs".to_owned();
        }
        if rustc_args.iter().any(|arg| arg == "core") {
            println!("[build core] rustc_args={rustc_args:?}");
        }
        run("rustc", &rustc_args, &[]);
    } else {
        run(
            "cargo",
            &["build", "-Zbuild-std=core"].map(String::from),
            &[("RUSTC", "tmp"), ("WRAPPER", "1")],
        );
    }
}

fn run(cmd: &str, args: &[String], vars: &[(&str, &str)]) {
    let library = var("VERIFY_RUST_STD_LIBRARY").unwrap();
    // CARGO_ENCODED_RUSTFLAGS takes a string that separte arguments by 0x1f
    let rustflags = rustc_flags().join("\u{1f}");
    let status = Command::new(cmd)
        .args(args)
        .env("__CARGO_TESTS_ONLY_SRC_ROOT", library)
        .env("CARGO_ENCODED_RUSTFLAGS", rustflags)
        .envs(vars.iter().copied())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    if !status.success() {
        eprintln!("[error] {cmd}: args={args:?} vars={vars:?}");
    }
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
    let kani_dir = var("KANI_DIR").unwrap();
    // -Lpath must be an absolute path
    let kani_lib = PathBuf::from(kani_dir).join("no_core").join("lib").canonicalize().unwrap();
    assert!(std::fs::exists(&kani_lib).unwrap());
    let kani_core = ["-L", kani_lib.to_str().unwrap(), "--extern=kani_core"];

    KANI_ARGS.iter().copied().chain(kani_core).map(|arg| arg.to_owned()).collect()
}

#[test]
fn test_rustc_flags() {
    dbg!(rustc_flags());
}
