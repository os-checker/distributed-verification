mod utils;
use distributed_verification::SimplifiedSerFunction;
use utils::{assert_eq, *};

#[test]
fn ensure_identical_hash() -> Result<()> {
    let proofs = get_proofs("tests/proofs")?;

    for path in &proofs {
        let snapshot = format!("simplified/{}.json", file_stem(path));
        dbg!(&snapshot);

        let path = path.to_str().unwrap();

        // run `distributed-verificatio file.rs`
        let text_full = cmd(&[path]);
        let v_ser_function_full: Vec<SerFunction> = serde_json::from_str(&text_full).unwrap();

        // run `distributed-verification file.rs --simplify-json`
        let text_simplified = cmd(&[path, "--simplify-json"]);
        let v_ser_function_simplified: Vec<SimplifiedSerFunction> =
            serde_json::from_str(&text_simplified).unwrap();
        expect_file![snapshot].assert_eq(&text_simplified);

        for (full, simplified) in v_ser_function_full.iter().zip(&v_ser_function_simplified) {
            assert_eq!(full.hash, simplified.hash, "Full={full:?}\nSimplified={simplified:?}");
        }
    }

    Ok(())
}
