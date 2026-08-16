//! Python's answers to the two seam questions, via tree-sitter.
//!
//! Targets Python 3.12+, which is what makes the type layer mechanically
//! checkable: PEP 695 gives aliases a keyword (`type X = int`) and inlines
//! `TypeVar`/`ParamSpec` out of module scope entirely. `NewType` is the one
//! straggler that stays an assignment, and its call is trivially detectable.

use super::{Denotation, LanguageProfile, PublicName};
use crate::config::Language;
use std::collections::HashSet;
use tree_sitter::Node;

pub struct Python;

impl LanguageProfile for Python {
    fn language(&self) -> Language {
        Language::Python
    }

    fn public_names(&self, source: &str) -> Vec<PublicName> {
        let mut parser = tree_sitter::Parser::new();
        if parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .is_err()
        {
            return Vec::new();
        }
        let Some(tree) = parser.parse(source, None) else {
            return Vec::new();
        };

        let mut found = Vec::new();
        let root = tree.root_node();
        let mut cursor = root.walk();
        for child in root.children(&mut cursor) {
            visit(child, source, &mut found);
        }

        // Dedupe by name, keeping the first: three `@overload` stubs plus an
        // implementation are four AST nodes and one public name.
        let mut seen = HashSet::new();
        found.retain(|name: &PublicName| seen.insert(name.name.clone()));
        found
    }

    fn type_alias_hint(&self, name: &str) -> Option<String> {
        Some(format!("type {name} = ..."))
    }
}

fn visit(node: Node, source: &str, out: &mut Vec<PublicName>) {
    match node.kind() {
        // Imports bind module-level names but are not this module's API.
        // Conditional blocks (`if TYPE_CHECKING:`, `try: ... except ImportError:`)
        // are skipped wholesale — nothing inside them is unconditional surface.
        "import_statement"
        | "import_from_statement"
        | "future_import_statement"
        | "if_statement"
        | "try_statement"
        | "while_statement"
        | "for_statement"
        | "with_statement"
        | "match_statement" => {}

        "decorated_definition" => {
            if let Some(inner) = node.child_by_field_name("definition") {
                visit(inner, source, out);
            }
        }

        "function_definition" => push(node, "name", Denotation::Callable, source, out),
        "class_definition" => push(node, "name", Denotation::Type, source, out),
        "type_alias_statement" => push(node, "left", Denotation::Type, source, out),

        "expression_statement" => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "assignment" {
                    assignment(child, source, out);
                }
            }
        }

        _ => {}
    }
}

/// A module-level binding denotes a value — unless its right-hand side is a
/// `NewType(...)` call, which is nominal rather than interchangeable and so
/// genuinely introduces a type.
fn assignment(node: Node, source: &str, out: &mut Vec<PublicName>) {
    let right = node.child_by_field_name("right");

    // `A = B = value` nests: recurse so both targets are counted.
    if let Some(right) = right
        && right.kind() == "assignment"
    {
        assignment(right, source, out);
    }

    let denotes = match right {
        Some(right) if is_new_type_call(right, source) => Denotation::Type,
        _ => Denotation::Value,
    };

    let Some(left) = node.child_by_field_name("left") else {
        return;
    };
    for target in targets(left) {
        record(text(target, source), target, denotes, out);
    }
}

/// The identifiers a left-hand side binds: one, or several for `A, B = ...`.
fn targets(left: Node) -> Vec<Node> {
    match left.kind() {
        "identifier" => vec![left],
        "pattern_list" | "tuple_pattern" | "list_pattern" => {
            let mut cursor = left.walk();
            left.children(&mut cursor)
                .filter(|c| c.kind() == "identifier")
                .collect()
        }
        // Attribute and subscript targets (`obj.x = 1`) bind nothing new here.
        _ => Vec::new(),
    }
}

fn is_new_type_call(node: Node, source: &str) -> bool {
    if node.kind() != "call" {
        return false;
    }
    let Some(function) = node.child_by_field_name("function") else {
        return false;
    };
    text(function, source)
        .rsplit('.')
        .next()
        .is_some_and(|name| name == "NewType")
}

fn push(node: Node, field: &str, denotes: Denotation, source: &str, out: &mut Vec<PublicName>) {
    let Some(target) = node.child_by_field_name(field) else {
        return;
    };
    // A generic alias binds `Pair` in `type Pair[T] = tuple[T, T]`.
    let name_node = first_identifier(target).unwrap_or(target);
    record(text(name_node, source), node, denotes, out);
}

fn record(name: &str, at: Node, denotes: Denotation, out: &mut Vec<PublicName>) {
    // The leading-underscore convention is the single source of truth, and it
    // excludes dunders (`__version__`, `__all__`) for free.
    if name.is_empty() || name.starts_with('_') {
        return;
    }
    out.push(PublicName {
        name: name.to_string(),
        denotes,
        line: at.start_position().row + 1,
    });
}

fn first_identifier(node: Node) -> Option<Node> {
    if node.kind() == "identifier" {
        return Some(node);
    }
    let mut cursor = node.walk();
    let children: Vec<Node> = node.children(&mut cursor).collect();
    children.into_iter().find_map(first_identifier)
}

fn text<'a>(node: Node, source: &'a str) -> &'a str {
    source.get(node.byte_range()).unwrap_or("")
}
