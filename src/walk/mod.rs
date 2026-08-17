//! Traversal.
//!
//! Uses the `ignore` crate — the one ripgrep uses — so `.gitignore` is
//! respected. "Is this file part of the project" is exactly what git already
//! knows, and matching ripgrep's semantics means users already know how the
//! walker behaves.

pub mod components;
pub mod display;
pub mod tree;
pub mod walk;

pub use components::components;
pub use display::display;
pub use tree::Tree;
pub use walk::walk;
