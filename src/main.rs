//! The CLI. Everything of substance lives in the library.

use clap::{Parser, Subcommand, ValueEnum};
use kdaquila_app_organizer::config::{self, CONFIG_FILE, Config};
use kdaquila_app_organizer::{Engine, diagnostics};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "app-organizer",
    version,
    about = "Validate folder, file naming, and file content conventions"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Check a tree against the conventions.
    Check {
        /// File or directory to check.
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
    },
    /// Print the effective configuration.
    Defaults {
        /// Project to read `app-organizer.toml` from, if it has one.
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Write the effective configuration to `app-organizer.toml`, to edit.
    Init {
        /// Directory to write the config file into.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Overwrite an existing config file.
        #[arg(long)]
        force: bool,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum Format {
    Text,
    Json,
}

/// Violations and internal failures are different outcomes, so they get
/// different codes: a CI job wants to fail on 1 and shout about 2.
const EXIT_VIOLATIONS: u8 = 1;
const EXIT_ERROR: u8 = 2;

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("app-organizer: {message}");
            ExitCode::from(EXIT_ERROR)
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode, String> {
    match cli.command {
        Command::Check { path, format } => check(&path, format),
        Command::Defaults { path } => {
            print!("{}", render_config(&effective_config(&path)?)?);
            Ok(ExitCode::SUCCESS)
        }
        Command::Init { path, force } => init(&path, force),
    }
}

fn check(path: &Path, format: Format) -> Result<ExitCode, String> {
    if !path.exists() {
        return Err(format!("{}: no such file or directory", path.display()));
    }
    let project_root = config::find_project_root(path);
    let config = Config::load(&project_root).map_err(|e| e.to_string())?;
    let engine = Engine::new(config).map_err(|e| e.to_string())?;
    let report = engine.check(path, &project_root);

    let output = match format {
        Format::Text => diagnostics::render_text(&report.diagnostics, report.files_checked),
        Format::Json => diagnostics::render_json(&report.diagnostics, report.files_checked),
    };
    println!("{}", output.trim_end());

    Ok(if report.is_clean() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(EXIT_VIOLATIONS)
    })
}

/// Seed a config file with the full defaults, so overrides are edits to
/// something visible rather than guesses at what is being replaced.
fn init(path: &Path, force: bool) -> Result<ExitCode, String> {
    if !path.is_dir() {
        return Err(format!("{}: not a directory", path.display()));
    }
    let target = path.join(CONFIG_FILE);
    if target.exists() && !force {
        return Err(format!(
            "{} already exists; pass --force to overwrite it",
            target.display()
        ));
    }

    let contents = format!("{HEADER}\n{}", render_config(&effective_config(path)?)?);
    std::fs::write(&target, contents).map_err(|e| format!("{}: {e}", target.display()))?;
    println!("wrote {}", target.display());
    Ok(ExitCode::SUCCESS)
}

const HEADER: &str = "\
# app-organizer configuration, seeded with the built-in defaults.
#
# Anything deleted here falls back to its default, so this file can be trimmed
# down to only the lines you actually change -- `[roots]` is usually the whole
# of it. `kinds`, `patterns` and `segments` replace the default when present;
# `exceptions` are *added* to the defaults, so deleting a default exception
# from this file does not switch it off.
#
# `app-organizer defaults` prints the effective result at any time.";

fn effective_config(path: &Path) -> Result<Config, String> {
    let project_root = config::find_project_root(path);
    Config::load(&project_root).map_err(|e| e.to_string())
}

fn render_config(config: &Config) -> Result<String, String> {
    toml::to_string_pretty(config).map_err(|e| format!("could not render config: {e}"))
}
