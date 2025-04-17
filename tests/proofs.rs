use indexmap::IndexMap;
use std::path::{Path, PathBuf};

mod utils;
use utils::{assert_eq, *};

fn get_proofs(dir: &str) -> Result<Vec<PathBuf>> {
    let mut proofs = vec![];
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file()
            && path.extension().and_then(|ext| Some(ext.to_str()? == "rs")).unwrap_or(false)
        {
            proofs.push(path);
        }
    }
    proofs.sort();
    Ok(proofs)
}

fn assert_unique_hash(proofs: &[PathBuf], v_json: &[Vec<SerFunction>]) {
    let mut map = IndexMap::<&str, Vec<(&Path, &str)>>::new();
    for (v, proof) in v_json.iter().zip(proofs) {
        for json in v {
            let def_id = &*json.def_id;
            let item = (&**proof, def_id);
            map.entry(&json.hash)
                .and_modify(|v_item| v_item.push(item))
                .or_insert_with(|| vec![item]);
        }
    }
    for (hash, v_item) in &map {
        let v_item_len = v_item.len();
        assert_eq!(
            v_item_len, 1,
            "{hash} should only correspond to single item, but \
             {v_item_len} items have the hash value:\n{v_item:?}",
        );
    }
}

#[test]
fn test_proofs() -> Result<()> {
    let proofs = get_proofs("tests/proofs")?;

    expect![[r#"
        [
            "tests/proofs/ad_hoc.rs",
            "tests/proofs/gen_by_macros.rs",
            "tests/proofs/proofs_for_contract.rs",
            "tests/proofs/standard_proofs.rs",
            "tests/proofs/standard_proofs_with_contracts.rs",
        ]
    "#]]
    .assert_debug_eq(&proofs);

    let mut v_json = Vec::<Vec<SerFunction>>::with_capacity(proofs.len());
    for path in &proofs {
        let file_stem = path.file_stem().and_then(|f| f.to_str()).unwrap();
        let text = cmd(&[&format!("tests/proofs/{file_stem}.rs")]);
        expect_file![format!("./snapshots/{file_stem}.json")].assert_eq(&text);
        v_json.push(serde_json::from_str(&text).unwrap());
    }

    assert_unique_hash(&proofs, &v_json);
    Ok(())
}

#[test]
fn test_compare() -> Result<()> {
    let proofs = get_proofs("tests/compare")?;

    expect![[r#"
        [
            "tests/compare/contract1.rs",
            "tests/compare/contract2.rs",
            "tests/compare/proof1.rs",
            "tests/compare/proof2.rs",
        ]
    "#]]
    .assert_debug_eq(&proofs);

    let mut v_json = Vec::<Vec<SerFunction>>::with_capacity(proofs.len());
    for path in &proofs {
        let file_stem = path.file_stem().and_then(|f| f.to_str()).unwrap();
        let text = cmd(&[&format!("tests/compare/{file_stem}.rs")]);
        // NOTE: don't write text to json file, since compare.rs write it in a different way
        v_json.push(serde_json::from_str(&text).unwrap());
    }

    assert_unique_hash(&proofs, &v_json);
    Ok(())
}

#[test]
/// Make sure latest kani is installed through cargo.
fn kani_installed() {
    let path = distributed_verification::kani_path();
    dbg!(&path);
}
