//! Parser for the Text-Based Stub (`.tbd`) library definitions used by Mach-O.
//!
//! This crate currently targets TBD format version 4 and extracts the
//! library versions and linker-visible symbol definitions (and weak symbols). To keep parsing
//! simple and efficient, the parser rejects escape sequences and returns
//! `&'data str` slices directly from the input.
//!
//! The parser accepts multi-document YAML TBD files. The first document is
//! treated as the main library. Additional documents are treated as child
//! libraries reexported by the main library. This covers a practical subset of
//! the full TBD v4 format, including the shape used by most system libraries.

use crate::args::macho::SemanticVersion;
use crate::ensure;
use crate::error::Context as _;
use crate::error::Result;
use itertools::Itertools;
use object::macho::Version;
use rayon::iter::IntoParallelRefIterator as _;
use rayon::iter::ParallelIterator as _;
use serde::Deserialize;
use std::collections::HashSet;

const ARM64_LIB_ARCH: &str = "arm64e-macos";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct TextBasedDefinition<'a> {
    tbd_version: u32,
    #[serde(borrow)]
    targets: Vec<&'a str>,
    #[serde(borrow)]
    install_name: &'a str,
    #[serde(default = "default_library_version")]
    current_version: &'a str,
    #[serde(default = "default_library_version")]
    compatibility_version: &'a str,
    #[serde(default)]
    parent_umbrella: Vec<ParentUmbrella<'a>>,
    #[serde(default)]
    reexported_libraries: Vec<ReexportedLibraries<'a>>,
    #[serde(default)]
    exports: Vec<Exports<'a>>,
    #[serde(default)]
    reexports: Vec<Exports<'a>>,
}

fn default_library_version() -> &'static str {
    "1.0.0"
}

impl<'a> TextBasedDefinition<'a> {
    fn all_exports(&self) -> impl Iterator<Item = &Exports<'a>> {
        self.exports.iter().chain(&self.reexports)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct ParentUmbrella<'a> {
    #[serde(borrow)]
    targets: Vec<&'a str>,
    #[serde(borrow)]
    umbrella: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct ReexportedLibraries<'a> {
    #[serde(borrow)]
    targets: Vec<&'a str>,
    #[serde(borrow)]
    libraries: Vec<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Exports<'a> {
    #[serde(borrow)]
    targets: Vec<&'a str>,
    #[serde(default)]
    #[serde(borrow)]
    symbols: Vec<&'a str>,
    #[serde(default)]
    #[serde(borrow)]
    weak_symbols: Vec<&'a str>,
}
// TODO: remove
#[allow(unused)]
#[derive(Debug, Clone)]
pub(crate) struct DefinedStubLibrary<'a> {
    /// Install name of the dynamic library, including its `.dylib` suffix.    
    pub(crate) install_name: &'a str,
    /// Current version of the main library, defaulting to 1.0.0 when omitted.
    pub(crate) current_version: Version,
    /// Compatibility version of the main library, defaulting to 1.0.0 when omitted.
    pub(crate) compatibility_version: Version,
    /// Global symbols defined by the library or by any reexported child library.
    pub(crate) symbols: Vec<&'a str>,
    /// Weak symbols defined by the library or by any reexported child library.
    pub(crate) weak_symbols: Vec<&'a str>,
}

impl DefinedStubLibrary<'_> {
    pub(crate) fn total_symbols(&self) -> usize {
        self.symbols.len() + self.weak_symbols.len()
    }
}

pub fn parse_defined_library<'data>(input: &'data str) -> Result<DefinedStubLibrary<'data>> {
    // We can benefit from a parallel parsing of some of the commonly used libraries
    // (e.g. libSystem.B.tbd has 40 sub-libraries). On the other hand, a commonly used one
    // `libc++.1.tbd` contains just a single library.
    let documents = input
        .split("--- !tapi-tbd")
        .filter(|document| !document.trim().is_empty())
        .collect_vec();
    let library_definitions = documents
        .par_iter()
        .map(|document| yaml_serde::from_str::<TextBasedDefinition<'data>>(document))
        .collect::<Result<Vec<_>, _>>()?;

    let main_library = library_definitions
        .first()
        .context("root library must be defined")?;
    ensure!(
        main_library.targets.contains(&ARM64_LIB_ARCH),
        "Library only supports {targets:?}, but we need {ARM64_LIB_ARCH}",
        targets = main_library.targets,
    );

    let mut defined_library = DefinedStubLibrary {
        install_name: main_library.install_name,
        current_version: SemanticVersion::try_from(main_library.current_version)
            .context("Invalid TBD current-version")?
            .get(),
        compatibility_version: SemanticVersion::try_from(main_library.compatibility_version)
            .context("Invalid TBD compatibility-version")?
            .get(),
        symbols: Vec::with_capacity(
            library_definitions
                .iter()
                .flat_map(TextBasedDefinition::all_exports)
                .map(|exp| exp.symbols.len())
                .sum(),
        ),
        weak_symbols: Vec::with_capacity(
            library_definitions
                .iter()
                .flat_map(TextBasedDefinition::all_exports)
                .map(|exp| exp.weak_symbols.len())
                .sum(),
        ),
    };

    // Main libraries commonly reexport symbols from child libraries. This parser
    // currently supports only a flat tree: one main library with leaf children.
    let exported_libraries = if let Some(exported_libraries) = main_library
        .reexported_libraries
        .iter()
        .at_most_one()
        .context("expected just a single exported library")?
    {
        ensure!(
            exported_libraries.targets.contains(&ARM64_LIB_ARCH),
            "Exported library only supports {:?}, but we need {ARM64_LIB_ARCH}",
            exported_libraries.targets
        );
        let exported_libraries: HashSet<_> = exported_libraries.libraries.iter().copied().collect();
        exported_libraries
    } else {
        HashSet::new()
    };

    for lib in &library_definitions {
        ensure!(
            lib.tbd_version == 4,
            "TBD version 4 expected, got {}",
            lib.tbd_version
        );
        if lib != main_library {
            ensure!(
                exported_libraries.contains(lib.install_name),
                "child library '{}' not listed as reexported by the main library",
                lib.install_name
            );
        }

        for export in lib.all_exports() {
            if export.targets.contains(&ARM64_LIB_ARCH) {
                defined_library.symbols.extend(export.symbols.iter());
                defined_library
                    .weak_symbols
                    .extend(export.weak_symbols.iter());
            }
        }
    }

    Ok(defined_library)
}

#[cfg(test)]
mod tests {
    use super::*;

    const VERSION_TEST_HEADER: &str = "--- !tapi-tbd\ntbd-version: 4\ntargets: [ arm64e-macos ]\ninstall-name: /usr/lib/libVersions.dylib\n";

    #[test]
    fn library_versions() {
        for (fields, current, compatibility) in [
            ("", Version::new(1, 0, 0), Version::new(1, 0, 0)),
            (
                "current-version: 3.2.1\n",
                Version::new(3, 2, 1),
                Version::new(1, 0, 0),
            ),
            (
                "compatibility-version: 2.1\n",
                Version::new(1, 0, 0),
                Version::new(2, 1, 0),
            ),
            (
                "current-version: '3.2'\ncompatibility-version: '2'\n",
                Version::new(3, 2, 0),
                Version::new(2, 0, 0),
            ),
            (
                "current-version: 0\ncompatibility-version: 0\n",
                Version(0),
                Version(0),
            ),
            (
                "current-version: 65535.255.255\ncompatibility-version: 65535.255.255\n",
                Version::new(65535, 255, 255),
                Version::new(65535, 255, 255),
            ),
        ] {
            let input = format!("{VERSION_TEST_HEADER}{fields}...\n");
            let library = parse_defined_library(&input).unwrap();
            assert_eq!(library.current_version, current, "{fields}");
            assert_eq!(library.compatibility_version, compatibility, "{fields}");
        }
    }

    #[test]
    fn invalid_library_versions() {
        for field in ["current-version", "compatibility-version"] {
            for value in ["", "abc", "-1", "1.2.3.4", "65536", "1.256", "1.2.256"] {
                let input = format!("{VERSION_TEST_HEADER}{field}: '{value}'\n...\n");
                let error = parse_defined_library(&input).unwrap_err().to_string();
                assert!(error.contains(&format!("Invalid TBD {field}")), "{error}");
            }
        }
    }

    #[test]
    fn parse_library_with_reexports() {
        let stub_library = parse_defined_library(
            r"--- !tapi-tbd
tbd-version:     4
targets:         [ x86_64-macos, arm64e-macos ]
install-name:    '/usr/lib/libMain.dylib'
current-version: 1.2.3
compatibility-version: 1.1
reexported-libraries:
  - targets:         [ x86_64-macos, arm64e-macos ]
    libraries:       [ '/usr/lib/libA.dylib', '/usr/lib/libB.dylib' ]
exports:
  - targets:         [ arm64e-macos ]
    symbols:         [ _main_arm64 ]
    weak-symbols:    [ _main_weak_arm64 ]
  - targets:         [ x86_64-macos ]
    symbols:         [ _main_x86_64 ]
    weak-symbols:    [ _main_weak_x86_64 ]
--- !tapi-tbd
tbd-version:     4
targets:         [ x86_64-macos, arm64e-macos ]
install-name:    '/usr/lib/libA.dylib'
current-version: 10
compatibility-version: 9
parent-umbrella:
  - targets:         [ x86_64-macos, arm64e-macos ]
    umbrella:        Main
exports:
  - targets:         [ arm64e-macos ]
    symbols:         [ _a_arm64 ]
    weak-symbols:    [ _a_weak_arm64 ]
  - targets:         [ x86_64-macos ]
    symbols:         [ _a_x86_64 ]
--- !tapi-tbd
tbd-version:     4
targets:         [ x86_64-macos, arm64e-macos ]
install-name:    '/usr/lib/libB.dylib'
current-version: 11
parent-umbrella:
  - targets:         [ x86_64-macos, arm64e-macos ]
    umbrella:        Main
exports:
  - targets:         [ arm64e-macos ]
    symbols:         [ _b_arm64 ]
reexports:
  - targets:         [ arm64e-macos ]
    symbols:         [ _b_exported_arm64 ]
    weak-symbols:    [ _b_weak_exported_arm64 ]
",
        )
        .expect("definition should parse");

        assert_eq!(stub_library.install_name, "/usr/lib/libMain.dylib");
        assert_eq!(stub_library.current_version, Version::new(1, 2, 3));
        assert_eq!(stub_library.compatibility_version, Version::new(1, 1, 0));
        assert_eq!(
            stub_library.symbols,
            ["_main_arm64", "_a_arm64", "_b_arm64", "_b_exported_arm64"]
        );
        assert_eq!(
            stub_library.weak_symbols,
            [
                "_main_weak_arm64",
                "_a_weak_arm64",
                "_b_weak_exported_arm64"
            ]
        );
    }
}
