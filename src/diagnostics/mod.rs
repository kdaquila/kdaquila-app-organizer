//! The `Diagnostic` type and its two renderers.
//!
//! This tool's entire user interface is its error output, so the shape is
//! designed rather than defaulted: a category tag, the offending path, notes
//! explaining what is wrong, and help lines that name the fix.

pub mod diagnostic;
pub mod render_json;
pub mod render_text;
pub mod tag;

pub use diagnostic::Diagnostic;
pub use render_json::render_json;
pub use render_text::render_text;
pub use tag::Tag;
