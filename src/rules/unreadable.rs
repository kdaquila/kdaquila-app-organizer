//! A file whose bytes the tool could not read at all.

use super::Rule;
use crate::diagnostics::{Diagnostic, Tag};
use crate::walk;
use std::path::Path;

/// Counting a file as checked and then saying nothing about it is the one
/// outcome that would be a lie.
pub fn unreadable(rel: &Path, reason: &str) -> Diagnostic {
    Diagnostic::new(
        Tag::Content,
        Rule::FileIsReadable,
        walk::display(rel),
        "could not read this file as UTF-8",
    )
    .note(reason.to_string())
    .note("its contents are not checked")
}
