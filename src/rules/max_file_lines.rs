//! The line budget, on the files carrying the logic.
//!
//! This overlaps `pylint`'s `too-many-lines` and ESLint's `max-lines`, and the
//! overlap is the point: the value is one threshold holding across every
//! language a project uses, whichever per-language linters it happens to have
//! switched on. Clippy has no equivalent at all.

use super::{Rule, Waivers};
use crate::config::Profile;
use crate::diagnostics::{Diagnostic, Tag};

pub fn max_file_lines(
    profile: &Profile,
    code_lines: usize,
    path: &str,
    waivers: &Waivers,
) -> Option<Diagnostic> {
    if code_lines <= profile.max_file_lines || !waivers.active(Rule::MaxFileLines) {
        return None;
    }
    Some(
        Diagnostic::new(
            Tag::Size,
            Rule::MaxFileLines,
            path,
            format!(
                "{code_lines} lines of code, and the budget is {}",
                profile.max_file_lines
            ),
        )
        .note("blank lines and comments are not counted")
        .help("pull a private helper out into its own file, or split the export in two"),
    )
}
