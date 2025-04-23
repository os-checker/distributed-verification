use super::{cache, utils::SourceCode};
use rustc_stable_hash::{FromStableHash, SipHasher128Hash, StableHasher, hashers::SipHasher128};
use serde::Serialize;
use stable_mir::{CrateDef, mir::mono::Instance};
use std::{cmp::Ordering, hash::Hasher};

/// A kani proof with its file source, attributes, and raw function content.
#[derive(Debug, Serialize)]
pub struct SerFunction {
    hash: String,
    /// DefId in stable_mir.
    def_id: String,
    /// Attributes are attached the function, but it seems that attributes
    /// and function must be separated to query.
    attrs: Vec<String>,
    /// Proof kind
    kind: Kind,
    /// Raw function string, including name, signature, and body.
    func: SourceCode,
    /// Count of callees.
    callees_len: usize,
    /// Recursive function calls inside the proof.
    callees: Vec<Callee>,
}

impl SerFunction {
    pub fn new(fun: super::Function) -> Self {
        let inst = fun.instance;
        let def_id = format_def_id(&inst);
        let attrs: Vec<_> = fun.attrs.iter().map(|a| a.as_str().to_owned()).collect();
        let kind = Kind::new(&attrs);
        // Though this is from body span, fn name and signature are included.
        let func = cache::get_source_code(&inst).unwrap_or_default();
        let callees: Vec<_> = fun.callees.iter().map(Callee::new).collect();
        let callees_len = callees.len();

        // Hash
        let mut hasher = StableHasher::<SipHasher128>::new();
        func.with_hasher(&mut hasher);
        hasher.write_length_prefix(attrs.len());
        attrs.iter().for_each(|attr| hasher.write_str(attr));
        hasher.write_length_prefix(callees_len);
        callees.iter().for_each(|callee| callee.func.with_hasher(&mut hasher));
        let Hash128(hash) = hasher.finish();

        SerFunction { hash, def_id, attrs, kind, func, callees_len, callees }
    }

    /// Compare by file and func string.
    pub fn cmp_by_file_and_func(&self, other: &Self) -> Ordering {
        self.func.cmp(&other.func)
    }
}

// ************* hash *************
struct Hash128(String);

impl FromStableHash for Hash128 {
    type Hash = SipHasher128Hash;

    fn from(SipHasher128Hash([a, b]): SipHasher128Hash) -> Hash128 {
        Hash128(format!("{a}{b}"))
    }
}
// ************* hash *************

fn format_def_id(inst: &Instance) -> String {
    format!("{:?}", inst.def.def_id())
}

#[derive(Debug, Serialize)]
pub struct Callee {
    def_id: String,
    func: SourceCode,
}

impl Callee {
    fn new(inst: &Instance) -> Self {
        let def_id = format_def_id(inst);
        let func = cache::get_source_code(inst).unwrap_or_default();
        Callee { def_id, func }
    }
}

/// kani proof kind
#[derive(Debug, Serialize)]
pub enum Kind {
    /// `#[kani::proof]` (actually `kanitool::proof`)
    Standard,
    /// `#[kani::proof_for_contract]` (actually `kanitool::proof_for_contract`)
    Contract,
}

impl Kind {
    /// ## Panic
    ///
    /// The given attributes must contain one of the proof kind macro.
    fn new(attrs: &[String]) -> Self {
        for attr in attrs {
            if attr.contains("kanitool::proof_for_contract") {
                return Kind::Contract;
            } else if attr.contains("kanitool::proof") {
                return Kind::Standard;
            }
        }
        panic!("{attrs:?} doesn't contain a proof kind.")
    }
}
