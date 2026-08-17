//! The one-call entry point the library exists to offer.

use super::{Engine, Error, Report};
use crate::config::{self, Config};
use std::path::Path;

/// Check a path, discovering the project root and config the way the CLI does.
pub fn check(start: &Path) -> Result<Report, Error> {
    let project_root = config::find_project_root(start);
    let config = Config::load(&project_root)?;
    let engine = Engine::new(config)?;
    Ok(engine.check(start, &project_root))
}
