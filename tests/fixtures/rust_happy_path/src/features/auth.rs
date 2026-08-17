//! `auth.rs` beside `auth/` -- Rust's own recommended module style.
//!
//! It declares submodules and re-exports; it exports no `fn`, `struct`, `enum`
//! or `trait` of its own, so nothing asks it to be named after one.

pub mod authenticate;
pub mod limits;
pub mod session;

pub use authenticate::authenticate;
pub use session::Session;
