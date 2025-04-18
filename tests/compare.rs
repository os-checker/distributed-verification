use std::fs::{copy, remove_file};

mod utils;
use utils::{assert_eq, *};

fn get(text: &str, start: &str) -> SerFunction {
    let json = &text[text.find("[\n").unwrap()..];
    let v: Vec<SerFunction> = serde_json::from_str(json).unwrap();
    v.into_iter().find(|f| f.func.src.starts_with(start)).unwrap()
}

const COMPARE: &str = "tests/compare";

fn compare(tmp: &str, v_file: &[&str], f: &str, assert: impl Fn(&SerFunction, &SerFunction, &str)) {
    let len = v_file.len();
    assert!(len > 1);
    let tmp = format!("{COMPARE}/{tmp}.rs");

    let mut v_func = vec![];
    for ele in v_file {
        copy(format!("{COMPARE}/{ele}.rs"), &tmp).unwrap();
        let text = cmd(&[&tmp]);
        expect_file![format!("./snapshots/{ele}.json")].assert_eq(&text);
        v_func.push(get(&text, f));
    }

    remove_file(tmp).unwrap();

    // For the same proof (w.r.t same path and body),
    // the hash value must be the same.
    for i in 0..len - 1 {
        for j in 1..len {
            assert(&v_func[i], &v_func[j], f);
        }
    }
}

#[test]
fn test_compare() {
    fn eq(fn1: &SerFunction, fn2: &SerFunction, f: &str) {
        assert_eq!(
            fn1.hash,
            fn2.hash,
            "Hash values of {f:?} are not equal: {f1:#?} ≠ {f2:#?}",
            f1 = simplify_ser_function(fn1),
            f2 = simplify_ser_function(fn2),
        );
    }
    compare("proof", &["proof1", "proof2"], "pub fn f()", eq);
    compare("contract", &["contract1", "contract2"], "pub fn f()", eq);

    fn not_eq(fn1: &SerFunction, fn2: &SerFunction, f: &str) {
        assert_ne!(
            fn1.hash,
            fn2.hash,
            "Hash values of {f:?} should not equal: {f1:#?} vs {f2:#?}",
            f1 = simplify_ser_function(fn1),
            f2 = simplify_ser_function(fn2),
        );
    }
    compare(
        "gen_proofs_by_nested_macros",
        &["gen_proofs_by_nested_macros1", "gen_proofs_by_nested_macros2"],
        "fn",
        not_eq,
    );
}

fn simplify_ser_function(fn1: &SerFunction) -> SerFunction {
    SerFunction {
        hash: fn1.hash.clone(),
        def_id: fn1.def_id.clone(),
        attrs: fn1.attrs.clone(),
        func: fn1.func.clone(),
        callees_len: fn1.callees_len,
        ..Default::default()
    }
}
