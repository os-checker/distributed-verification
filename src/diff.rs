use crate::{ProofKind, Result, SimplifiedSerFunction};
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use std::{
    hash::Hash,
    path::{MAIN_SEPARATOR, Path},
};

pub type BoxStr = Box<str>;

// ************ `kani list --json` ************

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(transparent)]
pub struct KaniListHarnesses {
    pub inner: IndexMap<BoxStr, IndexSet<BoxStr>>,
}

impl KaniListHarnesses {
    pub fn strip_path_prefix(&mut self, prefix: &str) -> Result<()> {
        let mut map = IndexMap::with_capacity(self.inner.len());
        for (key, val) in self.inner.iter_mut() {
            let val = std::mem::take(val);
            match key.strip_prefix(prefix) {
                Some(stripped) => _ = map.insert(stripped.into(), val),
                None => bail!(
                    "The key `{key}`\nis not stripped with prefix `{prefix}`, \
                     which probably a bug."
                ),
            }
        }
        map.sort_unstable_keys();
        self.inner = map;
        Ok(())
    }

    fn files(&self) -> Vec<&str> {
        self.inner.keys().map(|s| &**s).collect()
    }

    fn names<'harness>(&'harness self, v: &mut Vec<&'harness str>) {
        for harnesses in self.inner.values() {
            for name in harnesses {
                v.push(&**name);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct ContractedFunction {
    /// The fully qualified name the user gave to the function (i.e. includes the module path).
    pub function: BoxStr,
    /// The (currently full-) path to the file this function was declared within.
    pub file: BoxStr,
    /// The pretty names of the proof harnesses (`#[kani::proof_for_contract]`) for this function
    pub harnesses: Box<[BoxStr]>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub struct Totals {
    standard_harnesses: usize,
    contract_harnesses: usize,
    functions_under_contract: usize,
}

/// The datastructure generated from `kani list --json`.
///
/// ref: https://github.com/model-checking/kani/blob/b64e59de669cd77b625cc8c0b9a94f29117a0ff7/kani-driver/src/list/output.rs#L113
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct KaniListJson {
    pub kani_version: BoxStr,
    pub file_version: BoxStr,
    pub standard_harnesses: KaniListHarnesses,
    pub contract_harnesses: KaniListHarnesses,
    pub contracts: IndexSet<ContractedFunction>,
    pub totals: Totals,
}

impl KaniListJson {
    pub fn strip_path_prefix<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref().canonicalize()?;
        let prefix = &format!("{}{MAIN_SEPARATOR}", path.to_str().unwrap());

        self.standard_harnesses.strip_path_prefix(prefix)?;
        self.contract_harnesses.strip_path_prefix(prefix)?;
        Ok(())
    }

    /// This function is used in tests.
    pub fn strip_path_prefix_raw(&mut self, prefix: &str) -> Result<()> {
        self.standard_harnesses.strip_path_prefix(prefix)?;
        self.contract_harnesses.strip_path_prefix(prefix)?;
        Ok(())
    }

    pub fn files(&self) -> serde_json::Value {
        serde_json::json!({
            "standard_harnesses": self.standard_harnesses.files(),
            "contract_harnesses": self.contract_harnesses.files()
        })
    }

    pub fn harness_names(&self) -> Result<Vec<&str>> {
        let totals = &self.totals;
        let len = totals.standard_harnesses + totals.contract_harnesses;
        let mut v = Vec::with_capacity(len);

        self.standard_harnesses.names(&mut v);
        self.contract_harnesses.names(&mut v);

        ensure!(
            v.len() == len,
            "These harnesses are duplicated: {outliers:#?}",
            outliers = count_gt1(&v)
        );

        v.sort_unstable();
        Ok(v)
    }
}

fn count_gt1<T: Copy + Hash + Eq>(v: &[T]) -> Vec<(T, u32)> {
    let mut map = IndexMap::with_capacity(v.len());
    for key in v {
        map.entry(*key).and_modify(|n| *n += 1).or_insert(1u32);
    }
    map.into_iter().filter(|(_, n)| *n != 1).collect()
}

// ************ difference ************

/// Index to the `&[SimplifiedSerFunction]`
pub struct HarnessIdx(pub usize);

type Harnesses = Vec<HarnessIdx>;

pub struct MergedHarnesses {
    pub standard: Harnesses,
    pub contract: Harnesses,
}

impl MergedHarnesses {
    pub fn new(v: &[SimplifiedSerFunction]) -> Self {
        let cap = v.len();
        let mut standard = Vec::with_capacity(cap);
        let mut contract = Vec::with_capacity(cap);
        for (idx, func) in v.iter().enumerate() {
            add_harness(idx, func, &mut standard, &mut contract);
        }
        standard.shrink_to_fit();
        contract.shrink_to_fit();
        MergedHarnesses { standard, contract }
    }
}

fn add_harness(
    idx: usize,
    func: &SimplifiedSerFunction,
    standard: &mut Harnesses,
    contract: &mut Harnesses,
) {
    match func.proof_kind {
        Some(ProofKind::Standard) => standard.push(HarnessIdx(idx)),
        Some(ProofKind::Contract) => contract.push(HarnessIdx(idx)),
        None => (),
    }
}
