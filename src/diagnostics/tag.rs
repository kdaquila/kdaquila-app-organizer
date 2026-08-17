//! The category a diagnostic is filed under.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Tag {
    /// Where a folder sits.
    Folder,
    /// What a file or folder is called.
    Naming,
    /// What a file exports.
    Content,
    /// How long a file is.
    Size,
    /// A tracked file whose language contradicts its root's declaration.
    Root,
}

impl Tag {
    pub fn as_str(self) -> &'static str {
        match self {
            Tag::Folder => "folder",
            Tag::Naming => "naming",
            Tag::Content => "content",
            Tag::Size => "size",
            Tag::Root => "root",
        }
    }
}
