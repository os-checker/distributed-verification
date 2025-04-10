#![feature(rustc_private, let_chains, hash_set_entry)]
#![allow(unused)]

extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_smir;
extern crate rustc_span;
extern crate stable_mir;

use eyre::{Context, Ok, Result};
use functions::source_code_with;
use rustc_driver::{Compilation, run_compiler};
use rustc_smir::rustc_internal::internal;
use stable_mir::{
    CrateDef, ItemKind,
    mir::mono::Instance,
    ty::{RigidTy, TyKind},
};
use std::path::PathBuf;

mod cli;
mod functions;
mod logger;

#[macro_use]
extern crate tracing;

fn main() {
    logger::init();
    let cli = cli::parse();
    let mut v = Vec::from(
        [
            // the first argument to rustc is unimportant
            "rustc",
            "--crate-type=lib",
            "--cfg=kani",
            "-Z",
            "crate-attr=feature(register_tool)",
            "-Z",
            "crate-attr=register_tool(kanitool)",
            "--sysroot",
            "/home/zjp/rust/kani/target/kani",
            "-L",
            "/home/zjp/rust/kani/target/kani/lib",
            "--extern",
            "kani",
            "--extern",
            "noprelude:std=/home/zjp/rust/kani/target/kani/lib/libstd.rlib",
            "-Zunstable-options",
        ]
        .map(String::from),
    );
    v.extend(cli.rustc_args);
    run_compiler(&v, &mut Callback { json: cli.json });
}

struct Callback {
    json: Option<String>,
}

impl rustc_driver::Callbacks for Callback {
    fn after_analysis<'tcx>(
        &mut self,
        _compiler: &rustc_interface::interface::Compiler,
        tcx: rustc_middle::ty::TyCtxt<'tcx>,
    ) -> Compilation {
        let src_map = rustc_span::source_map::get_source_map().expect("No source map.");
        let mut output = Vec::new();

        rustc_smir::rustc_internal::run(tcx, || {
            for item in stable_mir::all_local_items() {
                let _span = debug_span!("all_local_items", ?item).entered();
                if let Some(fun) = functions::Function::new(item, tcx, &src_map) {
                    output.push(fun);
                }
            }
        })
        .expect("Failed to run rustc_smir.");

        let res = || match &self.json {
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
        Compilation::Stop
    }
}
