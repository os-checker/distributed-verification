use serde::{Deserialize, Serialize};

/// A Rust funtion with its file source, attributes, and raw function content.
#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Callee {
    pub def_id: String,
    pub func: SourceCode,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SourceCode {
    // A file path where src lies.
    // The path is stripped with pwd or sysroot prefix.
    file: String,

    /// Source that a stable_mir span points to.
    pub src: String,
    /// Is the stable_mir span from a macro expansion?
    /// If it is from an expansion, what's the source code before expansion?
    /// * Some(_) happens when the src (stable_mir) span comes from expansion, and tells
    ///   the source before the expansion.
    /// * None if the src is not from a macro expansion.
    pub before_expansion: Option<String>,
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
