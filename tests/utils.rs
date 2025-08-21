#![allow(dead_code, unused_imports)]
use assert_cmd::Command;
use std::path::{Path, PathBuf};

pub use distributed_verification::SerFunction;
pub use expect_test::{expect, expect_file};
pub use eyre::Result;
pub use pretty_assertions::assert_eq;

pub fn cmd(args: &[&str]) -> String {
    let mut command = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    command.env("DV_LOG", "off").args(args);
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "Failed to test standard_proof.rs:\n{}",
        std::str::from_utf8(&output.stderr).unwrap()
    );

    String::from_utf8(output.stdout).unwrap()
}

/// Get rs files under a dir.
pub fn get_proofs(dir: &str) -> Result<Vec<PathBuf>> {
    let mut proofs = vec![];
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file()
            && path.extension().and_then(|ext| Some(ext.to_str()? == "rs")).unwrap_or(false)
        {
            proofs.push(path);
        }
    }
    proofs.sort();
    Ok(proofs)
}

pub fn file_stem(path: &Path) -> &str {
    path.file_stem().and_then(|f| f.to_str()).unwrap()
}

pub fn read_file<T: serde::de::DeserializeOwned>(path: &str) -> Result<T> {
    let file = std::fs::File::open(path)?;
    let val = serde_json::from_reader(file)?;
    Ok(val)
}

pub fn read_proofs(path: &str) -> Result<Vec<SerFunction>> {
    let v_func: Vec<SerFunction> = serde_json::from_str(path)?;
    Ok(v_func.into_iter().filter(SerFunction::is_proof).collect())
}
