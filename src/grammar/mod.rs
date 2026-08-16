//! Path patterns: parsing the `"{root}/{folder1}/{kind}/{files}"` strings into
//! positional segment lists, and matching paths against the variant list.

pub mod matcher;
pub mod pattern;

pub use matcher::{MatchOutcome, Matched, match_path};
pub use pattern::{ParseError, Pattern};

/// The segment name that binds the file itself. Always last.
pub const FILES: &str = "files";
/// The segment name that binds a declared root. Always first.
pub const ROOT: &str = "root";
/// The segment name whose value is a kind folder, when a pattern has one.
pub const KIND: &str = "kind";
