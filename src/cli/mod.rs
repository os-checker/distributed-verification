use crate::functions::SourceCode;
use clap::Parser;
use distributed_verification::kani_path;
use std::sync::{LazyLock, Mutex};

/// Parse cli arguments.
pub fn parse() -> Run {
    Args::parse().into_args()
}

/// A helper tool for verify-rust-std repo to speed up verification.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Possible one of these values:
    /// * `--json false`: skip serializing to json
    /// * `--json path/to/file.json`
    /// * print to stdout if not set
    #[arg(long)]
    json: Option<String>,

    /// Rustc args for kani. Default to true, especially auto emitting
    /// kani args for rustc on single rs file.
    #[arg(long, default_value_t = false)]
    no_kani_args: bool,

    /// Run `kani list` after the analysis, and compare proofs to ensure
    /// analyzed proofs are identical to ones from kani list.
    #[arg(long, default_value_t = false)]
    check_kani_list: bool,

    /// Only emit def_id for callees in JSON.
    #[arg(long, default_value_t = false)]
    simplify_json: bool,

    /// Args for rustc. `distributed-verification -- [rustc_args]`
    /// No need to pass rustc as the first argument.
    rustc_args: Vec<String>,
}

impl Args {
    pub fn into_args(self) -> Run {
        let mut args = if self.no_kani_args {
            vec!["rustc".to_owned()]
        } else {
            let kani_path = kani_path();
            info!(kani_path, ?self);
            Vec::from(
                [
                    // the first argument to rustc is unimportant
                    "rustc",
                    "--crate-type=lib",
                    "--cfg=kani",
                    "-Zcrate-attr=feature(register_tool)",
                    "-Zcrate-attr=register_tool(kanitool)",
                    "--sysroot",
                    &kani_path,
                    "-L",
                    &format!("{kani_path}/lib"),
                    "--extern",
                    "kani",
                    "--extern",
                    &format!("noprelude:std={kani_path}/lib/libstd.rlib"),
                    "-Zunstable-options",
                    "-Zalways-encode-mir",
                    "-Zmir-enable-passes=-RemoveStorageMarkers",
                ]
                .map(String::from),
            )
        };
        args.extend(self.rustc_args);

        // set SIMPLIFY_JSON
        STATE.lock().unwrap().simplify_json = self.simplify_json;

        Run { json: self.json, check_kani_list: self.check_kani_list, rustc_args: args }
    }
}

pub struct Run {
    pub json: Option<String>,
    pub check_kani_list: bool,
    pub rustc_args: Vec<String>,
}

static STATE: LazyLock<Mutex<GlobalState>> =
    LazyLock::new(|| Mutex::new(GlobalState { simplify_json: false }));

struct GlobalState {
    simplify_json: bool,
}

/// simplify_json = false => don't skip
/// simplify_json = true => skip
pub fn skip_serialize_callee_souce_code(_: &SourceCode) -> bool {
    // NOTE: Stable MIR is run on another thread, which is not the same as main thread.
    // So this function need a synchronous global state.
    STATE.lock().unwrap().simplify_json
}
