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
    let names = language.public_names(source);
    let mut diagnostics = Vec::new();

    if waivers.active(Rule::SinglePublicName) && names.len() != 1 {
        diagnostics.push(single_public_name(&names, &path, stem));
    }

    if waivers.active(Rule::FilenameMatchesPublicName)
        && let [only] = names.as_slice()
    {
        let expected = to_snake_case(&only.name);
        // Compared in snake_case on both sides, so a badly *cased* filename is
        // the casing rule's business alone -- one fix, one diagnostic.
        if expected != to_snake_case(stem) {
            let extension = rel
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| format!(".{e}"))
                .unwrap_or_default();
            diagnostics.push(
                Diagnostic::new(
                    Tag::Naming,
                    Rule::FilenameMatchesPublicName,
                    &path,
                    format!("file name does not match public name `{}`", only.name),
                )
                .at_line(only.line)
                .help(format!("rename to {expected}{extension}")),
            );
        }
    }

    if waivers.active(Rule::KindMatchesDeclaration)
        && let Some(slot) = kind
    {
        for name in &names {
            let expected = name.denotes.kind();
            if expected == slot.name {
                continue;
            }
            // The tool knows the target path exactly, so it says it. There is
            // no `--fix`, and precision is worth a lot to whoever does the move.
            let target = replace_component(rel, slot.index, expected);
            let mut diagnostic = Diagnostic::new(
                Tag::Kind,
                Rule::KindMatchesDeclaration,
                &path,
                format!(
                    "`{}` denotes {} but lives in {}/",
                    name.name,
                    name.denotes.describe(),
                    slot.name
                ),
            )
            .at_line(name.line)
            .help(format!("move to {target}"));

            // A value in `types/` is usually a type alias someone spelled the
            // old way, so offer the other fix rather than only the move.
            if name.denotes == Denotation::Value
                && slot.name == Denotation::Type.kind()
                && let Some(hint) = language.type_alias_hint(&name.name)
            {
                diagnostic = diagnostic.help(format!("or write it as an alias: {hint}"));
            }
            diagnostics.push(diagnostic);
        }
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
