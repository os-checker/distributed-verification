use super::ty_to_fndef;
use indexmap::IndexSet;
use stable_mir::{CrateDef, mir::*, ty::FnDef};

pub fn calls_in_body(body: &Body) -> Vec<FnDef> {
    let locals = body.locals();
    let mut direct = Vec::new();
    for basic_block in &body.blocks {
        terminator_to_fn_def(&basic_block.terminator, locals, &mut direct);
    }
    // all direct calls in a body
    direct
}

/// Extract fn call from the terminator.
fn terminator_to_fn_def(terminator: &Terminator, locals: &[LocalDecl], direct: &mut Vec<FnDef>) {
    let TerminatorKind::Call { func, args, .. } = &terminator.kind else { return };

    push_fn_def(func, locals, direct);

    // FIXME: Will there be a FnDef in an arg in MIR? Probably no?
    // Anyway, need find a test for this. But now assume yes.
    // Maybe a function pointer or raw pointer cast to fn pointer suit?
    for arg in args {
        push_fn_def(arg, locals, direct);
    }

    // Will there ever be a FnDef in destination? Skip destination for now.
}

/// Push a FnDef Operand if any.
fn push_fn_def(func: &Operand, locals: &[LocalDecl], direct: &mut Vec<FnDef>) {
    let ty = func.ty(locals).expect("malformed operand or incompatible locals list");
    if let Some(fn_def) = ty_to_fndef(ty) {
        direct.push(fn_def);
    }
}

/// Recursively retrieve calls for a call.
pub fn recursive_callees(fn_def: FnDef, visited: &mut IndexSet<FnDef>) {
    // print!("[{}] ", fn_def.name());

    let Some(body) = fn_def.body() else { return };

    // let mut buf = Vec::with_capacity(1024);
    // body.dump(&mut buf, &format!("{fn_def:?}")).unwrap();
    // println!("{}", String::from_utf8(buf).unwrap());

    let mut direct = calls_in_body(&body);
    while let Some(call) = direct.pop() {
        // the call hasn't been reached, traverse
        if visited.insert(call) {
            recursive_callees(call, visited);
        }
    }
}
