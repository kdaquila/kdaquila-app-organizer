//! Layer 1 — folder grammar.
//!
//! Two of these checks need no pattern matching at all. Because kind names are
//! a closed set and folder names may not collide with them, any component
//! classifies as kind-or-folder by name alone.

use super::{KindSlot, Rule, Waivers};
use crate::Compiled;
use crate::config::Language;
use crate::diagnostics::{Diagnostic, Tag};
use crate::grammar::{KIND, MatchOutcome, match_path};
use crate::walk;
use std::path::Path;

/// Does this file sit at the `{files}` position of a legal pattern?
///
/// Returns the kind folder it landed in, when a pattern matched and named one.
pub fn check_placement(
    compiled: &Compiled,
    rel: &Path,
    waivers: &Waivers,
) -> (Option<KindSlot>, Vec<Diagnostic>) {
    if !waivers.active(Rule::FileMustBeInKindFolder) {
        return (None, Vec::new());
    }

    let path = walk::display(rel);
    let parent = rel.parent().unwrap_or(Path::new(""));
    let Some(dirs) = walk::components(parent) else {
        return (None, Vec::new());
    };

    match match_path(
        &compiled.patterns,
        &compiled.profile,
        &compiled.roots,
        &dirs,
    ) {
        MatchOutcome::Matched(matched) => {
            let slot = compiled.patterns[matched.pattern]
                .segments
                .iter()
                .position(|segment| segment == KIND)
                .zip(matched.kind())
                .map(|(index, name)| KindSlot {
                    name: name.to_string(),
                    index,
                });
            (slot, Vec::new())
        }
        MatchOutcome::NoMatch { notes } => {
            // Printing the tried patterns is precisely what the flat list buys
            // over a schema: it teaches the convention at the moment someone
            // breaks it.
            let tried = compiled
                .patterns
                .iter()
                .map(|pattern| pattern.raw.clone())
                .collect::<Vec<_>>()
                .join("\n");
            let diagnostic = Diagnostic::new(
                Tag::Folder,
                Rule::FileMustBeInKindFolder,
                path,
                "no pattern matched",
            )
            .notes(notes)
            .help(format!("tried: {tried}"));
            (None, vec![diagnostic])
        }
    }
}

/// Layer 2's casing half: the filename obeys the `{files}` casing rule.
pub fn check_filename_casing(
    compiled: &Compiled,
    rel: &Path,
    waivers: &Waivers,
) -> Option<Diagnostic> {
    if !waivers.active(Rule::FilenameCasing) {
        return None;
    }
    let casing = compiled
        .profile
        .segments
        .get(crate::grammar::FILES)
        .and_then(|segment| segment.casing)?;
    let stem = rel.file_stem()?.to_str()?;
    if casing.matches(stem) {
        return None;
    }

    let name = rel.file_name()?.to_str()?;
    let extension = rel
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{e}"))
        .unwrap_or_default();
    let mut diagnostic = Diagnostic::new(
        Tag::Naming,
        Rule::FilenameCasing,
        walk::display(rel),
        format!("`{name}` is not {}", casing.as_str()),
    );
    // Only offer a rename the rule would actually accept: snake_casing
    // `foo bar` or `Foo-Bar` leaves them non-conforming, and prescribing the
    // name that was just rejected is worse than saying nothing.
    if let Some(suggestion) = casing.suggest(stem)
        && suggestion != stem
        && casing.matches(&suggestion)
    {
        diagnostic = diagnostic.help(format!("rename to {suggestion}{extension}"));
    }
    Some(diagnostic)
}

/// A tracked file whose language contradicts the declaration of its root.
pub fn check_root_language(
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

/// The two structural checks: no mixed children, and kind folders are leaves.
pub fn check_directory(
    compiled: &Compiled,
    rel: &Path,
    children: &[String],
    waivers: &Waivers,
) -> Vec<Diagnostic> {
    let path = walk::display(rel);
    let Some(name) = rel.file_name().and_then(|n| n.to_str()) else {
        return Vec::new();
    };

    if compiled.profile.is_kind(name) {
        let leaf_only = compiled
            .profile
            .segments
            .get(KIND)
            .is_some_and(|segment| segment.leaf_only);
        if leaf_only && !children.is_empty() && waivers.active(Rule::KindFolderIsLeaf) {
            return vec![
                Diagnostic::new(
                    Tag::Folder,
                    Rule::KindFolderIsLeaf,
                    path,
                    format!("kind folder `{name}` contains subdirectories"),
                )
                .note(format!("subdirectories: {}", children.join(", ")))
                .help("kind folders hold files only"),
            ];
        }
        // A kind folder's children are already wrong; homogeneity adds nothing.
        return Vec::new();
    }

    if !waivers.active(Rule::NoMixedChildren) {
        return Vec::new();
    }
    let (kinds, folders): (Vec<&String>, Vec<&String>) = children
        .iter()
        .partition(|child| compiled.profile.is_kind(child));
    if kinds.is_empty() || folders.is_empty() {
        return Vec::new();
    }

    vec![
        Diagnostic::new(
            Tag::Folder,
            Rule::NoMixedChildren,
            path,
            format!("`{name}` mixes kind folders with ordinary folders"),
        )
        .note(format!("kind folders: {}", join(&kinds)))
        .note(format!("other folders: {}", join(&folders)))
        .help("a directory's children must be all kinds or all folders")
        .help("move the kind folders down into a sub-slice of their own, or flatten the folders back into this one"),
    ]
}

fn join(names: &[&String]) -> String {
    names
        .iter()
        .map(|name| name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}
