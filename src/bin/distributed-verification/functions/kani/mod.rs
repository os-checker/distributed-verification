mod coercion;
mod reachability;

pub use reachability::{CallGraph, collect_reachable_items};

use std::sync::LazyLock;

/// `#[kanitool::xxx]` attributes.
pub static KANI_TOOL_ATTRS: LazyLock<Vec<[String; 2]>> = LazyLock::new(|| {
    vec![
        [TOOL.into(), "proof".into()],
        [TOOL.into(), "proof_for_contract".into()],
        // attrs for contracts
        [TOOL.into(), "requires".into()],
        [TOOL.into(), "ensures".into()],
    ]
});

/// Tool attribute #[kani] expands.
pub const TOOL: &str = "kanitool";
