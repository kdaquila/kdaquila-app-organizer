//! The walk itself.

use super::Tree;
use std::path::Path;

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
        if entry.file_type().is_some_and(|t| t.is_dir()) {
            tree.dirs.push(rel.to_path_buf());
        } else {
            tree.files.push(rel.to_path_buf());
        }
    }

    tree.files.sort();
    tree.dirs.sort();
    tree
}
