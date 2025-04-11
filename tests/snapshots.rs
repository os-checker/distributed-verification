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
              "FnDef(DefId { id: 5, name: \"kani::any\" })",
              "FnDef(DefId { id: 6, name: \"kani::assert\" })",
              "FnDef(DefId { id: 7, name: \"kani::Arbitrary::any\" })"
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
              "FnDef(DefId { id: 3, name: \"top_level\" })",
              "FnDef(DefId { id: 4, name: \"m::func1\" })",
              "FnDef(DefId { id: 8, name: \"core::str::<impl str>::trim\" })",
              "FnDef(DefId { id: 9, name: \"core::str::<impl str>::trim_matches\" })",
              "FnDef(DefId { id: 15, name: \"core::str::<impl str>::get_unchecked\" })",
              "FnDef(DefId { id: 16, name: \"std::slice::SliceIndex::get_unchecked\" })",
              "FnDef(DefId { id: 14, name: \"std::str::pattern::ReverseSearcher::next_reject_back\" })",
              "FnDef(DefId { id: 18, name: \"std::str::pattern::ReverseSearcher::next_back\" })",
              "FnDef(DefId { id: 13, name: \"std::str::pattern::Searcher::next_reject\" })",
              "FnDef(DefId { id: 19, name: \"std::str::pattern::Searcher::next\" })",
              "FnDef(DefId { id: 12, name: \"std::str::pattern::Pattern::into_searcher\" })"
            ]
          },
          {
            "file": "tests/standard_proof.rs",
            "attrs": [],
            "func": "pub fn top_level() {\n    m::func1();\n}",
            "callees": [
              "FnDef(DefId { id: 4, name: \"m::func1\" })",
              "FnDef(DefId { id: 8, name: \"core::str::<impl str>::trim\" })",
              "FnDef(DefId { id: 9, name: \"core::str::<impl str>::trim_matches\" })",
              "FnDef(DefId { id: 15, name: \"core::str::<impl str>::get_unchecked\" })",
              "FnDef(DefId { id: 16, name: \"std::slice::SliceIndex::get_unchecked\" })",
              "FnDef(DefId { id: 14, name: \"std::str::pattern::ReverseSearcher::next_reject_back\" })",
              "FnDef(DefId { id: 18, name: \"std::str::pattern::ReverseSearcher::next_back\" })",
              "FnDef(DefId { id: 13, name: \"std::str::pattern::Searcher::next_reject\" })",
              "FnDef(DefId { id: 19, name: \"std::str::pattern::Searcher::next\" })",
              "FnDef(DefId { id: 12, name: \"std::str::pattern::Pattern::into_searcher\" })"
            ]
          },
          {
            "file": "tests/standard_proof.rs",
            "attrs": [],
            "func": "pub fn func1() {\n        let _a = \"\".trim();\n    }",
            "callees": [
              "FnDef(DefId { id: 8, name: \"core::str::<impl str>::trim\" })",
              "FnDef(DefId { id: 9, name: \"core::str::<impl str>::trim_matches\" })",
              "FnDef(DefId { id: 15, name: \"core::str::<impl str>::get_unchecked\" })",
              "FnDef(DefId { id: 16, name: \"std::slice::SliceIndex::get_unchecked\" })",
              "FnDef(DefId { id: 14, name: \"std::str::pattern::ReverseSearcher::next_reject_back\" })",
              "FnDef(DefId { id: 18, name: \"std::str::pattern::ReverseSearcher::next_back\" })",
              "FnDef(DefId { id: 13, name: \"std::str::pattern::Searcher::next_reject\" })",
              "FnDef(DefId { id: 19, name: \"std::str::pattern::Searcher::next\" })",
              "FnDef(DefId { id: 12, name: \"std::str::pattern::Pattern::into_searcher\" })"
            ]
          }
        ]"##]].assert_eq(&stdout);
}
