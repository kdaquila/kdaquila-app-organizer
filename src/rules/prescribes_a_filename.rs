//! Whether the content layer has already named the file.

use super::Rule;
use crate::diagnostics::Diagnostic;

/// A rename prescribed by the content layer fixes the casing too, so a
/// separate casing complaint would be redundant and possibly contradictory —
/// two different names offered for one file.
pub fn prescribes_a_filename(diagnostics: &[Diagnostic]) -> bool {
    diagnostics
        .iter()
        .any(|d| d.rule == Rule::FilenameMatchesExport)
}
