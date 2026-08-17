//! One casing, for every folder and every file the language owns.

use super::{Rule, Waivers};
use crate::config::Profile;
use crate::diagnostics::{Diagnostic, Tag};
use crate::walk;
use std::path::Path;

/// `name` is the folder's name, or the file's stem.
///
/// Files whose content already prescribes a filename are not passed here: that
/// rename fixes the casing too, and offering two different names for one file
/// would be worse than saying nothing.
pub fn name_casing(
    profile: &Profile,
    rel: &Path,
    name: &str,
    suffix: &str,
    waivers: &Waivers,
) -> Option<Diagnostic> {
    let casing = profile.name_case;
    if casing.matches(name) || !waivers.active(Rule::NameCasing) {
        return None;
    }

    let mut diagnostic = Diagnostic::new(
        Tag::Naming,
        Rule::NameCasing,
        walk::display(rel),
        format!("`{name}{suffix}` is not {}", casing.as_str()),
    );
    // Only offer a rename the rule would actually accept: snake_casing
    // `foo bar` or `Foo-Bar` leaves them non-conforming, and prescribing the
    // name that was just rejected is worse than saying nothing.
    if let Some(suggestion) = casing.suggest(name)
        && suggestion != name
        && casing.matches(&suggestion)
    {
        diagnostic = diagnostic.help(format!("rename to {suggestion}{suffix}"));
    }
    Some(diagnostic)
}
