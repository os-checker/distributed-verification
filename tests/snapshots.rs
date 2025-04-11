use assert_cmd::Command;
use expect_test::expect_file;

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
