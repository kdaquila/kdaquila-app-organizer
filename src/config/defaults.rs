//! The built-in configuration. Zero config means these defaults are the whole
//! contract, so `app-organizer defaults` prints them and they live in one place.

use super::{Casing, Config, Exception, Language, NameSet, Profile, SegmentRule};
use crate::rules::Rule;
use std::collections::BTreeMap;

pub fn config() -> Config {
    Config {
        roots: BTreeMap::from([
            ("src".to_string(), Language::Python),
            ("tests".to_string(), Language::Python),
        ]),
        python: Some(python_profile()),
        typescript: None,
        rust: None,
        cpp: None,
    }
}

fn python_profile() -> Profile {
    let free_folder = SegmentRule {
        not_one_of: Some(NameSet::Ref("@kinds".to_string())),
        casing: Some(Casing::SnakeCase),
        ..SegmentRule::default()
    };

    Profile {
        kinds: strings(&["functions", "types", "constants"]),
        patterns: strings(&[
            "{root}/{folder1}/{folder2}/{folder3}/{kind}/{files}",
            "{root}/{folder1}/{folder2}/{kind}/{files}",
            "{root}/{folder1}/{kind}/{files}",
        ]),
        segments: BTreeMap::from([
            (
                "folder1".to_string(),
                SegmentRule {
                    one_of: Some(NameSet::List(strings(&[
                        "app", "features", "pages", "shared",
                    ]))),
                    ..SegmentRule::default()
                },
            ),
            ("folder2".to_string(), free_folder.clone()),
            ("folder3".to_string(), free_folder),
            (
                "kind".to_string(),
                SegmentRule {
                    one_of: Some(NameSet::Ref("@kinds".to_string())),
                    leaf_only: true,
                    ..SegmentRule::default()
                },
            ),
            (
                "files".to_string(),
                SegmentRule {
                    casing: Some(Casing::SnakeCase),
                    ..SegmentRule::default()
                },
            ),
        ]),
        exceptions: vec![
            Exception {
                path: "**/constants/*.py".to_string(),
                waive: vec![Rule::SinglePublicName],
                reason: "constants files group related values by topic".to_string(),
            },
            Exception {
                path: "**/__init__.py".to_string(),
                waive: vec![Rule::FileMustBeInKindFolder, Rule::SinglePublicName],
                reason: "Python requires package markers at every level; they re-export"
                    .to_string(),
            },
            Exception {
                path: "**/py.typed".to_string(),
                waive: vec![Rule::FileMustBeInKindFolder],
                reason: "PEP 561 requires this exact path".to_string(),
            },
            Exception {
                path: "tests/**/conftest.py".to_string(),
                waive: vec![Rule::FileMustBeInKindFolder, Rule::SinglePublicName],
                reason: "pytest requires this exact filename for fixture discovery".to_string(),
            },
            Exception {
                path: "**/__main__.py".to_string(),
                waive: vec![Rule::FileMustBeInKindFolder, Rule::SinglePublicName],
                reason: "Python requires this exact filename for `python -m pkg`".to_string(),
            },
            Exception {
                path: "{root}/app/**".to_string(),
                waive: vec![Rule::FileMustBeInKindFolder],
                reason: "the composition root wires things together; kinds add nothing there"
                    .to_string(),
            },
        ],
    }
}

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}
