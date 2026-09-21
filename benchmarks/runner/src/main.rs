//! An over-engineered, opinionated tool for benchmarking linkers, in particular Wild.
//!
//! Things that make this specific to linkers and/or wild.
//!
//! * It assumes benchmarks are in the form of Wild-generated save-dirs. i.e. a directory (the name
//!   of which is the name of the benchmark) where that directory contains a rust-with script.
//! * It accommodates that some of the linkers fork on startup, then do shutdown work after the
//!   linker terminates. To prevent this from affecting subsequent runs, it inserts a delay based on
//!   how long the linker took to run.
//! * It handles querying the linkers for their version to include in the report.
//! * It allows per-benchmark configuration files that can specify things like the minimum supported
//!   version of wild that can run that benchmark or skipping particular linkers for particular
//!   benchmarks.
//! * Passing --no-fork to linkers that support it when measuring memory consumption.
//!
//! Basically, this is a big script, but written in Rust, because it makes it easier to maintain.
//!
//! We avoid calling out to external tools because it wouldn't really get us much. We'd need to
//! parse their output, which would probably be more complex than doing it ourselves. We'd also have
//! less control over how exactly things are done. So we just do it ourselves.
//!
//! Benchmarking and producing reports are two separate steps. This is useful since benchmarking is
//! slow, while producing reports is very fast. By being separate, we can run the slow benchmark
//! hopefully just once, then produce the report multiple times as we tweak the presentation. The
//! format of the intermediate file uses postcard because we already had a dependency on it. It's
//! very subject to change, so is only useful for short-term storage.

use crate::config::BenchConfig;
use crate::config::Config;
use anyhow::Context;
use anyhow::bail;
use clap::Parser;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

mod benchmarking;
mod config;
mod reporting;
mod scaling;
mod system;
mod table;

const RESULT_HEADER: &[u8] = b"wild-bench-results-v3\n";

type Result<T = (), E = anyhow::Error> = std::result::Result<T, E>;

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Subcommand,
}

#[derive(clap::Subcommand, Clone)]
enum Subcommand {
    Bench(BenchArgs),
    Report(ReportArgs),
    /// Print benchmark timings as a Markdown table.
    Table(table::TableArgs),
}

#[derive(Parser, Clone)]
struct BenchArgs {
    /// Path to benchmark.toml
    #[clap(long, default_value = "benchmarks/ryzen-9955hx.toml")]
    config: PathBuf,

    /// The directory containing the savedirs. Defaults to directory containing config.
    #[clap(long)]
    saves: Option<PathBuf>,

    /// Skip initial verification that we can run each benchmark.
    #[clap(long)]
    no_verify: bool,

    /// Skip checking that the system is suitably configured for benchmarking.
    #[clap(long)]
    no_check_system: bool,

    /// Output filename template. %fs is replaced with the configured filesystem type.
    #[clap(long, default_value = "/ram/%fs/linker-benchmark-out")]
    tmp: PathBuf,

    /// Number of runs per batch.
    #[clap(long, default_value = "8")]
    batch_size: u32,

    /// Number of batches.
    #[clap(long, default_value = "10")]
    num_batches: u32,

    /// Whether to skip checking memory consumption. Unless set, then the first batch will run with
    /// --no-fork for linkers that support it.
    #[clap(long)]
    no_mem: bool,

    /// Restrict to just the specified benchmarks.
    #[clap(long, value_delimiter = ',')]
    benches: Vec<String>,

    /// Filename to write results to. If not specified, will write to
    /// `benchmarks/[benchmark-name].bench-results`.
    #[clap(long)]
    output: Option<PathBuf>,

    /// Thread counts, comma-separated and/or inclusive ranges (e.g. 1-32 or 1,2,4,8).
    #[clap(long)]
    threads: Option<ThreadCounts>,

    /// Named linker configurations from the config. Defaults to all when no binaries are supplied.
    #[clap(long, value_delimiter = ',')]
    linkers: Vec<String>,

    /// The linker binaries to benchmark.
    binaries: Vec<PathBuf>,
}

#[derive(Parser, Clone)]
struct ReportArgs {
    /// Path to benchmark.toml. Can be repeated. If not specified, runs all toml files in the
    /// benchmarks dir.
    #[clap(long)]
    config: Vec<PathBuf>,

    /// The benchmarks directory. Reports are written here. We also by default look for config
    /// here.
    #[clap(long, default_value = "benchmarks")]
    dir: PathBuf,

    /// Override the filename containing previously written results to read from. Default is the
    /// same as for `--output` on the `bench` command.
    #[clap(long)]
    input: Option<PathBuf>,

    /// Whether to print stats to stdout.
    #[clap(long)]
    print_stats: bool,

    /// Label charts with absolute values (ms for time, MiB for memory) instead of percentage
    /// changes.
    #[clap(long)]
    absolute: bool,
}

fn main() -> Result {
    let args = Args::parse();

    match args.command {
        Subcommand::Bench(bench_args) => {
            let config = config::Config::load(&bench_args.config)?;
            if !bench_args.no_check_system {
                system::check_system_settings()?;
            }
            benchmarking::run_bench(&bench_args, &config)
        }
        Subcommand::Table(table_args) => table::run(&table_args),
        Subcommand::Report(report_args) => {
            for config_path in &report_args.configs()? {
                let config = config::Config::load(config_path)?;
                reporting::run_report(&report_args, &config)?;
            }
            Ok(())
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Benchmarks {
    benchmarks: Vec<BenchmarkResult>,
}

impl Benchmarks {
    fn load(path: &Path) -> Result<Self> {
        let bytes =
            std::fs::read(path).with_context(|| format!("Failed to read `{}`", path.display()))?;
        let bytes = bytes
            .strip_prefix(RESULT_HEADER)
            .context("Unsupported benchmark results format; rerun the bench command")?;
        postcard::from_bytes(bytes).with_context(|| format!("Failed to parse `{}`", path.display()))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct BenchmarkResult {
    config: Benchmark,
    batches: Vec<BatchResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BatchResult {
    threads: Option<u32>,
    bin: Bin,
    runs: Vec<Run>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Run {
    /// The pid of the process that we timed. Within a batch, we should expect to see evenly spaced
    /// pids. If we don't, that might be a sign that the system we're running in is busy doing
    /// something (e.g. applying updates). We don't actually use this at this stage, but might in
    /// future.
    pid: u32,
    extra_flags: Vec<String>,
    memory: bool,
    elapsed: std::time::Duration,
    pub(crate) max_rss: u64,
    pub(crate) stime: Duration,
    pub(crate) utime: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Bin {
    label: Option<String>,
    flags: Vec<String>,
    index: u32,
    path: PathBuf,
    identifier: LinkerIdentifier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LinkerIdentifier {
    kind: LinkerKind,
    version: String,
    variant: Option<String>,
    /// The commit hash reported by Wild, including a modification suffix when present.
    hash: Option<String>,
    /// If we've got a hash, then this is one patch level higher than version. Empty when the
    /// version output only contains a commit hash.
    effective_version: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LinkerKind {
    Wild,
    Lld,
    Mold,
    Bfd,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Benchmark {
    name: String,
    path: PathBuf,
    config: BenchConfig,
    output: PathBuf,
    // Used only when running benchmarks, not when reporting saved results.
    #[serde(skip)]
    min_wild_version: Option<Vec<u32>>,
}

impl LinkerKind {
    fn as_str(self) -> &'static str {
        match self {
            LinkerKind::Wild => "Wild",
            LinkerKind::Lld => "LLD",
            LinkerKind::Mold => "Mold",
            LinkerKind::Bfd => "GNU ld",
        }
    }

    fn supports_arg(self, arg: &str) -> bool {
        match arg {
            "--no-fork" => matches!(self, LinkerKind::Wild | LinkerKind::Mold),
            _ => true,
        }
    }
}

impl Bin {
    fn new(bin_path: &Path, index: u32) -> Result<Self> {
        let output = Command::new(bin_path)
            .arg("--version")
            .output()
            .with_context(|| format!("Failed to run `{}`", bin_path.display()))?;

        if !output.status.success() {
            bail!(
                "{} --version failed: {}",
                bin_path.display(),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let version_line = String::from_utf8_lossy(&output.stdout)
            .to_string()
            .lines()
            .next()
            .unwrap_or_default()
            .to_owned();

        let identifier = LinkerIdentifier::parse(&version_line, bin_path)
            .with_context(|| format!("Failed to parse linker version `{version_line}`"))?;

        let label = display_version_override(bin_path)?
            .map(|version| format!("{} {version}", identifier.kind));

        Ok(Self {
            label,
            flags: Vec::new(),
            index,
            path: bin_path.to_owned(),
            identifier,
        })
    }
}

/// Looks for a .version file alongside the binary. Only works if the path to the binary is
/// supplied. This is useful for overriding a version string for a linker that isn't actually the
/// release which its `--version` flag returns - e.g. because it was built from git at some later
/// point.
fn display_version_override(bin_path: &Path) -> Result<Option<String>> {
    let mut version_path = bin_path.to_owned().into_os_string();
    version_path.push(".version");
    let version_path = PathBuf::from(version_path);

    let contents = match std::fs::read_to_string(&version_path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("Failed to read `{}`", version_path.display()));
        }
    };

    let mut lines = contents.lines();
    let version = lines.next().unwrap_or_default().trim();

    if version.is_empty() || lines.next().is_some() {
        bail!(
            "{} must contain one non-empty version line",
            version_path.display()
        );
    }
    Ok(Some(version.to_owned()))
}

impl Benchmark {
    fn new(
        name: String,
        bench_dir: &Path,
        bench_config: BenchConfig,
        template: &Path,
    ) -> Result<Benchmark> {
        let filesystem = bench_config.filesystem();
        let min_wild_version = bench_config
            .min_wild_version
            .as_deref()
            .map(parse_version_number)
            .transpose()?;
        let output = std::path::absolute(
            template
                .to_str()
                .context("Output template must be UTF-8")?
                .replace("%fs", filesystem),
        )?;
        let path = bench_dir.join("run-with");
        if !path.exists() {
            bail!("{} doesn't exist", path.display())
        }

        Ok(Benchmark {
            name,
            path,
            config: bench_config,
            output,
            min_wild_version,
        })
    }

    fn supports_wild_version(&self, wild_version: &[u32]) -> bool {
        if wild_version.is_empty() {
            return true;
        }

        let Some(min_required) = &self.min_wild_version else {
            return true;
        };

        wild_version >= min_required.as_slice()
    }

    fn supports_bin(&self, bin: &Bin) -> bool {
        if self.config.skip_linkers.contains(&bin.identifier.kind) {
            return false;
        }
        if bin.identifier.kind == LinkerKind::Wild {
            return self.supports_wild_version(&bin.identifier.effective_version);
        }
        true
    }
}

impl LinkerIdentifier {
    fn parse(version_line: &str, bin_path: &Path) -> Option<Self> {
        let kind;
        let version;
        let mut hash = None;
        let mut variant = None;

        if let Some(mut rest) = version_line.strip_prefix("Wild ") {
            let legacy = rest.starts_with("version ");
            if let Some(r) = rest.strip_prefix("version ") {
                rest = r;
            }
            version = take_word(&mut rest).to_owned();
            if version.len() == 40 && version.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Some(Self {
                    kind: LinkerKind::Wild,
                    hash: Some(version.clone()),
                    version,
                    variant: None,
                    effective_version: Vec::new(),
                });
            }
            let token = take_word(&mut rest).trim_matches(['(', ')']);
            let commit = token.strip_suffix("-modified").unwrap_or(token);
            if (7..=40).contains(&commit.len()) && commit.bytes().all(|b| b.is_ascii_hexdigit()) {
                // Older releases include a hash too; release archive paths identify their version.
                let release = legacy
                    && !token.ends_with("-modified")
                    && bin_path.components().any(|component| {
                        component.as_os_str().to_str().is_some_and(|name| {
                            name.split(|c: char| !c.is_ascii_alphanumeric() && c != '.')
                                .any(|part| part == version)
                        })
                    });
                if !release {
                    hash = Some(token.to_owned());
                }
            } else if token == "non-git-build" {
                variant = Some(token.to_owned());
            }

            kind = LinkerKind::Wild;
        } else if let Some(mut rest) = version_line.strip_prefix("LLD ") {
            kind = LinkerKind::Lld;
            version = take_word(&mut rest).to_owned();
        } else if let Some(mut rest) = version_line.strip_prefix("Ubuntu LLD ") {
            kind = LinkerKind::Lld;
            version = take_word(&mut rest).to_owned();
            variant = Some("Ubuntu".to_owned());
        } else if let Some(mut rest) = version_line.strip_prefix("Debian LLD ") {
            kind = LinkerKind::Lld;
            version = take_word(&mut rest).to_owned();
            variant = Some("Debian".to_owned());
        } else if let Some(mut rest) = version_line.strip_prefix("mold ") {
            kind = LinkerKind::Mold;
            version = take_word(&mut rest).to_owned();
        } else {
            let rest = version_line.strip_prefix("GNU ld (")?;
            let (distribution, mut rest) = rest.split_once(") ")?;
            kind = LinkerKind::Bfd;
            version = take_word(&mut rest).to_owned();
            let distribution = distribution
                .strip_prefix("GNU Binutils")
                .unwrap_or(distribution)
                .trim();
            let distribution = distribution.strip_prefix("for ").unwrap_or(distribution);
            if !distribution.is_empty() {
                variant = Some(distribution.to_owned());
            }
        }

        let mut effective_version = parse_version_number(&version).ok()?;

        if hash.is_some()
            && let Some(patch) = effective_version.last_mut()
        {
            *patch += 1;
        }

        Some(LinkerIdentifier {
            kind,
            version,
            variant,
            hash,
            effective_version,
        })
    }

    fn name_parts(&self) -> Vec<String> {
        let mut parts = Vec::new();
        parts.push(self.kind.to_string());
        if let Some(hash) = &self.hash {
            parts.push(hash.chars().take(8).collect());
        } else {
            parts.push(self.version.clone());
        }
        if let Some(variant) = &self.variant {
            parts.push(variant.clone());
        }
        parts
    }
}

fn take_word<'a>(input: &mut &'a str) -> &'a str {
    *input = input.trim();
    let i = input.find(' ').unwrap_or(input.len());
    let (word, rest) = input.split_at(i);
    *input = rest;
    word
}

impl Display for Bin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(label) = &self.label {
            write!(f, "{label}")
        } else {
            write!(f, "{}", self.identifier)
        }
    }
}

impl Display for LinkerIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)?;

        if let Some(hash) = &self.hash {
            let prefix: String = hash.chars().take(8).collect();
            write!(f, " {prefix}")?;
        } else {
            write!(f, " {}", self.version)?;
        }

        if let Some(variant) = &self.variant {
            write!(f, " {variant}")?;
        }
        Ok(())
    }
}

impl Display for LinkerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Display for Benchmark {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

fn parse_version_number(v: &str) -> Result<Vec<u32>> {
    v.split('.')
        .map(|p| {
            p.parse()
                .with_context(|| format!("Failed to parse version `{v}`"))
        })
        .collect()
}

fn default_result_path(config: &Config, path_buf: Option<&PathBuf>) -> PathBuf {
    path_buf.cloned().unwrap_or_else(|| {
        PathBuf::from("benchmarks").join(format!("{}.bench-results", config.name))
    })
}

impl ReportArgs {
    fn configs(&self) -> Result<Vec<PathBuf>> {
        if !self.config.is_empty() {
            return Ok(self.config.clone());
        }

        let dir = std::fs::read_dir(&self.dir)
            .with_context(|| format!("Failed to read --dir `{}`", self.dir.display()))?;

        Ok(dir
            .filter_map(|ent| ent.ok())
            .map(|ent| ent.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version_less_than(a: &str, b: &str) -> bool {
        let (Ok(a), Ok(b)) = (parse_version_number(a), parse_version_number(b)) else {
            return false;
        };
        a < b
    }

    #[test]
    fn test_version_comparison() {
        assert!(version_less_than("0.5.0", "0.6.0"));
        assert!(!version_less_than("0.5.0", "0.5.0"));
        assert!(!version_less_than("0.6.0", "0.5.0"));
        assert!(version_less_than("0.5.0", "0.10.0"));
    }
}

#[derive(Clone, Debug)]
struct ThreadCounts(Vec<u32>);

impl std::str::FromStr for ThreadCounts {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self> {
        let mut counts = std::collections::BTreeSet::new();
        for part in input.split(',') {
            let part = part.trim();
            let (start, end) = part.split_once('-').unwrap_or((part, part));
            let start: u32 = start.parse().context("Invalid thread count")?;
            let end: u32 = end.parse().context("Invalid thread count")?;
            anyhow::ensure!(
                start > 0 && start <= end,
                "Thread ranges must be positive and ascending"
            );
            counts.extend(start..=end);
        }
        Ok(Self(counts.into_iter().collect()))
    }
}
