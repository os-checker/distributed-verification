use distributed_verification::statistics::*;
use indexmap::IndexMap;
use stable_mir::CrateDef;

fn new_stat() -> Stat {
    Stat { local: new_local_crate(), external: new_external_crates() }
}

fn new_external_crates() -> ExternalCrates {
    let external_crates = stable_mir::external_crates();
    let count = external_crates.len();
    // // NOTE: crate name may duplicate, like std will appear twice
    // let crates = external_crates.into_iter().map(|krate| krate.name).sorted().collect();
    ExternalCrates { count }
}

fn new_local_crate() -> LocalCrateFnDefs {
    let mut this = LocalCrateFnDefs::default();

    // for krate in stable_mir::find_crates("core") {
    let krate = stable_mir::local_crate();
    let fn_defs = krate.fn_defs();
    this.count.total = fn_defs.len();

    for fn_def in fn_defs {
        let name = fn_def.name();
        let attrs = fn_def.all_tool_attrs();
        if attrs.is_empty() {
            continue;
        }
        this.count.all_tool_attrs += 1;

        // Need robust tokens to recognize attributes
        // cc https://github.com/rust-lang/project-stable-mir/issues/83
        let kanitools_attrs = attrs
            .iter()
            .filter_map(|attr| {
                let attr = attr.as_str();
                attr.starts_with("#[kanitool::").then_some(attr)
            })
            .collect::<Vec<_>>();
        if kanitools_attrs.is_empty() {
            continue;
        }
        this.count.kanitools += 1;

        for attr in &kanitools_attrs {
            let attr = parse_kanitool(attr);
            if let Some(v) = this.kanitools.annotated_functions.get_mut(attr) {
                v.push(name.clone());
            } else {
                this.kanitools.annotated_functions.insert(attr.to_owned(), vec![name.clone()]);
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

pub fn analyze() -> crate::Result<()> {
    let stat = new_stat();
    dbg!(&stat);
    Ok(())
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
fn parse_kanitool(attr: &str) -> &str {
    // start from `#[`
    let end = match attr[2..].find(' ') {
        Some(pos) => 2 + pos,
        None => match attr[2..].find('(') {
            Some(pos) => 2 + pos,
            None => attr.len() - 1, // ignore `]`
        },
    };
    &attr[2..end]
}
