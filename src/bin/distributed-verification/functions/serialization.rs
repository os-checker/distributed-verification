use super::cache;
use distributed_verification::{Callee, SourceCode};
use rustc_stable_hash::{FromStableHash, SipHasher128Hash, StableHasher, hashers::SipHasher128};
use serde::Serialize;
use stable_mir::{CrateDef, mir::mono::Instance};
use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

/// A kani proof with its file source, attributes, and raw function content.
#[derive(Debug, Serialize)]
pub struct SerFunction {
    hash: String,
    /// DefId in stable_mir.
    def_id: String,
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

        // Though this is from body span, fn name and signature are included.
        let func = cache::get_source_code(&inst).unwrap();
        let callees: Vec<_> = fun.callees.iter().map(new_callee).collect();
        let callees_len = callees.len();

        // Hash: don't include def_id
        // NOTE: this hash considers callees.
        let mut hasher = StableHasher::<SipHasher128>::new();
        func.hash(&mut hasher);
        hasher.write_length_prefix(callees_len);
        callees.iter().for_each(|callee| callee.func.hash(&mut hasher));
        let Hash128(hash) = hasher.finish();

        SerFunction { hash, def_id, func, callees_len, callees }
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

fn new_callee(inst: &Instance) -> Callee {
    let def_id = format_def_id(inst);
    let func = cache::get_source_code(inst).unwrap();
    Callee { def_id, func }
}

/// Convertion from lib's SerFunction into the counterpart in main.rs
mod conversion {
    use super::*;
    use crate::functions::utils::vec_convertion;
    use distributed_verification as lib;

    impl From<SerFunction> for lib::SerFunction {
        fn from(value: SerFunction) -> Self {
            let SerFunction { hash, def_id, func, callees_len, callees } = value;
            let callees = vec_convertion(callees);
            Self { hash, def_id, func, callees_len, callees }
        }
    }

    impl From<&SerFunction> for lib::SimplifiedSerFunction {
        fn from(val: &SerFunction) -> Self {
            Self {
                hash: val.hash.clone(),
                name: val.func.name.clone(),
                file: val.func.file.clone(),
                proof_kind: val.func.proof_kind,
                callees_len: val.callees_len,
                callees: val.callees.iter().map(|c| c.func.name.clone()).collect(),
            }
        }
    }
}
