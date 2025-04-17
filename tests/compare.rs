use std::fs::{copy, remove_file};

mod utils;
use utils::{assert_eq, *};

fn get(text: &str, start: &str) -> SerFunction {
    let json = &text[text.find("[\n").unwrap()..];
    let v: Vec<SerFunction> = serde_json::from_str(json).unwrap();
    v.into_iter().find(|f| f.func.src.starts_with(start)).unwrap()
}

const COMPARE: &str = "tests/compare";

fn compare(tmp: &str, v_file: &[&str], f: &str) {
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
            assert_eq!(
                v_func[i].hash, v_func[j].hash,
                "Hash values of {f:?} are not equal: {} ≠ {}",
                v_file[i], v_file[j]
            );
        }
    }
}

#[test]
fn test_compare() {
    compare("proof", &["proof1", "proof2"], "pub fn f()");
    compare("contract", &["contract1", "contract2"], "pub fn f()");
}
