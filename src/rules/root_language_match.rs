//! A tracked file whose language contradicts the declaration of its root.

use super::{Rule, Waivers};
use crate::config::Language;
use crate::diagnostics::{Diagnostic, Tag};
use crate::walk;
use std::path::Path;

pub fn root_language_match(
    rel: &Path,
    root: &str,
    declared: Language,
    actual: Language,
    waivers: &Waivers,
) -> Option<Diagnostic> {
    if declared == actual || !waivers.active(Rule::RootLanguageMatch) {
        return None;
    }
    Some(Diagnostic::new(
        Tag::Root,
        Rule::RootLanguageMatch,
        walk::display(rel),
        format!(
            "`{root}/` is declared {}, but contains {} files",
            declared.as_str(),
            actual.as_str()
        ),
    ))
}
