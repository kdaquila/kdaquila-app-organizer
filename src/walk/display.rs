//! How a path is spelled in a diagnostic.

use std::path::Path;

/// Paths in diagnostics always read with forward slashes, on every platform.
pub fn display(path: &Path) -> String {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}
