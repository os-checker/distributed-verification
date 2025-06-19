use distributed_verification::{
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
        check_proofs(&kani_list, &v_ser_function).unwrap();

        // test `distributed-verification --check-kani-list`
        _ = cmd(&[path, "--check-kani-list=kani-list.json"]);
    }

    Ok(())
}

#[test]
fn kani_list_json() -> Result<()> {
    let path = "assets/kani-list_verify-rust-std.json";
    let file = std::fs::File::open(path)?;
    let mut kani_list: KaniListJson = serde_json::from_reader(file)?;

    kani_list.strip_path_prefix("./verify-rust-std/library")?;
    expect_file!["snapshots/kani_list/kani_list_json-files.json"]
        .assert_eq(&serde_json::to_string_pretty(&kani_list.files())?);

    expect![[r#"
        Totals {
            standard_harnesses: 622,
            contract_harnesses: 953,
            functions_under_contract: 337,
        }
    "#]]
    .assert_debug_eq(&kani_list.totals);

    Ok(())
}
