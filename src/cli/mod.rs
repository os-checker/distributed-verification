use crate::Result;
use clap::{Parser, builder::OsStr};
use distributed_verification::{
    kani_list::{KaniList, read_kani_list},
    kani_path,
};
use eyre::Context;
use std::fmt;

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
    #[arg(long, default_missing_value = Output::Stdout, default_value = Output::False, num_args= 0..=1)]
    json: Output,

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
    #[arg(long, default_missing_value = Output::Stdout, default_value = Output::False, num_args= 0..=1)]
    stat: Output,

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
    pub json: Output,
    pub kani_list: Option<KaniList>,
    pub simplify_json: bool,
    pub continue_compilation: bool,
    pub stat: Output,
    pub rustc_args: Vec<String>,
}

/// Emit an output, usually a JSON.
#[derive(Debug, Clone)]
pub enum Output {
    /// Don't emit any output.
    False,
    /// Write to stdout.
    Stdout,
    /// Write to a local file.
    Path(String),
}

impl Output {
    pub fn emit<T: serde::Serialize>(self, val: &T) -> Result<()> {
        let _span = error_span!("emit", ?self).entered();

        let writer: Box<dyn std::io::Write> = match self {
            Output::False => return Ok(()),
            Output::Stdout => Box::new(std::io::stdout()),
            Output::Path(path) => {
                let file = std::fs::File::create(path)?;
                Box::new(file)
            }
        };

        serde_json::to_writer_pretty(writer, val).context("Failed to write proof json")
    }

    /// Should emit an output?
    pub fn should_emit(&self) -> bool {
        !matches!(self, Output::False)
    }
}

impl From<String> for Output {
    fn from(s: String) -> Output {
        match &*s.to_ascii_lowercase() {
            "false" => Self::False,
            "" | "stdout" => Self::Stdout,
            _ => Self::Path(s),
        }
    }
}

impl fmt::Display for Output {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Output::False => "false",
            Output::Stdout => "stdout",
            Output::Path(path) => path,
        })
    }
}

impl From<Output> for OsStr {
    fn from(value: Output) -> Self {
        value.to_string().into()
    }
}
