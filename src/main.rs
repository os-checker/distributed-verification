#![feature(rustc_private, let_chains, hash_set_entry, hasher_prefixfree_extras)]

extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
#[macro_use]
extern crate rustc_smir;
extern crate rustc_span;
extern crate rustc_stable_hash;
extern crate stable_mir;

use distributed_verification::kani_path;
// FIXME: this is a bug for rustc_smir, because rustc_interface is used by
// run_with_tcx! without being imported inside.
use rustc_smir::rustc_internal;

mod cli;
mod functions;
mod logger;

#[macro_use]
extern crate tracing;

fn main() {
    logger::init();
    let cli = cli::parse();
    let kani_path = kani_path();
    assert!(std::fs::exists(&kani_path).unwrap());
    let mut args = Vec::from(
        [
            // the first argument to rustc is unimportant
            "rustc",
            "--crate-type=lib",
            "--cfg=kani",
            "-Zcrate-attr=feature(register_tool)",
            "-Zcrate-attr=register_tool(kanitool)",
            "--sysroot",
            &kani_path,
            "-L",
            &format!("{kani_path}/lib"),
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
    args.extend(cli.rustc_args);

    let res = run_with_tcx!(args, |tcx| {
        use eyre::{Context, Ok};
        let src_map = rustc_span::source_map::get_source_map().expect("No source map.");
        let output = functions::analyze(tcx, &src_map);

        let res = || match &cli.json {
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

        // Stop emitting artifact for the source code being compiled.
        ControlFlow::<(), ()>::Break(())
    });
    // rustc_smir uses `Err(CompilerError::Interrupted)` to represent ControlFlow::Break.
    assert!(res == Err(stable_mir::CompilerError::Interrupted(())), "Unexpected {res:?}");
}
