//! The built-in configuration. Zero config means these defaults are the whole
//! contract, so `app-organizer defaults` prints them and they live in one place.

use super::{Casing, Config, Exception, Language, Profile};
use crate::rules::Rule;
use std::collections::BTreeMap;

pub fn default_config() -> Config {
    Config {
        roots: BTreeMap::from([
            ("src".to_string(), Language::Python),
            ("tests".to_string(), Language::Python),
        ]),
        python: Some(python()),
        typescript: None,
        rust: Some(rust()),
        cpp: None,
    }
}

/// `def` and `class` carry an application's logic; everything else a module
/// declares is free to cluster. A module of twenty constants is a good file.
fn python() -> Profile {
    Profile {
        one_per_file: strings(&["def", "class"]),
        name_case: Casing::SnakeCase,
        exceptions: vec![
            mandated_name(
                "**/__init__.py",
                "Python requires package markers at every level",
            ),
            mandated_name(
                "**/__main__.py",
                "Python requires this exact filename for `python -m pkg`",
            ),
            Exception {
                path: "tests/**".to_string(),
                waive: vec![Rule::SinglePrimaryExport],
                reason: "a test module holds many test functions by design".to_string(),
            },
        ],
        ..Profile::default()
    }
}

/// `union` is left out because it is rare enough that a file built around one
/// is already unusual; `type`, `const` and `static` are left out for the same
/// reason `const` is in Python. Any project can add them back.
fn rust() -> Profile {
    Profile {
        one_per_file: strings(&["fn", "struct", "enum", "trait"]),
        name_case: Casing::SnakeCase,
        exceptions: vec![
            mandated_name(
                "**/mod.rs",
                "Rust requires this exact filename for a module",
            ),
            mandated_name("{root}/lib.rs", "cargo requires this exact filename"),
            mandated_name("{root}/main.rs", "cargo requires this exact filename"),
        ],
        ..Profile::default()
    }
}

/// The one shape of default exception left after the v2 redesign: a file whose
/// name the *language* dictates can never also be the name of its export, so
/// requiring a match would be requiring the impossible. Every other v1 default
/// — `**/constants/*.py`, `**/py.typed`, `{root}/app/**`, `conftest.py` — is
/// now derived, because a file with no governed export is already free.
fn mandated_name(path: &str, reason: &str) -> Exception {
    Exception {
        path: path.to_string(),
        waive: vec![Rule::FilenameMatchesExport],
        reason: reason.to_string(),
    }
}

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}
