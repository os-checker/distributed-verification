#![feature(rustc_private, let_chains, hash_set_entry, hasher_prefixfree_extras)]

extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
#[macro_use]
extern crate rustc_smir;
extern crate rustc_span;
extern crate rustc_stable_hash;
extern crate stable_mir;

use distributed_verification::{SimplifiedSerFunction, kani_list::check_proofs};
use eyre::Result;
use functions::{clear_rustc_ctx, set_rustc_ctx};
use rustc_middle::ty::TyCtxt;

mod cli;
mod functions;
mod logger;
mod stat;

#[macro_use]
extern crate tracing;
#[macro_use]
extern crate eyre;

fn main() -> Result<()> {
    logger::init();
    let mut run = cli::parse()?;
    let rustc_args = std::mem::take(&mut run.rustc_args);
    let stat = run.stat.clone();
    let run = run;

    let res = run_with_tcx!(rustc_args, move |tcx| {
        let continue_compilation = run.continue_compilation;
        let res = if stat.should_emit() {
            stat::analyze(stat)
        } else if run.json.should_emit() {
            analyze_proofs(tcx, run)
        } else {
            Ok(())
        };

        if continue_compilation {
            ControlFlow::<Result<()>, Result<()>>::Continue(res)
        } else {
            // Stop emitting artifact for the source code being compiled.
            ControlFlow::<Result<()>, Result<()>>::Break(res)
        }
    });

    match res {
        Ok(res_inner) | Err(stable_mir::CompilerError::Interrupted(res_inner)) => res_inner,
        Err(err) => Err(eyre!("Unexpected error {err:?}")),
    }
}

fn analyze_proofs(tcx: TyCtxt, run: cli::Run) -> Result<()> {
    set_rustc_ctx(tcx);
    let output = functions::analyze(tcx);
    clear_rustc_ctx();

    let output = functions::vec_convertion(output);
    let mut res_check_kani_list = Ok(());
    if let Some(kani_list) = run.kani_list {
        res_check_kani_list = check_proofs(&kani_list, &output);
    }

    let res_json = if run.simplify_json {
        let simplified: Vec<_> = output.iter().map(SimplifiedSerFunction::from).collect();
        run.json.emit(&simplified)
    } else {
        run.json.emit(&output)
    };

    match (res_check_kani_list, res_json) {
        (Ok(_), Ok(_)) => Ok(()),
        (Err(err), Ok(_)) | (Ok(_), Err(err)) => Err(err),
        (Err(err1), Err(err2)) => {
            Err(eyre!("1. Failed to match kani list:\n{err1:?}\n\n2. No json emitted:\n{err2:?}"))
        }
    }
}
