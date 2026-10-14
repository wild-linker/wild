use crate::BatchResult;
use crate::BenchArgs;
use crate::Benchmark;
use crate::BenchmarkResult;
use crate::Benchmarks;
use crate::Bin;
use crate::LinkerKind;
use crate::Result;
use crate::Run;
use crate::config::Config;
use anyhow::Context as _;
use anyhow::bail;
use std::collections::HashSet;
use std::io::Read as _;
use std::process::Command;
use std::process::Stdio;
use std::time::Instant;
use wait4::Wait4 as _;

pub(crate) fn run_bench(args: &BenchArgs, config: &Config) -> Result {
    let mut bins = args
        .binaries
        .iter()
        .enumerate()
        .map(|(i, bin_path)| Bin::new(bin_path, i as u32))
        .collect::<Result<Vec<Bin>>>()?;

    let names = if args.linkers.is_empty() && bins.is_empty() {
        config.linkers.keys().cloned().collect::<Vec<_>>()
    } else {
        args.linkers.clone()
    };
    for name in names {
        let settings = config
            .linkers
            .get(&name)
            .with_context(|| format!("Unknown linker configuration: {name}"))?;
        let mut bin = Bin::new(&settings.binary, bins.len() as u32)?;
        bin.label = Some(name);
        bin.flags = settings.flags.clone();
        bins.push(bin);
    }

    if bins.is_empty() {
        bail!("Need at least one binary");
    }

    let benchmarks = find_benchmarks(args, config)?;

    let benchmarks = filter_benchmarks_by_wild_version(benchmarks, &bins);

    for bench in &benchmarks {
        if bench.config.has_explicit_threads() {
            for bin in benchmark_bins(&bins, bench) {
                anyhow::ensure!(
                    bin.identifier.kind != LinkerKind::Bfd,
                    "Thread sweeps are not supported for GNU ld"
                );
                for flag in configured_flags(bin, bench) {
                    anyhow::ensure!(
                        !is_thread_flag(flag),
                        "Thread sweep conflicts with {flag:?} for {bin} / {bench}"
                    );
                }
            }
        }
        check_filesystem(bench)?;
    }

    println!("Binaries:");
    for bin in &bins {
        println!("  {bin}");
    }

    println!("Benchmarks:");
    for bench in &benchmarks {
        println!("  {bench}");
    }

    if !args.no_verify {
        verify(&bins, &benchmarks)?;
    }

    let results = run(&bins, &benchmarks, args)?;

    let output_path = crate::default_result_path(config, args.output.as_ref());

    let mut bytes = crate::RESULT_HEADER.to_vec();
    bytes.extend(postcard::to_stdvec(&results)?);
    std::fs::write(&output_path, bytes)
        .with_context(|| format!("Failed to write `{}`", output_path.display()))?;

    Ok(())
}

fn check_filesystem(bench: &Benchmark) -> Result {
    let target = if bench.output.exists() && !bench.config.delete_output.unwrap_or(false) {
        bench.output.as_path()
    } else {
        bench
            .output
            .parent()
            .context("Output must have a parent directory")?
    };

    let target = target.canonicalize().with_context(|| {
        format!(
            "Invalid output location for {}: {}",
            bench.name,
            target.display()
        )
    })?;

    let output = Command::new("findmnt")
        .args([
            "--first-only",
            "--noheadings",
            "--output",
            "FSTYPE",
            "--target",
        ])
        .arg(&target)
        .output()
        .context("Failed to run findmnt")?;

    if !output.status.success() {
        bail!(
            "Failed to determine filesystem for {}: {}",
            target.display(),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let actual = String::from_utf8_lossy(&output.stdout);
    let expected = bench.config.filesystem();

    if actual.trim() != expected {
        bail!(
            "{}: output {} is on {}, expected {}",
            bench.name,
            bench.output.display(),
            actual.trim(),
            expected
        );
    }
    Ok(())
}

fn remove_output(bench: &Benchmark) -> Result {
    if bench.config.delete_output.unwrap_or(false) {
        match std::fs::remove_file(&bench.output) {
            Ok(()) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => {
                return Err(e).with_context(|| {
                    format!("Failed to delete output {}", bench.output.display())
                });
            }
        }
    }
    Ok(())
}

fn run(bins: &[Bin], benchmarks: &[Benchmark], args: &BenchArgs) -> Result<Benchmarks> {
    let mut out = Vec::new();
    let start = Instant::now();

    for (bench_index, bench) in benchmarks.iter().enumerate() {
        let cases = benchmark_cases(bins, bench);
        let bench_start = Instant::now();
        let message = format!(
            "Benchmark {} of {}: {bench}",
            bench_index + 1,
            benchmarks.len()
        );

        let progress_bar = indicatif::ProgressBar::new(u64::from(
            args.num_batches * args.batch_size * cases.len() as u32,
        ))
        .with_style(indicatif::ProgressStyle::with_template(
            "{msg} {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}]",
        )?)
        .with_message(message.clone());

        if cases.is_empty() {
            progress_bar.finish_and_clear();
            continue;
        }
        for &(bin, threads) in &cases {
            run_once(bin, bench, &[], threads)?;
        }
        let mut bench_results = Vec::new();
        for batch_num in 0..args.num_batches {
            for offset in 0..cases.len() {
                let (bin, threads) = cases[(offset + batch_num as usize) % cases.len()];
                let mut bin_results = Vec::new();
                for _ in 0..args.batch_size {
                    let extra_flags = if !args.no_mem && batch_num == 0 {
                        ["--no-fork"].as_slice()
                    } else {
                        &[]
                    };
                    bin_results.push(run_once(bin, bench, extra_flags, threads)?);
                    progress_bar.inc(1);
                }
                bench_results.push(BatchResult {
                    bin: bin.clone(),
                    threads,
                    runs: bin_results,
                });
            }
        }
        bench_results.sort_by_key(|b| b.bin.index);
        let r = BenchmarkResult {
            config: bench.clone(),
            batches: bench_results,
        };
        out.push(r);
        progress_bar.finish_and_clear();
        println!("{message}: done in {} s", bench_start.elapsed().as_secs());
    }

    let elapsed = start.elapsed();
    println!(
        "All done in {}h {}m {}s",
        elapsed.as_secs() / 3600,
        (elapsed.as_secs() / 60) % 60,
        elapsed.as_secs() % 60
    );

    Ok(Benchmarks { benchmarks: out })
}

/// Runs each benchmark once with each linker.
fn verify(bins: &[Bin], benchmarks: &[Benchmark]) -> Result {
    let mut success = true;
    for bench in benchmarks {
        println!("Verifying: {bench}");
        for (bin, threads) in benchmark_cases(bins, bench) {
            if let Err(error) = run_once(bin, bench, &[], threads) {
                eprintln!("{error}");
                success = false;
            }
        }
    }

    if !success {
        bail!("One or more benchmark/linker combinations failed");
    }

    Ok(())
}

fn benchmark_cases<'a>(bins: &'a [Bin], bench: &'a Benchmark) -> Vec<(&'a Bin, Option<u32>)> {
    bench
        .config
        .thread_counts()
        .into_iter()
        .flat_map(|threads| benchmark_bins(bins, bench).map(move |bin| (bin, threads)))
        .collect()
}

fn benchmark_bins<'a>(bins: &'a [Bin], bench: &'a Benchmark) -> impl Iterator<Item = &'a Bin> {
    bins.iter().filter(move |bin| {
        bench.supports_bin(bin)
            && (!bench.config.latest_only.unwrap_or(false)
                || !bins.iter().any(|other| {
                    other.identifier.kind == bin.identifier.kind
                        && bench.supports_bin(other)
                        && version_key(other) > version_key(bin)
                }))
    })
}

fn version_key(bin: &Bin) -> (bool, &[u32]) {
    let version = &bin.identifier.effective_version;
    // Legacy hash-only Wild builds have no numeric version to compare against releases.
    (version.is_empty(), version)
}

fn configured_flags<'a>(bin: &'a Bin, bench: &'a Benchmark) -> impl Iterator<Item = &'a String> {
    bench
        .config
        .flags
        .iter()
        .chain(
            bench
                .config
                .linker_flags
                .get(&bin.identifier.kind)
                .into_iter()
                .flatten(),
        )
        .chain(&bin.flags)
}

fn run_once(
    bin: &Bin,
    bench: &Benchmark,
    extra_flags: &[&str],
    threads: Option<u32>,
) -> Result<Run> {
    let mut flags: Vec<_> = configured_flags(bin, bench).cloned().collect();
    if let Some(threads) = threads {
        flags.push(format!("--threads={threads}"));
    }
    flags.extend(
        extra_flags
            .iter()
            .filter(|f| bin.identifier.kind.supports_arg(f))
            .map(|f| (*f).to_owned()),
    );

    remove_output(bench)?;
    let mut command = Command::new(&bench.path);
    command
        .env("OUT", &bench.output)
        .arg(&bin.path)
        .args(&flags);

    let (mut pipe_read, pipe_write) = std::io::pipe()?;
    command
        .stderr(pipe_write.try_clone()?)
        .stdout(pipe_write)
        .stdin(Stdio::null());

    let start = Instant::now();

    let child = command
        .spawn()
        .with_context(|| format!("Failed to run {command:?}"))?;

    // Ensure we're not holding any copies of the write-end of the pipe in the parent process,
    // otherwise the read below won't terminate.
    command.stdout(Stdio::null());
    command.stderr(Stdio::null());

    let mut text_out = Vec::new();
    let read_result = pipe_read.read_to_end(&mut text_out);

    let pid = child.id();
    let wait_result = child.wait4();
    let elapsed = start.elapsed();
    remove_output(bench)?;
    read_result?;
    let res_use = wait_result?;
    let text_out = String::from_utf8_lossy(&text_out);

    if !res_use.status.success() {
        bail!("Error returned from {command:?}\n{text_out}")
    }

    // Make sure that the linker runs without warning. Specifically what we care about is that the
    // linker is being invoked without any flags that it doesn't properly support, since that might
    // be unfair to other linkers that do support that option.
    if text_out.contains("WARN") || text_out.contains("warning:") {
        bail!("Command produced warnings: {command:?}\n{text_out}");
    }

    // However long we took to run, sleep for half of that. If the linker forked on startup, then
    // this gives the subprocess a chance to shutdown in the background before we run the next
    // command.
    std::thread::sleep(elapsed / 2);

    Ok(Run {
        pid,
        extra_flags: flags,
        memory: extra_flags.contains(&"--no-fork"),
        elapsed,
        max_rss: res_use.rusage.maxrss,
        stime: res_use.rusage.stime,
        utime: res_use.rusage.utime,
    })
}

fn find_benchmarks(args: &BenchArgs, config: &Config) -> Result<Vec<Benchmark>> {
    let dir = args
        .saves
        .as_deref()
        .or_else(|| config.filename.parent())
        .context("Missing --saves")?;

    let mut benchmarks = Vec::new();

    let expanded = config.expanded()?;
    for requested in &args.benches {
        if !expanded.contains_key(requested) && !config.benches.contains_key(requested) {
            bail!("Unknown benchmark: {requested}");
        }
    }

    let keep: HashSet<&str> = args.benches.iter().map(String::as_str).collect();
    for (name, (directory, settings)) in expanded {
        if !settings.skip
            && (keep.is_empty()
                || keep.contains(name.as_str())
                || keep.contains(directory.as_str()))
        {
            benchmarks.push(Benchmark::new(
                name,
                &dir.join(directory),
                settings,
                &args.tmp,
            )?);
        }
    }

    Ok(benchmarks)
}

/// Filter benchmarks to just those that have at least one supported Wild version.
fn filter_benchmarks_by_wild_version(benchmarks: Vec<Benchmark>, bins: &[Bin]) -> Vec<Benchmark> {
    let wild_versions: Vec<_> = bins
        .iter()
        .filter(|bin| bin.identifier.kind == LinkerKind::Wild)
        .map(|bin| &bin.identifier.effective_version)
        .collect();
    if wild_versions.is_empty() {
        return benchmarks;
    }

    benchmarks
        .into_iter()
        .filter(|bench| {
            let supported = wild_versions
                .iter()
                .any(|version| bench.supports_wild_version(version));
            if !supported {
                println!("Skipping benchmark {bench} due to minimum version requirement");
            }
            supported
        })
        .collect()
}

fn is_thread_flag(flag: &str) -> bool {
    matches!(
        flag,
        "--threads" | "--no-threads" | "-threads" | "-no-threads" | "--thread-count"
    ) || flag.starts_with("--threads=")
        || flag.starts_with("-threads=")
        || flag.starts_with("--thread-count=")
        || flag.starts_with("-j")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latest_versions_per_linker() -> Result {
        let bins: Vec<_> = [
            "LLD 19.0.0",
            "mold 2.9.0",
            "LLD 20.0.0",
            "mold 2.10.0",
            "LLD 20.0.0",
        ]
        .into_iter()
        .enumerate()
        .map(|(index, version)| Bin {
            label: None,
            flags: Vec::new(),
            index: index as u32,
            path: "ld".into(),
            identifier: crate::LinkerIdentifier::parse(version, std::path::Path::new("ld"))
                .unwrap(),
        })
        .collect();
        let config: Config = toml::from_str(
            r#"
name = "latest"
[defaults]
latest-only = true
[bench.example]
[bench.example.variants.all]
latest-only = false
"#,
        )?;
        for (name, (_, settings)) in config.expanded()? {
            let bench = Benchmark {
                name: name.clone(),
                path: "input".into(),
                config: settings,
                output: "output".into(),
                min_wild_version: None,
            };
            let indices: Vec<_> = benchmark_cases(&bins, &bench)
                .iter()
                .map(|(bin, _)| bin.index)
                .collect();
            assert_eq!(
                indices,
                if name == "example" {
                    vec![2, 3, 4]
                } else {
                    vec![0, 1, 2, 3, 4]
                }
            );
        }
        Ok(())
    }
}
