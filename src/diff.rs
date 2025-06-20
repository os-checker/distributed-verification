use crate::{Result, SimplifiedSerFunction};
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use std::path::{MAIN_SEPARATOR, Path};
use syn::parse::Parser;

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
    const KANITOOL: &str = "kanitool";
    const PROOF: &str = "proof";
    const CONTRACT: &str = "proof_for_contract";

    for attr in &func.attrs {
        if let Ok(v_attr) = syn::Attribute::parse_outer.parse_str(attr) {
            // Usually an attr string corresponds to single attr item,
            // but syn API donesn't provide one like that.
            for attr in &v_attr {
                let path = &attr.path().segments;
                if path.first().map(|first| first.ident == KANITOOL).unwrap_or(false) {
                    let ident = path.get(1).map(|p| &p.ident);
                    if let Some(second) = ident {
                        if second == PROOF {
                            standard.push(HarnessIdx(idx));
                        } else if second == CONTRACT {
                            contract.push(HarnessIdx(idx));
                        }
                    }
                }
            }
        }
    }
}
