mod coercion;
mod reachability;

pub use reachability::{CallGraph, collect_reachable_items};

// use std::sync::LazyLock;
//
// /// `#[kanitool::xxx]` attributes.
// pub static KANI_TOOL_ATTRS: LazyLock<Vec<[String; 2]>> = LazyLock::new(|| {
//     vec![
//         [TOOL.into(), PROOF.into()],
//         [TOOL.into(), PROOF_FOR_CONTRACT.into()],
//         // attrs for contracts
//         [TOOL.into(), REQUIRES.into()],
//         [TOOL.into(), ENSURES.into()],
//     ]
// });

/// Tool attribute #[kani] expands.
pub const TOOL: &str = "kanitool";
pub const PROOF: &str = "proof";
pub const PROOF_FOR_CONTRACT: &str = "proof_for_contract";
// pub const REQUIRES: &str = "requires";
// pub const ENSURES: &str = "ensures";
