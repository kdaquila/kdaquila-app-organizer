//! Scoped rule waivers.
//!
//! An exception is not a path allowlist — it names a glob and the rules that
//! do not apply beneath it. Globs may use `{root}`, which expands to whatever
//! the `[roots]` map declares for that profile: hardcoding `src/lib.rs` would
//! break a crate whose root is `source/`, and `**/lib.rs` would wrongly match
//! a nested file that happens to be called that.

use super::{Rule, Waivers};
use crate::config::Profile;
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};

/// Compiled exception globs, and what each one waives.
pub struct Exceptions {
    globs: GlobSet,
    /// Parallel to the globs: the rules each one waives.
    waived: Vec<Vec<Rule>>,
}

impl Exceptions {
    pub fn build(profile: &Profile, roots: &[String]) -> Result<Exceptions, String> {
        let mut builder = GlobSetBuilder::new();
        let mut waived = Vec::new();

        for exception in &profile.exceptions {
            for pattern in expand_roots(&exception.path, roots) {
                let glob = GlobBuilder::new(&pattern)
                    // `*` must not cross a separator, so `**/constants/*.py`
                    // means what it looks like.
                    .literal_separator(true)
                    .build()
                    .map_err(|e| format!("invalid exception path `{}`: {e}", exception.path))?;
                builder.add(glob);
                waived.push(exception.waive.clone());
            }
        }

        let globs = builder
            .build()
            .map_err(|e| format!("could not build exception globs: {e}"))?;
        Ok(Exceptions { globs, waived })
    }

    /// The union of rules waived for one path.
    pub fn waivers_for(&self, path: &str) -> Waivers {
        let mut waivers = Waivers::default();
        for index in self.globs.matches(path) {
            waivers.0.extend(self.waived[index].iter().copied());
        }
        waivers
    }
}

/// `{root}/app/**` becomes one glob per declared root. A pattern with no
/// placeholder is passed through untouched.
fn expand_roots(pattern: &str, roots: &[String]) -> Vec<String> {
    if !pattern.contains("{root}") {
        return vec![pattern.to_string()];
    }
    roots
        .iter()
        .map(|root| pattern.replace("{root}", root))
        .collect()
}
