use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Stat {
    pub local: LocalCrateFnDefs,
    pub external: ExternalCrates,
}

/// External crates excluding the local one.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ExternalCrates {
    /// Count of external crates.
    pub count: usize,
    // /// Sorted by name.
    //pub crates: Vec<String>,
}

/// Metrics based on `Vec<FnDef>`.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LocalCrateFnDefs {
    pub count: CountFunctions,
    pub kanitools: KaniTools,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct KaniTools {
    /// The FnDef count in each attribute from annotated_functions.
    pub count: IndexMap<String, usize>,
    /// FnDefs that are annotated with `#[kanitools]`, group by attributes.
    /// A function may appear under multiple attributes.
    pub annotated_functions: IndexMap<String, Vec<String>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CountFunctions {
    /// Count of FnDefs. A FnDef is from like a normal function, method, or that in a trait.
    pub total: usize,
    /// FnDefs that annotated with tool attributes, including kanitools, clippy, and others.
    pub all_tool_attrs: usize,
    /// FnDefs annotated with `#[kanitools::*]`.
    pub kanitools: usize,
}
