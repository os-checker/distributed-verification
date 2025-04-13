use assert_cmd::Command;
use distributed_verification::SerFunction;
use expect_test::expect_file;
use std::fs::{copy, remove_file};

fn cmd(args: &[&str]) -> String {
    let output = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap().args(args).output().unwrap();
    assert!(
        output.status.success(),
        "Failed to test standard_proof.rs:\n{}",
        std::str::from_utf8(&output.stderr).unwrap()
    );

    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn standard_proofs() {
    let json = cmd(&["tests/standard_proofs.rs"]);
    expect_file!["./snapshots/standard_proofs.json"].assert_eq(&json);
}

#[test]
fn standard_proofs_with_contracts() {
    let json = cmd(&["tests/standard_proofs_with_contracts.rs"]);
    expect_file!["./snapshots/standard_proofs_with_contracts.json"].assert_eq(&json);
}

fn get_hash(text: &str, start: &str) -> String {
    let v: Vec<SerFunction> = serde_json::from_str(text).unwrap();
    v.into_iter().find(|f| f.func.starts_with(start)).unwrap().hash
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
    assert_eq!(get_hash(text1, f), get_hash(text2, f));
}
