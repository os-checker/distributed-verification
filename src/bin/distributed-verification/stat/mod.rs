use crate::{
    cli::Output,
    functions::kani::{PROOF, PROOF_FOR_CONTRACT, TOOL},
};
use distributed_verification::{ProofKind, statistics::*};
use indexmap::IndexMap;
use rustc_hir::def_id::DefId;
use rustc_middle::ty::TyCtxt;
use rustc_public::{CrateDef, external_crates, local_crate, rustc_internal::internal};

fn new_stat(tcx: TyCtxt) -> Stat {
    Stat { local: new_local_crate(tcx), external: new_external_crates() }
}

fn new_external_crates() -> ExternalCrates {
    let external_crates = external_crates();
    let count = external_crates.len();
    // // NOTE: crate name may duplicate, like std will appear twice
    // let crates = external_crates.into_iter().map(|krate| krate.name).sorted().collect();
    ExternalCrates { count }
}

fn new_local_crate(tcx: TyCtxt) -> LocalCrateFnDefs {
    let mut this = LocalCrateFnDefs::default();

    let krate = local_crate();
    let crate_name = &*krate.name;
    this.crate_name = crate_name.to_owned();
    let fn_defs = krate.fn_defs();
    this.fn_defs.total = fn_defs.len();

    let module = &mut String::with_capacity(64);

    for fn_def in fn_defs {
        let name = fn_def.name();
        let mut kanitool_fn = false;

        let did = internal(tcx, fn_def.def_id());

        update_module_path(tcx, did, crate_name, module);

        // cc https://github.com/rust-lang/project-stable-mir/issues/83
        let kanitools_attrs = tcx.get_all_attrs(did).iter().filter_map(|attr| {
            if let rustc_hir::Attribute::Unparsed(attr) = attr {
                this.attrs.all_tool_attrs += 1;
                let paths = &attr.path.segments;
                if paths.first().map(|ident| ident.as_str() == TOOL).unwrap_or(false) {
                    kanitool_fn = true;
                    this.attrs.kanitools += 1;
                    return Some(paths.iter().map(|ident| ident.as_str()).collect::<Vec<_>>());
                }
            }
            None
        });

        let mut added_proof = false;
        let map = &mut this.count_in_module;
        for kani_attr in kanitools_attrs {
            let attr_str = kani_attr.join("::");
            if let Some(v) = this.kanitools.annotated_functions.get_mut(&attr_str) {
                v.push(name.clone());
            } else {
                this.kanitools.annotated_functions.insert(attr_str, vec![name.clone()]);
            }

            added_proof = add_proof_in_module(module, &kani_attr, map);
        }

        if !added_proof {
            // This function is not a proof.
            with_map_count_in_module(module, |c| c.not_proof += 1, map);
        }

        // Only metric on fns annotated with kani.
        if !kanitool_fn {
            continue;
        }

        this.fn_defs.kanitools.count += 1;
        this.fn_defs.kanitools.names.push(name);
    }

    this.fn_defs.kanitools.names.sort_unstable();

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
    out.emit(&stat, &stat.local.crate_name)
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

/// For the local crate being compiled, classify the function as per ProofKind, and add the count.
/// If the attr is not as ProofKind, nothing happens.
fn add_proof_in_module(module: &str, kani_attr: &[&str], map: &mut MapCountInModule) -> bool {
    let proof_kind = proof_kind(kani_attr);
    if proof_kind.is_none() {
        return false;
    }

    let increment = |val: &mut CountInModule| {
        let count = match proof_kind {
            Some(ProofKind::Standard) => &mut val.standard,
            Some(ProofKind::Contract) => &mut val.contract,
            None => unreachable!("None ProofKind has been handled for {kani_attr:?}"),
        };
        *count += 1;
    };
    with_map_count_in_module(module, increment, map);

    true
}

fn update_module_path(tcx: TyCtxt, did: DefId, crate_name: &str, module: &mut String) {
    module.clear();
    module.push_str(crate_name);
    // This is based on def_path returns path segment instead of `<...>`
    // e.g. rustc_public generates the following path for monomorphized fn
    // <&num::saturating::Saturating<u128> as ops::bit::BitOr<num::saturating::Saturating<u128>>>::bitor
    let fn_path = &tcx.def_path(did).data;
    if fn_path.len() > 1 {
        // Push the first submodule.
        let sym = match fn_path[0].data.name() {
            rustc_hir::definitions::DefPathDataName::Named(symbol) => symbol,
            rustc_hir::definitions::DefPathDataName::Anon { namespace } => namespace,
        };
        module.push_str("::");
        module.push_str(sym.as_str());
    }
}

fn proof_kind(attr: &[&str]) -> Option<ProofKind> {
    Some(match attr {
        [TOOL, PROOF] => ProofKind::Standard,
        [TOOL, PROOF_FOR_CONTRACT] => ProofKind::Contract,
        _ => return None,
    })
}
