use distributed_verification::statistics::Stat;

mod utils;
use utils::*;

#[test]
fn stat() -> Result<()> {
    let proofs = get_proofs("tests/proofs")?;

    for path in &proofs {
        let file_stem = file_stem(path);
        let stat_path = format!("snapshots/stat/{file_stem}.json");
        dbg!(&stat_path);

        let path = path.to_str().unwrap();
        // run `distributed-verification path --json=false --stat`
        let text = &cmd(&[path, "--json=false", "--stat"]);
        // ensure this can be deserialized
        let _: Stat = serde_json::from_str(text).unwrap();

        expect_file![stat_path].assert_eq(text);
    }

    Ok(())
}
