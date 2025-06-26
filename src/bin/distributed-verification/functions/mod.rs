use distributed_verification::SerFunction;
use indexmap::IndexSet;
use kani::{CallGraph, collect_reachable_items};
use rustc_data_structures::fx::FxHashSet;
use rustc_middle::ty::TyCtxt;
use stable_mir::mir::mono::{Instance, MonoItem};

mod cache;
pub use cache::{clear_rustc_ctx, set_rustc_ctx};

mod kani;
pub use kani::TOOL;

mod utils;

pub fn analyze(tcx: TyCtxt) -> Vec<SerFunction> {
    let local_items = stable_mir::all_local_items();
    let cap = local_items.len();

    let mut entries = Vec::with_capacity(cap);

    for item in local_items {
        let _span = error_span!("all_local_items", ?item).entered();

        let Ok(inst) = Instance::try_from(item).inspect_err(|err| error!(?err)) else { continue };
        entries.push(MonoItem::from(inst));
    }

    let (mono_items, callgraph) = collect_reachable_items(tcx, &entries);
    let mut set_of_instance = FxHashSet::with_capacity_and_hasher(1024, Default::default());

    // Filter out non kanitool functions.
    let mut v_func: Vec<_> = mono_items
        .iter()
        .filter_map(|f| Function::new(f, &callgraph))
        .map(|f| {
            let instance = f.set_callees();
            (cache::get_func_with_recursive_hash(&instance, &mut set_of_instance), instance)
        })
        .collect();
    // Sort by file path and function name.
    v_func.sort_by(|a, b| cache::cmp_callees(&a.1, &b.1));
    v_func.into_iter().map(|f| f.0).collect()
}

/// A Rust funtion with its file source, attributes, and raw function content.
#[derive(Debug)]
pub struct Function {
    /// Instance of the function.
    instance: Instance,

    /// Recursive fnction calls inside the body.
    /// The elements are sorted by file path and fn source code to keep hash value stable.
    callees: IndexSet<Instance>,
}

impl Function {
    pub fn new(item: &MonoItem, callgraph: &CallGraph) -> Option<Self> {
        // Skip non fn items
        let &MonoItem::Fn(instance) = item else {
            return None;
        };

        // Skip if no body.
        if !cache::has_body(&instance) {
            return None;
        }

        let mut callees = IndexSet::new();
        callgraph.recursive_callees(item, &mut callees);
        callees.sort_by(cache::cmp_callees);

        Some(Function { instance, callees })
    }

    fn set_callees(self) -> Instance {
        let callees = self.callees.into_iter().collect();
        cache::set_callees(&self.instance, callees);
        self.instance
    }
}
