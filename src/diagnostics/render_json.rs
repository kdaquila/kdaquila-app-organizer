//! The `--format json` renderer: the same values, for wrapper packages.

use super::Diagnostic;

pub fn render_json(diagnostics: &[Diagnostic], files_checked: usize) -> String {
    let report = serde_json::json!({
        "files_checked": files_checked,
        "error_count": diagnostics.len(),
        "diagnostics": diagnostics,
    });
    serde_json::to_string_pretty(&report).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"))
}
