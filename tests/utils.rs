use assert_cmd::Command;

#[allow(unused_imports)] // seems a bug in clippy
pub use expect_test::{expect, expect_file};
#[allow(unused_imports)] // seems a bug in clippy
pub use pretty_assertions::assert_eq;

pub fn cmd(args: &[&str]) -> String {
    let mut command = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    command.env("RUST_LOG", "off").args(args);
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "Failed to test standard_proof.rs:\n{}",
        std::str::from_utf8(&output.stderr).unwrap()
    );

    String::from_utf8(output.stdout).unwrap()
}
