//! One exported item.

/// A top-level item that forms part of a module's public surface.
#[derive(Debug, Clone)]
pub struct PublicName {
    pub name: String,
    /// The language keyword that introduced it — `fn`, `class`, `struct`.
    ///
    /// The engine never interprets this string. It only asks whether the
    /// profile's `one_per_file` list contains it, which is what keeps the core
    /// free of any language's vocabulary: each profile supplies its own.
    pub construct: &'static str,
    /// 1-based line of the declaration.
    pub line: usize,
}
