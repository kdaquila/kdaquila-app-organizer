//! A file the language's parser could not make sense of.

use super::Rule;
use crate::diagnostics::{Diagnostic, Tag};

/// Nothing downstream can be trusted once the parse falls over, and saying
/// "exports nothing" about a file that does not compile buries the real
/// problem.
pub fn unparsable(path: &str) -> Diagnostic {
    Diagnostic::new(
        Tag::Content,
        Rule::FileIsReadable,
        path,
        "could not parse this file",
    )
    .note("its contents are not checked; fix the syntax error first")
}
