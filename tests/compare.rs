use std::fs::{copy, create_dir_all, remove_file};
mod utils;
use utils::{assert_eq, *};

fn get(text: &str, start: &str) -> SerFunction {
    let json = &text[text.find("[\n").unwrap()..];
    let v: Vec<SerFunction> = serde_json::from_str(json).unwrap();
    v.into_iter().find(|f| f.name.starts_with(start)).unwrap()
}

const COMPARE: &str = "tests/compare";
const SNAP_COMPARE: &str = "tests/snapshots/compare";
const EXPECT_COMPARE: &str = "./snapshots/compare";

fn compare(
    shared_file: &str,
    v_file: &[&str],
    f: &str,
    assert: impl Fn(&SerFunction, &SerFunction, &str, &str, &str),
) {
    struct ProofPath {
        src_file: String,
        hash_json: String,
        hash_json_expected: String,
    }

    let len = v_file.len();
    assert!(len > 1);
    let v_path: Vec<_> = v_file
        .iter()
        .map(|file| ProofPath {
            src_file: format!("{COMPARE}/{file}.rs"),
            hash_json: format!("{SNAP_COMPARE}/{file}.json"),
            hash_json_expected: format!("{EXPECT_COMPARE}/{file}.json"),
        })
        .collect();
    let tmp = format!("{COMPARE}/{shared_file}.rs");

    let mut v_func = vec![];
    for path in &v_path {
        copy(&path.src_file, &tmp).unwrap();
        let text = run_dv(&[&tmp]);
        expect_file![&path.hash_json_expected].assert_eq(&text);
        v_func.push(get(&text, f));
    }

    remove_file(tmp).unwrap();

    // For the same proof (w.r.t same path and body),
    // the hash value must be the same.
    for i in 0..len - 1 {
        for j in 1..len {
            assert(&v_func[i], &v_func[j], f, v_file[i], v_file[j]);
        }
    }

    if let [old, new] = v_path.as_slice() {
        let diff = run_vrs_diff(&old.hash_json, &new.hash_json);
        let diff_json = format!("{EXPECT_COMPARE}/diff/{shared_file}.json");
        expect_file![diff_json].assert_eq(&diff);
    }
}

#[test]
fn test_compare() {
    _ = create_dir_all(SNAP_COMPARE);
    _ = create_dir_all(format!("{SNAP_COMPARE}/diff"));

    fn eq(fn1: &SerFunction, fn2: &SerFunction, f: &str, f1: &str, f2: &str) {
        assert_eq!(
            fn1.hash, fn2.hash,
            "Adding irrelevant code shouldn't change the hash of {f:?}:\n{f1}: {fn1:#?}\n ≠ \n{f2}: {fn2:#?}"
        );
    }
    compare("proof", &["proof1", "proof2"], "verify::f", eq);
    compare("contract", &["contract1", "contract2"], "verify::f", eq);

    fn not_eq(fn1: &SerFunction, fn2: &SerFunction, f: &str, f1: &str, f2: &str) {
        assert_ne!(
            fn1.hash, fn2.hash,
            "Hash values of {f:?} should not equal:\n{f1}: {fn1:#?}\n vs \n{f2}: {fn2:#?}"
        );
    }
    compare(
        "gen_proofs_by_nested_macros",
        &["gen_proofs_by_nested_macros1", "gen_proofs_by_nested_macros2"],
        "verify::proof",
        not_eq,
    );
}
