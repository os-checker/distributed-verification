use crate::Result;
use clap::Parser;
use distributed_verification::db::split_to_json::split_to_json;
use distributed_verification::logger;

pub fn run(args: &[String]) -> Result<()> {
    logger::init();
    SubCmdSplit::parse_from(args).run()
}

/// `verify_rust_std --db path --base folder` to extract functions into local files.
#[derive(Parser, Debug)]
struct SubCmdSplit {
    /// Path to a sqlite file containing DbFunctions.
    #[arg(long)]
    db: String,

    /// Path to base folder to store json files.
    #[arg(long)]
    base: String,
}

impl SubCmdSplit {
    fn run(self) -> Result<()> {
        split_to_json(&self.db, &self.base)
    }
}
