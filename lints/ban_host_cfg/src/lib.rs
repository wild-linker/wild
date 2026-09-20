#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_errors;
extern crate rustc_span;

use rustc_errors::DiagDecorator;
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_span::{FileName, RemapPathScopeComponents, Span};
use std::collections::HashSet;

#[derive(Default)]
struct BanHostCfg {
    scanned_files: HashSet<String>,
}

dylint_linting::impl_pre_expansion_lint! {
    /// ### What it does
    ///
    /// Keeps host selection inside the host trees. The only sources allowed to select the host
    /// operating system or call native OS APIs are `libwild/src/host/mod.rs` and the trees
    /// `libwild/src/host/<tree>.rs` and `libwild/src/host/<tree>/` for `<tree>` in `unix`,
    /// `linux`, `macos`, `illumos`, `other_unix`, `windows` and `wasi`.
    ///
    /// Everywhere else in the workspace — libwild, the `wild` binary and its integration tests,
    /// and the helper crates, including the facade modules in `host/` — this denies:
    ///
    /// - `#[cfg]`, `#[cfg_attr]`, `#![cfg]`, `#![cfg_attr]`, `cfg!()` and `cfg_select!`
    ///   mentioning `windows`, `unix`, `target_os`, `target_family`, `target_env`, `target_abi`,
    ///   `target_vendor` or `target_pointer_width`;
    /// - native OS references: `std::os::{unix,windows,linux,macos,fd,wasi}`, `windows_sys`,
    ///   `windows::Win32`, `libc::` and `nix::`;
    /// - outside `libwild/src/host/`, naming a host tree directly (`host::imp`, `host::linux`,
    ///   …) instead of going through the facades.
    ///
    /// ### The ratchet
    ///
    /// `BASELINE` lists the files that still select the host, with how many findings each has
    /// today. Exceeding an allowance fails, and so does dropping below it without lowering the
    /// number, so the list can only shrink. New host-dependent code has to go into a tree.
    ///
    /// ### Why one CI host is enough
    ///
    /// Each source file is scanned pre-expansion, so code that this host would `cfg` away is
    /// still checked. Only whole module files reached through a feature-gated `mod` need their
    /// feature active, which is why CI lints both the default and `--no-default-features` sets.
    ///
    /// ### Why `target_arch` and `target_endian` aren't checked
    ///
    /// They are host facts too, but they're deliberately outside this boundary: their uses
    /// outside the trees derive a default from the host architecture, such as
    /// `default_emulation`, which picks what we link for when the arguments don't say.
    pub BAN_HOST_CFG,
    Deny,
    "keep host selection and native OS APIs inside libwild's host trees",
    BanHostCfg::default()
}

const SELECTORS: [&str; 8] = [
    "windows",
    "unix",
    "target_os",
    "target_family",
    "target_env",
    "target_abi",
    "target_vendor",
    "target_pointer_width",
];

const HOST_DIR: &str = "libwild/src/host/";

const HOST_TREES: [&str; 7] = [
    "unix",
    "linux",
    "macos",
    "illumos",
    "other_unix",
    "windows",
    "wasi",
];

/// Qualified forms, so that a plain `windows` or `unix` elsewhere isn't mistaken for a tree.
const HOST_TREE_REFS: [&str; 8] = [
    "host::imp",
    "host::unix",
    "host::linux",
    "host::macos",
    "host::illumos",
    "host::other_unix",
    "host::windows",
    "host::wasi",
];

const NATIVE_MARKERS: [&str; 10] = [
    "std::os::windows",
    "std::os::unix",
    "std::os::linux",
    "std::os::macos",
    "std::os::fd",
    "std::os::wasi",
    "windows_sys",
    "windows::Win32",
    "libc::",
    "nix::",
];

/// Files outside the host trees that still select the host, and how many findings each has.
/// These are the ones that predate the host module; everything else must be at zero. Lower a
/// number when you remove a finding: the lint fails if a count no longer matches, so the
/// allowances can only shrink.
const BASELINE: [(&str, usize); 3] = [
    ("wild/tests/integration_tests.rs", 14),
    ("wild/tests/external_tests/lld_tests.rs", 1),
    ("linker-diff/src/utils.rs", 1),
];

/// The crate directories of this workspace, used to make paths workspace-relative.
const CRATE_DIRS: [&str; 6] = [
    "libwild/",
    "wild/",
    "linker-diff/",
    "linker-utils/",
    "linker-layout/",
    "linker-trace/",
];

impl EarlyLintPass for BanHostCfg {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &rustc_ast::ast::Item) {
        let current_file = source_filename(cx, item.span);
        if !in_scope(&current_file) || !self.scanned_files.insert(current_file.clone()) {
            return;
        }
        // Scan the physical source once, rather than only the AST that survived `cfg`.
        let Ok(source) = std::fs::read_to_string(&current_file)
            .or_else(|_| cx.sess().source_map().span_to_snippet(item.span))
        else {
            return;
        };

        let mut findings = Vec::new();
        for invocation in host_cfg_invocations(&source) {
            findings.push(format!("host cfg `{invocation}`"));
        }
        for reference in native_host_references(&source) {
            findings.push(format!("native OS reference `{reference}`"));
        }
        if !inside_host_dir(&current_file) {
            for reference in host_tree_references(&source) {
                findings.push(format!("host tree named directly: `{reference}`"));
            }
        }

        let allowed = baseline_allowance(&current_file);
        if findings.len() > allowed {
            for finding in findings.iter().skip(allowed) {
                emit(cx, item.span, finding.clone());
            }
        } else if findings.len() < allowed {
            emit_baseline_too_high(cx, item.span, allowed, findings.len());
        }
    }
}

fn emit(cx: &EarlyContext<'_>, span: Span, detail: String) {
    cx.opt_span_lint(
        BAN_HOST_CFG,
        Some(span),
        DiagDecorator(move |diag| {
            diag.primary_message(format!(
                "host selection outside libwild's host trees: {detail}; only \
                 libwild/src/host/mod.rs and the host trees may select the host, everything \
                 else goes through crate::host"
            ));
        }),
    );
}

fn emit_baseline_too_high(cx: &EarlyContext<'_>, span: Span, allowed: usize, found: usize) {
    cx.opt_span_lint(
        BAN_HOST_CFG,
        Some(span),
        DiagDecorator(move |diag| {
            diag.primary_message(format!(
                "this file's BASELINE allowance in the ban_host_cfg lint is {allowed}, but it \
                 now has {found} findings; please lower the allowance so it can't grow back"
            ));
        }),
    );
}

fn baseline_allowance(filename: &str) -> usize {
    let normalized = filename.replace('\\', "/");
    workspace_relative(&normalized)
        .and_then(|relative| {
            BASELINE
                .iter()
                .find(|(path, _)| *path == relative)
                .map(|(_, allowed)| *allowed)
        })
        .unwrap_or(0)
}

/// Rust sources the boundary applies to: every workspace crate's targets (production code, unit
/// tests, integration tests and bins), plus this lint's own `ui/` fixtures. Only `host/mod.rs`
/// and the trees are exempt; the facade modules stay in scope.
fn in_scope(filename: &str) -> bool {
    let normalized = filename.replace('\\', "/");
    if !normalized.ends_with(".rs") {
        return false;
    }
    if let Some(relative) = workspace_relative(&normalized) {
        return !is_host_tree_file(relative);
    }
    normalized.starts_with("ui/") || normalized.contains("/ui/")
}

/// `host/mod.rs`, `host/<tree>.rs` and anything under `host/<tree>/`. A facade such as
/// `host/fs.rs`, or a lookalike such as `host/unixish.rs`, is not a tree.
fn is_host_tree_file(relative: &str) -> bool {
    let Some(rest) = relative.strip_prefix(HOST_DIR) else {
        return false;
    };
    if rest == "mod.rs" {
        return true;
    }
    HOST_TREES.iter().any(|tree| {
        rest.strip_prefix(tree)
            .is_some_and(|after| after == ".rs" || after.starts_with('/'))
    })
}

/// The facades and the trees may name the trees directly; nothing else may.
fn inside_host_dir(filename: &str) -> bool {
    let normalized = filename.replace('\\', "/");
    workspace_relative(&normalized).is_some_and(|relative| relative.starts_with(HOST_DIR))
}

/// The workspace-relative part of a path, anchored at a path-component boundary. The deepest
/// match wins, since the repository itself is called `wild`, as is one of its crates.
fn workspace_relative(normalized: &str) -> Option<&str> {
    CRATE_DIRS
        .iter()
        .filter_map(|dir| {
            normalized
                .match_indices(dir)
                .filter(|(offset, _)| *offset == 0 || normalized.as_bytes()[offset - 1] == b'/')
                .map(|(offset, _)| offset)
                .last()
        })
        .max()
        .map(|offset| &normalized[offset..])
}

fn host_cfg_invocations(source: &str) -> Vec<String> {
    let compact = compact_code(&code_without_comments_or_strings(source));
    let mut invocations = Vec::new();
    for start in [
        "#[cfg(",
        "#[cfg_attr(",
        "#![cfg(",
        "#![cfg_attr(",
        "cfg!(",
        "cfg_select!{",
        "cfg_select!(",
    ] {
        for (offset, _) in compact.match_indices(start) {
            // Attributes need no leading boundary; `my_cfg!(unix)` is not `cfg!`.
            if !start.starts_with('#') && !standalone_before(&compact, offset) {
                continue;
            }
            let Some(clause) = balanced_invocation(&compact, offset, start.len() - 1) else {
                continue;
            };
            if SELECTORS
                .iter()
                .any(|selector| contains_identifier(clause, selector))
            {
                let shown = if start.starts_with("cfg_select!") {
                    "cfg_select!"
                } else {
                    clause.trim_start_matches("#![").trim_start_matches("#[")
                };
                invocations.push(shown.to_owned());
            }
        }
    }
    invocations
}

fn host_tree_references(source: &str) -> Vec<String> {
    let code = code_without_comments_or_strings(source);
    let mut references = Vec::new();
    for name in HOST_TREE_REFS {
        for _ in standalone_matches(&code, name) {
            references.push(name.to_owned());
        }
    }
    references
}

fn native_host_references(source: &str) -> Vec<String> {
    let code = code_without_comments_or_strings(source);
    NATIVE_MARKERS
        .into_iter()
        .filter(|marker| standalone_matches(&code, marker).next().is_some())
        .map(str::to_owned)
        .collect()
}

fn is_identifier_char(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

fn standalone_before(code: &str, offset: usize) -> bool {
    code[..offset]
        .chars()
        .next_back()
        .is_none_or(|character| !is_identifier_char(character))
}

/// Offsets of `needle` not embedded in a longer identifier. A trailing boundary is only
/// required when the needle itself ends in an identifier character (so `libc::` still
/// matches `libc::getpid`).
fn standalone_matches<'a>(code: &'a str, needle: &'a str) -> impl Iterator<Item = usize> + 'a {
    let needs_trailing = needle.chars().next_back().is_some_and(is_identifier_char);
    code.match_indices(needle)
        .map(|(offset, _)| offset)
        .filter(move |offset| {
            standalone_before(code, *offset)
                && (!needs_trailing
                    || code[offset + needle.len()..]
                        .chars()
                        .next()
                        .is_none_or(|character| !is_identifier_char(character)))
        })
}

fn contains_identifier(code: &str, identifier: &str) -> bool {
    standalone_matches(code, identifier).next().is_some()
}

/// Drops whitespace so `# [cfg (unix)]` matches `#[cfg(unix)]`, but keeps one space between
/// identifier characters so `return cfg!(unix)` does not become `returncfg!(unix)`.
fn compact_code(code: &str) -> String {
    let mut compact = String::with_capacity(code.len());
    let mut pending_space = false;
    for character in code.chars() {
        if character.is_whitespace() {
            pending_space = true;
            continue;
        }
        if pending_space
            && is_identifier_char(character)
            && compact.chars().next_back().is_some_and(is_identifier_char)
        {
            compact.push(' ');
        }
        pending_space = false;
        compact.push(character);
    }
    compact
}

/// The invocation starting at `offset` whose opening delimiter sits at `offset + open`, up to
/// and including its matching closing delimiter.
fn balanced_invocation(source: &str, offset: usize, open: usize) -> Option<&str> {
    let mut depth = 0_u32;
    for (relative, character) in source[offset + open..].char_indices() {
        match character {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(&source[offset..offset + open + relative + 1]);
                }
            }
            _ => {}
        }
    }
    None
}

fn code_without_comments_or_strings(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut output = String::with_capacity(source.len());
    let mut index = 0;
    while index < bytes.len() {
        if let Some((prefix_len, hashes)) = raw_string_prefix(&bytes[index..]) {
            let start = index;
            index += prefix_len;
            while index < bytes.len() {
                let suffix = &bytes[index + 1..];
                if bytes[index] == b'"'
                    && suffix.len() >= hashes
                    && suffix[..hashes].iter().all(|byte| *byte == b'#')
                {
                    index += 1 + hashes;
                    break;
                }
                index += 1;
            }
            mask_range(&mut output, bytes, start, index);
        } else if bytes[index..].starts_with(b"//") {
            while index < bytes.len() && bytes[index] != b'\n' {
                output.push(' ');
                index += 1;
            }
        } else if bytes[index..].starts_with(b"/*") {
            let mut depth = 1_u32;
            output.push_str("  ");
            index += 2;
            while index < bytes.len() && depth > 0 {
                if bytes[index..].starts_with(b"/*") {
                    depth += 1;
                    output.push_str("  ");
                    index += 2;
                } else if bytes[index..].starts_with(b"*/") {
                    depth -= 1;
                    output.push_str("  ");
                    index += 2;
                } else {
                    output.push(if bytes[index] == b'\n' { '\n' } else { ' ' });
                    index += 1;
                }
            }
        } else if let Some(len) = char_literal_len(&bytes[index..]) {
            // Masked so a `'"'` literal cannot open a phantom string.
            mask_range(&mut output, bytes, index, index + len);
            index += len;
        } else if bytes[index] == b'"' {
            output.push(' ');
            index += 1;
            while index < bytes.len() {
                let byte = bytes[index];
                output.push(if byte == b'\n' { '\n' } else { ' ' });
                index += 1;
                if byte == b'\\' && index < bytes.len() {
                    output.push(' ');
                    index += 1;
                } else if byte == b'"' {
                    break;
                }
            }
        } else {
            output.push(bytes[index] as char);
            index += 1;
        }
    }
    output
}

/// Length of an ASCII or escaped char literal at the start of `source`. Lifetimes and
/// multi-byte char literals return `None`; neither can contain a quote that confuses masking.
fn char_literal_len(source: &[u8]) -> Option<usize> {
    if source.first() != Some(&b'\'') {
        return None;
    }
    if source.get(1) == Some(&b'\\') {
        let close = source.iter().skip(3).position(|byte| *byte == b'\'')?;
        return Some(3 + close + 1);
    }
    (source.get(2) == Some(&b'\'') && source.get(1) != Some(&b'\'')).then_some(3)
}

fn raw_string_prefix(source: &[u8]) -> Option<(usize, usize)> {
    let mut index = usize::from(source.starts_with(b"br"));
    if source.get(index) != Some(&b'r') {
        return None;
    }
    index += 1;
    let hashes_start = index;
    while source.get(index) == Some(&b'#') {
        index += 1;
    }
    (source.get(index) == Some(&b'"')).then_some((index + 1, index - hashes_start))
}

fn mask_range(output: &mut String, source: &[u8], start: usize, end: usize) {
    for byte in &source[start..end] {
        output.push(if *byte == b'\n' { '\n' } else { ' ' });
    }
}

fn source_filename(cx: &EarlyContext<'_>, span: Span) -> String {
    match cx.sess().source_map().span_to_filename(span) {
        FileName::Real(real_filename) => real_filename
            .local_path()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_else(|| {
                real_filename
                    .path(RemapPathScopeComponents::DIAGNOSTICS)
                    .to_string_lossy()
                    .into_owned()
            }),
        filename => filename
            .display(RemapPathScopeComponents::DIAGNOSTICS)
            .to_string(),
    }
}

#[test]
fn ui() {
    dylint_testing::ui_test(env!("CARGO_PKG_NAME"), "ui");
}

#[test]
fn host_trees_are_exempt() {
    for path in [
        "libwild/src/host/mod.rs",
        "libwild/src/host/unix.rs",
        "libwild/src/host/linux.rs",
        "libwild/src/host/linux/perf.rs",
        "libwild/src/host/macos.rs",
        "libwild/src/host/illumos.rs",
        "libwild/src/host/other_unix.rs",
        "libwild/src/host/windows.rs",
        "libwild/src/host/wasi.rs",
        "libwild/src/host/wasi/path.rs",
        "/home/dev/wild/libwild/src/host/linux.rs",
        r"C:\work\wild\libwild\src\host\windows.rs",
    ] {
        assert!(!in_scope(path), "{path}");
    }
}

#[test]
fn facades_and_lookalikes_are_in_scope() {
    for path in [
        "libwild/src/host/fs.rs",
        "libwild/src/host/os.rs",
        "libwild/src/host/perf.rs",
        "libwild/src/host/process.rs",
        "libwild/src/host/linker_plugin.rs",
        "libwild/src/host/common.rs",
        "libwild/src/host/unixish.rs",
        "libwild/src/host/linux_extra.rs",
        "libwild/src/host/mod_extra.rs",
        "libwild/src/host/other/mod.rs",
        "libwild/src/host/tests/linux.rs",
        "libwild/src/fs.rs",
        "libwild/src/platform.rs",
        "wild/src/host/linux.rs",
    ] {
        assert!(in_scope(path), "{path}");
    }
}

#[test]
fn every_workspace_crate_target_is_in_scope() {
    for path in [
        "libwild/src/lib.rs",
        "libwild/src/subprocess.rs",
        "wild/src/main.rs",
        "wild/tests/integration_tests.rs",
        "wild/tests/external_tests/lld_tests.rs",
        "linker-diff/src/lib.rs",
        "linker-utils/src/elf.rs",
        "linker-layout/src/lib.rs",
        "linker-trace/src/lib.rs",
        "/abs/wild/wild/src/main.rs",
        "/home/runner/work/wild/wild/libwild/src/lib.rs",
        r"C:\work\wild\wild\tests\integration_tests.rs",
        "ui/host_cfg.rs",
        "/tmp/lint/ui/host_cfg.rs",
    ] {
        assert!(in_scope(path), "{path}");
    }
    assert!(!in_scope("wild/Cargo.toml"));
    assert!(!in_scope(
        "/home/dev/.cargo/registry/src/libc-0.2/src/lib.rs"
    ));
}

#[test]
fn tree_references_are_allowed_only_under_host() {
    assert!(inside_host_dir("libwild/src/host/fs.rs"));
    assert!(inside_host_dir("/abs/libwild/src/host/mod.rs"));
    assert!(!inside_host_dir("libwild/src/lib.rs"));
    assert!(!inside_host_dir("wild/tests/integration_tests.rs"));
    assert!(!inside_host_dir("mylibwild/src/host/fs.rs"));
}

#[test]
fn baseline_allowances_apply_to_their_file_only() {
    assert_eq!(baseline_allowance("libwild/src/fs.rs"), 0);
    for (path, allowed) in BASELINE {
        assert_eq!(baseline_allowance(path), allowed, "{path}");
        assert_eq!(baseline_allowance(&format!("/abs/wild/{path}")), allowed);
    }
}

#[test]
fn banned_selectors_are_detected_in_every_cfg_form() {
    assert_eq!(
        host_cfg_invocations("fn selected() { if cfg!(windows) {} }"),
        vec!["cfg!(windows)"]
    );
    assert_eq!(
        host_cfg_invocations("#[cfg(all(test, target_os = \"windows\"))] fn f() {}"),
        vec!["cfg(all(test,target_os=))"]
    );
    assert_eq!(
        host_cfg_invocations("#[cfg_attr(unix, allow(dead_code))] fn f() {}"),
        vec!["cfg_attr(unix,allow(dead_code))"]
    );
    assert_eq!(
        host_cfg_invocations("#![cfg(target_family = \"wasm\")]\nfn f() {}"),
        vec!["cfg(target_family=)"]
    );
    assert_eq!(
        host_cfg_invocations("#![cfg_attr(target_env = \"musl\", allow(unused))]"),
        vec!["cfg_attr(target_env=,allow(unused))"]
    );
    assert_eq!(
        host_cfg_invocations("fn f() -> bool { return cfg!(target_pointer_width = \"64\"); }"),
        vec!["cfg!(target_pointer_width=)"]
    );
    assert_eq!(
        host_cfg_invocations("# [ cfg ( target_vendor = \"apple\" ) ] fn f() {}"),
        vec!["cfg(target_vendor=)"]
    );
    assert_eq!(
        host_cfg_invocations("#[cfg(not(target_abi = \"eabihf\"))] fn f() {}"),
        vec!["cfg(not(target_abi=))"]
    );
}

#[test]
fn cfg_select_host_selection_is_detected() {
    for source in [
        "cfg_select! { unix => { fn f() {} } _ => {} }",
        "std::cfg_select! {\n    target_os = \"linux\" => { mod a; }\n    _ => { mod b; }\n}",
        "core::cfg_select!(windows => { fn f() {} } _ => {});",
        "fn f() -> u8 { cfg_select! { all(test, target_env = \"musl\") => 1, _ => 2 } }",
    ] {
        assert_eq!(
            host_cfg_invocations(source),
            vec!["cfg_select!"],
            "{source}"
        );
    }
    for source in [
        "cfg_select! { feature = \"zstd\" => { fn f() {} } _ => {} }",
        "std::cfg_select! { target_arch = \"x86_64\" => {} target_endian = \"big\" => {} }",
        "cfg_select! { my_unix_cfg => { let s = \"windows\"; } _ => {} } // unix",
        "my_cfg_select! { unix => {} }",
    ] {
        assert!(host_cfg_invocations(source).is_empty(), "{source}");
    }
}

#[test]
fn allowed_selectors_are_not_detected() {
    for source in [
        "#[cfg(feature = \"plugins\")] fn f() {}",
        "#[cfg(test)] mod tests {}",
        "#[cfg(debug_assertions)] fn f() {}",
        "#[cfg(target_arch = \"x86_64\")] fn f() {}",
        "#[cfg(any(target_arch = \"aarch64\", target_endian = \"big\"))] fn f() {}",
        "#[cfg_attr(feature = \"windows\", allow(dead_code))] fn f() {}",
        "#[cfg_attr(not(feature = \"plugins\"), path = \"linker_plugins_disabled.rs\")] mod x;",
        "fn f() -> bool { cfg!(feature = \"macho\") }",
        "cfg_select! { feature = \"zstd\" => { fn f() {} } _ => {} }",
    ] {
        assert!(host_cfg_invocations(source).is_empty(), "{source}");
    }
}

#[test]
fn selectors_match_on_identifier_boundaries() {
    for source in [
        "#[cfg(my_unix_cfg)] fn f() {}",
        "#[cfg(unix_like)] fn f() {}",
        "#[cfg(not(windows_host))] fn f() {}",
        "#[cfg(custom_target_os)] fn f() {}",
        "#[cfg(target_os_extra)] fn f() {}",
        "fn f() { my_cfg!(unix); }",
        "fn f() { notcfg!(windows); }",
    ] {
        assert!(host_cfg_invocations(source).is_empty(), "{source}");
    }
    assert_eq!(
        host_cfg_invocations("#[cfg(any(my_unix_cfg, unix))] fn f() {}"),
        vec!["cfg(any(my_unix_cfg,unix))"]
    );
    assert_eq!(
        host_cfg_invocations("fn f() { std::cfg!(windows); }"),
        vec!["cfg!(windows)"]
    );
}

#[test]
fn cfg_in_comments_strings_and_char_literals_is_ignored() {
    assert!(host_cfg_invocations(
        r####"fn neutral() {
            let _ = "#[cfg(windows)]";
            let _ = r###"cfg!(target_os = "linux")"###;
            let _ = b"cfg!(unix)";
            /* cfg!(unix) /* #[cfg(windows)] */ */
            // #[cfg(target_os = "macos")]
            let _ = '"';
            let _ = "cfg!(windows)";
        }"####
    )
    .is_empty());
    assert_eq!(
        host_cfg_invocations("fn f() { let q = '\"'; if cfg!(unix) {} let r = '\\''; }"),
        vec!["cfg!(unix)"]
    );
    assert_eq!(
        host_cfg_invocations("fn f<'a>(x: &'a str) -> bool { cfg!(windows) }"),
        vec!["cfg!(windows)"]
    );
}

#[test]
fn native_platform_references_are_detected_outside_strings() {
    assert_eq!(
        native_host_references(
            "use std::os::unix::fs::PermissionsExt; use std::os::fd::AsRawFd; libc::getpid();"
        ),
        vec!["std::os::unix", "std::os::fd", "libc::"]
    );
    assert_eq!(
        native_host_references(
            "use std::os::windows::fs::FileExt; use windows_sys::Win32::Foundation::HANDLE; \
             use windows::Win32::System; use std::os::linux::fs::MetadataExt; \
             use std::os::macos::raw; use std::os::wasi::ffi::OsStrExt;"
        ),
        vec![
            "std::os::windows",
            "std::os::linux",
            "std::os::macos",
            "std::os::wasi",
            "windows_sys",
            "windows::Win32",
        ]
    );
    let masked = "let text = \"windows_sys libc::\"; // std::os::unix";
    assert!(native_host_references(masked).is_empty());
    assert!(native_host_references(
        "use std::os::raw::c_int; use std::ffi::c_void; my_libc::f(); let windows_sysfoo = 1;"
    )
    .is_empty());
}

#[test]
fn tree_references_are_word_boundary_matched() {
    assert_eq!(
        host_tree_references("crate::host::imp::fs::x(); host::windows::y();"),
        vec!["host::imp", "host::windows"]
    );
    assert_eq!(
        host_tree_references(
            "use crate::host::unix::fs; host::other_unix::f(); host::illumos::g(); \
             host::linux::h(); host::macos::i(); host::wasi::j();"
        ),
        vec![
            "host::unix",
            "host::linux",
            "host::macos",
            "host::illumos",
            "host::other_unix",
            "host::wasi",
        ]
    );
    assert!(
        host_tree_references(
            "host::imported; host::unixish; my_host::unix; crate::host::fs::read_input(); \
             \"host::linux\"; // host::macos"
        )
        .is_empty()
    );
}
