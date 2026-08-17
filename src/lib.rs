//! `app-organizer` — an opinionated, multi-language validator for folder
//! conventions, file naming conventions, and file content conventions.
//!
//! One substantial export per file, a filename that names it, a line budget on
//! the files carrying the logic, and a cap on folder nesting. Linters check the
//! code *inside* files and say nothing about where files live; this checks the
//! other half.
//!
//! The library exists on day one so the future pip/npm wrappers have something
//! to bind to; `main.rs` is a thin CLI over it.

// Modules are named after the single thing they export, which puts `Config` in
// `config/config.rs` and `walk` in `walk/walk.rs`. Clippy reads that as an
// accident; here it is the convention this crate exists to enforce.
#![allow(clippy::module_inception)]

pub mod config;
pub mod diagnostics;
pub mod engine;
pub mod lang;
pub mod rules;
pub mod walk;

pub use engine::{Compiled, Engine, Error, Report, check};
