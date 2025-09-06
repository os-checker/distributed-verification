//! Access cached data from local thread set for the given Instance.
//! If the data hasn't been available, generate one and insert it.
//! The data is always behind a borrow through the `get_*` callbacks.

use crate::functions::utils::{StreamHasher, source_code_with, stable_hash};
use distributed_verification::{SerFunction, SourceCode};
use indexmap::IndexSet;
use rustc_data_structures::fx::FxHashMap;
use rustc_middle::ty::TyCtxt;
use rustc_public::mir::mono::Instance;
use rustc_span::source_map::{SourceMap, get_source_map};
use std::{
    cell::RefCell,
    cmp::{Ordering, Reverse},
    sync::Arc,
};

mod db;

thread_local! {
    static CACHE: RefCell<Cache> = RefCell::new(Cache::new());
}

pub fn set_rustc_ctx(tcx: TyCtxt) {
    // Safety: TyCtxt<'short> is extended to TyCtxt<'static>,
    // and we only use TyCtxt<'static> in stable_mir's callback.
    let tcx = unsafe { std::mem::transmute::<TyCtxt<'_>, TyCtxt<'static>>(tcx) };
    let src_map = get_source_map().expect("No source map.");
    let rustc = RustcCxt { tcx, src_map };
    CACHE.with(|c| c.borrow_mut().rustc = Some(rustc));
}

pub fn clear_rustc_ctx() {
    CACHE.with(|c| c.borrow_mut().rustc = None);
}

fn get_cache<T>(f: impl FnOnce(&mut Cache) -> T) -> T {
    CACHE.with(|c| f(&mut c.borrow_mut()))
}

pub fn has_body(inst: &Instance) -> bool {
    get_cache(|c| c.get_or_insert(inst).is_some())
}

/// Set direct callees in a function. This should be called once.
pub fn set_callees(inst: &Instance, callees: Box<[Instance]>) {
    get_cache(move |c| {
        for callee in &*callees {
            _ = c.get_or_insert(callee);
        }
        c.get_mut(inst).set_callees(callees)
    })
}

/// Get a function whose hash is computed by traversing callees.
pub fn get_func_with_recursive_hash(inst: &Instance, set: &mut IndexSet<Instance>) -> SerFunction {
    get_cache(|c| c.get_func_with_recursive_hash(inst, set))
}

pub fn cmp_callees(a: &Instance, b: &Instance) -> Ordering {
    get_cache(|cache| {
        cache.get_or_insert(a);
        cache.get_or_insert(b);
        let func_a = cache.get(a);
        let func_b = cache.get(b);
        func_a.compare(func_b)
    })
}

pub fn store_to_db() {
    get_cache(|cache| cache.store_to_db());
}

type Functions = FxHashMap<Instance, Function>;

struct Cache {
    /// The reason to have Instance as the key is
    /// https://github.com/os-checker/distributed-verification/issues/42
    functions: Functions,
    rustc: Option<RustcCxt>,
    path_prefixes: PathPrefixes,
}

impl Cache {
    fn new() -> Self {
        let (set, rustc) = Default::default();
        let path_prefixes = PathPrefixes::new();
        Cache { functions: set, rustc, path_prefixes }
    }

    fn get_or_insert(&mut self, inst: &Instance) -> Option<&SerFunction> {
        self.functions
            .entry(*inst)
            .or_insert_with(|| {
                let Some(body) = inst.body() else { return Function::default() };
                let rustc = self.rustc.as_ref().expect("No TyCtxt available.");
                let prefix = self.path_prefixes.prefixes();
                let src = source_code_with(inst, body.span, rustc.tcx, &rustc.src_map, prefix);
                Function::new_non_recurisve(src)
            })
            .inner
            .as_ref()
    }

    fn get_mut(&mut self, inst: &Instance) -> &mut Function {
        self.functions
            .get_mut(inst)
            .unwrap_or_else(|| panic!("{} {inst:?} must be inserted before", inst.name()))
    }

    fn get(&self, inst: &Instance) -> &Function {
        self.functions
            .get(inst)
            .unwrap_or_else(|| panic!("{} {inst:?} must be inserted before", inst.name()))
    }

    fn get_func_with_recursive_hash(
        &mut self,
        inst: &Instance,
        set: &mut IndexSet<Instance>,
    ) -> SerFunction {
        fn new(hash: Box<str>, func: &SerFunction) -> SerFunction {
            let SerFunction { name, file, proof_kind, .. } = func;
            SerFunction { hash, name: name.clone(), file: file.clone(), proof_kind: *proof_kind }
        }

        let hash = self.get_recursive_hash(inst, set);
        let func = self.get(inst);
        new(hash, func.inner.as_ref().unwrap())
    }

    fn push_recursive_callees(&self, inst: &Instance, set: &mut IndexSet<Instance>) {
        for callee in &self.functions.get(inst).unwrap().callees {
            if set.insert(*callee) {
                // traverse this call
                self.push_recursive_callees(callee, set);
            }
        }
    }

    fn get_recursive_hash(&mut self, inst: &Instance, set: &mut IndexSet<Instance>) -> Box<str> {
        if let Some(recursive_hash) = self.get(inst).recursive_hash.clone() {
            recursive_hash
        } else {
            set.clear();
            set.insert(*inst);
            self.push_recursive_callees(inst, set);

            // stable sort through file, fn name, (direct) hash
            set.sort_unstable_by(|a, b| {
                let fields = |inst: &Instance| {
                    let func = self.get(inst).inner.as_ref()?;
                    Some((&*func.file, &*func.name, &*func.hash))
                };
                fields(a).cmp(&fields(b))
            });

            let mut hasher = StreamHasher::new();
            for inst in &*set {
                let hash = self.get(inst).inner.as_ref().map(|val| &*val.hash);
                hasher.append(hash);
            }
            let recursive_hash = hasher.finish();

            self.get_mut(inst).recursive_hash = Some(recursive_hash.clone());
            recursive_hash
        }
    }

    /// Store functions info to db
    fn store_to_db(&self) {
        let mut db = db::Db::new().unwrap();
        let crate_name = &rustc_public::local_crate().name;
        db.store(&self.functions, crate_name).unwrap();
    }
}

/// A function hash and its callees.
#[derive(Debug, Default)]
struct Function {
    /// Sorce information.
    src: SourceCode,
    /// This can be None due to the exsitence of Instance body.
    ///
    /// NOTE:
    /// * the hash is computed from current caller, not recursively obtained from callees
    /// * callees_len is thus zero, because we only know body string ATM
    inner: Option<SerFunction>,
    /// Direct calls in the body.
    callees: Box<[Instance]>,
    /// A hash computed by traversing callees and the function itself.
    recursive_hash: Option<Box<str>>,
}

impl Function {
    fn new_non_recurisve(src: SourceCode) -> Self {
        Function {
            inner: Some(SerFunction {
                hash: stable_hash(&src),
                name: src.name.clone().into(),
                file: src.file.clone().into(),
                proof_kind: src.proof_kind,
            }),
            // traverse callees later
            callees: Box::default(),
            // compute hash after recursive callees are available
            recursive_hash: None,
            src,
        }
    }

    fn set_callees(&mut self, callees: Box<[Instance]>) {
        if !self.callees.is_empty() {
            error!(?callees, ?self.callees, "self.callees should be empty, while actually not");
        }
        self.callees = callees;
    }

    fn compare(&self, other: &Self) -> Ordering {
        match (&self.inner, &other.inner) {
            (Some(a), Some(b)) => {
                // Sort by file, proof_kind, name, and recursive_hash.
                // None is less than Some, so reverse the order to make proof first.
                let x = (&*a.file, Reverse(a.proof_kind), &*a.name);
                let y = (&*b.file, Reverse(b.proof_kind), &*b.name);
                match x.cmp(&y) {
                    Ordering::Equal => {
                        // recursive_hash is still possible to be None, but why?
                        let h1 = self.recursive_hash.as_deref();
                        let h2 = other.recursive_hash.as_deref();
                        h1.cmp(&h2)
                    }
                    ord => ord,
                }
            }
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
        }
    }
}

struct RustcCxt {
    tcx: TyCtxt<'static>,
    src_map: Arc<SourceMap>,
}

struct PathPrefixes {
    pwd: String,
    sysroot: String,
}

impl PathPrefixes {
    fn new() -> Self {
        // Path from crate folder, especially relative to `verify-rust-std/library`.
        //
        // cc https://github.com/os-checker/distributed-verification/issues/82
        let mut pwd =
            std::env::current_dir().unwrap().parent().unwrap().to_str().unwrap().to_owned();
        pwd.push('/');

        let out = std::process::Command::new("rustc").arg("--print=sysroot").output().unwrap();
        let sysroot = std::str::from_utf8(&out.stdout).unwrap().trim();
        let sysroot = format!("{sysroot}/lib/rustlib/src/rust/");
        PathPrefixes { pwd, sysroot }
    }

    fn prefixes(&self) -> [&str; 2] {
        [&self.sysroot, &*self.pwd]
    }
}

#[test]
fn parse_attr() {
    let attr = r##"
#[attr = ConstStability {stability: PartialConstStability {level:
Stable {since: Version(RustcVersion {major: 1, minor: 65, patch: 0})},
feature: "ptr_const_cast", promotable: false}}]"##;

    use proc_macro2::TokenStream;
    use quote::ToTokens;
    use syn::parse::Parser;
    let attrs = syn::Attribute::parse_outer.parse_str(attr).unwrap();
    let mut ts = TokenStream::new();
    attrs.into_iter().for_each(|attr| ts.extend(attr.into_token_stream()));
    println!("{ts}");
}
