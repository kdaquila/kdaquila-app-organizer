//! `app-organizer` — an opinionated, multi-language validator for folder
//! conventions, file naming conventions, and file content conventions.
//!
//! Linters check the code *inside* files and say nothing about where files
//! live. This checks the other half.
//!
//! The library exists on day one so the future pip/npm wrappers have something
//! to bind to; `main.rs` is a thin CLI over it.

pub mod config;
pub mod diagnostics;
pub mod grammar;
pub mod lang;
pub mod rules;
pub mod walk;

use config::{Config, ConfigError, Language, Profile};
use diagnostics::Diagnostic;
use grammar::Pattern;
use lang::LanguageProfile;
use rules::exceptions::Exceptions;
use rules::{content, folder};
use std::collections::BTreeMap;
use std::path::Path;

/// One language's configuration, with everything precomputed that can be.
pub struct Compiled {
    pub language: Language,
    pub profile: Profile,
    pub patterns: Vec<Pattern>,
    /// The roots declared for this language, in the `[roots]` map's order.
    pub roots: Vec<String>,
    pub exceptions: Exceptions,
    /// The content-layer profile, absent for languages that have not shipped one.
    pub content: Option<Box<dyn LanguageProfile>>,
}

#[derive(Debug)]
pub struct Report {
    pub diagnostics: Vec<Diagnostic>,
    pub files_checked: usize,
}

impl Report {
    pub fn is_clean(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

#[derive(Debug)]
pub enum Error {
    Config(ConfigError),
    Invalid(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Config(e) => write!(f, "{e}"),
            Error::Invalid(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<ConfigError> for Error {
    fn from(e: ConfigError) -> Self {
        Error::Config(e)
    }
}

pub struct Engine {
    config: Config,
    languages: BTreeMap<Language, Compiled>,
}

impl Engine {
    pub fn new(config: Config) -> Result<Engine, Error> {
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

            let patterns = profile
                .patterns
                .iter()
                .map(|raw| grammar::pattern::parse(raw))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| Error::Invalid(e.to_string()))?;
            let exceptions =
                Exceptions::build(profile, &roots).map_err(|e| Error::Invalid(e.to_string()))?;

            languages.insert(
                language,
                Compiled {
                    language,
                    profile: profile.clone(),
                    patterns,
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
            let Some(compiled) = self.owner(file) else {
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
            let root = walk::components(file)
                .and_then(|c| c.first().copied())
                .unwrap_or_default();

            if let Some(diagnostic) =
                folder::check_root_language(file, root, compiled.language, actual, &waivers)
            {
                // A Rust file in a Python root is not then judged as Python.
                diagnostics.push(diagnostic);
                continue;
            }

            let (kind, placement) = folder::check_placement(compiled, file, &waivers);
            diagnostics.extend(placement);
            diagnostics.extend(folder::check_filename_casing(compiled, file, &waivers));

            if let Some(language) = &compiled.content
                && let Ok(source) = std::fs::read_to_string(project_root.join(file))
            {
                diagnostics.extend(content::check(
                    language.as_ref(),
                    &source,
                    file,
                    kind.as_ref(),
                    &waivers,
                ));
            }
        }

        for (dir, children) in &tree.dirs {
            let Some(compiled) = self.owner(dir) else {
                continue;
            };
            let waivers = compiled.exceptions.waivers_for(&walk::display(dir));
            diagnostics.extend(folder::check_directory(compiled, dir, children, &waivers));
        }

        diagnostics.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
        Report {
            diagnostics,
            files_checked,
        }
    }

    /// The profile governing a path, by its first component. Paths outside
    /// every declared root are invisible to the tool.
    fn owner(&self, rel: &Path) -> Option<&Compiled> {
        let root = walk::components(rel)?.first().copied()?;
        let language = self.config.roots.get(root)?;
        self.languages.get(language)
    }
}

/// Check a path, discovering the project root and config the way the CLI does.
pub fn check(start: &Path) -> Result<Report, Error> {
    let project_root = config::find_project_root(start);
    let config = Config::load(&project_root)?;
    let engine = Engine::new(config)?;
    Ok(engine.check(start, &project_root))
}
