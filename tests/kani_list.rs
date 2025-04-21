use distributed_verification::KaniList;
use std::process::Command;

mod utils;
use utils::*;

#[test]
fn validate_kani_list_json() -> Result<()> {
    let proofs = get_proofs("tests/proofs")?;

    for path in &proofs {
        let kani_list = get_kani_list(path.to_str().unwrap());
        let file_stem = file_stem(path);
        let list_path = format!("kani_list/{file_stem}.txt");
        dbg!(&list_path);
        expect_file![list_path].assert_debug_eq(&kani_list);
    }

    Ok(())
}

fn get_kani_list(file: &str) -> KaniList {
    // kani list -Zlist -Zfunction-contracts --format=json file.rs
    let args = ["list", "-Zlist", "-Zfunction-contracts", "--format=json", file];
    let output = Command::new("kani").args(args).output().unwrap();
    assert!(
        output.status.success(),
        "Failed to run `kani list -Zlist -Zfunction-contracts --format=json {file}`:\n{}",
        std::str::from_utf8(&output.stderr).unwrap()
    );

    // read kani-list.json
    let file_json = std::fs::File::open("kani-list.json").unwrap();
    serde_json::from_reader(file_json).unwrap()
}
