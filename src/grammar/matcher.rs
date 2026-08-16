//! Matching a path against the pattern variant list.
//!
//! On failure the interesting output is not "no", it is *why* — so a failed
//! match carries the best available explanation, chosen as the failure that
//! got deepest into a pattern of the right shape.

use super::{KIND, ROOT, pattern::Pattern};
use crate::config::{NameSet, Profile};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Matched {
    /// Index into the pattern list, for reporting which variant applied.
    pub pattern: usize,
    /// Segment name -> the path component that bound to it.
    pub bindings: BTreeMap<String, String>,
}

impl Matched {
    pub fn kind(&self) -> Option<&str> {
        self.bindings.get(KIND).map(String::as_str)
    }
}

#[derive(Debug, Clone)]
pub enum MatchOutcome {
    Matched(Matched),
    /// Lines explaining the closest miss, ready to render as diagnostic notes.
    NoMatch {
        notes: Vec<String>,
    },
}

/// Match a file's *directory* components against every pattern variant.
///
/// The `{files}` terminator is deliberately not matched here: a file in the
/// right folder with a badly cased name deserves a naming diagnostic, not
/// "no pattern matched".
pub fn match_path(
    patterns: &[Pattern],
    profile: &Profile,
    roots: &[String],
    dirs: &[&str],
) -> MatchOutcome {
    let mut best: Option<(usize, Vec<String>)> = None;

    for (index, pattern) in patterns.iter().enumerate() {
        let segments = pattern.dir_segments();
        if segments.len() != dirs.len() {
            continue;
        }
        match match_one(pattern, profile, roots, dirs) {
            Ok(bindings) => {
                return MatchOutcome::Matched(Matched {
                    pattern: index,
                    bindings,
                });
            }
            Err((depth, notes)) => {
                if best.as_ref().is_none_or(|(d, _)| depth > *d) {
                    best = Some((depth, notes));
                }
            }
        }
    }

    let notes = match best {
        Some((_, notes)) => notes,
        None => vec![format!(
            "no pattern places files {} folder{} below the project root",
            dirs.len(),
            if dirs.len() == 1 { "" } else { "s" }
        )],
    };
    MatchOutcome::NoMatch { notes }
}

/// `Ok(bindings)`, or `Err((index of the failing segment, explanation))`.
fn match_one(
    pattern: &Pattern,
    profile: &Profile,
    roots: &[String],
    dirs: &[&str],
) -> Result<BTreeMap<String, String>, (usize, Vec<String>)> {
    let mut bindings = BTreeMap::new();

    for (index, segment) in pattern.dir_segments().iter().enumerate() {
        let component = dirs[index];

        if segment == ROOT {
            if !roots.iter().any(|r| r == component) {
                return Err((
                    index,
                    vec![
                        format!("`{component}` is not a declared root"),
                        format!("declared roots: {}", roots.join(", ")),
                    ],
                ));
            }
            bindings.insert(segment.clone(), component.to_string());
            continue;
        }

        let Some(rule) = profile.segments.get(segment) else {
            bindings.insert(segment.clone(), component.to_string());
            continue;
        };

        if let Some(set) = &rule.one_of {
            let allowed = profile.resolve(set);
            if !allowed.iter().any(|a| a == component) {
                let headline = if segment == KIND {
                    format!("`{component}` is not a kind folder")
                } else {
                    format!("`{component}` is not allowed as {{{segment}}}")
                };
                return Err((
                    index,
                    vec![headline, format!("expected one of: {}", allowed.join(", "))],
                ));
            }
        }

        if let Some(set) = &rule.not_one_of {
            let denied = profile.resolve(set);
            if denied.iter().any(|d| d == component) {
                let what = match set {
                    NameSet::Ref(name) if name == "@kinds" => "a kind name",
                    _ => "reserved",
                };
                return Err((
                    index,
                    vec![format!(
                        "`{component}` is {what}, so it cannot also be a folder name"
                    )],
                ));
            }
        }

        if let Some(casing) = rule.casing
            && !casing.matches(component)
        {
            return Err((
                index,
                vec![format!("`{component}` is not {}", casing.as_str())],
            ));
        }

        bindings.insert(segment.clone(), component.to_string());
    }

    Ok(bindings)
}
