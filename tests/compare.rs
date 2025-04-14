use distributed_verification::SerFunction;
use std::fs::{copy, remove_file};

mod utils;
use utils::{assert_eq, cmd, expect_file};

fn get(text: &str, start: &str) -> SerFunction {
    let json = &text[text.find("[\n").unwrap()..];
    let v: Vec<SerFunction> = serde_json::from_str(json).unwrap();
    v.into_iter().find(|f| f.func.starts_with(start)).unwrap()
}

#[test]
fn compare_proof() {
    let file = "tests/compare/proof.rs";

    copy("tests/compare/proof1.rs", file).unwrap();
    let text1 = &cmd(&["tests/compare/proof.rs"]);
    expect_file!["./snapshots/proof1.json"].assert_eq(text1);

    // Add another proof won't affact the previous one.
    copy("tests/compare/proof2.rs", file).unwrap();
    let text2 = &cmd(&["tests/compare/proof.rs"]);
    expect_file!["./snapshots/proof2.json"].assert_eq(text2);

    remove_file(file).unwrap();

    let f = "pub fn f()";
    // For the same proof (w.r.t same path and body),
    // the hash value must be the same.
    assert_eq!(get(text1, f).hash, get(text2, f).hash);
}

#[test]
fn compare_contract() {
    let file = "tests/compare/contract.rs";

    copy("tests/compare/contract1.rs", file).unwrap();
    let text1 = &cmd(&["tests/compare/contract.rs"]);
    expect_file!["./snapshots/contract1.json"].assert_eq(text1);

    copy("tests/compare/contract2.rs", file).unwrap();
    let text2 = &cmd(&["tests/compare/contract.rs"]);
    expect_file!["./snapshots/contract2.json"].assert_eq(text2);

    remove_file(file).unwrap();

    let f = "pub fn f()";
    let f1 = get(text1, f);
    let f2 = get(text2, f);
    let callees1 = f1.callee_sorted_by_file_func();
    let callees2 = f2.callee_sorted_by_file_func();

    assert_eq!(callees1, callees2);
    assert_eq!(f1.hash, f2.hash);
}
