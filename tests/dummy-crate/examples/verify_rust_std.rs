//! `VERIFY_RUST_STD_LIBRARY=path/to/verify-rust-std/library` and
//! `KANI_DIR=path/to/kani` should be set beforehand.
#![feature(rustc_private)]

extern crate indexmap;
extern crate rustc_driver;
extern crate rustc_interface;
extern crate rustc_middle;

#[macro_use]
extern crate rustc_smir;
extern crate stable_mir;

use indexmap::IndexMap;
use stable_mir::CrateDef;

use std::{
    env::var,
    path::PathBuf,
    process::{Command, Stdio},
};

const RUSTC: &str = env!("CARGO_CRATE_NAME");
const JSON_FILE: &str = "rustflags.json";

fn main() {
    let rustc_wrapper = &*format!("target/debug/examples/{RUSTC}");
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
    const CORE_JSON: &str = "/home/zjp/rust/distributed-verification/core.json";
    let mut new_args = Vec::with_capacity(args.len() + 2);
    let core_json = var("CORE_JSON");
    new_args.extend(
        [
            "--no-kani-args",
            "--simplify-json",
            "--json",
            core_json.as_deref().unwrap_or(CORE_JSON),
            "--",
        ]
        .map(String::from),
    );
    new_args.extend(args);
    run("distributed-verification", &new_args, &[]);
}

#[allow(dead_code)]
#[allow(unused_must_use)]
fn build_core_old(args: Vec<String>) {
    rustc_smir::run_with_tcx!(args, |_tcx| {
        let external_crates = stable_mir::external_crates();
        dbg!(external_crates.len(), external_crates);

        let mut kanitools = IndexMap::<String, Vec<String>>::new();

        for krate in stable_mir::find_crates("core") {
            let fn_defs = krate.fn_defs();
            dbg!(fn_defs.len());
            for fn_def in fn_defs {
                let name = fn_def.name();
                let attrs = fn_def.all_tool_attrs();
                if attrs.is_empty() {
                    continue;
                }
                // Need robust tokens to recognize attributes
                // cc https://github.com/rust-lang/project-stable-mir/issues/83
                let attrs = attrs
                    .iter()
                    .filter_map(|attr| {
                        let attr = attr.as_str();
                        attr.starts_with("#[kanitool::").then_some(attr)
                    })
                    .collect::<Vec<_>>();
                if attrs.is_empty() {
                    continue;
                }
                for attr in &attrs {
                    let attr = parse_kanitool(attr);
                    if let Some(v) = kanitools.get_mut(attr) {
                        v.push(name.clone());
                    } else {
                        kanitools.insert(attr.to_owned(), vec![name.clone()]);
                    }
                }
                // println!("{name}: {attrs:?}", attrs = attrs.join(" "));
            }
        }

        kanitools.sort_unstable_keys();
        kanitools.values_mut().for_each(|v| v.sort_unstable());
        let counts = kanitools.iter().map(|(k, v)| (k, v.len())).collect::<IndexMap<_, _>>();
        dbg!(&counts, &kanitools);
        ControlFlow::<(), ()>::Continue(())
    });
}

// From verify-rust-std CI:
// * Standard proofs: 371 (diff: -2)
// * Contract proofs: 955 ✅
//
// counts = {
//     "kanitool::asserted_with": 21,
//     "kanitool::checked_with": 21,
//     "kanitool::disable_checks": 19,
//     "kanitool::fn_marker": 91,
//     "kanitool::modifies_wrapper": 21,
//     "kanitool::proof": 369,
//     "kanitool::proof_for_contract": 955,
//     "kanitool::recursion_check": 21,
//     "kanitool::replaced_with": 21,
//     "kanitool::should_panic": 98,
//     "kanitool::solver": 9,
//     "kanitool::stub_verified": 2,
//     "kanitool::unstable(feature": 11,
//     "kanitool::unwind": 16,
// }

// `#[kanitool::proof]`
// `#[kanitool::proof_for_contract = ...]`
// `#[kanitool::recursion_check = ...]`
// `#[kanitool::disable_checks(pointer)]`
// `#[kanitool::unstable(feature = \"ghost-state\", issue = 3946, reason =...]`
fn parse_kanitool(attr: &str) -> &str {
    // start from `#[`
    let end = match attr[2..].find(' ') {
        Some(pos) => 2 + pos,
        None => match attr[2..].find('(') {
            Some(pos) => 2 + pos,
            None => attr.len() - 1, // ignore `]`
        },
    };
    &attr[2..end]
}
