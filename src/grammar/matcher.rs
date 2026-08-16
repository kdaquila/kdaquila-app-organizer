//! Matching a path against the pattern variant list.
//!
//! On failure the interesting output is not "no", it is *why* — so a failed
//! match carries the best available explanation, chosen as the failure that
//! got deepest into a pattern of the right shape.

use super::{KIND, ROOT, pattern::Pattern};
use crate::config::{NameSet, Profile};
use std::collections::BTreeMap;

/// The declared root a path was found under, already resolved.
///
/// `{root}` is matched before the pattern rather than by it, because a root
/// may span several components (`src/my_package`) while every other segment
/// is exactly one.
#[derive(Debug, Clone, Copy)]
pub struct Root<'a> {
    pub name: &'a str,
    pub depth: usize,
}

#[derive(Debug, Clone)]
pub struct Matched {
    /// Index into the pattern list, for reporting which variant applied.
    pub pattern: usize,
    /// Segment name -> the path component that bound to it.
    pub bindings: BTreeMap<String, String>,
    /// Position of the `{kind}` component in the *full* path, when the
    /// matched variant has one.
    pub kind_index: Option<usize>,
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
    root: Root<'_>,
    dirs: &[&str],
) -> MatchOutcome {
    // Everything below the root; the root itself is already accounted for.
    let below = &dirs[root.depth.min(dirs.len())..];
    let mut best: Option<(usize, Vec<String>)> = None;

    for (index, pattern) in patterns.iter().enumerate() {
        // The `{root}` segment is the first, and it is already matched.
        let segments = &pattern.dir_segments()[1..];
        if segments.len() != below.len() {
            continue;
        }
        match match_one(segments, profile, below) {
            Ok(mut bindings) => {
                bindings.insert(ROOT.to_string(), root.name.to_string());
                let kind_index = pattern
                    .segments
                    .iter()
                    .position(|segment| segment == KIND)
                    .map(|at| root.depth + at - 1);
                return MatchOutcome::Matched(Matched {
                    pattern: index,
                    bindings,
                    kind_index,
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
            "no pattern places files {} folder{} below `{}/`",
            below.len(),
            if below.len() == 1 { "" } else { "s" },
            root.name
        )],
    };
    MatchOutcome::NoMatch { notes }
}

/// `Ok(bindings)`, or `Err((index of the failing segment, explanation))`.
fn match_one(
    segments: &[String],
    profile: &Profile,
    dirs: &[&str],
) -> Result<BTreeMap<String, String>, (usize, Vec<String>)> {
    let mut bindings = BTreeMap::new();

    for (index, segment) in segments.iter().enumerate() {
        let component = dirs[index];

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
