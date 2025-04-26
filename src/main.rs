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

use functions::{clear_rustc_ctx, set_rustc_ctx};
use rustc_middle::ty::TyCtxt;
use stable_mir::CrateDef;

mod cli;
mod functions;
mod logger;

#[macro_use]
extern crate tracing;

fn main() {
    logger::init();
    let run = cli::parse();

    let res = run_with_tcx!(run.args, |tcx| {
        use eyre::{Context, Ok};

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

        let res = || match &run.json {
            Some(path) => {
                if path == "false" {
                    return Ok(());
                }
                let _span = error_span!("write_json", path).entered();
                let file = std::fs::File::create(path)?;
                serde_json::to_writer_pretty(file, &output)
                    .with_context(|| "Failed to write functions to json")
            }
            None => serde_json::to_writer_pretty(std::io::stdout(), &output)
                .with_context(|| "Failed to write functions to stdout"),
        };

        res().unwrap();
        check_kani_list(output, tcx);

        // Stop emitting artifact for the source code being compiled.
        ControlFlow::<(), ()>::Break(())
    });
    // rustc_smir uses `Err(CompilerError::Interrupted)` to represent ControlFlow::Break.
    assert!(res == Err(stable_mir::CompilerError::Interrupted(())), "Unexpected {res:?}");
}

fn check_kani_list(output: Vec<functions::SerFunction>, tcx: TyCtxt) {
    let output: Vec<distributed_verification::SerFunction> = functions::vec_convertion(output);
    let crate_file = tcx.sess.local_crate_source_file().expect("No real crate root file.");
    let crate_file = crate_file.local_path().unwrap().to_str().unwrap();
    distributed_verification::kani_list::check(crate_file, &output);
}
