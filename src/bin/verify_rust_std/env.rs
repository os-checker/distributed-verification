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
