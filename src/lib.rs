use eyre::Result;
use serde::{Deserialize, Serialize};

#[macro_use]
extern crate tracing;
#[macro_use]
extern crate eyre;

pub mod diff;
pub mod error_handling;
pub mod kani_list;
pub mod logger;
pub mod statistics;

/// A kani proof with its file source, attributes, and raw function content.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct SerFunction {
    pub hash: String,
    /// DefId in stable_mir.
    pub def_id: String,
    /// Raw function string, including name, signature, and body.
    pub func: SourceCode,
    /// Count of callees.
    pub callees_len: usize,
    /// Recursive function calls inside the body.
    pub callees: Vec<Callee>,
}

/// Kani proof kind.
///
/// Suppose each proof only belongs to single kind.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProofKind {
    /// `#[kani::proof]`
    Standard,
    /// `#[kani::proof_for_contract]`
    Contract,
}

/// [`InstanceKind`], but remove Virtual idx and make Item as None to save space.
///
/// [`InstanceKind`]: https://doc.rust-lang.org/nightly/nightly-rustc/stable_mir/mir/mono/enum.InstanceKind.html
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InstKind {
    Intrinsic,
    Virtual,
    Shim,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Callee {
    pub def_id: String,
    pub func: SourceCode,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceCode {
    // A file path where src lies.
    // The path is stripped with pwd or sysroot prefix.
    pub file: String,

    /// Function name.
    pub name: String,

    /// InstanceKind, but normal Item is represented as None.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub inst_kind: Option<InstKind>,

    /// Potential kani proof kind: standard or contract.
    /// This tool will never identify if a function is an auto harness.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub proof_kind: Option<ProofKind>,

    /// Attributes are attached the function, but it seems that attributes
    /// and function must be separated to query.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub attrs: Vec<String>,

    /// Source that a stable_mir span points to.
    pub src: String,

    /// The count of macro backtraces.
    #[serde(skip_serializing_if = "zero")]
    #[serde(default)]
    pub macro_backtrace_len: usize,

    /// Is the stable_mir span from a macro expansion?
    /// If it is from an expansion, what's the source code before expansion?
    /// * Some(_) happens when the src (stable_mir) span comes from expansion, and tells
    ///   the source before the expansion.
    /// * None if the src is not from a macro expansion.
    ///
    /// Refer to [#31] to know sepecific cases.
    ///
    /// [#31]: https://github.com/os-checker/distributed-verification/issues/31
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub macro_backtrace: Vec<MacroBacktrace>,
}

fn zero(n: &usize) -> bool {
    *n == 0
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MacroBacktrace {
    pub callsite: String,
    pub defsite: String,
}

/// A local path to kani's artifacts.
///
/// Choose the following if found
/// * `$KANI_DIR`
/// * or `$KANI_HOME/kani-{version}`
/// * or `$HOME/.kani/kani-{version}`
pub fn kani_path() -> String {
    use std::env::var;
    let path = if let Ok(path) = var("KANI_DIR") {
        path
    } else {
        let kani = std::process::Command::new("kani").arg("--version").output().unwrap();
        let kani_folder = std::str::from_utf8(&kani.stdout).unwrap().trim().replace(' ', "-");
        let home = var("KANI_HOME").or_else(|_| var("HOME")).unwrap();
        format!("{home}/.kani/{kani_folder}")
    };
    assert!(std::fs::exists(&path).unwrap());
    path
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct SimplifiedSerFunction {
    pub hash: String,
    pub name: String,
    pub file: String,
    pub proof_kind: Option<ProofKind>,
    pub callees_len: usize,
    pub callees: Vec<String>,
}

impl From<&SerFunction> for SimplifiedSerFunction {
    fn from(val: &SerFunction) -> Self {
        SimplifiedSerFunction {
            hash: val.hash.clone(),
            name: val.func.name.clone(),
            file: val.func.file.clone(),
            proof_kind: val.func.proof_kind,
            callees_len: val.callees_len,
            callees: val.callees.iter().map(|c| c.func.name.clone()).collect(),
        }
    }
}
