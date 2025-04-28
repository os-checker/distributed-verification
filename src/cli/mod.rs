use crate::Result;
use clap::Parser;
use distributed_verification::{
    kani_list::{KaniList, read_kani_list},
    kani_path,
};

/// Parse cli arguments.
pub fn parse() -> Result<Run> {
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
    #[arg(long)]
    check_kani_list: Option<String>,

    /// Only emit def_id for callees in JSON.
    #[arg(long, default_value_t = false)]
    simplify_json: bool,

    /// Continue compilation. Default to false, meaning compilation stops
    /// once proofs are analyzed.
    #[arg(long, default_value_t = false)]
    continue_compilation: bool,

    /// Emit statistics for proofs as an alternative JSON.
    /// No normal JSON is emitted.
    #[arg(long)]
    stat: Option<String>,

    /// Args for rustc. `distributed-verification -- [rustc_args]`
    /// No need to pass rustc as the first argument.
    rustc_args: Vec<String>,
}

impl Args {
    pub fn into_args(self) -> Result<Run> {
        let mut rustc_args = if self.no_kani_args {
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
        rustc_args.extend(self.rustc_args);

        let kani_list = self.check_kani_list.map(|path| read_kani_list(&path)).transpose()?;

        Ok(Run {
            json: self.json,
            kani_list,
            simplify_json: self.simplify_json,
            continue_compilation: self.continue_compilation,
            stat: self.stat,
            rustc_args,
        })
    }
}

pub struct Run {
    pub json: Option<String>,
    pub kani_list: Option<KaniList>,
    pub simplify_json: bool,
    pub continue_compilation: bool,
    pub stat: Option<String>,
    pub rustc_args: Vec<String>,
}
