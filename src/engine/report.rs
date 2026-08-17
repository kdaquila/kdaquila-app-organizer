//! What one check produced.

use crate::diagnostics::Diagnostic;

#[derive(Debug)]
pub struct Report {
    pub diagnostics: Vec<Diagnostic>,
    pub files_checked: usize,
    /// The roots in play, so a report over zero files can say why.
    pub declared_roots: Vec<String>,
}

impl Report {
    pub fn is_clean(&self) -> bool {
        self.diagnostics.is_empty()
    }
}
