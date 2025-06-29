use std::fs::{copy, remove_file};

mod utils;
use utils::{assert_eq, *};

fn get(text: &str, start: &str) -> SerFunction {
    let json = &text[text.find("[\n").unwrap()..];
    let v: Vec<SerFunction> = serde_json::from_str(json).unwrap();
    v.into_iter().find(|f| f.name.starts_with(start)).unwrap()
}

const COMPARE: &str = "tests/compare";

fn compare(
    tmp: &str,
    v_file: &[&str],
    f: &str,
    assert: impl Fn(&SerFunction, &SerFunction, &str, &str, &str),
) {
    let len = v_file.len();
    assert!(len > 1);
    let tmp = format!("{COMPARE}/{tmp}.rs");

    let mut v_func = vec![];
    for ele in v_file {
        copy(format!("{COMPARE}/{ele}.rs"), &tmp).unwrap();
        let text = cmd(&[&tmp]);
        expect_file![format!("./snapshots/compare/{ele}.json")].assert_eq(&text);
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
}

#[test]
fn test_compare() {
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
