//! How deep folders may nest below a root.
//!
//! All that survives of v1's path patterns. Once files may live anywhere, the
//! three-variant pattern list said exactly one thing that a number does not:
//! nothing. What it was actually enforcing was a nesting cap.

use super::{Rule, Waivers};
use crate::config::Profile;
use crate::diagnostics::{Diagnostic, Tag};
use crate::walk;
use std::path::Path;

/// `depth` is how many folders deep this directory sits below its root.
///
/// Only the shallowest offenders are reported. A tree that is two levels too
/// deep would otherwise produce a diagnostic for every directory beneath the
/// first one, all of which are fixed by the same move.
pub fn folder_depth(
    profile: &Profile,
    rel: &Path,
    depth: usize,
    waivers: &Waivers,
) -> Option<Diagnostic> {
    if depth != profile.max_folder_depth + 1 || !waivers.active(Rule::FolderDepth) {
        return None;
    }
    Some(
        Diagnostic::new(
            Tag::Folder,
            Rule::FolderDepth,
            walk::display(rel),
            format!(
                "folders nest {depth} deep below the root, and the limit is {}",
                profile.max_folder_depth
            ),
        )
        .help("flatten a level, or split the tree into two roots"),
    )
}
