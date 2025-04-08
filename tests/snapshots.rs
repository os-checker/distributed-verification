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
            "func": "fn standard_proof() {\n        let val: u8 = kani::any();\n        assert_eq!(val, 1, \"Not eq to 1.\");\n    }",
            "callees": [
              "DefId(3:10310 ~ core[abab]::panicking::panic_fmt)",
              "DefId(20:281 ~ kani[75bd]::assert)",
              "DefId(20:283 ~ kani[75bd]::any)"
            ]
          },
          {
            "file": "tests/standard_proof.rs",
            "attrs": [
              "#[kani::proof]",
              "#[kani::proof]",
              "#[allow(unused_mut)]"
            ],
            "func": "fn standard_proof_empty() {}",
            "callees": []
          },
          {
            "file": "tests/standard_proof.rs",
            "attrs": [
              "#[kani::proof]",
              "#[kani::proof]"
            ],
            "func": "fn recursive_callees() {\n        crate::top_level();\n    }",
            "callees": [
              "DefId(0:4 ~ standard_proof[50eb]::top_level)"
            ]
          },
          {
            "file": "tests/standard_proof.rs",
            "attrs": [],
            "func": "pub fn top_level() {\n    m::func1();\n}",
            "callees": [
              "DefId(0:6 ~ standard_proof[50eb]::m::func1)"
            ]
          },
          {
            "file": "tests/standard_proof.rs",
            "attrs": [],
            "func": "pub fn func1() {}",
            "callees": []
          }
        ]"##]].assert_eq(&stdout);
}
