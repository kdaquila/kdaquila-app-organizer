//! The content layer: one parse, three rules derived from it.

use super::Waivers;
use super::filename_matches_export::filename_matches_export;
use super::max_file_lines::max_file_lines;
use super::single_primary_export::single_primary_export;
use super::unparsable::unparsable;
use crate::config::Profile;
use crate::diagnostics::Diagnostic;
use crate::lang::{LanguageProfile, PublicName};
use std::path::Path;

pub fn check_content(
    language: &dyn LanguageProfile,
    profile: &Profile,
    source: &str,
    rel: &Path,
    waivers: &Waivers,
) -> Vec<Diagnostic> {
    let path = crate::walk::display(rel);
    let stem = rel.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
    let module = language.read(source);

    if module.has_syntax_errors && module.names.is_empty() {
        return vec![unparsable(&path)];
    }

    let governed: Vec<&PublicName> = module
        .names
        .iter()
        .filter(|name| profile.governs(name.construct))
        .collect();

    let mut diagnostics = Vec::new();
    diagnostics.extend(single_primary_export(&governed, &path, waivers));

    // A file that still has to be split cannot be told what to be called: the
    // answer depends on which export survives the split.
    if let [only] = governed.as_slice() {
        diagnostics.extend(filename_matches_export(
            profile, only, rel, &path, stem, waivers,
        ));
    }

    // The budget rides on having a primary export, not on having exactly one,
    // so a file that fails the split is still told it is too long.
    if !governed.is_empty() {
        diagnostics.extend(max_file_lines(profile, module.code_lines, &path, waivers));
    }

    diagnostics
}
