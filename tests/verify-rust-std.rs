use distributed_verification::diff::{KaniListJson, MergedHarnesses};

mod utils;
use utils::*;

// path to distributed-verification
const PREFIX_LOCAL_DIR: &str = "/home/gh-zjp-CN/distributed-verification/";
const PREFIX_CI_DIR: &str = "/home/runner/work/distributed-verification/distributed-verification/";

/// Read kani-list.json generated from verify-rust-std CI.
fn read_kani_list_json() -> KaniListJson {
    const KANI_LIST_JSON: &str = "tmp/ubuntu-latest-kani-list.json/kani-list.json";
    const PREFIX: &str = "/home/runner/work/verify-rust-std/verify-rust-std/library/";

    let mut kani_list: KaniListJson = read_file(KANI_LIST_JSON).unwrap();
    kani_list.normalize_file_path();
    kani_list.strip_path_prefix_raw(PREFIX);
    kani_list.strip_path_closure_name(&[PREFIX, PREFIX_LOCAL_DIR, PREFIX_CI_DIR]);
    kani_list
}

fn read_core_json() -> Vec<SerFunction> {
    const CORE_JSON: &str = "./assets/core.json";
    const PREFIX_LOCAL: &str = "/home/gh-zjp-CN/distributed-verification/verify-rust-std/library/";
    const PREFIX_CI: &str = "/home/runner/work/distributed-verification/distributed-verification/verify-rust-std/library/";

    let mut v: Vec<SerFunction> = read_file(CORE_JSON).unwrap();
    for func in &mut v {
        // strip_path_closure_name
        func.name = func
            .name
            .replace(PREFIX_LOCAL, "")
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
            standard_harnesses: 8350,
            contract_harnesses: 1154,
            functions_under_contract: 356,
        }
    "#]]
    .assert_debug_eq(&kani_list.totals);

    let harness_names = kani_list.harness_names(|file, _| file.starts_with("core/"));
    expect_file!["snapshots/verify-rust-std/harness_names.txt"].assert_debug_eq(&harness_names);

    let v_func = read_core_json();
    let merged = MergedHarnesses::new(&v_func);

    let function_names = merged.function_names(|f| f.file.starts_with("core/"));
    expect_file!["snapshots/verify-rust-std/function_names.txt"].assert_debug_eq(&function_names);

    let names_not_in_functions: Vec<_> = harness_names
        .iter()
        .filter_map(|&h| function_names.get(h).is_none().then_some(h))
        .collect();
    expect_file!["snapshots/verify-rust-std/names_not_in_functions.txt"]
        .assert_debug_eq(&names_not_in_functions);
}
