//! Parsing a pattern string into positional segments.
//!
//! Depth is data: nothing here knows that three folder levels are the default.
//! A project that wants a fourth adds a variant and a segment definition.

use super::{FILES, ROOT};

#[derive(Debug, Clone)]
pub struct Pattern {
    /// The pattern exactly as written, for printing back in diagnostics.
    pub raw: String,
    /// Segment names in order, e.g. `["root", "folder1", "kind", "files"]`.
    pub segments: Vec<String>,
}

impl Pattern {
    /// The segments that describe directories — everything but the terminator.
    pub fn dir_segments(&self) -> &[String] {
        &self.segments[..self.segments.len() - 1]
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub pattern: String,
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid pattern `{}`: {}", self.pattern, self.message)
    }
}

pub fn parse(raw: &str) -> Result<Pattern, ParseError> {
    let fail = |message: &str| ParseError {
        pattern: raw.to_string(),
        message: message.to_string(),
    };

    let mut segments = Vec::new();
    for part in raw.split('/') {
        let name = part
            .strip_prefix('{')
            .and_then(|p| p.strip_suffix('}'))
            .ok_or_else(|| fail(&format!("segment `{part}` is not of the form {{name}}")))?;
        if name.is_empty() {
            return Err(fail("empty segment name"));
        }
        segments.push(name.to_string());
    }

    if segments.len() < 2 {
        return Err(fail("a pattern needs at least a root and a terminator"));
    }
    if segments[0] != ROOT {
        return Err(fail("the first segment must be {root}"));
    }
    if segments[segments.len() - 1] != FILES {
        return Err(fail("the last segment must be {files}"));
    }
    if segments[1..segments.len() - 1]
        .iter()
        .any(|s| s == ROOT || s == FILES)
    {
        return Err(fail("{root} and {files} may only appear at the ends"));
    }

    Ok(Pattern {
        raw: raw.to_string(),
        segments,
    })
}
