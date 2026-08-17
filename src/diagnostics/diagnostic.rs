//! One thing that is wrong, and what to do about it.

use super::Tag;
use crate::rules::Rule;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    pub tag: Tag,
    #[serde(serialize_with = "serialize_rule")]
    pub rule: Rule,
    /// The headline, printed after `error[tag]: `.
    pub message: String,
    /// Project-root-relative, always with forward slashes.
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    /// Explanation lines, rendered in the `|` gutter.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    /// Fix lines, rendered after `=`. May contain newlines for aligned lists.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub help: Vec<String>,
}

fn serialize_rule<S: serde::Serializer>(rule: &Rule, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(rule.as_str())
}

impl Diagnostic {
    pub fn new(tag: Tag, rule: Rule, path: impl Into<String>, message: impl Into<String>) -> Self {
        Diagnostic {
            tag,
            rule,
            message: message.into(),
            path: path.into(),
            line: None,
            notes: Vec::new(),
            help: Vec::new(),
        }
    }

    pub fn at_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    pub fn help(mut self, help: impl Into<String>) -> Self {
        self.help.push(help.into());
        self
    }

    /// The key diagnostics are sorted by, so output order never depends on
    /// filesystem order.
    pub fn sort_key(&self) -> (&str, usize, Tag) {
        (&self.path, self.line.unwrap_or(0), self.tag)
    }
}
