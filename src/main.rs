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

mod cli;
mod functions;
mod logger;

#[macro_use]
extern crate tracing;
#[macro_use]
extern crate eyre;

fn main() -> Result<()> {
    logger::init();
    let run = cli::parse()?;

    let res = run_with_tcx!(run.rustc_args, |tcx| {
        // let crates = stable_mir::external_crates();
        // dbg!(crates.len(), crates);
        // for krate in stable_mir::find_crates("core") {
        //     let fn_defs = krate.fn_defs();
        //     dbg!(fn_defs.len());
        //     for fn_def in fn_defs {
        //         let name = fn_def.name();
        //         let attrs = fn_def.all_tool_attrs();
        //         // let attrs = fn_def.tool_attrs(&["kanitool".into(), "proof".into()]);
        //         if attrs.is_empty() {
        //             continue;
        //         }
        //         let attrs = attrs.iter().map(|attr| attr.as_str()).collect::<Vec<_>>().join(" ");
        //         println!("{name}: {attrs:?}");
        //     }
        // }

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

        let res = match (res_check_kani_list, res_json) {
            (Ok(_), Ok(_)) => Ok(()),
            (Err(err), Ok(_)) | (Ok(_), Err(err)) => Err(err),
            (Err(err1), Err(err2)) => Err(eyre!(
                "1. Failed to match kani list:\n{err1:?}\n\n2. No json emitted:\n{err2:?}"
            )),
        };

        if run.continue_compilation {
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
