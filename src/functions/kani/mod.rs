mod coercion;
mod reachability;

pub use reachability::{CallGraph, collect_reachable_items};

use std::sync::LazyLock;

/// `#[kanitool::xxx]` attributes.
pub static KANI_TOOL_ATTRS: LazyLock<Vec<[String; 2]>> = LazyLock::new(|| {
    vec![
        ["kanitool".into(), "proof".into()],
        ["kanitool".into(), "proof_for_contract".into()],
        // attrs for contracts
        ["kanitool".into(), "requires".into()],
        ["kanitool".into(), "ensures".into()],
    ]
});
