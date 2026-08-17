//! What one walk found.

use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct Tree {
    /// Project-root-relative file paths, sorted.
    pub files: Vec<PathBuf>,
    /// Project-root-relative directory paths, sorted.
    ///
    /// v1 also carried each directory's child *directories*, because two rules
    /// compared siblings. Both are gone, and so is the bookkeeping that fed
    /// them.
    pub dirs: Vec<PathBuf>,
}
