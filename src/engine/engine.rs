//! The compiled config, and the loop that runs it over a tree.

use super::{Compiled, Error, Report};
use crate::config::{Config, Language};
use crate::lang;
use crate::rules::check_content::check_content;
use crate::rules::folder_depth::folder_depth;
use crate::rules::name_casing::name_casing;
use crate::rules::prescribes_a_filename::prescribes_a_filename;
use crate::rules::root_language_match::root_language_match;
use crate::rules::unreadable::unreadable;
use crate::rules::{Exceptions, Rule};
use crate::walk;
use std::collections::BTreeMap;
use std::path::Path;

pub struct Engine {
    config: Config,
    languages: BTreeMap<Language, Compiled>,
}

impl Engine {
    pub fn new(config: Config) -> Result<Engine, Error> {
        config.check_roots().map_err(Error::Invalid)?;
        let mut languages = BTreeMap::new();

        for language in Language::ALL {
            let roots = config.roots_for(language);
            if roots.is_empty() {
                continue;
            }
            let Some(profile) = config.profile(language) else {
                return Err(Error::Invalid(format!(
                    "roots {} are declared {}, but there is no [{}] profile",
                    roots.join(", "),
                    language.as_str(),
                    language.as_str()
                )));
            };
            let exceptions =
                Exceptions::build(profile, &roots).map_err(|e| Error::Invalid(e.to_string()))?;

            languages.insert(
                language,
                Compiled {
                    language,
                    profile: profile.clone(),
                    roots,
                    exceptions,
                    content: lang::profile_for(language),
                },
            );
        }

        Ok(Engine { config, languages })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Check everything under `start`, reporting paths relative to `project_root`.
    pub fn check(&self, start: &Path, project_root: &Path) -> Report {
        let tree = walk::walk(start, project_root);
        let mut diagnostics = Vec::new();
        let mut files_checked = 0;

        for file in &tree.files {
            let Some((compiled, root, _)) = self.owner(file) else {
                continue;
            };
            // Extensions outside every language's list are untracked: a
            // README.md or fixtures.json may sit anywhere.
            let Some(actual) = file
                .extension()
                .and_then(|e| e.to_str())
                .and_then(Language::for_extension)
            else {
                continue;
            };
            files_checked += 1;
            let waivers = compiled.exceptions.waivers_for(&walk::display(file));

            if let Some(diagnostic) =
                root_language_match(file, root, compiled.language, actual, &waivers)
            {
                // A Rust file in a Python root is not then judged as Python.
                diagnostics.push(diagnostic);
                continue;
            }

            let mut content = Vec::new();
            if let Some(language) = &compiled.content {
                match std::fs::read_to_string(project_root.join(file)) {
                    Ok(source) => {
                        content = check_content(
                            language.as_ref(),
                            &compiled.profile,
                            &source,
                            file,
                            &waivers,
                        );
                    }
                    Err(error) if waivers.active(Rule::FileIsReadable) => {
                        content.push(unreadable(file, &error.to_string()));
                    }
                    Err(_) => {}
                }
            }

            if !prescribes_a_filename(&content)
                && let Some(stem) = file.file_stem().and_then(|s| s.to_str())
            {
                let suffix = file
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| format!(".{e}"))
                    .unwrap_or_default();
                diagnostics.extend(name_casing(
                    &compiled.profile,
                    file,
                    stem,
                    &suffix,
                    &waivers,
                ));
            }
            diagnostics.extend(content);
        }

        for dir in &tree.dirs {
            let Some((compiled, _, root_depth)) = self.owner(dir) else {
                continue;
            };
            let Some(components) = walk::components(dir) else {
                continue;
            };
            // A root's own name is the user's declaration, not the tool's
            // business — `src/MyPackage` is theirs to spell.
            let depth = components.len() - root_depth;
            if depth == 0 {
                continue;
            }
            let waivers = compiled.exceptions.waivers_for(&walk::display(dir));
            diagnostics.extend(folder_depth(&compiled.profile, dir, depth, &waivers));
            if let Some(name) = dir.file_name().and_then(|n| n.to_str()) {
                diagnostics.extend(name_casing(&compiled.profile, dir, name, "", &waivers));
            }
        }

        diagnostics.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
        Report {
            diagnostics,
            files_checked,
            declared_roots: self.config.roots.keys().cloned().collect(),
        }
    }

    /// The profile governing a path, the root it was found under, and how many
    /// components that root is. Paths outside every declared root are invisible.
    fn owner<'a>(&'a self, rel: &Path) -> Option<(&'a Compiled, &'a str, usize)> {
        let components = walk::components(rel)?;
        let (name, language, depth) = self.config.root_for(&components)?;
        Some((self.languages.get(&language)?, name, depth))
    }
}
