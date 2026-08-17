//! Rust's answer to the seam question, via tree-sitter.
//!
//! Exported means "carries a visibility modifier" — `pub`, `pub(crate)`,
//! `pub(super)`, or `pub(in path)`. A bare item is private, which is the exact
//! analogue of Python's `_helper`, except that here the compiler enforces it.
//!
//! `pub(crate)` counts. The rule is about one substantial thing per file, and
//! `pub(crate)` *is* the module's surface to the rest of the crate; excluding
//! it would gut the rule for exactly the kind of lib+bin crate this one is,
//! where almost nothing is bare `pub`.

use super::{LanguageProfile, Module, PublicName, code_lines};
use crate::config::Language;
use std::ops::Range;
use tree_sitter::Node;

pub struct Rust;

impl LanguageProfile for Rust {
    fn language(&self) -> Language {
        Language::Rust
    }

    fn read(&self, source: &str) -> Module {
        let mut parser = tree_sitter::Parser::new();
        if parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .is_err()
        {
            return Module::default();
        }
        let Some(tree) = parser.parse(source, None) else {
            return Module {
                has_syntax_errors: true,
                ..Module::default()
            };
        };

        let mut names = Vec::new();
        let mut not_shipped = Vec::new();
        let root = tree.root_node();
        let mut cursor = root.walk();
        for item in root.children(&mut cursor) {
            if let Some(range) = cfg_test_range(item, source) {
                not_shipped.push(range);
            }
            let Some(construct) = construct(item.kind()) else {
                continue;
            };
            if !is_exported(item, source) {
                continue;
            }
            let Some(name) = item.child_by_field_name("name") else {
                continue;
            };
            names.push(PublicName {
                name: text(name, source).to_string(),
                construct,
                line: item.start_position().row + 1,
            });
        }

        Module {
            names,
            has_syntax_errors: root.has_error(),
            code_lines: code_lines(source, &tree, &not_shipped),
        }
    }
}

/// The span of a `#[cfg(test)] mod tests { … }` block, attribute included.
///
/// Rust is the only language here with no convention for putting unit tests in
/// another file, and the compiler strips these from what the file ships. The
/// budget measures shipped code, so taxing a crate for testing the idiomatic
/// way would be measuring the wrong thing.
fn cfg_test_range(item: Node, source: &str) -> Option<Range<usize>> {
    if item.kind() != "mod_item" {
        return None;
    }
    let attribute = item.prev_sibling()?;
    if text(attribute, source).replace(' ', "") != "#[cfg(test)]" {
        return None;
    }
    Some(attribute.start_byte()..item.end_byte())
}

/// The keyword an item node stands for, or `None` for nodes that declare no
/// module-level name of their own.
///
/// `mod`, `use`, `impl`, `extern crate` and attributes are all skipped: a `mod`
/// names a file rather than a thing, and the rest either re-export or attach to
/// something already counted. Counting `impl` blocks would make every type's
/// own file illegal.
fn construct(kind: &str) -> Option<&'static str> {
    Some(match kind {
        "function_item" => "fn",
        "struct_item" => "struct",
        "enum_item" => "enum",
        "trait_item" => "trait",
        "union_item" => "union",
        "type_item" => "type",
        "const_item" => "const",
        "static_item" => "static",
        "macro_definition" => "macro_rules",
        _ => return None,
    })
}

fn is_exported(item: Node, source: &str) -> bool {
    // `macro_rules!` takes no visibility modifier; it leaves the crate only
    // via `#[macro_export]`, which the grammar hangs as a *preceding sibling*
    // rather than a child — the one place Rust's tree does not nest what reads
    // like it should.
    if item.kind() == "macro_definition" {
        return has_macro_export(item, source);
    }
    let mut cursor = item.walk();
    item.children(&mut cursor)
        .any(|child| child.kind() == "visibility_modifier")
}

fn has_macro_export(item: Node, source: &str) -> bool {
    let mut previous = item.prev_sibling();
    while let Some(node) = previous {
        match node.kind() {
            "attribute_item" if text(node, source).contains("macro_export") => return true,
            // Doc comments and other attributes may sit between the two.
            "attribute_item" => {}
            kind if kind.contains("comment") => {}
            _ => return false,
        }
        previous = node.prev_sibling();
    }
    false
}

fn text<'a>(node: Node, source: &'a str) -> &'a str {
    source.get(node.byte_range()).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(source: &str) -> Module {
        Rust.read(source)
    }

    fn exports(source: &str) -> Vec<(String, &'static str)> {
        read(source)
            .names
            .into_iter()
            .map(|name| (name.name, name.construct))
            .collect()
    }

    #[test]
    fn every_visibility_modifier_exports() {
        let found = exports(
            "pub fn a() {}\n\
             pub(crate) struct B;\n\
             pub(super) enum C { X }\n\
             pub(in crate::d) trait D {}\n",
        );
        assert_eq!(
            found,
            [
                ("a".to_string(), "fn"),
                ("B".to_string(), "struct"),
                ("C".to_string(), "enum"),
                ("D".to_string(), "trait"),
            ]
        );
    }

    #[test]
    fn bare_items_are_private() {
        assert!(exports("fn a() {}\nstruct B;\nconst C: u8 = 1;\n").is_empty());
    }

    #[test]
    fn linkage_items_export_nothing_of_their_own() {
        // The whole reason `auth.rs` beside `auth/`, `mod.rs` and `lib.rs` need
        // no special case: they declare and re-export, and neither counts.
        assert!(exports("pub mod auth;\npub use auth::login;\nimpl Auth {}\n").is_empty());
    }

    #[test]
    fn macros_export_only_with_the_attribute() {
        assert!(exports("macro_rules! quiet { () => {} }").is_empty());
        assert_eq!(
            exports("#[macro_export]\nmacro_rules! shout { () => {} }"),
            [("shout".to_string(), "macro_rules")]
        );
        // The attribute may sit behind a doc comment and other attributes.
        assert_eq!(
            exports(
                "#[macro_export]\n\
                 #[allow(unused)]\n\
                 /// doc\n\
                 macro_rules! shout { () => {} }"
            ),
            [("shout".to_string(), "macro_rules")]
        );
    }

    #[test]
    fn cfg_test_modules_do_not_count_toward_the_budget() {
        let shipped = "pub fn a() {\n    let x = 1;\n}\n";
        let tested =
            format!("{shipped}\n#[cfg(test)]\nmod tests {{\n    #[test]\n    fn t() {{}}\n}}\n");
        assert_eq!(read(shipped).code_lines, 3);
        assert_eq!(read(&tested).code_lines, 3);
    }

    #[test]
    fn comments_and_blank_lines_do_not_count() {
        let source = "// leading\n\
                      pub fn a() {\n\
                      \n\
                      /* block\n\
                         comment */\n\
                          let x = 1; // trailing\n\
                      }\n";
        assert_eq!(read(source).code_lines, 3);
    }
}
