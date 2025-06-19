use crate::Result;
use eyre::Context;
use std::{
    env::var,
    path::{Path, PathBuf},
    sync::LazyLock,
};

#[allow(non_snake_case)]
pub struct EnvVar {
    /// distributed-verification CLI or path
    pub DISTRIBUTED_VERIFICATION: String,
    /// verify_rust_std CLI or path
    pub VERIFY_RUST_STD: String,
    /// Path to verify-rust-std/library
    pub VERIFY_RUST_STD_LIBRARY: PathBuf,
    // Path to kani folder where bin, lib, no_core, and playback are sub directories
    // pub KANI_DIR: PathBuf,
    /// Path to folder where artifacts like core.json and rustflags.json are put.
    pub OUTPUT_DIR: PathBuf,
    /// RUSTFLAGS including kani lib path and more, separated by 0x1f
    pub CARGO_ENCODED_RUSTFLAGS: String,
}

impl EnvVar {
    pub fn write_rustflags_json(&self, json: &serde_json::Value) -> Result<()> {
        const JSON_FILE: &str = "rustflags.json";

        let path = self.OUTPUT_DIR.join(JSON_FILE);
        let writer = std::fs::File::create(&path)?;
        serde_json::to_writer_pretty(writer, json)?;
        let path = path.canonicalize()?;
        info!("{path:?} is written.");
        Ok(())
    }

    pub fn core_json(&self) -> PathBuf {
        self.OUTPUT_DIR.join("core.json")
    }
}

fn var_to_path(env: &str) -> PathBuf {
    let s = var(env).unwrap();
    let path = Path::new(&s);
    assert!(path.exists(), "{env}={s:?} doesn't point to a valid path.");
    path.canonicalize().unwrap()
}

fn var_or_string(env: &str, default: &str) -> String {
    var(env).unwrap_or_else(|_| default.to_owned())
}

pub static ENV: LazyLock<EnvVar> = LazyLock::new(|| EnvVar {
    DISTRIBUTED_VERIFICATION: var_or_string("DISTRIBUTED_VERIFICATION", "distributed-verification"),
    VERIFY_RUST_STD: var_or_string("VERIFY_RUST_STD", "verify_rust_std"),
    VERIFY_RUST_STD_LIBRARY: var_to_path("VERIFY_RUST_STD_LIBRARY"),
    // KANI_DIR: var_to_path("KANI_DIR"),
    OUTPUT_DIR: var_to_path("OUTPUT_DIR"),
    CARGO_ENCODED_RUSTFLAGS: cargo_encoded_rustflags().unwrap(),
});

const WRAPPER: &str = "WRAPPER";
/// Inner env var to know if the process is cargo wrapper (verify_rust_std).
pub fn is_wrapper() -> bool {
    var(WRAPPER).as_deref() == Ok("1")
}
/// Set inner env var when cargo wrapper is to run.
pub fn set_wrapper() -> (&'static str, &'static str) {
    (WRAPPER, "1")
}

pub fn set_rustc_wrapper() -> (&'static str, &'static str) {
    ("RUSTC", &ENV.VERIFY_RUST_STD)
}

const KANI_ARGS: &[&str] = &[
    "-C",
    "overflow-checks=on",
    "-Z",
    "unstable-options",
    "-Z",
    "trim-diagnostic-paths=no",
    "-Z",
    "human_readable_cgu_names",
    "-Z",
    "always-encode-mir",
    "--cfg=kani",
    "-Z",
    "crate-attr=feature(register_tool)",
    "-Z",
    "crate-attr=register_tool(kanitool)",
    // "-L",
    // "/home/zjp/rust/distributed-verification/kani/target/kani/no_core/lib",
    // "--extern",
    // "kani_core",
    "-C",
    "panic=abort",
    "-C",
    "symbol-mangling-version=v0",
    "-Z",
    "panic_abort_tests=yes",
    "-Z",
    "mir-enable-passes=-RemoveStorageMarkers",
    "--check-cfg=cfg(kani)",
];

fn kani_lib_path(kani_dir: &Path) -> Result<PathBuf> {
    // inject kani_core dependency to recognize kani module in core
    // -Lpath must be an absolute path
    let kani_lib = kani_dir.join("no_core").join("lib");
    kani_lib.canonicalize().with_context(|| format!("{kani_lib:?} can't be canonicalized"))
}

fn rustc_flags(kani_lib: &Path) -> Result<Vec<String>> {
    let kani_core = ["-L", kani_lib.to_str().unwrap(), "--extern=kani_core"];
    Ok(KANI_ARGS.iter().copied().chain(kani_core).map(|arg| arg.to_owned()).collect())
}

fn cargo_encoded_rustflags() -> Result<String> {
    let kani_dir = &var_to_path("KANI_DIR");
    let kani_lib = kani_lib_path(kani_dir)?;
    Ok(rustc_flags(&kani_lib)?.join("\u{1f}"))
}

#[test]
fn test_rustc_flags() {
    dbg!(cargo_encoded_rustflags().unwrap());
}
