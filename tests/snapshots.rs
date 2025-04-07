use assert_cmd::Command;

fn cmd() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
}

#[test]
fn standard_proof() {
    let output = cmd().args(["tests/standard_proof.rs"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{stdout}");
}
