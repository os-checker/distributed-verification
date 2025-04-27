use serde::{Deserialize, Serialize};

pub mod kani_list;

/// A kani proof with its file source, attributes, and raw function content.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct SerFunction {
    pub hash: String,
    /// DefId in stable_mir.
    pub def_id: String,
    /// Attributes are attached the function, but it seems that attributes
    /// and function must be separated to query.
    pub attrs: Vec<String>,
    /// Proof kind
    pub kind: Kind,
    /// Raw function string, including name, signature, and body.
    pub func: SourceCode,
    /// Count of callees.
    pub callees_len: usize,
    /// Recursive function calls inside the body.
    pub callees: Vec<Callee>,
}

/// kani proof kind
#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy)]
pub enum Kind {
    /// `#[kani::proof]` (actually `kanitool::proof`)
    #[default]
    Standard,
    /// `#[kani::proof_for_contract]` (actually `kanitool::proof_for_contract`)
    Contract,
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
    pub attrs: Vec<String>,
    pub name: String,
    pub file: String,
    pub callees_len: usize,
    pub callees: Vec<String>,
}

impl From<&SerFunction> for SimplifiedSerFunction {
    fn from(val: &SerFunction) -> Self {
        SimplifiedSerFunction {
            hash: val.hash.clone(),
            attrs: val.attrs.clone(),
            name: val.func.name.clone(),
            file: val.func.file.clone(),
            callees_len: val.callees_len,
            callees: val.callees.iter().map(|c| c.func.name.clone()).collect(),
        }
    }
}
