#![feature(rustc_private, let_chains)]

extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_span;
extern crate stable_mir;

use rustc_driver::{Compilation, run_compiler};

mod functions;

fn main() {
    let mut v: Vec<_> = std::env::args().collect();
    v.extend(
        [
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
    run_compiler(&v, &mut Callback {});
}

struct Callback {}

impl rustc_driver::Callbacks for Callback {
    fn after_analysis<'tcx>(
        &mut self,
        _compiler: &rustc_interface::interface::Compiler,
        tcx: rustc_middle::ty::TyCtxt<'tcx>,
    ) -> Compilation {
        let src_map = rustc_span::source_map::get_source_map().expect("No source map.");
        for id in tcx.hir_free_items() {
            let item = tcx.hir_item(id);
            let func = functions::Function::new(item, &src_map, tcx);
            dbg!(func);
        }
        // let items = stable_mir::all_local_items();
        // dbg!(&items);
        Compilation::Stop
    }
}
