//! The `--format text` renderer.

use super::Diagnostic;

pub fn render_text(diagnostics: &[Diagnostic], files_checked: usize, roots: &[String]) -> String {
    let mut out = String::new();
    for group in group(diagnostics) {
        out.push_str(&render_one(group[0], &group));
        out.push('\n');
    }
    out.push_str(&summary(diagnostics.len(), files_checked));
    out.push('\n');
    // "no violations" after looking at nothing is a lie of omission: the
    // usual cause is being pointed somewhere above or below the roots.
    if files_checked == 0 {
        out.push_str(&format!(
            "note: nothing here sits under a declared root ({}); check from the project root, or declare roots in {}\n",
            roots.join(", "),
            crate::config::CONFIG_FILE,
        ));
    }
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

/// One structural mistake usually shows up in many files at once — a nesting
/// cap blown in one place fails the same way for every folder beneath.
/// Repeating the explanation per file buries the diagnostics that really are
/// per-file, so identical ones share a block.
///
/// Only the text renderer groups; `--format json` stays one entry per file,
/// which is what a consumer iterating over files wants.
fn group(diagnostics: &[Diagnostic]) -> Vec<Vec<&Diagnostic>> {
    let mut groups: Vec<Vec<&Diagnostic>> = Vec::new();
    for diagnostic in diagnostics {
        let mergeable = |other: &Diagnostic| {
            // A line number makes a diagnostic specific to one file's contents.
            diagnostic.line.is_none()
                && other.line.is_none()
                && other.tag == diagnostic.tag
                && other.rule == diagnostic.rule
                && other.message == diagnostic.message
                && other.notes == diagnostic.notes
                && other.help == diagnostic.help
        };
        match groups.iter_mut().find(|g| mergeable(g[0])) {
            Some(existing) => existing.push(diagnostic),
            None => groups.push(vec![diagnostic]),
        }
    }
    groups
}

fn render_one(diagnostic: &Diagnostic, group: &[&Diagnostic]) -> String {
    let mut out = format!(
        "error[{}]: {}\n",
        diagnostic.tag.as_str(),
        diagnostic.message
    );
    match diagnostic.line {
        Some(line) => out.push_str(&format!("  --> {}:{}\n", diagnostic.path, line)),
        None => out.push_str(&format!("  --> {}\n", diagnostic.path)),
    }
    for also in &group[1..] {
        out.push_str(&format!("      {}\n", also.path));
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
