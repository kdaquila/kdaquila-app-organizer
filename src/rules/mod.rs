//! The rule vocabulary, and the dependency graph that makes rules deactivate
//! when whatever they depend on has been waived.

pub mod content;
pub mod exceptions;
pub mod folder;

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Every rule the tool can report, and every name an exception may waive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rule {
    /// Layer 1: a file may only sit at the `{files}` position of a legal pattern.
    FileMustBeInKindFolder,
    /// Layer 1: a directory's child directories are all kinds or all folders.
    NoMixedChildren,
    /// Layer 1: a kind directory contains files only.
    KindFolderIsLeaf,
    /// A tracked file's language matches the language its root is declared as.
    RootLanguageMatch,
    /// Layer 2: the filename obeys the casing declared for `{files}`.
    FilenameCasing,
    /// Layer 2: the filename is the snake_case of the module's public name.
    FilenameMatchesPublicName,
    /// Layer 3: exactly one top-level binding without a leading underscore.
    SinglePublicName,
    /// Layer 3: what the public name denotes matches its kind folder.
    KindMatchesDeclaration,
}

impl Rule {
    pub fn as_str(self) -> &'static str {
        match self {
            Rule::FileMustBeInKindFolder => "file_must_be_in_kind_folder",
            Rule::NoMixedChildren => "no_mixed_children",
            Rule::KindFolderIsLeaf => "kind_folder_is_leaf",
            Rule::RootLanguageMatch => "root_language_match",
            Rule::FilenameCasing => "filename_casing",
            Rule::FilenameMatchesPublicName => "filename_matches_public_name",
            Rule::SinglePublicName => "single_public_name",
            Rule::KindMatchesDeclaration => "kind_matches_declaration",
        }
    }

    /// The rule this one has nothing left to check without.
    ///
    /// This is the whole deactivation cascade, expressed once as a property of
    /// the rule graph rather than as special cases at each call site.
    pub fn depends_on(self) -> Option<Rule> {
        match self {
            // No single public name to derive a filename from.
            Rule::FilenameMatchesPublicName => Some(Rule::SinglePublicName),
            // No kind folder to compare a declaration against.
            Rule::KindMatchesDeclaration => Some(Rule::FileMustBeInKindFolder),
            _ => None,
        }
    }
}

/// Which path component bound to the `{kind}` segment, and what it was called.
/// Knowing the position is what lets a kind mismatch name the exact target path.
#[derive(Debug, Clone)]
pub struct KindSlot {
    pub name: String,
    pub index: usize,
}

/// The snake_case spelling of a name — `Credentials` becomes `credentials`,
/// `HTTPClient` becomes `http_client`.
pub fn to_snake_case(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    let mut out = String::with_capacity(name.len() + 4);
    for (index, &current) in chars.iter().enumerate() {
        if current.is_uppercase() {
            let previous = if index == 0 {
                None
            } else {
                Some(chars[index - 1])
            };
            let next = chars.get(index + 1).copied();
            let boundary = match previous {
                None | Some('_') => false,
                // `userId` -> `user_id`, and `HTTPClient` -> `http_client`.
                Some(previous) => {
                    previous.is_lowercase()
                        || previous.is_numeric()
                        || next.is_some_and(char::is_lowercase)
                }
            };
            if boundary {
                out.push('_');
            }
            out.extend(current.to_lowercase());
        } else {
            out.push(current);
        }
    }
    out
}

/// The rules waived for one path, and the resulting active/inactive answer.
#[derive(Debug, Clone, Default)]
pub struct Waivers(pub BTreeSet<Rule>);

impl Waivers {
    /// A rule is active unless it was waived directly, or transitively lost
    /// what it depends on.
    pub fn active(&self, rule: Rule) -> bool {
        if self.0.contains(&rule) {
            return false;
        }
        match rule.depends_on() {
            Some(dep) => self.active(dep),
            None => true,
        }
    }
}
