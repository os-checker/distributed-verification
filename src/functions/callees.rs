use super::ty_to_fndef;
use indexmap::IndexSet;
use stable_mir::{mir::*, ty::FnDef};

pub fn calls_in_body(body: &Body, visited: &mut IndexSet<FnDef>) -> Vec<FnDef> {
    let locals = body.locals();
    let mut direct = Vec::new();
    for basic_block in &body.blocks {
        terminator_to_fn_def(&basic_block.terminator, locals, visited, &mut direct);
    }
    // all direct calls in a body
    direct
}

/// Extract fn call from the terminator.
fn terminator_to_fn_def(
    terminator: &Terminator,
    locals: &[LocalDecl],
    visited: &mut IndexSet<FnDef>,
    direct: &mut Vec<FnDef>,
) {
    let TerminatorKind::Call { func, args, .. } = &terminator.kind else { return };

    push_fn_def(func, visited, locals, direct);

    // FIXME: Will there be a FnDef in an arg in MIR? Probably no?
    // Anyway, need find a test for this. But now assume yes.
    // Maybe a function pointer or raw pointer cast to fn pointer suit?
    for arg in args {
        push_fn_def(arg, visited, locals, direct);
    }

    // Will there ever be a FnDef in destination? Skip destination for now.
}

/// Push a FnDef Operand if any.
fn push_fn_def(
    func: &Operand,
    visited: &mut IndexSet<FnDef>,
    locals: &[LocalDecl],
    direct: &mut Vec<FnDef>,
) {
    let ty = func.ty(locals).expect("malformed operand or incompatible locals list");
    if let Some(fn_def) = ty_to_fndef(ty) {
        direct.push(fn_def);
        visited.insert(fn_def);
    }
}

/// Recursively retrieve calls for a call.
pub fn recursive_callees(fn_def: FnDef, visited: &mut IndexSet<FnDef>) {
    let Some(body) = fn_def.body() else { return };
    let direct = calls_in_body(&body, visited);
    if direct.iter().any(|f| !visited.contains(f)) {
        // some calls haven't been reached, recurse
        for call in direct {
            recursive_callees(call, visited);
        }
    }
}
