//! One language's configuration, with everything precomputed that can be.

use crate::config::{Language, Profile};
use crate::lang::LanguageProfile;
use crate::rules::Exceptions;

pub struct Compiled {
    pub language: Language,
    pub profile: Profile,
    /// The roots declared for this language, in the `[roots]` map's order.
    pub roots: Vec<String>,
    pub exceptions: Exceptions,
    /// The content-layer profile, absent for languages that cannot be parsed yet.
    pub content: Option<Box<dyn LanguageProfile>>,
}
