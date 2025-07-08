use distributed_verification::{
    SerFunction,
    diff::KaniListJson,
    kani_list::{check_proofs, get_kani_list},
};

mod utils;
use tracing::error_span;
use utils::*;

#[test]
fn validate_kani_list_json() -> Result<()> {
    distributed_verification::logger::init();
    let proofs = get_proofs("tests/proofs")?;

    for path in &proofs {
        let file_stem = file_stem(path);
        let list_path = format!("snapshots/kani_list/{file_stem}.txt");

        let path = path.to_str().unwrap();
        let _span = error_span!("validate", list_path, path).entered();

        // run `kani list`
        let kani_list = get_kani_list(path)?;
        expect_file![list_path].assert_debug_eq(&kani_list);

        // run `distributed-verification`
        let text = cmd(&[path]);
        let v_ser_function: Vec<SerFunction> = serde_json::from_str(&text).unwrap();
        let v_proof: Vec<_> = v_ser_function.iter().filter(|f| f.is_proof()).collect();
        check_proofs(&kani_list, &v_proof).unwrap();

        // test `distributed-verification --check-kani-list`
        _ = cmd(&[path, "--check-kani-list=kani-list.json"]);
    }

    Ok(())
}

fn read_kani_list_json() -> KaniListJson {
    const KANI_LIST_JSON: &str = "assets/kani-list_verify-rust-std.json";
    const PREFIX: &str = "/home/gh-zjp-CN/distributed-verification/verify-rust-std/library/";

    let mut kani_list: KaniListJson = read_file(KANI_LIST_JSON).unwrap();
    kani_list.strip_path_prefix_raw(PREFIX);
    kani_list
}

#[test]
fn kani_list_json() {
    let kani_list = read_kani_list_json();

    expect_file!["snapshots/kani_list/kani_list_json-files.json"]
        .assert_eq(&serde_json::to_string_pretty(&kani_list.files()).unwrap());

    expect![[r#"
        Totals {
            standard_harnesses: 622,
            contract_harnesses: 953,
            functions_under_contract: 337,
        }
    "#]]
    .assert_debug_eq(&kani_list.totals);
}
