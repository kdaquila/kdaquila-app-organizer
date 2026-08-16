//! The `Diagnostic` type and its two renderers.
//!
//! This tool's entire user interface is its error output, so the shape is
//! designed rather than defaulted: a category tag, the offending path, notes
//! explaining what is wrong, and help lines that name the fix.

use crate::rules::Rule;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Tag {
    /// Layer 1 — where the file or directory sits.
    Folder,
    /// Layer 2 — what the file is called.
    Naming,
    /// Layer 3 — how many public names the module has.
    Content,
    /// Layer 3 — whether the public name's declaration matches its kind folder.
    Kind,
    /// A tracked file whose language contradicts its root's declaration.
    Root,
}

impl Tag {
    pub fn as_str(self) -> &'static str {
        match self {
            Tag::Folder => "folder",
            Tag::Naming => "naming",
            Tag::Content => "content",
            Tag::Kind => "kind",
            Tag::Root => "root",
        }
    }
}

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

    pub fn notes(mut self, notes: impl IntoIterator<Item = String>) -> Self {
        self.notes.extend(notes);
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

/// The `--format text` renderer.
pub fn render_text(diagnostics: &[Diagnostic], files_checked: usize) -> String {
    let mut out = String::new();
    for diagnostic in diagnostics {
        out.push_str(&render_one(diagnostic));
        out.push('\n');
    }
    out.push_str(&summary(diagnostics.len(), files_checked));
    out.push('\n');
    out
}

fn summary(errors: usize, files_checked: usize) -> String {
    let files = plural(files_checked, "file", "files");
    if errors == 0 {
        format!("checked {files} -- no violations")
    } else {
        format!(
            "checked {files} -- {}",
            plural(errors, "violation", "violations")
        )
    }
}

fn plural(count: usize, one: &str, many: &str) -> String {
    format!("{count} {}", if count == 1 { one } else { many })
}

fn render_one(diagnostic: &Diagnostic) -> String {
    let mut out = format!(
        "error[{}]: {}\n",
        diagnostic.tag.as_str(),
        diagnostic.message
    );
    match diagnostic.line {
        Some(line) => out.push_str(&format!("  --> {}:{}\n", diagnostic.path, line)),
        None => out.push_str(&format!("  --> {}\n", diagnostic.path)),
    }
    if !diagnostic.notes.is_empty() {
        out.push_str("   |\n");
        for note in &diagnostic.notes {
            out.push_str(&format!("   | {note}\n"));
        }
    }
    if !diagnostic.help.is_empty() {
        out.push_str("   |\n");
        for help in &diagnostic.help {
            out.push_str(&render_help(help));
        }
    }
    out
}

/// Help text may be a labelled list (`tried: a\nb\nc`); continuation lines are
/// indented to line up under the first item.
fn render_help(help: &str) -> String {
    let mut lines = help.lines();
    let Some(first) = lines.next() else {
        return String::new();
    };
    let indent = match first.find(": ") {
        Some(at) => " ".repeat("   = ".len() + at + 2),
        None => " ".repeat("   = ".len()),
    };
    let mut out = format!("   = {first}\n");
    for line in lines {
        out.push_str(&format!("{indent}{line}\n"));
    }
    out
}

/// The `--format json` renderer: the same values, for wrapper packages.
pub fn render_json(diagnostics: &[Diagnostic], files_checked: usize) -> String {
    let report = serde_json::json!({
        "files_checked": files_checked,
        "error_count": diagnostics.len(),
        "diagnostics": diagnostics,
    });
    serde_json::to_string_pretty(&report).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"))
}
