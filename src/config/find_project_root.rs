//! Where a check considers itself to be.

use super::CONFIG_FILE;
use std::path::{Path, PathBuf};

/// Walk up from `start` looking for the file that marks the project root.
/// Absent one, `start` itself is the root.
pub fn find_project_root(start: &Path) -> PathBuf {
    let start = if start.is_dir() {
        start.to_path_buf()
    } else {
        start.parent().unwrap_or(start).to_path_buf()
    };
    for dir in start.ancestors() {
        if dir.join(CONFIG_FILE).is_file() {
            return dir.to_path_buf();
        }
    }
    start
}
