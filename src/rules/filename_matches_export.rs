//! The filename is the export's name, transformed into the language's casing.

use super::{Rule, Waivers};
use crate::config::Profile;
use crate::diagnostics::{Diagnostic, Tag};
use crate::lang::PublicName;
use std::path::Path;

pub fn filename_matches_export(
    profile: &Profile,
    export: &PublicName,
    rel: &Path,
    path: &str,
    stem: &str,
    waivers: &Waivers,
) -> Option<Diagnostic> {
    if !waivers.active(Rule::FilenameMatchesExport) {
        return None;
    }
    // A casing with no converter can prescribe nothing, and guessing would be
    // worse than staying quiet.
    let expected = profile.name_case.suggest(&export.name)?;
    if expected == stem {
        return None;
    }

    Some(
        Diagnostic::new(
            Tag::Naming,
            Rule::FilenameMatchesExport,
            path,
            format!(
                "file name does not match its export `{} {}`",
                export.construct, export.name
            ),
        )
        .at_line(export.line)
        .help(format!("rename to {expected}{}", extension(rel))),
    )
}

fn extension(rel: &Path) -> String {
    rel.extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{e}"))
        .unwrap_or_default()
}
