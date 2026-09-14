use crate::LinkerKind;
use crate::Result;
use anyhow::Context as _;
use anyhow::bail;
use anyhow::ensure;
use indexmap::IndexMap;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Config {
    pub(crate) name: String,

    #[serde(default)]
    pub(crate) linkers: IndexMap<String, LinkerConfig>,

    #[serde(default)]
    pub(crate) defaults: BenchConfig,

    #[serde(default, rename = "bench")]
    pub(crate) benches: IndexMap<String, BenchConfig>,

    #[serde(skip)]
    pub(crate) filename: PathBuf,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct BenchConfig {
    #[serde(default)]
    pub(crate) skip: bool,
    pub(crate) min_wild_version: Option<String>,
    #[serde(default)]
    pub(crate) skip_linkers: Vec<LinkerKind>,
    pub(crate) filesystem: Option<String>,
    #[serde(default, rename = "delete-output")]
    pub(crate) delete_output: Option<bool>,
    #[serde(default)]
    pub(crate) flags: Vec<String>,
    #[serde(default, rename = "linker-flags")]
    pub(crate) linker_flags: BTreeMap<LinkerKind, Vec<String>>,
    #[serde(default)]
    pub(crate) variants: IndexMap<String, BenchConfig>,
}

impl Config {
    pub(crate) fn load(config_path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(config_path)
            .with_context(|| format!("Failed to read `{}`", config_path.display()))?;

        let mut config: Config = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse `{}`", config_path.display()))?;

        config.filename = config_path.to_path_buf();

        Ok(config)
    }
}

impl BenchConfig {
    pub(crate) fn filesystem(&self) -> &str {
        self.filesystem.as_deref().unwrap_or("tmpfs")
    }

    fn inherit(&self, parent: &Self) -> Self {
        let mut flags = parent.flags.clone();
        flags.extend(self.flags.iter().cloned());
        let mut linker_flags = parent.linker_flags.clone();
        for (linker, flags) in &self.linker_flags {
            linker_flags
                .entry(*linker)
                .or_default()
                .extend(flags.iter().cloned());
        }
        let mut skip_linkers = parent.skip_linkers.clone();

        skip_linkers.extend_from_slice(&self.skip_linkers);

        Self {
            skip: parent.skip || self.skip,
            min_wild_version: self
                .min_wild_version
                .clone()
                .or_else(|| parent.min_wild_version.clone()),
            skip_linkers,
            filesystem: self
                .filesystem
                .clone()
                .or_else(|| parent.filesystem.clone()),
            delete_output: self.delete_output.or(parent.delete_output),
            flags,
            linker_flags,
            variants: IndexMap::new(),
        }
    }
}

impl Config {
    pub(crate) fn expanded(&self) -> Result<IndexMap<String, (String, BenchConfig)>> {
        ensure!(
            self.defaults.variants.is_empty(),
            "defaults cannot contain variants"
        );

        let mut out = IndexMap::new();

        for (directory, bench) in &self.benches {
            validate_name(directory)?;
            let base = bench.inherit(&self.defaults);

            let entries = if bench.variants.is_empty() {
                vec![(directory.clone(), base)]
            } else {
                let mut entries = Vec::new();
                for (variant, settings) in &bench.variants {
                    validate_name(variant)?;
                    if !settings.variants.is_empty() {
                        bail!("Nested variants are not permitted: {directory}.{variant}");
                    }
                    entries.push((format!("{directory}.{variant}"), settings.inherit(&base)));
                }
                entries
            };

            for (name, settings) in entries {
                validate_name(settings.filesystem())?;

                if out
                    .insert(name.clone(), (directory.clone(), settings))
                    .is_some()
                {
                    bail!("Duplicate benchmark name: {name}");
                }
            }
        }
        Ok(out)
    }
}

fn validate_name(name: &str) -> Result {
    if name.is_empty()
        || name == "."
        || name == ".."
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
    {
        bail!("Invalid benchmark, variant or filesystem name: {name:?}");
    }
    Ok(())
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct LinkerConfig {
    pub(crate) binary: PathBuf,
    #[serde(default)]
    pub(crate) flags: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_configuration_order() -> Result {
        let config: Config = toml::from_str(
            r#"
name = "ordering"
[linkers.z]
binary = "wild"
[linkers.a]
binary = "mold"
[bench.z.variants.z]
[bench.z.variants.a]
[bench.a]
"#,
        )?;
        assert_eq!(
            config
                .linkers
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["z", "a"]
        );
        let expanded = config.expanded()?;
        assert_eq!(
            expanded.keys().map(String::as_str).collect::<Vec<_>>(),
            ["z.z", "z.a", "a"]
        );
        let bytes = postcard::to_stdvec(&config)?;
        let restored: Config = postcard::from_bytes(&bytes)?;
        assert_eq!(
            restored.linkers.keys().collect::<Vec<_>>(),
            config.linkers.keys().collect::<Vec<_>>()
        );
        assert_eq!(
            restored.expanded()?.keys().collect::<Vec<_>>(),
            expanded.keys().collect::<Vec<_>>()
        );
        Ok(())
    }
}
