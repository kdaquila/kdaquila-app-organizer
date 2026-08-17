//! The one casing a language uses for every folder and file name it owns.
//!
//! One value per language, not a list and not a per-construct matrix. A list
//! lets `Button.tsx` and `button.tsx` coexist in one repo, which enforces
//! nothing; a matrix breaks on the first React component, which is a function
//! named in PascalCase. Picking one also sidesteps a real bug class, since
//! macOS and Windows are case-insensitive and git handles case-only renames
//! badly.

use crate::rules::to_snake_case;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Casing {
    SnakeCase,
    PascalCase,
    CamelCase,
    KebabCase,
}

impl Casing {
    pub fn as_str(self) -> &'static str {
        match self {
            Casing::SnakeCase => "snake_case",
            Casing::PascalCase => "PascalCase",
            Casing::CamelCase => "camelCase",
            Casing::KebabCase => "kebab-case",
        }
    }

    /// The name rewritten into this casing.
    ///
    /// This is how an export's name becomes a filename, so it is a *transform*
    /// and never the identity: `pub struct HTTPClient` prescribes
    /// `http_client.rs` whatever the export was called. Badly cased exports are
    /// therefore laundered rather than propagated, which is why export naming
    /// can be left to each language's own toolchain.
    ///
    /// Returns `None` for the two casings no shipping profile uses — better to
    /// check nothing than to prescribe a name from a converter nobody has
    /// exercised.
    pub fn suggest(self, name: &str) -> Option<String> {
        match self {
            Casing::SnakeCase => Some(to_snake_case(name)),
            Casing::KebabCase => Some(to_snake_case(name).replace('_', "-")),
            Casing::PascalCase | Casing::CamelCase => None,
        }
    }

    pub fn matches(self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        match self {
            Casing::SnakeCase => name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
            Casing::KebabCase => name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
            Casing::PascalCase => {
                name.chars().all(|c| c.is_ascii_alphanumeric())
                    && name.starts_with(|c: char| c.is_ascii_uppercase())
            }
            Casing::CamelCase => {
                name.chars().all(|c| c.is_ascii_alphanumeric())
                    && name.starts_with(|c: char| c.is_ascii_lowercase())
            }
        }
    }
}
