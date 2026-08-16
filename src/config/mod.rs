//! Config types, loading, and the merge of a user's `app-organizer.toml` over
//! the built-in defaults.

pub mod defaults;

use crate::rules::Rule;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const CONFIG_FILE: &str = "app-organizer.toml";

/// The languages the tool knows how to recognise by extension.
///
/// Recognising a language is separate from having a profile for it: a `.rs`
/// file under a root declared python must be an `error[root]`, which requires
/// knowing that `.rs` is tracked by *something*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Python,
    Typescript,
    Rust,
    Cpp,
}

impl Language {
    pub const ALL: [Language; 4] = [
        Language::Python,
        Language::Typescript,
        Language::Rust,
        Language::Cpp,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Language::Python => "python",
            Language::Typescript => "typescript",
            Language::Rust => "rust",
            Language::Cpp => "cpp",
        }
    }

    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Language::Python => &["py", "pyi"],
            Language::Typescript => &["ts", "tsx"],
            Language::Rust => &["rs"],
            Language::Cpp => &["cpp", "cc", "hpp", "h"],
        }
    }

    /// The language that claims this extension, if any. Extensions outside
    /// every list are untracked — invisible to the tool.
    pub fn for_extension(ext: &str) -> Option<Language> {
        Language::ALL
            .into_iter()
            .find(|lang| lang.extensions().contains(&ext))
    }
}

/// Casings a segment may require.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Casing {
    SnakeCase,
    PascalCase,
    CamelCase,
    KebabCase,
}

impl Casing {
    pub fn as_str(self) -> &'static str {
        match self {
            Casing::SnakeCase => "snake_case",
            Casing::PascalCase => "PascalCase",
            Casing::CamelCase => "camelCase",
            Casing::KebabCase => "kebab-case",
        }
    }

    /// The name rewritten into this casing, when the tool can say so
    /// confidently. Only snake_case ships a converter — the other casings
    /// exist for segment rules and will grow one when a profile needs it.
    pub fn suggest(self, name: &str) -> Option<String> {
        match self {
            Casing::SnakeCase => Some(crate::rules::to_snake_case(name)),
            _ => None,
        }
    }

    pub fn matches(self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        match self {
            Casing::SnakeCase => name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
            Casing::KebabCase => name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
            Casing::PascalCase => {
                name.chars().all(|c| c.is_ascii_alphanumeric())
                    && name.starts_with(|c: char| c.is_ascii_uppercase())
            }
            Casing::CamelCase => {
                name.chars().all(|c| c.is_ascii_alphanumeric())
                    && name.starts_with(|c: char| c.is_ascii_lowercase())
            }
        }
    }
}

/// A set of names, either spelled out or referencing a profile-level list.
///
/// `"@kinds"` is kept unresolved so that `app-organizer defaults` prints the
/// same thing a human would have written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NameSet {
    Ref(String),
    List(Vec<String>),
}

/// Constraints on one positional segment of a path pattern.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SegmentRule {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub one_of: Option<NameSet>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub not_one_of: Option<NameSet>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub casing: Option<Casing>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub leaf_only: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// A scoped rule waiver: a glob, and the rules that do not apply beneath it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Exception {
    pub path: String,
    pub waive: Vec<Rule>,
    pub reason: String,
}

impl Exception {
    /// Same scope, same effect — the `reason` is prose and does not count.
    fn same(&self, other: &Exception) -> bool {
        self.path == other.path && self.waive == other.waive
    }
}

/// Everything one language declares about its own shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub kinds: Vec<String>,
    pub patterns: Vec<String>,
    pub segments: BTreeMap<String, SegmentRule>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exceptions: Vec<Exception>,
}

impl Profile {
    /// Resolve a `NameSet`, expanding `"@kinds"` against this profile.
    pub fn resolve(&self, set: &NameSet) -> Vec<String> {
        match set {
            NameSet::List(names) => names.clone(),
            NameSet::Ref(name) => match name.as_str() {
                "@kinds" => self.kinds.clone(),
                _ => Vec::new(),
            },
        }
    }

    pub fn is_kind(&self, name: &str) -> bool {
        self.kinds.iter().any(|k| k == name)
    }
}

/// The effective configuration: defaults, with any user file merged over them.
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
        let mut config = defaults::config();
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
            match slot {
                Some(profile) => {
                    if let Some(kinds) = &over.kinds {
                        profile.kinds = kinds.clone();
                    }
                    if let Some(patterns) = &over.patterns {
                        profile.patterns = patterns.clone();
                    }
                    if let Some(segments) = &over.segments {
                        profile.segments = segments.clone();
                    }
                    // A file seeded by `app-organizer init` already contains
                    // the defaults verbatim; appending them again would double
                    // every entry.
                    for exception in &over.exceptions {
                        if !profile.exceptions.iter().any(|kept| kept.same(exception)) {
                            profile.exceptions.push(exception.clone());
                        }
                    }
                }
                None => {
                    *slot = Some(Profile {
                        kinds: over.kinds.clone().unwrap_or_default(),
                        patterns: over.patterns.clone().unwrap_or_default(),
                        segments: over.segments.clone().unwrap_or_default(),
                        exceptions: over.exceptions.clone(),
                    })
                }
            }
        }
    }
}

/// A user's config file. Every field is optional — the defaults supply the rest.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigFile {
    pub roots: Option<BTreeMap<String, Language>>,
    pub python: Option<ProfileOverride>,
    pub typescript: Option<ProfileOverride>,
    pub rust: Option<ProfileOverride>,
    pub cpp: Option<ProfileOverride>,
}

impl ConfigFile {
    fn profile_override(&self, language: Language) -> Option<&ProfileOverride> {
        match language {
            Language::Python => self.python.as_ref(),
            Language::Typescript => self.typescript.as_ref(),
            Language::Rust => self.rust.as_ref(),
            Language::Cpp => self.cpp.as_ref(),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileOverride {
    pub kinds: Option<Vec<String>>,
    pub patterns: Option<Vec<String>>,
    pub segments: Option<BTreeMap<String, SegmentRule>>,
    #[serde(default)]
    pub exceptions: Vec<Exception>,
}

#[derive(Debug)]
pub struct ConfigError {
    pub path: PathBuf,
    pub message: String,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path.display(), self.message)
    }
}

impl std::error::Error for ConfigError {}

/// Walk up from `start` looking for the file that marks the project root.
/// Absent one, `start` itself is the root.
pub fn find_project_root(start: &Path) -> PathBuf {
    let start = if start.is_dir() {
        start.to_path_buf()
    } else {
        start.parent().unwrap_or(start).to_path_buf()
    };
    for dir in start.ancestors() {
        if dir.join(CONFIG_FILE).is_file() {
            return dir.to_path_buf();
        }
    }
    start
}
