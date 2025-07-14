use assert_cmd::cargo::CommandCargoExt;
use distributed_verification::diff::{KaniListJson, MergeHashKaniList, MergedHarnesses};
use std::process::{Command, Stdio};

mod utils;
use utils::*;

// path to distributed-verification
const PREFIX_LOCAL_DIR: &str = "/home/gh-zjp-CN/distributed-verification/";
const PREFIX_CI_DIR: &str = "/home/runner/work/distributed-verification/distributed-verification/";
const PREFIX_KANI_LIST_LIBRARY: &str = "/home/runner/work/verify-rust-std/verify-rust-std/library/";
const PREFIX_LOCAL_LIBRARY: &str =
    "/home/gh-zjp-CN/distributed-verification/verify-rust-std/library/";

const CLI: &str = "verify_rust_std";
const CORE_JSON: &str = "./assets/core.json";

const KANI_LIST_JSON: &str = "assets/kani-list_verify-rust-std-CI.json";

fn snapshot_file(file_name: &str) -> String {
    format!("snapshots/verify-rust-std/{file_name}")
}

/// Read kani-list.json generated from verify-rust-std CI.
fn read_kani_list_json() -> KaniListJson {
    let mut kani_list: KaniListJson = read_file(KANI_LIST_JSON).unwrap();
    kani_list.normalize_file_path();
    kani_list.strip_path_prefix_raw(PREFIX_KANI_LIST_LIBRARY);
    kani_list.strip_path_closure_name(&[PREFIX_KANI_LIST_LIBRARY, PREFIX_LOCAL_DIR, PREFIX_CI_DIR]);
    kani_list
}

fn read_core_json() -> Vec<SerFunction> {
    const PREFIX_CI: &str = "/home/runner/work/distributed-verification/distributed-verification/verify-rust-std/library/";

    let mut v: Vec<SerFunction> = read_file(CORE_JSON).unwrap();
    for func in &mut v {
        // strip_path_closure_name
        func.name = func
            .name
            .replace(PREFIX_LOCAL_LIBRARY, "")
            .replace(PREFIX_CI, "")
            .replace(PREFIX_LOCAL_DIR, "")
            .replace(PREFIX_CI_DIR, "")
            .into();
    }
    v
}

#[test]
fn core_json() {
    let v_func = read_core_json();
    let merged = MergedHarnesses::new(&v_func);

    #[derive(Debug)]
    #[allow(dead_code)]
    struct Count {
        standard: usize,
        contract: usize,
    }

    let count = Count { standard: merged.standard.len(), contract: merged.contract.len() };
    expect![[r#"
        Count {
            standard: 621,
            contract: 995,
        }
    "#]]
    .assert_debug_eq(&count);
}

#[test]
fn read() {
    let kani_list = read_kani_list_json();
    expect![[r#"
        Totals {
            standard_harnesses: 8385,
            contract_harnesses: 1232,
            functions_under_contract: 356,
        }
    "#]]
    .assert_debug_eq(&kani_list.totals);

    let harness_names = kani_list.harness_names(|file, _| file.starts_with("core/"));
    expect_file![snapshot_file("harness_names.txt")].assert_debug_eq(&harness_names);

    let v_func = read_core_json();
    let merged = MergedHarnesses::new(&v_func);

    let function_names = merged.function_names(|f| f.file.starts_with("core/"));
    expect_file![snapshot_file("function_names.txt")].assert_debug_eq(&function_names);

    let names_not_in_functions: Vec<_> = harness_names
        .iter()
        .filter_map(|&h| function_names.get(h).is_none().then_some(h))
        .collect();
    expect_file![snapshot_file("names_not_in_functions.txt")]
        .assert_debug_eq(&names_not_in_functions);
}

// verify_rust_std merge --hash-json assets/core.json --kani-list assets/kani-list_verify-rust-std-CI.json \
// --strip-kani-list-prefix /home/runner/work/verify-rust-std/verify-rust-std/library/ > merge.json
#[test]
fn merge() {
    let mut cmd = Command::new(CLI);
    let args = [
        "merge",
        "--hash-json",
        CORE_JSON,
        "--kani-list",
        KANI_LIST_JSON,
        "--strip-kani-list-prefix",
        PREFIX_KANI_LIST_LIBRARY,
    ];
    let output = cmd.args(args).stderr(Stdio::inherit()).output().unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    assert!(output.status.success(), "stdout={stdout}");
    expect_file![snapshot_file("merge.json")].assert_eq(stdout);

    let v_hash: Vec<MergeHashKaniList> = serde_json::from_str(stdout).unwrap();
    expect!["9616"].assert_eq(&v_hash.len().to_string());
    expect!["5937"].assert_eq(&v_hash.iter().filter(|h| h.hash.is_some()).count().to_string());
}
