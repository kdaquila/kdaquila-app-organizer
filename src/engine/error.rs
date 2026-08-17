//! A check that could not be set up at all — as opposed to one that found
//! violations, which is a `Report`.

use crate::config::ConfigError;

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
