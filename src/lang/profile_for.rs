//! Which languages can be read, as opposed to merely recognised.

use super::LanguageProfile;
use super::python::Python;
use super::rust::Rust;
use crate::config::Language;

/// The content-layer profile for a language, if one has shipped yet.
///
/// `None` means the content layer simply does not run — folder depth and name
/// casing still do, so a language can be organised before it can be parsed.
pub fn profile_for(language: Language) -> Option<Box<dyn LanguageProfile>> {
    match language {
        Language::Python => Some(Box::new(Python)),
        Language::Rust => Some(Box::new(Rust)),
        _ => None,
    }
}
