//! One language's section of a user's config file.

use super::{Casing, Exception, Profile};
use serde::Deserialize;

/// Every field optional: a config file says only what it changes.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileOverride {
    pub one_per_file: Option<Vec<String>>,
    pub max_file_lines: Option<usize>,
    pub max_folder_depth: Option<usize>,
    pub name_case: Option<Casing>,
    #[serde(default)]
    pub exceptions: Vec<Exception>,
}

impl ProfileOverride {
    /// Scalars replace; exceptions append.
    pub fn apply_to(&self, profile: &mut Profile) {
        if let Some(one_per_file) = &self.one_per_file {
            profile.one_per_file = one_per_file.clone();
        }
        if let Some(max_file_lines) = self.max_file_lines {
            profile.max_file_lines = max_file_lines;
        }
        if let Some(max_folder_depth) = self.max_folder_depth {
            profile.max_folder_depth = max_folder_depth;
        }
        if let Some(name_case) = self.name_case {
            profile.name_case = name_case;
        }
        // A file seeded by `app-organizer init` already contains the defaults
        // verbatim; appending them again would double every entry.
        for exception in &self.exceptions {
            if !profile.exceptions.iter().any(|kept| kept.same(exception)) {
                profile.exceptions.push(exception.clone());
            }
        }
    }
}
