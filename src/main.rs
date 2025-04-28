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
use eyre::{Context, Result};
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
    let run = run;

    let res = run_with_tcx!(rustc_args, move |tcx| {
        let continue_compilation = run.continue_compilation;
        let res = if run.stat { stat::analyze() } else { analyze_proofs(tcx, run) };

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

    let res_json = (|| {
        let writer: Box<dyn std::io::Write>;
        match &run.json {
            Some(path) => {
                if path == "false" {
                    return eyre::Ok(());
                }
                let _span = error_span!("write_json", path).entered();
                let file = std::fs::File::create(path)?;
                writer = Box::new(file);
            }
            None => writer = Box::new(std::io::stdout()),
        }

        if run.simplify_json {
            let simplified: Vec<_> = output.iter().map(SimplifiedSerFunction::from).collect();
            serde_json::to_writer_pretty(writer, &simplified)
        } else {
            serde_json::to_writer_pretty(writer, &output)
        }
        .context("Failed to write proof json")
    })();

    match (res_check_kani_list, res_json) {
        (Ok(_), Ok(_)) => Ok(()),
        (Err(err), Ok(_)) | (Ok(_), Err(err)) => Err(err),
        (Err(err1), Err(err2)) => {
            Err(eyre!("1. Failed to match kani list:\n{err1:?}\n\n2. No json emitted:\n{err2:?}"))
        }
    }
}
