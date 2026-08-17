//! Python's answer to the seam question, via tree-sitter.
//!
//! Exported means "top-level and not `_`-prefixed" — the language has no
//! visibility keyword, so the advisory convention is the only signal there is.
//! That excludes dunders (`__version__`, `__all__`) for free.

use super::{LanguageProfile, Module, PublicName, code_lines};
use crate::config::Language;
use std::collections::HashSet;
use tree_sitter::Node;

pub struct Python;

impl LanguageProfile for Python {
    fn language(&self) -> Language {
        Language::Python
    }

    fn read(&self, source: &str) -> Module {
        let mut parser = tree_sitter::Parser::new();
        if parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
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
        let root = tree.root_node();
        let mut cursor = root.walk();
        for child in root.children(&mut cursor) {
            visit(child, source, &mut names);
        }

        // Dedupe by name, keeping the first: three `@overload` stubs plus an
        // implementation are four AST nodes and one export.
        let mut seen = HashSet::new();
        names.retain(|name: &PublicName| seen.insert(name.name.clone()));

        Module {
            names,
            has_syntax_errors: root.has_error(),
            code_lines: code_lines(source, &tree, &[]),
        }
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

        "function_definition" => push(node, "name", "def", source, out),
        "class_definition" => push(node, "name", "class", source, out),
        "type_alias_statement" => push(node, "left", "type", source, out),

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

/// Module-level bindings. `TypeVar`/`NewType` used to be picked out here so
/// they could be steered into `types/`; with kind folders gone an assignment is
/// just an assignment, and none of them are governed by default anyway.
fn assignment(node: Node, source: &str, out: &mut Vec<PublicName>) {
    // `A = B = value` nests: recurse so both targets are counted.
    if let Some(right) = node.child_by_field_name("right")
        && right.kind() == "assignment"
    {
        assignment(right, source, out);
    }

    let Some(left) = node.child_by_field_name("left") else {
        return;
    };
    for target in targets(left) {
        record(text(target, source), target, "assignment", out);
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

fn push(node: Node, field: &str, construct: &'static str, source: &str, out: &mut Vec<PublicName>) {
    let Some(target) = node.child_by_field_name(field) else {
        return;
    };
    // A generic alias binds `Pair` in `type Pair[T] = tuple[T, T]`.
    let name_node = first_identifier(target).unwrap_or(target);
    record(text(name_node, source), node, construct, out);
}

fn record(name: &str, at: Node, construct: &'static str, out: &mut Vec<PublicName>) {
    if name.is_empty() || name.starts_with('_') {
        return;
    }
    out.push(PublicName {
        name: name.to_string(),
        construct,
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
