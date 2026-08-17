//! What a language profile makes of one file.

use super::PublicName;

#[derive(Debug, Clone, Default)]
pub struct Module {
    /// Exported names in source order, deduped by name.
    pub names: Vec<PublicName>,
    /// The parser hit syntax it could not make sense of, so `names` may be
    /// incomplete. Reporting *that* beats reporting its consequences.
    pub has_syntax_errors: bool,
    /// Lines carrying something other than whitespace and comments.
    pub code_lines: usize,
}
