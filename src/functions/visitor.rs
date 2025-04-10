use rustc_hir::{Block, Expr, ExprKind, HirId, QPath, def_id::DefId, intravisit::*};
use rustc_middle::ty::{TyCtxt, TypeckResults};
use std::collections::HashSet;

type Callees = HashSet<DefId>;

/// All nested callees for the fn body after recursive visit.
pub fn get_callees<'tcx>(block: &'tcx Block, tcx: TyCtxt<'tcx>) -> Callees {
    let hir_id = block.hir_id;
    let _span = info_span!("get_callees", ?hir_id);
    let mut visitor = VistFnBlock::new(hir_id, tcx);
    visitor.visit_block(block);
    visitor.callee
}

/// Visitor for body from top-level fn.
pub struct VistFnBlock<'tcx> {
    /// Nested calls.
    callee: HashSet<DefId>,
    hir_id: HirId,
    /// 'tcx seems to strict here
    tyck: &'tcx TypeckResults<'tcx>,
    tcx: TyCtxt<'tcx>,
}

impl<'tcx> VistFnBlock<'tcx> {
    pub fn new(hir_id: HirId, tcx: TyCtxt<'tcx>) -> Self {
        let hir_owner = tcx.hir_get_parent_item(hir_id);
        // let tyck = TypeckResults::new(hir_owner);
        let tyck = tcx.typeck(hir_owner);
        let callee = HashSet::new();
        VistFnBlock { callee, hir_id, tyck, tcx }
    }

    /// Add a callee from a function path.
    fn add_callee(&mut self, qpath: QPath<'_>) {
        let qpath_res = self.tyck.qpath_res(&qpath, self.hir_id);
        let def_id = qpath_res.def_id();
        self.callee.get_or_insert(def_id);
    }
}

impl<'tcx> Visitor<'tcx> for VistFnBlock<'tcx> {
    type MaybeTyCtxt = TyCtxt<'tcx>;
    type NestedFilter = rustc_middle::hir::nested_filter::OnlyBodies;

    fn maybe_tcx(&mut self) -> Self::MaybeTyCtxt {
        self.tcx
    }

    fn visit_expr(&mut self, ex: &'tcx Expr<'tcx>) -> Self::Result {
        // TODO: fn calls in closures;
        // Type-related paths (e.g., <T>::default or <T>::Output;
        // handle macro expansion.
        match ex.kind {
            ExprKind::Call(expr, exprs) => {
                if let ExprKind::Path(qpath) = expr.kind {
                    self.add_callee(qpath);
                } else {
                    walk_expr(self, expr);
                }
                exprs.iter().for_each(|e| walk_expr(self, e));
            }
            _ => walk_expr(self, ex),
        }
    }
}
