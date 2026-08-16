//! End-to-end tests: every fixture is a small fake repo, checked through the
//! real CLI, with the exact rendered output snapshotted.
//!
//! For a tool whose output *is* its product, reviewing wording changes as
//! snapshot diffs is the right surface.
//!
//! Note: these fixtures rely on there being no `app-organizer.toml` at the
//! crate root. One there would become the project root for every fixture that
//! does not carry its own, and paths in the output would shift.

use assert_cmd::Command;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn check(name: &str) -> String {
    let output = Command::cargo_bin("app-organizer")
        .unwrap()
        .arg("check")
        .arg(fixture(name))
        .output()
        .unwrap();

    format!(
        "exit: {}\n{}{}",
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    )
}

macro_rules! scenarios {
    ($($name:ident),* $(,)?) => {
        $(
            #[test]
            fn $name() {
                insta::assert_snapshot!(check(stringify!($name)));
            }
        )*
    };
}

scenarios!(
    happy_path,
    // layer 1 — folder grammar
    non_kind_folder,
    mixed_children,
    kind_subdirectory,
    too_deep,
    // layer 2 — file naming
    name_mismatch,
    bad_filename_casing,
    unfixable_filename,
    conflicting_rename,
    // layer 3 — content
    two_public_names,
    no_public_names,
    class_in_functions,
    bare_alias_in_types,
    new_type_in_types,
    pep695_alias,
    overload,
    type_checking,
    type_var,
    stray_names_in_constants,
    invalid_syntax,
    not_utf8,
    // one structural cause across many files
    src_layout,
    // roots, extensions, and config
    foreign_extension,
    default_exceptions,
    user_exception,
    custom_roots,
    package_root,
    nested_roots,
);

#[test]
fn json_format_is_valid_json() {
    let output = Command::cargo_bin("app-organizer")
        .unwrap()
        .args(["check", "--format", "json"])
        .arg(fixture("class_in_functions"))
        .output()
        .unwrap();

    let text = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("output is valid JSON");
    insta::assert_snapshot!(serde_json::to_string_pretty(&parsed).unwrap());
}

#[test]
fn defaults_are_printable() {
    let output = Command::cargo_bin("app-organizer")
        .unwrap()
        .arg("defaults")
        .arg(fixture("happy_path"))
        .output()
        .unwrap();
    insta::assert_snapshot!(String::from_utf8_lossy(&output.stdout));
}

/// A seeded config must round-trip: reading it back yields the same effective
/// config it was written from, with no exceptions doubled.
#[test]
fn init_seeds_a_config_that_round_trips() {
    let dir = tempdir();
    Command::cargo_bin("app-organizer")
        .unwrap()
        .arg("init")
        .arg(&dir)
        .assert()
        .success();

    let seeded = Command::cargo_bin("app-organizer")
        .unwrap()
        .arg("defaults")
        .arg(&dir)
        .output()
        .unwrap();
    let pristine = Command::cargo_bin("app-organizer")
        .unwrap()
        .arg("defaults")
        .arg(fixture("happy_path"))
        .output()
        .unwrap();

    assert_eq!(
        String::from_utf8_lossy(&seeded.stdout),
        String::from_utf8_lossy(&pristine.stdout),
    );

    // And it refuses to clobber what it just wrote.
    Command::cargo_bin("app-organizer")
        .unwrap()
        .arg("init")
        .arg(&dir)
        .assert()
        .failure();

    std::fs::remove_dir_all(&dir).ok();
}

fn tempdir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("app-organizer-init-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::remove_file(dir.join("app-organizer.toml")).ok();
    dir
}
