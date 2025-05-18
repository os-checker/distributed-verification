use crate::{cli::Output, functions::TOOL};
use distributed_verification::statistics::*;
use indexmap::IndexMap;
use rustc_middle::ty::TyCtxt;
use rustc_smir::rustc_internal::internal;
use stable_mir::CrateDef;

fn new_stat(tcx: TyCtxt) -> Stat {
    Stat { local: new_local_crate(tcx), external: new_external_crates() }
}

fn new_external_crates() -> ExternalCrates {
    let external_crates = stable_mir::external_crates();
    let count = external_crates.len();
    // // NOTE: crate name may duplicate, like std will appear twice
    // let crates = external_crates.into_iter().map(|krate| krate.name).sorted().collect();
    ExternalCrates { count }
}

fn new_local_crate(tcx: TyCtxt) -> LocalCrateFnDefs {
    let mut this = LocalCrateFnDefs::default();

    // for krate in stable_mir::find_crates("core") {
    let krate = stable_mir::local_crate();
    let fn_defs = krate.fn_defs();
    this.count.total = fn_defs.len();

    for fn_def in fn_defs {
        let name = fn_def.name();

        let did = internal(tcx, fn_def.def_id());
        // cc https://github.com/rust-lang/project-stable-mir/issues/83
        let kanitools_attrs: Vec<Vec<_>> = tcx
            .get_all_attrs(did)
            .filter_map(|attr| {
                if let rustc_hir::Attribute::Unparsed(attr) = attr {
                    this.count.all_tool_attrs += 1;
                    let paths = &attr.path.segments;
                    if paths.first().map(|ident| ident.as_str() == TOOL).unwrap_or(false) {
                        this.count.kanitools += 1;
                        return Some(paths.iter().map(|ident| ident.as_str()).collect());
                    }
                }
                None
            })
            .collect();

        for attr in &kanitools_attrs {
            let attr_str = attr.join("::");
            if let Some(v) = this.kanitools.annotated_functions.get_mut(&attr_str) {
                v.push(name.clone());
            } else {
                this.kanitools.annotated_functions.insert(attr_str, vec![name.clone()]);
            }
        }
    }

    this.kanitools.annotated_functions.sort_unstable_keys();
    this.kanitools.annotated_functions.values_mut().for_each(|v| v.sort_unstable());
    this.kanitools.count = this
        .kanitools
        .annotated_functions
        .iter()
        .map(|(k, v)| (k.to_owned(), v.len()))
        .collect::<IndexMap<_, _>>();
    this
}

pub fn analyze(out: Output, tcx: TyCtxt) -> crate::Result<()> {
    let stat = new_stat(tcx);
    out.emit(&stat)
}

// From verify-rust-std CI:
// * Standard proofs: 371 (diff: -2)
// * Contract proofs: 955 ✅
//
// counts = {
//     "kanitool::asserted_with": 21,
//     "kanitool::checked_with": 21,
//     "kanitool::disable_checks": 19,
//     "kanitool::fn_marker": 91,
//     "kanitool::modifies_wrapper": 21,
//     "kanitool::proof": 369,
//     "kanitool::proof_for_contract": 955,
//     "kanitool::recursion_check": 21,
//     "kanitool::replaced_with": 21,
//     "kanitool::should_panic": 98,
//     "kanitool::solver": 9,
//     "kanitool::stub_verified": 2,
//     "kanitool::unstable(feature": 11,
//     "kanitool::unwind": 16,
// }

// `#[kanitool::proof]`
// `#[kanitool::proof_for_contract = ...]`
// `#[kanitool::recursion_check = ...]`
// `#[kanitool::disable_checks(pointer)]`
// `#[kanitool::unstable(feature = \"ghost-state\", issue = 3946, reason =...]`
