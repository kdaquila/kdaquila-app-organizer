//! A scoped rule waiver.

use crate::rules::Rule;
use serde::{Deserialize, Serialize};

/// A glob, and the rules that do not apply beneath it.
///
/// Not a path allowlist: an exception names which rules stop applying, so
/// waiving one thing never silently waives the rest. `reason` is required so
/// that every waiver carries the argument for its own existence.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Exception {
    pub path: String,
    pub waive: Vec<Rule>,
    pub reason: String,
}

impl Exception {
    /// Same scope, same effect — the `reason` is prose and does not count.
    pub fn same(&self, other: &Exception) -> bool {
        self.path == other.path && self.waive == other.waive
    }
}
