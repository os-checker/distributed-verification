use crate::Result;
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
    /// Path to kani folder where bin, lib, no_core, and playback are sub directories
    pub KANI_DIR: PathBuf,
    /// Path to folder where artifacts like core.json and rustflags.json are put.
    pub OUTPUT_DIR: PathBuf,
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
    KANI_DIR: var_to_path("KANI_DIR"),
    OUTPUT_DIR: var_to_path("OUTPUT_DIR"),
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
