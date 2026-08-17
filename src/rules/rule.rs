//! Every rule the tool can report, and every name an exception may waive.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rule {
    /// A tracked file's language matches the language its root is declared as.
    RootLanguageMatch,
    /// The tool could read and parse the file at all. Not a convention, but
    /// staying silent about a file it could not open would be worse.
    FileIsReadable,
    /// Folders nest no deeper than the profile allows.
    FolderDepth,
    /// Folder and file names obey the language's one casing.
    NameCasing,
    /// The filename is the export's name, transformed into that casing.
    FilenameMatchesExport,
    /// At most one export built from a governed construct.
    SinglePrimaryExport,
    /// A file with a governed export stays inside the line budget.
    MaxFileLines,
}

impl Rule {
    pub fn as_str(self) -> &'static str {
        match self {
            Rule::RootLanguageMatch => "root_language_match",
            Rule::FileIsReadable => "file_is_readable",
            Rule::FolderDepth => "folder_depth",
            Rule::NameCasing => "name_casing",
            Rule::FilenameMatchesExport => "filename_matches_export",
            Rule::SinglePrimaryExport => "single_primary_export",
            Rule::MaxFileLines => "max_file_lines",
        }
    }

    /// The rule this one has nothing left to check without.
    ///
    /// This is the whole deactivation cascade, expressed once as a property of
    /// the rule graph rather than as special cases at each call site. Both
    /// edges point at the same place, which is the v2 design in one line: a
    /// governed export is what activates the two extra standards, so waiving
    /// the export rule lifts both of them with it.
    pub fn depends_on(self) -> Option<Rule> {
        match self {
            // No primary export to derive a filename from.
            Rule::FilenameMatchesExport => Some(Rule::SinglePrimaryExport),
            // The budget is on the files carrying the logic; without a primary
            // export this is a constants table or a config map, and its length
            // is its author's business.
            Rule::MaxFileLines => Some(Rule::SinglePrimaryExport),
            _ => None,
        }
    }

    /// Every variant, for tests and for documenting the vocabulary.
    pub const ALL: [Rule; 7] = [
        Rule::RootLanguageMatch,
        Rule::FileIsReadable,
        Rule::FolderDepth,
        Rule::NameCasing,
        Rule::FilenameMatchesExport,
        Rule::SinglePrimaryExport,
        Rule::MaxFileLines,
    ];
}
