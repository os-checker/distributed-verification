use distributed_verification::KaniList;
use std::{collections::HashMap, process::Command};

mod utils;
use utils::{assert_eq, *};

#[test]
fn validate_kani_list_json() -> Result<()> {
    let proofs = get_proofs("tests/proofs")?;

    for path in &proofs {
        let file_stem = file_stem(path);
        let list_path = format!("kani_list/{file_stem}.txt");
        dbg!(&list_path);

        let path = path.to_str().unwrap();
        // run `kani list`
        let kani_list = get_kani_list(path);
        expect_file![list_path].assert_debug_eq(&kani_list);

        // run `distributed_verification`
        let text = cmd(&[path]);
        let v_ser_function: Vec<SerFunction> = serde_json::from_str(&text).unwrap();
        merge(&kani_list, &v_ser_function);
    }

    Ok(())
}

fn get_kani_list(file: &str) -> KaniList {
    // kani list -Zlist -Zfunction-contracts --format=json file.rs
    let args = ["list", "-Zlist", "-Zfunction-contracts", "--format=json", file];
    let output = Command::new("kani").args(args).output().unwrap();
    assert!(
        output.status.success(),
        "Failed to run `kani list -Zlist -Zfunction-contracts --format=json {file}`:\n{}",
        std::str::from_utf8(&output.stderr).unwrap()
    );

    // read kani-list.json
    let file_json = std::fs::File::open("kani-list.json").unwrap();
    serde_json::from_reader(file_json).unwrap()
}

fn merge(list: &KaniList, v_ser_fun: &[SerFunction]) {
    // sanity check
    let totals = &list.totals;
    assert_eq!(v_ser_fun.len(), totals.standard_harnesses + totals.contract_harnesses);
    assert_eq!(
        list.standard_harnesses.values().map(|s| s.len()).sum::<usize>(),
        totals.standard_harnesses
    );
    assert_eq!(
        list.contract_harnesses.values().map(|s| s.len()).sum::<usize>(),
        totals.contract_harnesses
    );

    let map: HashMap<_, _> = v_ser_fun
        .iter()
        .enumerate()
        .map(|(idx, f)| ((&*f.func.file, &*f.func.name), idx))
        .collect();

    // check all standard proofs are in distributed-verification json
    for (path, proofs) in &list.standard_harnesses {
        for proof in proofs {
            let key = (path.as_str(), proof.as_str());
            let idx = map.get(&key).unwrap();
            dbg!(idx);
        }
    }

    // check all contract proofs are in distributed-verification json
    for (path, proofs) in &list.contract_harnesses {
        for proof in proofs {
            let key = (path.as_str(), proof.as_str());
            let idx = map.get(&key).unwrap();
            dbg!(idx);
        }
    }

    // douoble check
    for &(path, proof) in map.keys() {
        // FIXME: split standard and contract proofs if kind supported
        match list.standard_harnesses.get(path) {
            Some(set) => _ = set.get(proof).unwrap(),
            None => match list.contract_harnesses.get(path) {
                Some(set) => _ = set.get(proof).unwrap(),
                None => panic!("{path} {proof} does not exist"),
            },
        };
    }
}
