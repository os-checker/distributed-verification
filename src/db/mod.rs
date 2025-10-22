use crate::{BoxStr, InstKind, MacroBacktrace, ProofKind};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

pub mod split_to_json;
pub mod sql;

/// All information for a function stored in db.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DbFunction {
    // A file path where src lies.
    // The path is stripped with pwd or sysroot prefix.
    pub file: String,

    /// Function name.
    pub name: String,

    /// A hash considering recursive calls.
    pub hash: BoxStr,

    /// A hash only computed from direct calls.
    pub hash_direct: BoxStr,

    /// InstanceKind, but normal Item is represented as None.
    pub inst_kind: Option<InstKind>,

    /// Potential kani proof kind: standard or contract.
    /// This tool will never identify if a function is an auto harness.
    pub proof_kind: Option<ProofKind>,

    /// Attributes are attached the function, but it seems that attributes
    /// and function must be separated to query.
    pub attrs: Vec<String>,

    /// Source that a stable_mir span points to.
    pub src: BoxStr,

    /// The count of macro backtraces.
    pub macro_backtrace_len: usize,

    /// For a function that is generated through macros.
    pub macro_backtrace: Vec<MacroBacktrace>,

    /// The count of callees recursively traversed.
    pub callees_len: usize,

    /// Recurisve callees where the string refers to recursive hash of the function.
    pub callees: Vec<BoxStr>,

    /// Crate name.
    #[serde(rename = "crate")]
    pub krate: BoxStr,

    /// Item path, separated by `::`. This should not contain generics (i.e. `<...>`).
    pub path: BoxStr,
}

impl DbFunction {
    /// Path to  erialize JSON. The string buffer may contain base folder, like `folder/`.
    fn json_path(&self, path_buf: &mut PathBuf) {
        path_buf.push(&self.file);

        // Create parent folder if not exists.
        if !fs::exists(&path_buf).unwrap() {
            fs::create_dir_all(&path_buf).unwrap();
        }

        path_buf.push(&self.name);
    }
}
