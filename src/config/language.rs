//! The languages the tool knows how to recognise by extension.

use serde::{Deserialize, Serialize};

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
