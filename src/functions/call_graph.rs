use indexmap::{IndexMap, IndexSet};
use rustc_hir::def_id::DefId;
use std::{cell::RefCell, rc::Rc};

/// Context across calls and top-level functions.
#[derive(Default)]
pub struct CtxInner {
    /// Functions that are reached along the whole graph.
    /// In case of recursive calls, if all direct called functions are
    /// in this set, no longer to go deeper.
    /// This set is always cleared at each top-level function.
    reached: IndexSet<DefId>,
    /// CallGraph cache: each call will be cached, so that the next time
    /// the callee is found, clone the callgraph from the cache to use.
    cache: IndexMap<DefId, Rc<CallGraph>>,
}

#[derive(Default, Clone)]
pub struct Ctx {
    inner: Rc<RefCell<CtxInner>>,
}

impl Ctx {
    pub fn new() -> Self {
        Ctx { inner: Default::default() }
    }

    fn borrow_mut<T>(&self, f: impl FnOnce(&mut CtxInner) -> T) -> T {
        f(&mut self.inner.borrow_mut())
    }

    /// Clear reached set. Should only be called at each top-level function.
    pub fn clear_reached_set(&self) {
        self.borrow_mut(|ctx| ctx.reached.clear());
    }

    /// Push a fn call (callee). Returns
    /// * true if the call is reached before
    /// * and a full call graph for the funcion if any
    pub fn push_reached_fn(&self, id: DefId) -> (bool, Option<Rc<CallGraph>>) {
        self.borrow_mut(|ctx| {
            let callgraph = ctx.cache.get(&id).cloned();
            let reached = ctx.reached.insert(id);
            (reached, callgraph)
        })
    }

    /// Push a full call graph. Panic if inserted more than once.
    pub fn push_call_graph(&self, id: DefId, callgraph: Rc<CallGraph>) {
        self.borrow_mut(|ctx| {
            let exist = ctx.cache.insert(id, callgraph);
            assert!(exist.is_none(), "Call graph for {id:?} must be inserted only fully and once.")
        });
    }
}

/// A call graph from inside a function.
pub struct CallGraph {
    /// Caller.
    func: DefId,
    /// Callees, inserted by the occurence order.
    callees: IndexMap<DefId, Rc<CallGraph>>,
    /// State on context.
    ctx: Ctx,
}

impl CallGraph {
    pub fn new(func: DefId, ctx: &Ctx) -> Self {
        let callees = IndexMap::new();
        let ctx = ctx.clone();
        CallGraph { func, callees, ctx }
    }

    pub fn push(&mut self, callee: DefId) -> PushState {
        let (reached, callgraph) = self.ctx.push_reached_fn(callee);
        if !reached {
            PushState::FirstPush
        } else if let Some(call_graph) = callgraph {
            PushState::FullCallGraph(call_graph)
        } else {
            PushState::Reached
        }
    }
}

/// State of pushing a function node.
pub enum PushState {
    /// On first push.
    FirstPush,
    /// Non first push.
    Reached,
    /// Fully visited with a full call graph.
    FullCallGraph(Rc<CallGraph>),
}

impl PushState {
    pub fn is_fist_push(&self) -> bool {
        matches!(self, Self::FirstPush)
    }

    pub fn get_call_graph(self) -> Option<Rc<CallGraph>> {
        match self {
            Self::FullCallGraph(callgraph) => Some(callgraph),
            _ => None,
        }
    }
}
