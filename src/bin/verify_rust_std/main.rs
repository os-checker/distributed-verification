//! `VERIFY_RUST_STD_LIBRARY=path/to/verify-rust-std/library` and
//! `KANI_DIR=path/to/kani` should be set beforehand.

use std::{
    env::var,
    path::{Path, PathBuf},
    process::{Command, Stdio, abort},
};

const RUSTC: &str = env!("CARGO_CRATE_NAME");
const JSON_FILE: &str = "rustflags.json";

mod env;

fn main() {
    let rustc_wrapper = &var("VERIFY_RUST_STD").unwrap();
    // dbg!(RUSTC, rustc_wrapper);

    let mut args = std::env::args().collect::<Vec<_>>();
    rustc_flags();

    if args.len() == 2 && args[1].as_str() == "-vV" {
        // cargo invokes `rustc -vV` first
        run("rustc", &["-vV".to_owned()], &[]);
    } else if std::env::var("WRAPPER").as_deref() == Ok("1") {
        // then cargo invokes `rustc - --crate-name ___ --print=file-names`
        if args[1] == "-" {
            // `rustc -` is a substitute file name from stdin
            // see https://rust-lang.zulipchat.com/#narrow/channel/182449-t-compiler.2Fhelp/topic/.E2.9C.94.20What.20does.20.60rustc.20-.60do.3F/with/514494493
            args[1] = "src/lib.rs".to_owned();
        }

        let rustc_args = &args[1..];
        if args.iter().any(|arg| arg == "core") {
            println!("[build core] rustc_args={rustc_args:?}");
            let writer = std::fs::File::create(JSON_FILE).unwrap();
            let json = serde_json::json!({
                "rustflags": &rustc_args,
                "rustc": format!("rustc {}", rustc_args.join(" "))
            });
            serde_json::to_writer_pretty(writer, &json).unwrap();
            let path = PathBuf::from(JSON_FILE).canonicalize().unwrap();
            println!("{path:?} is written.");
            build_core(args.split_off(1));
        } else {
            // build non-core crates
            run("rustc", rustc_args, &[]);
        }
    } else {
        run(
            "cargo",
            &["build", "-Zbuild-std=core"].map(String::from),
            &[("RUSTC", rustc_wrapper), ("WRAPPER", "1")],
        );
    }
}

fn run(cmd: &str, args: &[String], vars: &[(&str, &str)]) {
    let library = var("VERIFY_RUST_STD_LIBRARY").unwrap();
    // CARGO_ENCODED_RUSTFLAGS takes a string that separte arguments by 0x1f
    let rustc_flags = rustc_flags();
    let rustflags = rustc_flags.join("\u{1f}");
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
        eprintln!("[error] {cmd}: args={args:?} vars={vars:?} rustc_flags={rustc_flags:?}");
        abort();
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
    let kani_lib = PathBuf::from(kani_dir).join("no_core").join("lib");
    let kani_lib = kani_lib.canonicalize().unwrap_or_else(|err| panic!("{kani_lib:?}: {err}"));
    assert!(std::fs::exists(&kani_lib).unwrap());
    let kani_core = ["-L", kani_lib.to_str().unwrap(), "--extern=kani_core"];

    KANI_ARGS.iter().copied().chain(kani_core).map(|arg| arg.to_owned()).collect()
}

#[test]
fn test_rustc_flags() {
    dbg!(rustc_flags());
}

fn build_core(args: Vec<String>) {
    const OUTPUT_DIR: &str = "/home/zjp/rust/distributed-verification";
    let mut new_args = Vec::with_capacity(args.len() + 2);
    let output_dir = var("OUTPUT_DIR");
    let output_dir: &Path = output_dir.as_deref().unwrap_or(OUTPUT_DIR).as_ref();
    let core_json = output_dir.join("core.json");
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
    run("distributed-verification", &new_args, &[]);
}
