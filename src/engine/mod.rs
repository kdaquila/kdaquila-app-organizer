//! Compiling a config into something that can check a tree, and running it.

pub mod check;
pub mod compiled;
pub mod engine;
pub mod error;
pub mod report;

pub use check::check;
pub use compiled::Compiled;
pub use engine::Engine;
pub use error::Error;
pub use report::Report;
