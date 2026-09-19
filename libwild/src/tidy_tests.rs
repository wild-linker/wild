//! Tests that assert properties of our source files, such as formatting.

use crate::bail;
use crate::env;
use crate::error::Context as _;
use crate::error::Result;
use std::fs::read_dir;
use std::path::Path;

#[test]
fn check_sources_format() -> Result {
    use std::process::Command;
    use std::process::Stdio;

    // Sandboxed hosts (WASI) can't see the source tree or run formatters.
    if crate::platforms::host::SANDBOXED {
        return Ok(());
    }

    if env::var("WILD_TEST_IGNORE_FORMAT").is_ok() {
        return Ok(());
    }

    fn collect_files(dir: &Path, extensions: &[&str]) -> Vec<std::path::PathBuf> {
        read_dir(dir)
            .into_iter()
            .flatten()
            .flatten()
            .flat_map(|entry| {
                let path = entry.path();
                if path.is_dir() {
                    collect_files(&path, extensions)
                } else if path.is_file()
                    && path
                        .extension()
                        .is_some_and(|ext| extensions.contains(&ext.to_str().unwrap()))
                {
                    vec![path]
                } else {
                    vec![]
                }
            })
            .collect()
    }

    let extensions = ["c", "cc", "h"];
    let sources_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("wild")
        .join("tests")
        .join("sources");

    assert!(sources_path.is_dir());

    let source_files = collect_files(Path::new(&sources_path), &extensions);

    let clang_format_out = Command::new("clang-format")
        .arg("--dry-run")
        .arg("-Werror")
        // Undocumented option that forces the colours: https://github.com/llvm/llvm-project/issues/119224
        .arg("--color")
        .args(source_files)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to spawn `clang-format`, is it installed?");

    if !clang_format_out.status.success() {
        let stdout = String::from_utf8_lossy(&clang_format_out.stdout);
        let stderr = String::from_utf8_lossy(&clang_format_out.stderr);
        let mut out = String::with_capacity(stdout.len() + stderr.len() + 1);
        if !stdout.is_empty() {
            out.push_str(&stdout);
            if !stderr.is_empty() {
                out.push('\n');
            }
        }
        if !stderr.is_empty() {
            out.push_str(&stderr);
        }
        let clang_out = Command::new("clang-format")
            .arg("--version")
            .output()
            .expect("Failed to spawn `clang-format --version`");
        let clang_version = String::from_utf8_lossy(&clang_out.stdout);
        let version_no_endline = clang_version.trim_end_matches('\n');
        bail!(
            "clang-format ({version_no_endline}) check failed:\n{out}\n\
            Run `clang-format -i {sources_path}/*/*/*.{{{extensions_str}}}` to fix it.",
            sources_path = sources_path.display(),
            extensions_str = extensions.join(",")
        );
    }

    Ok(())
}

#[test]
fn check_toml_format() -> Result {
    use std::process::Command;
    use std::process::Stdio;

    // Sandboxed hosts (WASI) can't see the source tree or run formatters.
    if crate::platforms::host::SANDBOXED {
        return Ok(());
    }

    if env::var("WILD_TEST_IGNORE_FORMAT").is_ok() {
        return Ok(());
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();

    let taplo_out = Command::new("taplo")
        .arg("format")
        .arg("--check")
        .arg("--diff")
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to spawn `taplo`, is it installed?");

    if !taplo_out.status.success() {
        let stdout = String::from_utf8_lossy(&taplo_out.stdout);
        let stderr = String::from_utf8_lossy(&taplo_out.stderr);
        let mut out = String::with_capacity(stdout.len() + stderr.len() + 1);
        if !stdout.is_empty() {
            out.push_str(&stdout);
            if !stderr.is_empty() {
                out.push('\n');
            }
        }
        if !stderr.is_empty() {
            out.push_str(&stderr);
        }

        bail!("TOML format check failed:\n{out}\nRun `taplo format` to fix it.");
    }

    Ok(())
}

#[test]
fn check_text_files() -> Result {
    // Sandboxed hosts (WASI) can't see the source tree or run formatters.
    if crate::platforms::host::SANDBOXED {
        return Ok(());
    }
    const EXCLUDE_DIR: &[&str] = &[
        "target",
        "build",
        "external_test_suites",
        "fakes-debug",
        "fakes",
    ];

    fn verify_path(path: &Path, problems: &mut Vec<String>) -> crate::error::Result {
        if EXCLUDE_DIR.iter().any(|e| path.ends_with(e)) {
            return Ok(());
        }

        if path.is_dir() {
            for entry in read_dir(path)
                .with_context(|| format!("Failed to read directory {}", path.display()))?
            {
                let entry = entry?;
                let file_name = entry.file_name();
                let Some(file_name) = file_name.to_str() else {
                    continue;
                };

                // Ignore hidden files / directories.
                if file_name.starts_with('.') {
                    continue;
                }

                verify_path(&entry.path(), problems)?;
            }
        } else if path.is_symlink() {
            // Ignore symlinks.
        } else {
            let content = std::fs::read(path)
                .with_context(|| format!("Failed to read file {}", path.display()))?;

            let is_valid_utf8 = std::str::from_utf8(&content).is_ok();
            let is_text = is_valid_utf8 && !content.contains(&0);

            if is_text {
                if content.contains(&b'\r') {
                    problems.push(format!(
                        "The file {} uses Windows line-endings. Please convert it to Unix-style.",
                        path.display()
                    ));
                }

                let allow_no_trailing_newline =
                    content.is_empty() || path.extension().is_some_and(|ext| ext == "json");

                if !allow_no_trailing_newline && !content.ends_with(b"\n") {
                    problems.push(format!(
                        "The file {} is missing a trailing newline",
                        path.display()
                    ));
                }
            }
        }
        Ok(())
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();

    let mut problems = Vec::new();
    verify_path(root, &mut problems)?;

    if !problems.is_empty() {
        bail!("{}\n", problems.join("\n"));
    }

    Ok(())
}

/// Checks that we don't put ELF-specific code in files where it shouldn't be.
#[test]
fn check_elf_specific_code() -> Result {
    // Sandboxed hosts (WASI) can't see the source tree or run formatters.
    if crate::platforms::host::SANDBOXED {
        return Ok(());
    }
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");

    // Files where we don't allow ELF-specific code.
    const DISALLOWED: &[&str] = &[
        "input_data.rs",
        "layout.rs",
        "parsing.rs",
        "resolution.rs",
        "symbol_db.rs",
        "thunks.rs",
    ];

    // Patterns that we still allow. These should probably be dealt with, either by renaming these
    // types if we conclude that they're not really ELF-specific, or by removing references to them.
    const EXEMPTIONS: &[&str] = &["linker_utils::elf::RelocationKind"];

    for name in DISALLOWED {
        let path = src_dir.join(name);
        let contents = std::fs::read_to_string(&path)?;
        let mut skip = false;
        for (i, line) in contents.lines().enumerate() {
            if line.starts_with("#[test]") {
                skip = true;
            } else if line.starts_with('}') {
                skip = false;
            } else if skip {
                continue;
            }

            if line.contains("::elf") && !EXEMPTIONS.iter().any(|e| line.contains(e)) {
                bail!(
                    "{path}:{line} contains ELF-specific code. \
                    Please move code, likely by extending Platform trait",
                    path = path.display(),
                    line = i + 1,
                );
            }
        }
    }

    Ok(())
}

/// Host-specific code (`cfg` on the host OS, and native OS APIs) must live in the `platforms`
/// host trees. See `platforms/mod.rs`.
#[test]
fn check_host_code_confined_to_platforms() -> Result {
    // Sandboxed hosts (WASI) can't see the source tree or run formatters.
    if crate::platforms::host::SANDBOXED {
        return Ok(());
    }
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");

    // `cfg` keys that select the host. `target_arch` and `target_endian` aren't here: outside
    // `platforms`, they select what we're linking for.
    const HOST_CFG_KEYS: &[&str] = &[
        "unix",
        "windows",
        "target_os",
        "target_family",
        "target_env",
        "target_abi",
        "target_vendor",
        "target_pointer_width",
    ];

    // Native OS APIs.
    const HOST_PATHS: &[&str] = &["std::os::", "libc::", "nix::", "windows_sys"];

    fn is_host_tree(relative: &Path) -> bool {
        let mut components = relative
            .components()
            .map(|c| c.as_os_str().to_string_lossy());
        if components.next().as_deref() != Some("platforms") {
            return false;
        }
        components
            .next()
            .is_some_and(|name| name == "mod.rs" || name.starts_with("platform_"))
    }

    fn has_word(line: &str, word: &str) -> bool {
        line.match_indices(word).any(|(start, _)| {
            let before = line[..start].chars().next_back();
            let after = line[start + word.len()..].chars().next();
            let is_ident = |c: Option<char>| c.is_some_and(|c| c.is_alphanumeric() || c == '_');
            !is_ident(before) && !is_ident(after)
        })
    }

    fn collect_rs_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> Result {
        for entry in read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                collect_rs_files(&path, out)?;
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                out.push(path);
            }
        }
        Ok(())
    }

    let mut files = Vec::new();
    collect_rs_files(&src_dir, &mut files)?;

    for path in files {
        let relative = path.strip_prefix(&src_dir).unwrap_or(&path);
        // This file names the patterns it checks for.
        if is_host_tree(relative) || relative == Path::new("tidy_tests.rs") {
            continue;
        }
        let in_platforms = relative.starts_with("platforms");
        let contents = std::fs::read_to_string(&path)?;
        for (i, line) in contents.lines().enumerate() {
            let code = line.trim_start();
            if code.starts_with("//") {
                continue;
            }
            let host_cfg = (code.contains("cfg")
                && HOST_CFG_KEYS.iter().any(|k| has_word(code, k)))
                || code.contains("cfg_select!");
            let host_path = HOST_PATHS.iter().any(|p| code.contains(p));
            // Only the facade modules in `platforms/` may name the host trees.
            let host_tree = !in_platforms && code.contains("platforms::platform_");
            if host_cfg || host_path || host_tree {
                bail!(
                    "{path}:{line} contains host-specific code. Please move it into the host \
                    trees under `platforms/` and use it via `crate::platforms`. See \
                    `platforms/mod.rs`",
                    path = path.display(),
                    line = i + 1,
                );
            }
        }
    }

    Ok(())
}
