use distributed_verification::kani_list::{check_proofs, get_kani_list};

mod utils;
use utils::*;

#[test]
fn validate_kani_list_json() -> Result<()> {
    let proofs = get_proofs("tests/proofs")?;

    for path in &proofs {
        let file_stem = file_stem(path);
        let list_path = format!("kani_list/{file_stem}.txt");
        dbg!(&list_path);

        let path = path.to_str().unwrap();
        // run `kani list`
        let kani_list = get_kani_list(path);
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
