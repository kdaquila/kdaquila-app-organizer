//! The effective configuration: defaults, with any user file merged over them.

use super::{CONFIG_FILE, ConfigError, ConfigFile, Language, Profile, default_config};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub roots: BTreeMap<String, Language>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub python: Option<Profile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub typescript: Option<Profile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rust: Option<Profile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpp: Option<Profile>,
}

impl Config {
    pub fn profile(&self, language: Language) -> Option<&Profile> {
        match language {
            Language::Python => self.python.as_ref(),
            Language::Typescript => self.typescript.as_ref(),
            Language::Rust => self.rust.as_ref(),
            Language::Cpp => self.cpp.as_ref(),
        }
    }

    fn profile_slot(&mut self, language: Language) -> &mut Option<Profile> {
        match language {
            Language::Python => &mut self.python,
            Language::Typescript => &mut self.typescript,
            Language::Rust => &mut self.rust,
            Language::Cpp => &mut self.cpp,
        }
    }

    /// The declared root that owns a path, if any.
    ///
    /// A root may be more than one component deep — `src/my_package` — because
    /// in an installable Python project the package directory *is* the root of
    /// the source tree, and nothing above it is part of the graded structure.
    /// The longest match wins, so a nested root beats the one containing it.
    pub fn root_for(&self, components: &[&str]) -> Option<(&str, Language, usize)> {
        self.roots
            .iter()
            .filter_map(|(root, language)| {
                let depth = root.split('/').count();
                (components.len() >= depth && components[..depth].join("/") == *root).then_some((
                    root.as_str(),
                    *language,
                    depth,
                ))
            })
            .max_by_key(|(_, _, depth)| *depth)
    }

    /// Roots may not overlap: a path must belong to exactly one, and nesting
    /// one inside another would make "which profile governs this" ambiguous
    /// in ways no longest-match rule can make honest.
    pub fn check_roots(&self) -> Result<(), String> {
        for root in self.roots.keys() {
            if root.is_empty() || root.starts_with('/') || root.ends_with('/') {
                return Err(format!(
                    "root `{root}` must be a relative path with no leading or trailing slash"
                ));
            }
            if let Some(outer) = self
                .roots
                .keys()
                .find(|other| *other != root && root.starts_with(&format!("{other}/")))
            {
                return Err(format!("root `{root}` is nested inside root `{outer}`"));
            }
        }
        Ok(())
    }

    /// The roots declared for one language, in declaration order.
    pub fn roots_for(&self, language: Language) -> Vec<String> {
        self.roots
            .iter()
            .filter(|(_, lang)| **lang == language)
            .map(|(root, _)| root.clone())
            .collect()
    }

    /// Defaults with `app-organizer.toml` (if present at `project_root`) merged over them.
    pub fn load(project_root: &Path) -> Result<Config, ConfigError> {
        let path = project_root.join(CONFIG_FILE);
        let mut config = default_config();
        if !path.exists() {
            return Ok(config);
        }
        let text = std::fs::read_to_string(&path).map_err(|e| ConfigError {
            path: path.clone(),
            message: e.to_string(),
        })?;
        let file: ConfigFile = toml::from_str(&text).map_err(|e| ConfigError {
            path: path.clone(),
            message: e.to_string(),
        })?;
        config.merge(file);
        Ok(config)
    }

    /// User settings win field by field; exceptions are the exception — those
    /// append, so nobody has to re-declare the `__init__.py` waiver.
    fn merge(&mut self, file: ConfigFile) {
        if let Some(roots) = &file.roots {
            self.roots = roots.clone();
        }
        for language in Language::ALL {
            let Some(over) = file.profile_override(language) else {
                continue;
            };
            let slot = self.profile_slot(language);
            // A language with no built-in profile still starts from the shared
            // baseline, so `[typescript] name_case = "kebab-case"` is a
            // one-line declaration rather than a full profile.
            over.apply_to(slot.get_or_insert_with(Profile::default));
        }
    }
}
