use assert_cmd::Command;
use expect_test::expect_file;

fn cmd() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
}

#[test]
fn standard_proof() {
    let output = cmd().args(["tests/standard_proof.rs"]).output().unwrap();
    assert!(
        output.status.success(),
        "Failed to test standard_proof.rs:\n{}",
        std::str::from_utf8(&output.stderr).unwrap()
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    expect_file!["./snapshots/standard_proof.json"].assert_eq(&stdout);
}
