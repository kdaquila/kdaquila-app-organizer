//! The multi-language seam.
//!
//! A language profile answers exactly two questions — *what are this module's
//! public names?* and *does each one denote a callable, a type, or a value?*
//! Everything else in the engine is language-agnostic, which is what makes
//! adding TypeScript, Rust, or C++ additive rather than invasive.

pub mod python;

use crate::config::Language;

/// What a public name denotes. There are only three answers a program gives,
/// and each maps to exactly one kind folder — in every language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Denotation {
    Callable,
    Type,
    Value,
}

impl Denotation {
    /// The kind folder this denotation belongs in.
    pub fn kind(self) -> &'static str {
        match self {
            Denotation::Callable => "functions",
            Denotation::Type => "types",
            Denotation::Value => "constants",
        }
    }

    /// Phrased for a diagnostic: "`X` denotes a type but lives in functions/".
    pub fn describe(self) -> &'static str {
        match self {
            Denotation::Callable => "a callable",
            Denotation::Type => "a type",
            Denotation::Value => "a value",
        }
    }

    /// Pluralised, for the one diagnostic that can name several at once.
    pub fn describe_plural(self) -> &'static str {
        match self {
            Denotation::Callable => "callables",
            Denotation::Type => "types",
            Denotation::Value => "values",
        }
    }
}

/// One top-level binding that forms part of a module's public surface.
#[derive(Debug, Clone)]
pub struct PublicName {
    pub name: String,
    pub denotes: Denotation,
    /// 1-based line of the declaration.
    pub line: usize,
    /// How this language would spell the same binding as a type alias, when
    /// that is a plausible thing to have meant. Only the language knows when
    /// it is: `X = int` could have been an alias, `X = compute()` could not.
    pub type_alias_hint: Option<String>,
}

/// What a language profile makes of one file.
#[derive(Debug, Clone, Default)]
pub struct Module {
    /// Public names in source order, deduped by name.
    pub names: Vec<PublicName>,
    /// The parser hit syntax it could not make sense of, so `names` may be
    /// incomplete. Reporting *that* beats reporting its consequences.
    pub has_syntax_errors: bool,
}

pub trait LanguageProfile {
    fn language(&self) -> Language;

    fn read(&self, source: &str) -> Module;
}

/// The content-layer profile for a language, if one has shipped yet.
///
/// `None` means layer 3 simply does not run — layers 1 and 2 still do, so a
/// language can be organised before it can be parsed.
pub fn profile_for(language: Language) -> Option<Box<dyn LanguageProfile>> {
    match language {
        Language::Python => Some(Box::new(python::Python)),
        _ => None,
    }
}
