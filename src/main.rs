#![feature(rustc_private, let_chains, hash_set_entry)]

extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_smir;
extern crate rustc_span;
extern crate stable_mir;

use eyre::{Context, Ok, Result};
use functions::analyze;
use rustc_driver::{Compilation, run_compiler};

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
            "-Zcrate-attr=feature(register_tool)",
            "-Zcrate-attr=register_tool(kanitool)",
            "--sysroot",
            "/home/zjp/rust/kani/target/kani",
            "-L",
            "/home/zjp/rust/kani/target/kani/lib",
            "--extern",
            "kani",
            "--extern",
            "noprelude:std=/home/zjp/rust/kani/target/kani/lib/libstd.rlib",
            "-Zunstable-options",
            "-Zalways-encode-mir",
            "-Zmir-enable-passes=-RemoveStorageMarkers",
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

        let output = rustc_smir::rustc_internal::run(tcx, || analyze(tcx, &src_map, &mut output))
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
