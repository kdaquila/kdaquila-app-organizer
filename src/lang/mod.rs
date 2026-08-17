//! The multi-language seam.
//!
//! A language profile answers one question: *what does this module export, and
//! by which construct?* Everything else in the engine is language-agnostic,
//! which is what makes adding TypeScript or C++ additive rather than invasive.
//!
//! v1 asked a second question — whether each name denoted a callable, a type,
//! or a value — so that a kind folder could be derived from it. Dropping kind
//! folders dropped that question with them, and the seam halved.

pub mod code_lines;
pub mod language_profile;
pub mod module;
pub mod profile_for;
pub mod public_name;
pub mod python;
pub mod rust;

pub use code_lines::code_lines;
pub use language_profile::LanguageProfile;
pub use module::Module;
pub use profile_for::profile_for;
pub use public_name::PublicName;
