use super::ty_to_fndef;
use stable_mir::{mir::*, ty::FnDef};

pub fn calls_in_body(body: &Body, v: &mut Vec<FnDef>) {
    let locals = body.locals();
    for basic_block in &body.blocks {
        terminator_to_fn_def(&basic_block.terminator, locals, v);
    }
}

/// Extract fn call from the terminator.
fn terminator_to_fn_def(terminator: &Terminator, locals: &[LocalDecl], v: &mut Vec<FnDef>) {
    let TerminatorKind::Call { func, args, .. } = &terminator.kind else { return };

    push_fn_def(func, v, locals);

    // FIXME: Will there be a FnDef in an arg in MIR? Probably no?
    // Anyway, need find a test for this. But now assume yes.
    // Maybe a function pointer or raw pointer cast to fn pointer suit?
    for arg in args {
        push_fn_def(arg, v, locals);
    }

    // Will there ever be a FnDef in destination? Skip destination for now.
}

/// Push a FnDef Operand if any.
fn push_fn_def(func: &Operand, v: &mut Vec<FnDef>, locals: &[LocalDecl]) {
    let ty = func.ty(locals).expect("malformed operand or incompatible locals list");
    if let Some(fn_def) = ty_to_fndef(ty) {
        v.push(fn_def);
    }
}
