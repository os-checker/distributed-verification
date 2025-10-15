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
    pub crate_name: String,
    pub attrs: CountAttrs,
    pub fn_defs: FnDefs,
    pub kanitools: KaniTools,
    /// Function counts directy in root and first submodule.
    /// The count in root module is not traversally collected.
    pub count_in_module: MapCountInModule,
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
pub struct CountAttrs {
    /// FnDefs that annotated with tool attributes, including kanitools, clippy, and others.
    pub all_tool_attrs: usize,
    /// FnDefs annotated with `#[kanitools::*]`.
    pub kanitools: usize,
}

// A FnDef is from like a normal function, method, or that in a trait.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FnDefs {
    /// Count of all FnDefs.
    pub total: usize,
    /// FnDefs annotated with kanitool.
    pub kanitools: KaniToolsFnDefs,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct KaniToolsFnDefs {
    pub count: usize,
    pub names: Vec<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CountInModule {
    pub not_proof: usize,
    pub standard: usize,
    pub contract: usize,
}

pub type MapCountInModule = IndexMap<String, CountInModule>;

/// Mainly used to increment the count.
pub fn with_map_count_in_module(
    module: &str,
    fn_val: impl FnOnce(&mut CountInModule),
    map: &mut MapCountInModule,
) {
    if let Some(val) = map.get_mut(module) {
        fn_val(val);
    } else {
        let mut val = CountInModule::default();
        fn_val(&mut val);
        map.insert(module.to_owned(), val);
    }
}
