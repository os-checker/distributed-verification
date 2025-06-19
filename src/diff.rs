use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};

pub type BoxStr = Box<str>;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(transparent)]
pub struct Harnesses {
    pub inner: IndexMap<BoxStr, IndexSet<BoxStr>>,
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
    pub standard_harnesses: Harnesses,
    pub contract_harnesses: Harnesses,
    pub contracts: IndexSet<ContractedFunction>,
    pub totals: Totals,
}
