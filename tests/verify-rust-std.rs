use distributed_verification::diff::{KaniListJson, MergedHarnesses};

mod utils;
use utils::*;

/// Read kani-list.json generated from verify-rust-std CI.
fn read_kani_list_json() -> Result<KaniListJson> {
    const KANI_LIST_JSON: &str = "tmp/ubuntu-latest-kani-list.json/kani-list.json";
    const PREFIX: &str = "/home/runner/work/verify-rust-std/verify-rust-std/library/";

    let mut kani_list: KaniListJson = read_file(KANI_LIST_JSON)?;
    kani_list.strip_path_prefix_raw(PREFIX)?;
    Ok(kani_list)
}

fn read_core_json() -> Result<Vec<SerFunction>> {
    const CORE_JSON: &str = "./assets/core.json";
    read_file(CORE_JSON)
}

#[test]
fn core_json() -> Result<()> {
    let v_func = read_core_json()?;
    let merged = MergedHarnesses::new(&v_func);

    #[derive(Debug)]
    #[allow(dead_code)]
    struct Count {
        standard: usize,
        contract: usize,
    }

    let count = Count { standard: merged.standard.len(), contract: merged.contract.len() };
    dbg!(count);

    Ok(())
}

#[test]
fn read() -> Result<()> {
    let kani_list = read_kani_list_json()?;

    let harness_names = kani_list.harness_names(|file, _| file.starts_with("core/"));

    let v_func = read_core_json()?;
    let merged = MergedHarnesses::new(&v_func);
    let function_names = merged.function_names(|f| f.file.starts_with("core/"));

    dbg!(&kani_list.totals, &harness_names, &function_names);

    let names_not_in_functions: Vec<_> = harness_names
        .iter()
        .filter_map(|&h| function_names.get(h).is_none().then_some(h))
        .collect();

    assert!(
        names_not_in_functions.is_empty(),
        "Some harnesses are not found: {names_not_in_functions:#?}"
    );

    Ok(())
}
