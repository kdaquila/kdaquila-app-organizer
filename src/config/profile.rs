//! Everything one language declares about its own shape.

use super::{Casing, Exception};
use serde::{Deserialize, Serialize};

/// Five values. v1's `kinds`, `patterns` and `segments` are gone with the kind
/// folders they existed to place.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    /// The constructs a file may declare at most one of — the *substantial*
    /// exports. Spelled as this language's own keywords, which the engine
    /// treats as opaque strings.
    pub one_per_file: Vec<String>,
    /// The line budget, in non-blank non-comment lines. Applies only to files
    /// that have a governed export.
    pub max_file_lines: usize,
    /// How deep folders may nest below a root.
    pub max_folder_depth: usize,
    /// The casing every folder and file name under this language obeys.
    pub name_case: Casing,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exceptions: Vec<Exception>,
}

impl Profile {
    /// Whether a construct is one this language holds to one per file.
    pub fn governs(&self, construct: &str) -> bool {
        self.one_per_file.iter().any(|c| c == construct)
    }
}

impl Default for Profile {
    /// The baseline a language starts from before its own profile speaks.
    ///
    /// `one_per_file` is empty because the governed constructs are the one
    /// thing no language can inherit from another. The other three are the
    /// tool's cross-language position and are the same everywhere they ship.
    fn default() -> Profile {
        Profile {
            one_per_file: Vec::new(),
            max_file_lines: 200,
            max_folder_depth: 3,
            name_case: Casing::SnakeCase,
            exceptions: Vec::new(),
        }
    }
}
