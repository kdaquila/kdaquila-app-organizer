//! Layer 3 — content, plus the half of layer 2 that consumes it.
//!
//! The key move: don't ask "what architectural category is this", ask what the
//! module's public name *denotes*. There are only three answers a program can
//! give, and each maps to exactly one kind folder.

use super::{KindSlot, Rule, Waivers, to_snake_case};
use crate::diagnostics::{Diagnostic, Tag};
use crate::lang::{Denotation, LanguageProfile, PublicName};
use std::path::Path;

pub fn check(
    language: &dyn LanguageProfile,
    source: &str,
    rel: &Path,
    kind: Option<&KindSlot>,
    waivers: &Waivers,
) -> Vec<Diagnostic> {
    let path = crate::walk::display(rel);
    let stem = rel.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
    let module = language.read(source);

    // Nothing downstream can be trusted if the parse fell over, and saying
    // "declares no public names" about a file that does not compile buries
    // the actual problem.
    if module.has_syntax_errors && module.names.is_empty() {
        return vec![
            Diagnostic::new(
                Tag::Content,
                Rule::FileIsReadable,
                &path,
                "could not parse this file",
            )
            .note("its contents are not checked; fix the syntax error first"),
        ];
    }

    let names = &module.names;
    let mut diagnostics = Vec::new();
    let single_name_failed = waivers.active(Rule::SinglePublicName) && names.len() != 1;

    if single_name_failed {
        diagnostics.push(single_public_name(names, &path, stem));
    }

    if waivers.active(Rule::FilenameMatchesPublicName)
        && let [only] = names.as_slice()
    {
        let expected = to_snake_case(&only.name);
        // Compared in snake_case on both sides, so a badly *cased* filename is
        // the casing rule's business alone -- one fix, one diagnostic.
        if expected != to_snake_case(stem) {
            diagnostics.push(
                Diagnostic::new(
                    Tag::Naming,
                    Rule::FilenameMatchesPublicName,
                    &path,
                    format!("file name does not match public name `{}`", only.name),
                )
                .at_line(only.line)
                .help(format!("rename to {expected}{}", extension(rel))),
            );
        }
    }

    // A file that still has to be split cannot be told where to move: the
    // answer depends on which names survive the split.
    if waivers.active(Rule::KindMatchesDeclaration)
        && !single_name_failed
        && let Some(slot) = kind
    {
        diagnostics.extend(kind_mismatches(names, rel, &path, slot));
    }

    diagnostics
}

/// One diagnostic per *wrong kind*, not per name — a `constants/` file holding
/// three stray functions has one problem, not three.
fn kind_mismatches(
    names: &[PublicName],
    rel: &Path,
    path: &str,
    slot: &KindSlot,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for denotes in [Denotation::Callable, Denotation::Type, Denotation::Value] {
        if denotes.kind() == slot.name {
            continue;
        }
        let strays: Vec<&PublicName> = names.iter().filter(|n| n.denotes == denotes).collect();
        let Some(first) = strays.first() else {
            continue;
        };

        let mut diagnostic = if let [only] = strays.as_slice()
            && names.len() == 1
        {
            // The whole file is in the wrong place, so the tool knows the
            // exact target path. There is no `--fix`, and precision is worth
            // a lot to whoever does the move.
            Diagnostic::new(
                Tag::Kind,
                Rule::KindMatchesDeclaration,
                path,
                format!(
                    "`{}` denotes {} but lives in {}/",
                    only.name,
                    denotes.describe(),
                    slot.name
                ),
            )
            .help(format!(
                "move to {}",
                replace_component(rel, slot.index, denotes.kind())
            ))
        } else {
            // The file holds a legitimate mix, so only the strays move.
            Diagnostic::new(
                Tag::Kind,
                Rule::KindMatchesDeclaration,
                path,
                format!(
                    "{} public name{} denote{} {} but live in {}/",
                    strays.len(),
                    if strays.len() == 1 { "" } else { "s" },
                    if strays.len() == 1 { "s" } else { "" },
                    denotes.describe_plural(),
                    slot.name
                ),
            )
            .note(format!(
                "{}: {}",
                denotes.describe_plural(),
                strays
                    .iter()
                    .map(|n| n.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
            .help(format!("move them into {}/", denotes.kind()))
        };

        diagnostic = diagnostic.at_line(first.line);

        // A value in `types/` is usually a type alias someone spelled the old
        // way, so offer the other fix rather than only the move.
        if denotes == Denotation::Value && slot.name == Denotation::Type.kind() {
            for stray in &strays {
                if let Some(hint) = &stray.type_alias_hint {
                    diagnostic = diagnostic.help(format!("or write it as an alias: {hint}"));
                }
            }
        }
        diagnostics.push(diagnostic);
    }

    diagnostics
}

fn single_public_name(names: &[PublicName], path: &str, stem: &str) -> Diagnostic {
    if names.is_empty() {
        return Diagnostic::new(
            Tag::Content,
            Rule::SinglePublicName,
            path,
            "file declares no public names, expected 1",
        )
        .note("every module names one public thing, and the filename is that name");
    }

    // If one name already matches the filename, the others are the strays.
    let extras: Vec<&PublicName> = match names.iter().find(|n| to_snake_case(&n.name) == stem) {
        Some(anchor) => names.iter().filter(|n| n.name != anchor.name).collect(),
        None => names.iter().skip(1).collect(),
    };

    let advice = match extras.as_slice() {
        [only] => format!(
            "rename `{0}` to `_{0}`, or move it to its own file",
            only.name
        ),
        _ => "prefix the extras with `_`, or move them to their own files".to_string(),
    };

    let diagnostic = Diagnostic::new(
        Tag::Content,
        Rule::SinglePublicName,
        path,
        format!("file declares {} public names, expected 1", names.len()),
    )
    .note(format!(
        "public names: {}",
        names
            .iter()
            .map(|n| n.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    ))
    .note(advice);

    match extras.first() {
        Some(first) => diagnostic.at_line(first.line),
        None => diagnostic,
    }
}

/// A file whose bytes the tool could not read at all.
pub fn unreadable(rel: &Path, reason: &str) -> Diagnostic {
    Diagnostic::new(
        Tag::Content,
        Rule::FileIsReadable,
        crate::walk::display(rel),
        "could not read this file as UTF-8",
    )
    .note(reason.to_string())
    .note("its contents are not checked")
}

fn extension(rel: &Path) -> String {
    rel.extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{e}"))
        .unwrap_or_default()
}

/// Swap one component of a path, for "move to …" help text.
fn replace_component(rel: &Path, index: usize, replacement: &str) -> String {
    let mut components: Vec<String> = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    if let Some(component) = components.get_mut(index) {
        *component = replacement.to_string();
    }
    components.join("/")
}

/// Whether a set of diagnostics already prescribes a filename, which makes a
/// separate casing complaint redundant and possibly contradictory.
pub fn prescribes_a_filename(diagnostics: &[Diagnostic]) -> bool {
    diagnostics
        .iter()
        .any(|d| d.rule == Rule::FilenameMatchesPublicName)
}
