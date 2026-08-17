//! The shape of an `app-organizer.toml` on disk.

use super::{Language, ProfileOverride};
use serde::Deserialize;
use std::collections::BTreeMap;

/// Every field is optional — the defaults supply the rest.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFile {
    pub roots: Option<BTreeMap<String, Language>>,
    pub python: Option<ProfileOverride>,
    pub typescript: Option<ProfileOverride>,
    pub rust: Option<ProfileOverride>,
    pub cpp: Option<ProfileOverride>,
}

impl ConfigFile {
    pub fn profile_override(&self, language: Language) -> Option<&ProfileOverride> {
        match language {
            Language::Python => self.python.as_ref(),
            Language::Typescript => self.typescript.as_ref(),
            Language::Rust => self.rust.as_ref(),
            Language::Cpp => self.cpp.as_ref(),
        }
    }
}
