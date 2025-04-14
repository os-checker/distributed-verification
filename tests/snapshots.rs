mod utils;
use utils::*;

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
