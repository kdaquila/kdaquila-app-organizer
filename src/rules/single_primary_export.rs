//! At most one export built from a governed construct.
//!
//! *At most*, not exactly one. Zero governed exports is a legal, deliberate
//! shape — a constants table, a module of type aliases, `mod.rs`, `__init__.py`
//! — and making it legal is what retires most of v1's exception list.

use super::{Rule, Waivers};
use crate::diagnostics::{Diagnostic, Tag};
use crate::lang::PublicName;

pub fn single_primary_export(
    governed: &[&PublicName],
    path: &str,
    waivers: &Waivers,
) -> Option<Diagnostic> {
    if governed.len() < 2 || !waivers.active(Rule::SinglePrimaryExport) {
        return None;
    }

    let listed = governed
        .iter()
        .map(|name| format!("{} {}", name.construct, name.name))
        .collect::<Vec<_>>()
        .join(", ");

    Some(
        Diagnostic::new(
            Tag::Content,
            Rule::SinglePrimaryExport,
            path,
            format!(
                "file exports {} substantial things, expected 1",
                governed.len()
            ),
        )
        .note(format!("exports: {listed}"))
        .note("move all but one to files of their own, or make them private")
        .at_line(governed[1].line),
    )
}
