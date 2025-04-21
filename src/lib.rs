use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};

/// A Rust funtion with its file source, attributes, and raw function content.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct SerFunction {
    pub hash: String,
    /// DefId in stable_mir.
    pub def_id: String,
    /// Attributes are attached the function, but it seems that attributes
    /// and function must be separated to query.
    pub attrs: Vec<String>,
    /// Raw function string, including name, signature, and body.
    pub func: SourceCode,
    /// Count of callees.
    pub callees_len: usize,
    /// Recursive function calls inside the body.
    pub callees: Vec<Callee>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Callee {
    pub def_id: String,
    pub func: SourceCode,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct SourceCode {
    /// Function name.
    pub name: String,

    /// Mangled function name.
    pub mangled_name: String,

    /// String of [`InstanceKind`].
    ///
    /// [`InstanceKind`]: https://doc.rust-lang.org/nightly/nightly-rustc/stable_mir/mir/mono/enum.InstanceKind.html
    pub kind: String,

    // A file path where src lies.
    // The path is stripped with pwd or sysroot prefix.
    pub file: String,

    /// Source that a stable_mir span points to.
    pub src: String,

    /// The count of macro backtraces.
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
    pub macro_backtrace: Vec<MacroBacktrace>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct MacroBacktrace {
    callsite: String,
    defsite: String,
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
