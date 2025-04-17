//! Access cached data from local thread set for the given Instance.
//! If the data hasn't been available, generate one and insert it.
//! The data is always behind a borrow through the `get_*` callbacks.

use super::utils::{SourceCode, source_code_with};
use rustc_data_structures::fx::FxHashMap;
use rustc_middle::ty::TyCtxt;
use rustc_span::source_map::{SourceMap, get_source_map};
use stable_mir::{
    CrateDef, DefId,
    mir::{Body, mono::Instance},
};
use std::{cell::RefCell, cmp::Ordering, sync::Arc};

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

fn get_cache_func<T>(inst: &Instance, f: impl FnOnce(&CacheFunction) -> T) -> Option<T> {
    get_cache(|cache| cache.get_or_insert(inst).map(f))
}

pub fn get_body<T>(inst: &Instance, f: impl FnOnce(&Body) -> T) -> Option<T> {
    get_cache_func(inst, |cf| f(&cf.body))
}

pub fn get_source_code(inst: &Instance) -> Option<SourceCode> {
    get_cache_func(inst, |cf| cf.src.clone())
}

/// NOTE: this function doesn't auto insert SourceCode if Instance isn't handled.
pub fn share_same_source_code(a: &Instance, b: &Instance) -> bool {
    get_cache(|cache| {
        let defid_a = a.def.def_id();
        let defid_b = b.def.def_id();
        let src_a = cache.set.get(&defid_a);
        let src_b = cache.set.get(&defid_b);
        match (src_a, src_b) {
            (Some(cache_a), Some(cache_b)) => match (cache_a, cache_b) {
                (Some(func_a), Some(func_b)) => func_a.src == func_b.src,
                (None, None) => defid_a == defid_b,
                (None, Some(_)) => false,
                (Some(_), None) => false,
            },
            (None, None) => defid_a == defid_b,
            (None, Some(_)) => false,
            (Some(_), None) => false,
        }
    })
}

pub fn cmp_callees(a: &Instance, b: &Instance) -> Ordering {
    get_cache(|cache| {
        cache.get_or_insert(a);
        cache.get_or_insert(b);
        let func_a = cache.set.get(&a.def.def_id()).unwrap().as_ref().map(|f| &f.src);
        let func_b = cache.set.get(&b.def.def_id()).unwrap().as_ref().map(|f| &f.src);
        func_a.cmp(&func_b)
    })
}

struct Cache {
    set: FxHashMap<DefId, Option<CacheFunction>>,
    rustc: Option<RustcCxt>,
    path_prefixes: PathPrefixes,
}

impl Cache {
    fn new() -> Self {
        let (set, rustc) = Default::default();
        let path_prefixes = PathPrefixes::new();
        Cache { set, rustc, path_prefixes }
    }

    fn get_or_insert(&mut self, inst: &Instance) -> Option<&CacheFunction> {
        self.set
            .entry(inst.def.def_id())
            .or_insert_with(|| {
                let body = inst.body()?;
                let rustc = self.rustc.as_ref()?;
                let prefix = self.path_prefixes.prefixes();
                let src = source_code_with(inst, body.span, rustc.tcx, &rustc.src_map, prefix);
                Some(CacheFunction { body, src })
            })
            .as_ref()
    }
}

struct RustcCxt {
    tcx: TyCtxt<'static>,
    src_map: Arc<SourceMap>,
}

struct CacheFunction {
    body: Body,
    src: SourceCode,
}

struct PathPrefixes {
    pwd: String,
    sysroot: String,
}

impl PathPrefixes {
    fn new() -> Self {
        let mut pwd = std::env::current_dir().unwrap().into_os_string().into_string().unwrap();
        pwd.push('/');

        let out = std::process::Command::new("rustc").arg("--print=sysroot").output().unwrap();
        let sysroot = std::str::from_utf8(&out.stdout).unwrap().trim();
        let sysroot = format!("{sysroot}/lib/rustlib/src/rust/");
        PathPrefixes { pwd, sysroot }
    }

    fn prefixes(&self) -> [&str; 2] {
        [&*self.pwd, &self.sysroot]
    }
}
