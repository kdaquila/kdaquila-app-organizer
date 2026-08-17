//! Config types, loading, and the merge of a user's `app-organizer.toml` over
//! the built-in defaults.

pub mod casing;
pub mod config;
pub mod config_error;
pub mod config_file;
pub mod default_config;
pub mod exception;
pub mod find_project_root;
pub mod language;
pub mod profile;
pub mod profile_override;

pub use casing::Casing;
pub use config::Config;
pub use config_error::ConfigError;
pub use config_file::ConfigFile;
pub use default_config::default_config;
pub use exception::Exception;
pub use find_project_root::find_project_root;
pub use language::Language;
pub use profile::Profile;
pub use profile_override::ProfileOverride;

pub const CONFIG_FILE: &str = "app-organizer.toml";
