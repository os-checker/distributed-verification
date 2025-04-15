mod utils;
use utils::*;

#[test]
fn test_proofs() -> eyre::Result<()> {
    let mut proofs = vec![];
    for entry in std::fs::read_dir("tests/proofs")? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file()
            && path.extension().and_then(|ext| Some(ext.to_str()? == "rs")).unwrap_or(false)
        {
            proofs.push(path);
        }
    }

    proofs.sort();
    expect![[r#"
        [
            "tests/proofs/ad_hoc.rs",
            "tests/proofs/proofs_for_contract.rs",
            "tests/proofs/standard_proofs.rs",
            "tests/proofs/standard_proofs_with_contracts.rs",
        ]
    "#]]
    .assert_debug_eq(&proofs);

    for path in &proofs {
        let file_stem = path.file_stem().and_then(|f| f.to_str()).unwrap();
        let json = cmd(&[&format!("tests/proofs/{file_stem}.rs")]);
        expect_file![format!("./snapshots/{file_stem}.json")].assert_eq(&json);
    }

    Ok(())
}

#[test]
/// Make sure latest kani is installed through cargo.
fn kani_installed() {
    let path = distributed_verification::kani_path();
    dbg!(&path);
}
