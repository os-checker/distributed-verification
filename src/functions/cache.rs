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
use std::{cell::RefCell, sync::Arc};

thread_local! {
    static CACHE: RefCell<Cache> = RefCell::new(Cache::default());
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

fn get_cache_func<T>(inst: &Instance, f: impl FnOnce(&CacheFunction) -> T) -> Option<T> {
    CACHE.with(|c| c.borrow_mut().get_or_insert(inst).map(f))
}

pub fn get_body<T>(inst: &Instance, f: impl FnOnce(&Body) -> T) -> Option<T> {
    get_cache_func(inst, |cf| f(&cf.body))
}

pub fn get_source_code(inst: &Instance) -> Option<SourceCode> {
    get_cache_func(inst, |cf| cf.src.clone())
}

#[derive(Default)]
struct Cache {
    rustc: Option<RustcCxt>,
    set: FxHashMap<DefId, Option<CacheFunction>>,
}

impl Cache {
    fn get_or_insert(&mut self, inst: &Instance) -> Option<&CacheFunction> {
        self.set
            .entry(inst.def.def_id())
            .or_insert_with(|| {
                let body = inst.body()?;
                let rustc = self.rustc.as_ref()?;
                let src = source_code_with(body.span, rustc.tcx, &rustc.src_map);
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
