//! End-to-end tests: every fixture is a small fake repo, checked through the
//! real CLI, with the exact rendered output snapshotted.
//!
//! For a tool whose output *is* its product, reviewing wording changes as
//! snapshot diffs is the right surface.
//!
//! Note: every fixture carries its own `app-organizer.toml`. Without one, the
//! crate root's config would become their project root — the paths in the
//! output would shift and nothing would be governed at all.

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
    // folders
    too_deep,
    folder_casing,
    // naming
    name_mismatch,
    bad_filename_casing,
    unfixable_filename,
    conflicting_rename,
    // content
    two_governed_exports,
    topic_file,
    pep695_alias,
    overload,
    type_checking,
    invalid_syntax,
    not_utf8,
    // the line budget, and the files it deliberately does not apply to
    too_many_lines,
    long_constants_file,
    // Rust
    rust_happy_path,
    rust_two_exports,
    rust_pub_crate,
    rust_private_only,
    rust_too_long,
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
        .arg(fixture("two_governed_exports"))
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
