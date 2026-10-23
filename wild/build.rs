//! A build script that writes `version.rs`. This will then be used by wild as the version written
//! into the .comment section and reported when the --version flag is used.
//!
//! The version will be the version from `Cargo.toml`, but only if there's a git tag with the same
//! name and we're building from that commit without other changes. Otherwise, our version will be
//! the git hash at which our changes, if any, diverge from origin/main.

use std::path::Path;
use std::process::Command;

fn main() {
    let out_dir = std::env::var("OUT_DIR").expect("Needs OUT_DIR to be set");
    let out = Path::new(&out_dir).join("version.rs");
    let version = version_info();

    println!("cargo:rerun-if-changed=../.git/HEAD");

    if let Ok(existing) = std::fs::read_to_string(&out)
        && existing == version
    {
        return;
    }

    std::fs::write(&out, &version).expect("Failed to write version.rs");
}

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

fn version_info() -> String {
    let has_tag = get_version_tag().is_some() && !has_changes_relative_to(PKG_VERSION);
    let commit_hash = if has_tag {
        git_output_line(&["rev-parse", "HEAD"])
    } else {
        main_hash()
    };
    let modified = !has_tag && commit_hash.as_deref().is_some_and(has_changes_relative_to);
    let commit_date = commit_hash
        .as_deref()
        .and_then(|hash| git_output_line(&["show", "-s", "--format=%cs", hash]));

    format!(
        "libwild::WildVersionInfo {{ \
            name: libwild::DEFAULT_LINKER_NAME,
            commit_hash: {commit_hash:?}, \
            commit_date: {commit_date:?}, \
            has_tag: {has_tag}, \
            modified: {modified}, \
         }}"
    )
}

fn has_changes_relative_to(version_ref: &str) -> bool {
    Command::new("git")
        .arg("diff")
        .arg("--quiet")
        .arg(version_ref)
        .output()
        .map_or(true, |output| !output.status.success())
}

/// Returns the hash of the merge-point with origin/main.
fn main_hash() -> Option<String> {
    git_output_line(&["merge-base", "HEAD", "origin/main"])
}

fn git_output_line(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;

    if !output.status.success() {
        return None;
    }

    String::from_utf8(output.stdout)
        .ok()?
        .lines()
        .next()
        .map(|line| line.to_owned())
}

/// Returns the current version, but only if there's a corresponding tag at the current commit.
fn get_version_tag() -> Option<String> {
    let output = Command::new("git")
        .arg("tag")
        .arg("--points-at")
        .arg("HEAD")
        .output()
        .ok()?;
    let stdout = String::from_utf8(output.stdout).ok()?;
    if stdout.lines().any(|line| line == PKG_VERSION) {
        Some(PKG_VERSION.to_owned())
    } else {
        None
    }
}
