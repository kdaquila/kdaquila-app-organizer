//! A path split into the pieces the root map is matched against.

use std::path::Path;

/// The path components as `&str`, or `None` if any component is not UTF-8.
pub fn components(path: &Path) -> Option<Vec<&str>> {
    path.components()
        .map(|c| c.as_os_str().to_str())
        .collect::<Option<Vec<_>>>()
}
