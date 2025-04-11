use super::ty_to_fndef;
use indexmap::IndexSet;
use rustc_middle::ty::TyCtxt;
use rustc_smir::rustc_internal::{internal, stable};
use stable_mir::{CrateDef, mir::*, ty::FnDef};

pub fn calls_in_body(body: &Body, v: &mut IndexSet<FnDef>) -> Vec<FnDef> {
    let locals = body.locals();
    let mut direct = Vec::new();
    for basic_block in &body.blocks {
        terminator_to_fn_def(&basic_block.terminator, locals, v, &mut direct);
    }
    // all direct calls in a body
    direct
}

/// Extract fn call from the terminator.
fn terminator_to_fn_def(
    terminator: &Terminator,
    locals: &[LocalDecl],
    v: &mut IndexSet<FnDef>,
    direct: &mut Vec<FnDef>,
) {
    let TerminatorKind::Call { func, args, .. } = &terminator.kind else { return };

    push_fn_def(func, v, locals, direct);

    // FIXME: Will there be a FnDef in an arg in MIR? Probably no?
    // Anyway, need find a test for this. But now assume yes.
    // Maybe a function pointer or raw pointer cast to fn pointer suit?
    for arg in args {
        push_fn_def(arg, v, locals, direct);
    }

    // Will there ever be a FnDef in destination? Skip destination for now.
}

/// Push a FnDef Operand if any.
fn push_fn_def(
    func: &Operand,
    v: &mut IndexSet<FnDef>,
    locals: &[LocalDecl],
    direct: &mut Vec<FnDef>,
) {
    let ty = func.ty(locals).expect("malformed operand or incompatible locals list");
    if let Some(fn_def) = ty_to_fndef(ty) {
        direct.push(fn_def);
        v.insert(fn_def);
    }
}

/// Recursively retrieve calls for a call.
pub fn recursive_callees(fn_def: FnDef, tcx: TyCtxt, v: &mut IndexSet<FnDef>) {
    let def_id = internal(tcx, fn_def.def_id());
    let mir = tcx.promoted_mir(def_id);
    dbg!(&mir, def_id);
    for body in mir {
        let stable_mir_body = stable(body);
        stable_mir_body.dump(&mut std::io::stdout(), &format!("{fn_def:?}")).unwrap();
        let direct = calls_in_body(&stable_mir_body, v);
        if dbg!(direct.iter().any(|f| !v.contains(f))) {
            // some calls haven't been reached, recurse
            for call in direct {
                recursive_callees(call, tcx, v);
            }
        }
    }
}
