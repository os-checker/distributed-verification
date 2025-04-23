use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, process::Command};

use crate::{Kind, SerFunction};

/// Output of `kani list` command.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct KaniList {
    pub kani_version: String,
    pub file_version: String,
    pub standard_harnesses: IndexMap<String, IndexSet<String>>,
    pub contract_harnesses: IndexMap<String, IndexSet<String>>,
    pub contracts: IndexSet<ContractedFunction>,
    pub totals: Total,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContractedFunction {
    pub function: String,
    pub file: String,
    pub harnesses: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Total {
    pub standard_harnesses: usize,
    pub contract_harnesses: usize,
    pub functions_under_contract: usize,
}

/// Get kani list and check if it complies with Vec<SerFunction>.
pub fn check(file: &str, v_ser_fun: &[SerFunction]) {
    let list = get_kani_list(file);
    check_proofs(&list, v_ser_fun);
}

pub fn get_kani_list(file: &str) -> KaniList {
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

pub fn check_proofs(list: &KaniList, v_ser_fun: &[SerFunction]) {
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
        .map(|(idx, f)| ((&*f.func.file, &*f.func.name), (idx, f.kind)))
        .collect();

    // check all standard proofs are in distributed-verification json
    for (path, proofs) in &list.standard_harnesses {
        for proof in proofs {
            let key = (path.as_str(), proof.as_str());
            let val = map.get(&key).unwrap();
            dbg!(val);
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

    // double check
    for (&(path, proof), &(_, kind)) in &map {
        let harnesses = match kind {
            Kind::Standard => &list.standard_harnesses[path],
            Kind::Contract => &list.contract_harnesses[path],
        };
        _ = harnesses.get(proof).unwrap();
    }
}
