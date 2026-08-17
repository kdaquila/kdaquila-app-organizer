//! The line count the budget is measured against.

use std::ops::Range;
use tree_sitter::{Node, Tree};

/// Lines carrying something other than whitespace and comments.
///
/// Counted off the parse tree the profile already built, so a `#` inside a
/// string literal is code, a trailing `// note` still leaves its line counted,
/// and a block comment costs nothing however it is wrapped. A regex could not
/// get any of those three right.
///
/// `excluded` is for regions a language wants left out entirely — Rust's
/// `#[cfg(test)]` modules, which the compiler strips from what the file ships.
/// Rust is the only language here with no convention for putting unit tests
/// somewhere else, so counting them would tax it for being idiomatic.
pub fn code_lines(source: &str, tree: &Tree, excluded: &[Range<usize>]) -> usize {
    // Regions are blanked in place rather than removed, so line boundaries —
    // and therefore the count — survive the edit.
    let mut masked = source.as_bytes().to_vec();
    for range in excluded {
        blank(range.clone(), &mut masked);
    }
    blank_comments(tree.root_node(), &mut masked);
    String::from_utf8_lossy(&masked)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count()
}

/// Matching on `contains("comment")` rather than an exact list is deliberate:
/// grammars spell it `comment`, `line_comment`, `block_comment`, and
/// `outer_doc_comment_marker` depending on the language and the version, and a
/// missed spelling would silently inflate every count.
fn blank_comments(node: Node, out: &mut [u8]) {
    if node.kind().contains("comment") {
        blank(node.byte_range(), out);
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        blank_comments(child, out);
    }
}

/// Overwrite a byte range with spaces, keeping its newlines intact.
fn blank(range: Range<usize>, out: &mut [u8]) {
    let end = range.end.min(out.len());
    let Some(slice) = out.get_mut(range.start..end) else {
        return;
    };
    for byte in slice {
        if *byte != b'\n' {
            *byte = b' ';
        }
    }
}
