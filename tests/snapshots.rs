use assert_cmd::Command;
use expect_test::expect;

fn cmd() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
}

#[test]
fn standard_proof() {
    let output = cmd().args(["tests/standard_proof.rs"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    expect![[r##"
        [
          {
            "file": "tests/standard_proof.rs",
            "attrs": [
              "#[kani::proof]",
              "#[kani::proof]"
            ],
            "func": "fn standard_proof() {\n        let val: u8 = kani::any();\n        assert_eq!(val, 1, \"Not eq to 1.\");\n    }"
          },
          {
            "file": "tests/standard_proof.rs",
            "attrs": [
              "#[kani::proof]",
              "#[kani::proof]",
              "#[allow(unused_mut)]"
            ],
            "func": "fn standard_proof_empty() {}"
          }
        ]"##]].assert_eq(&stdout);
}
