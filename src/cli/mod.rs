use clap::Parser;
use std::path::PathBuf;

/// Parse cli arguments.
pub fn parse() -> Args {
    Args::parse()
}

/// A helper tool for verify-rust-std repo to speed up verification.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Json output file path. Print to stdout if not set.
    #[arg(long)]
    pub json: Option<PathBuf>,

    /// Args for rustc.
    pub rustc_args: Vec<String>,
}
