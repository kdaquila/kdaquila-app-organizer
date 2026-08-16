//! Traversal.
//!
//! Uses the `ignore` crate — the one ripgrep uses — so `.gitignore` is
//! respected. "Is this file part of the project" is exactly what git already
//! knows, and matching ripgrep's semantics means users already know how the
//! walker behaves.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Skipped even without a `.gitignore`. An extension filter cannot replace
/// this list: the worst offenders are full of *tracked* extensions — `.venv/`
/// holds thousands of `.py` files, `node_modules/` thousands of `.ts`.
const ALWAYS_SKIP: [&str; 6] = [
    ".git",
    ".venv",
    "venv",
    "node_modules",
    "target",
    "__pycache__",
];

#[derive(Debug, Default)]
pub struct Tree {
    /// Project-root-relative file paths, sorted.
    pub files: Vec<PathBuf>,
    /// Project-root-relative directory paths -> names of their child directories.
    pub dirs: BTreeMap<PathBuf, Vec<String>>,
}

pub fn walk(start: &Path, project_root: &Path) -> Tree {
    let mut tree = Tree::default();

    let walker = ignore::WalkBuilder::new(start)
        // Fixtures and freshly-cloned trees are not always git repos; honour
        // their ignore files anyway.
        .require_git(false)
        .filter_entry(|entry| {
            entry
                .file_name()
                .to_str()
                .is_none_or(|name| !ALWAYS_SKIP.contains(&name))
        })
        .build();

    for entry in walker.flatten() {
        let Ok(rel) = entry.path().strip_prefix(project_root) else {
            continue;
        };
        if rel.as_os_str().is_empty() {
            continue;
        }
        let is_dir = entry.file_type().is_some_and(|t| t.is_dir());
        if is_dir {
            tree.dirs.entry(rel.to_path_buf()).or_default();
            // Only record the child against a parent the walk actually
            // descended into; a parent outside the walk has an incomplete
            // child list and must not be judged on it.
            if let Some(parent) = rel.parent()
                && let Some(name) = rel.file_name().and_then(|n| n.to_str())
                && let Some(siblings) = tree.dirs.get_mut(parent)
            {
                siblings.push(name.to_string());
            }
        } else {
            tree.files.push(rel.to_path_buf());
        }
    }

    tree.files.sort();
    for children in tree.dirs.values_mut() {
        children.sort();
    }
    tree
}

/// Paths in diagnostics always read with forward slashes, on every platform.
pub fn display(path: &Path) -> String {
    path.components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

/// The path components as `&str`, or `None` if any component is not UTF-8.
pub fn components(path: &Path) -> Option<Vec<&str>> {
    path.components()
        .map(|c| c.as_os_str().to_str())
        .collect::<Option<Vec<_>>>()
}
