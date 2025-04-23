use indexmap::IndexSet;
use kani::{CallGraph, KANI_TOOL_ATTRS, collect_reachable_items};
use rustc_middle::ty::TyCtxt;
use stable_mir::{
    CrateDef,
    crate_def::Attribute,
    mir::mono::{Instance, MonoItem},
};

mod cache;
pub use cache::{clear_rustc_ctx, set_rustc_ctx};

mod kani;

mod utils;
pub use utils::vec_convertion;

mod serialization;
pub use serialization::SerFunction;

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

    // Filter out non kanitool functions.
    let mut proofs: Vec<_> = mono_items
        .iter()
        .filter_map(|f| Function::new(f, &callgraph, |x| !x.attrs.is_empty()))
        .map(SerFunction::new)
        .collect();
    // Sort proofs by file path and source code.
    proofs.sort_by(|a, b| a.cmp_by_file_and_func(b));
    proofs
}

/// A Rust funtion with its file source, attributes, and raw function content.
#[derive(Debug)]
pub struct Function {
    /// Instance of the function.
    instance: Instance,

    /// kanitool's attributs.
    attrs: Vec<Attribute>,

    /// Recursive fnction calls inside the body.
    /// The elements are sorted by file path and fn source code to keep hash value stable.
    callees: IndexSet<Instance>,
}

impl Function {
    pub fn new(
        item: &MonoItem,
        callgraph: &CallGraph,
        filter: impl FnOnce(&Self) -> bool,
    ) -> Option<Self> {
        // Skip non fn items
        let &MonoItem::Fn(instance) = item else {
            return None;
        };

        // Skip if no body.
        cache::get_body(&instance, |_| ())?;

        // Only need kanitool attrs: proof, proof_for_contract, contract, ...
        let attrs = KANI_TOOL_ATTRS.iter().flat_map(|v| instance.def.tool_attrs(v)).collect();

        let mut callees = IndexSet::new();
        callgraph.recursive_callees(item, &mut callees);
        callees.sort_by(cache::cmp_callees);

        let this = Function { instance, attrs, callees };
        filter(&this).then_some(this)
    }
}
