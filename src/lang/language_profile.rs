//! The trait every language implements.

use super::Module;
use crate::config::Language;

pub trait LanguageProfile {
    fn language(&self) -> Language;

    fn read(&self, source: &str) -> Module;
}
