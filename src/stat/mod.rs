use indexmap::IndexMap;
use stable_mir::CrateDef;

pub fn analyze() -> crate::Result<()> {
    let external_crates = stable_mir::external_crates();
    dbg!(external_crates.len(), external_crates);

    let mut kanitools = IndexMap::<String, Vec<String>>::new();

    // for krate in stable_mir::find_crates("core") {
    let krate = stable_mir::local_crate();
    let fn_defs = krate.fn_defs();
    dbg!(fn_defs.len());
    for fn_def in fn_defs {
        let name = fn_def.name();
        let attrs = fn_def.all_tool_attrs();
        if attrs.is_empty() {
            continue;
        }
        // Need robust tokens to recognize attributes
        // cc https://github.com/rust-lang/project-stable-mir/issues/83
        let attrs = attrs
            .iter()
            .filter_map(|attr| {
                let attr = attr.as_str();
                attr.starts_with("#[kanitool::").then_some(attr)
            })
            .collect::<Vec<_>>();
        if attrs.is_empty() {
            continue;
        }
        for attr in &attrs {
            let attr = parse_kanitool(attr);
            if let Some(v) = kanitools.get_mut(attr) {
                v.push(name.clone());
            } else {
                kanitools.insert(attr.to_owned(), vec![name.clone()]);
            }
        }
        // println!("{name}: {attrs:?}", attrs = attrs.join(" "));
    }

    kanitools.sort_unstable_keys();
    kanitools.values_mut().for_each(|v| v.sort_unstable());
    let counts = kanitools.iter().map(|(k, v)| (k, v.len())).collect::<IndexMap<_, _>>();
    dbg!(&counts, &kanitools);
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
