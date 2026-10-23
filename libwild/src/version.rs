const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Copy)]
pub struct WildVersionInfo {
    /// What this linker is called. Generally "wild", but could be different when embedding
    /// libwild.
    pub name: &'static str,

    /// The release commit for clean tagged builds, otherwise the merge base with origin/main.
    pub commit_hash: Option<&'static str>,

    /// The date of `commit_hash` in YYYY-MM-DD format.
    pub commit_date: Option<&'static str>,

    /// Whether this is a clean build of a tag matching the package version.
    pub has_tag: bool,

    /// Whether the working tree differs from the commit identified by `commit_hash`.
    pub modified: bool,
}

pub const DEFAULT_LINKER_NAME: &str = "wild";

impl std::fmt::Display for WildVersionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            writeln!(f, "linker: {}", self.name)?;
            writeln!(f, "version: {PACKAGE_VERSION}")?;
            writeln!(f, "hash: {}", self.commit_hash.unwrap_or("unknown"))?;
            writeln!(f, "commit_date: {}", self.commit_date.unwrap_or("unknown"))?;
            writeln!(f, "has_tag: {}", self.has_tag)?;
            writeln!(f, "modified: {}", self.modified)?;
            write!(f, "features: ")?;
            let mut separator = "";
            for (name, enabled) in [
                ("fork", cfg!(feature = "fork")),
                ("macho", cfg!(feature = "macho")),
                ("perfetto", cfg!(feature = "perfetto")),
                ("plugins", cfg!(feature = "plugins")),
                ("wasm", cfg!(feature = "wasm")),
                ("wip", cfg!(feature = "wip")),
                ("zstd", cfg!(feature = "zstd")),
            ] {
                if enabled {
                    write!(f, "{separator}{name}")?;
                    separator = ", ";
                }
            }
            return writeln!(f);
        }

        if self.has_tag && !self.modified {
            f.write_str(PACKAGE_VERSION)
        } else if let Some(hash) = self.commit_hash {
            if self.modified {
                write!(f, "{PACKAGE_VERSION} {hash}-modified")
            } else {
                f.write_str(hash)
            }
        } else {
            write!(f, "{PACKAGE_VERSION} non-git-build")
        }
    }
}
