use crate::{Kind, Result, SerFunction};
use eyre::ContextCompat;
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, process::Command};

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
    check_proofs(&list, v_ser_fun).unwrap();
}

/// Run `kani list` on single rust file.
pub fn get_kani_list(rs_file_path: &str) -> KaniList {
    // kani list -Zlist -Zfunction-contracts --format=json file.rs
    let args = ["list", "-Zlist", "-Zfunction-contracts", "--format=json", rs_file_path];
    let output = Command::new("kani").args(args).output().unwrap();
    assert!(
        output.status.success(),
        "Failed to run `kani list -Zlist -Zfunction-contracts --format=json {rs_file_path}`:\n{}",
        std::str::from_utf8(&output.stderr).unwrap()
    );

    // read kani-list.json
    read_kani_list("kani-list.json").unwrap()
}

/// Read a kani-list.json
pub fn read_kani_list(kani_list_path: &str) -> Result<KaniList> {
    let _span = debug_span!("read_kani_list", kani_list_path).entered();
    let file_json = std::fs::File::open(kani_list_path)?;
    Ok(serde_json::from_reader(file_json)?)
}

/// Check if all proofs matches in kani-list.json and SerFunctions.
pub fn check_proofs(list: &KaniList, v_ser_fun: &[SerFunction]) -> Result<()> {
    // sanity check
    let totals = &list.totals;
    {
        let len = v_ser_fun.len();
        let total = totals.standard_harnesses + totals.contract_harnesses;
        ensure!(
            len == total,
            "The length of Vec<SerFunction> is {len}, but total {total} proofs in kani list."
        );
    }
    {
        let len = list.standard_harnesses.values().map(|s| s.len()).sum::<usize>();
        let total = totals.standard_harnesses;
        ensure!(
            len == total,
            "Found {len} standard proofs, but kani list reports {total} of them."
        );
    }
    {
        let len = list.contract_harnesses.values().map(|s| s.len()).sum::<usize>();
        let total = totals.contract_harnesses;
        ensure!(
            len == total,
            "Found {len} contract proofs, but kani list reports {total} of them."
        );
    }

    let map: HashMap<_, _> = v_ser_fun
        .iter()
        .enumerate()
        .map(|(idx, f)| ((&*f.func.file, &*f.func.name), (idx, f.kind)))
        .collect();

    // check all standard proofs are in distributed-verification json
    for (path, proofs) in &list.standard_harnesses {
        for proof in proofs {
            let key = (path.as_str(), proof.as_str());
            map.get(&key).with_context(|| {
                format!(
                    "The standard proof {key:?} exists in kani list, \
                     but not in distributed-verification JSON."
                )
            })?;
        }
    }

    // check all contract proofs are in distributed-verification json
    for (path, proofs) in &list.contract_harnesses {
        for proof in proofs {
            let key = (path.as_str(), proof.as_str());
            map.get(&key).with_context(|| {
                format!(
                    "The contract proof {key:?} exists in kani list, \
                     but not in distributed-verification JSON."
                )
            })?;
        }
    }

    // double check
    for (&(path, proof), &(_, kind)) in &map {
        let harnesses = match kind {
            Kind::Standard => &list.standard_harnesses[path],
            Kind::Contract => &list.contract_harnesses[path],
        };
        harnesses.get(proof).with_context(|| {
            format!(
                "The {kind:?} proof {harnesses:?} exists in \
                     distributed-verification JSON, but not in kani list."
            )
        })?;
    }

    Ok(())
}
