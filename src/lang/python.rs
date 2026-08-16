//! Python's answers to the two seam questions, via tree-sitter.
//!
//! Targets Python 3.12+, which is what makes the type layer mechanically
//! checkable: PEP 695 gives aliases a keyword (`type X = int`) and inlines
//! `TypeVar`/`ParamSpec` out of module scope entirely. The stragglers that
//! stay assignments are all trivially detectable calls.

use super::{Denotation, LanguageProfile, Module, PublicName};
use crate::config::Language;
use std::collections::HashSet;
use tree_sitter::Node;

/// Calls that introduce a type even though they are spelled as assignments.
///
/// `NewType` is nominal rather than interchangeable, so it is not an alias and
/// cannot use the `type` keyword. The type-parameter constructors are legacy
/// under PEP 695 but still common, and they are certainly not *values* — the
/// worst outcome here would be advising someone to move a `TypeVar` into
/// `constants/`.
const TYPE_CONSTRUCTORS: [&str; 4] = ["NewType", "TypeVar", "ParamSpec", "TypeVarTuple"];

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
                names: Vec::new(),
                has_syntax_errors: true,
            };
        };

        let mut names = Vec::new();
        let root = tree.root_node();
        let mut cursor = root.walk();
        for child in root.children(&mut cursor) {
            visit(child, source, &mut names);
        }

        // Dedupe by name, keeping the first: three `@overload` stubs plus an
        // implementation are four AST nodes and one public name.
        let mut seen = HashSet::new();
        names.retain(|name: &PublicName| seen.insert(name.name.clone()));

        Module {
            names,
            has_syntax_errors: root.has_error(),
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

/// A module-level binding denotes a value, unless its right-hand side is one
/// of the calls that genuinely introduce a type.
fn assignment(node: Node, source: &str, out: &mut Vec<PublicName>) {
    let right = node.child_by_field_name("right");

    // `A = B = value` nests: recurse so both targets are counted.
    if let Some(right) = right
        && right.kind() == "assignment"
    {
        assignment(right, source, out);
    }

    let is_call = right.is_some_and(|right| right.kind() == "call");
    let denotes = match right {
        Some(right) if is_type_constructor(right, source) => Denotation::Type,
        _ => Denotation::Value,
    };

    let Some(left) = node.child_by_field_name("left") else {
        return;
    };
    for target in targets(left) {
        let name = text(target, source);
        // `X = int` may well have been meant as an alias; `X = compute()`
        // certainly was not, so only the former earns the suggestion.
        let hint = (!is_call).then(|| format!("type {name} = ..."));
        record(name, target, denotes, hint, out);
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

fn is_type_constructor(node: Node, source: &str) -> bool {
    if node.kind() != "call" {
        return false;
    }
    let Some(function) = node.child_by_field_name("function") else {
        return false;
    };
    text(function, source)
        .rsplit('.')
        .next()
        .is_some_and(|name| TYPE_CONSTRUCTORS.contains(&name))
}

fn push(node: Node, field: &str, denotes: Denotation, source: &str, out: &mut Vec<PublicName>) {
    let Some(target) = node.child_by_field_name(field) else {
        return;
    };
    // A generic alias binds `Pair` in `type Pair[T] = tuple[T, T]`.
    let name_node = first_identifier(target).unwrap_or(target);
    record(text(name_node, source), node, denotes, None, out);
}

fn record(
    name: &str,
    at: Node,
    denotes: Denotation,
    type_alias_hint: Option<String>,
    out: &mut Vec<PublicName>,
) {
    // The leading-underscore convention is the single source of truth, and it
    // excludes dunders (`__version__`, `__all__`) for free.
    if name.is_empty() || name.starts_with('_') {
        return;
    }
    out.push(PublicName {
        name: name.to_string(),
        denotes,
        line: at.start_position().row + 1,
        type_alias_hint,
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
