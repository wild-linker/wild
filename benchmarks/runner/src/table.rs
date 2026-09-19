use crate::Benchmarks;
use crate::Result;
use crate::config::Config;
use crate::reporting::ReportMode;
use crate::reporting::mean;
use crate::reporting::merge_batches;
use anyhow::Context as _;
use clap::Parser;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::PathBuf;

#[derive(Parser, Clone)]
pub(crate) struct TableArgs {
    /// Locate results using this config and apply its benchmark and linker skips.
    #[arg(long, required_unless_present = "input")]
    config: Option<PathBuf>,

    /// Read this results file. Can be used without a config.
    #[arg(long, required_unless_present = "config")]
    input: Option<PathBuf>,
}

pub(crate) fn run(args: &TableArgs) -> Result {
    let config = args.config.as_deref().map(Config::load).transpose()?;
    let input = match &args.input {
        Some(input) => input.clone(),
        None => crate::default_result_path(
            config.as_ref().context("Expected --config or --input")?,
            None,
        ),
    };

    let results = Benchmarks::load(&input)?;
    let markdown = render(&results, config.as_ref())?;
    std::io::stdout().lock().write_all(markdown.as_bytes())?;
    Ok(())
}

fn render(results: &Benchmarks, config: Option<&Config>) -> Result<String> {
    let expanded = config.map(Config::expanded).transpose()?;
    let mut bins = BTreeMap::new();

    for benchmark in &results.benchmarks {
        for batch in &benchmark.batches {
            bins.entry(batch.bin.index).or_insert(&batch.bin);
        }
    }

    let bin_order: Vec<_> = bins.keys().copied().collect();
    let mut benchmarks: Vec<_> = results.benchmarks.iter().collect();

    if let Some(expanded) = &expanded {
        benchmarks.sort_by_key(|benchmark| {
            expanded
                .get_index_of(&benchmark.config.name)
                .unwrap_or(usize::MAX)
        });
    }

    let baseline = *bin_order.last().context("Results contain no linkers")?;
    let mut names = BTreeMap::new();

    for (&index, bin) in &bins {
        let same_kind = bins
            .values()
            .filter(|other| other.identifier.kind == bin.identifier.kind)
            .count();

        let name = if same_kind == 1 && bin.label.is_none() {
            bin.identifier.kind.to_string()
        } else {
            bin.to_string()
        };

        names.insert(index, name);
    }

    let original_names = names.clone();

    for (&index, name) in &mut names {
        if original_names
            .values()
            .filter(|other| *other == name)
            .count()
            > 1
        {
            write!(name, " #{}", u64::from(index) + 1)?;
        }
    }

    let mut header = vec!["Benchmark".to_owned()];
    for index in &bin_order {
        header.push(format!("{} (s)", escape(&names[index])));
    }

    for &index in &bin_order {
        let name = &names[&index];
        if index != baseline {
            header.push(format!("{}/{}", escape(name), escape(&names[&baseline])));
        }
    }

    let mut rows = vec![header];

    for benchmark in benchmarks {
        let settings = if let Some(expanded) = &expanded {
            let Some((_, settings)) = expanded.get(&benchmark.config.name) else {
                continue;
            };
            settings
        } else {
            &benchmark.config.config
        };

        if settings.skip {
            continue;
        }

        let mut timing = ReportMode::Time.filter(benchmark, settings);
        merge_batches(&mut timing);
        let mut times_by_threads = BTreeMap::<_, BTreeMap<_, _>>::new();
        for batch in &benchmark.batches {
            times_by_threads.entry(batch.threads).or_default();
        }
        for batch in &timing.batches {
            times_by_threads
                .entry(batch.threads)
                .or_default()
                .insert(batch.bin.index, mean(batch, ReportMode::Time));
        }

        for (threads, times) in times_by_threads {
            let name = threads.map_or_else(
                || benchmark.config.name.clone(),
                |n| format!("{} ({n} threads)", benchmark.config.name),
            );

            let mut row = vec![escape(&name)];

            for index in &bin_order {
                match times.get(index) {
                    Some(time) => row.push(format!("{time:.2}")),
                    None => row.push("-".to_owned()),
                }
            }

            for index in bin_order.iter().filter(|&&index| index != baseline) {
                let ratio = times
                    .get(index)
                    .zip(times.get(&baseline))
                    .filter(|(_, baseline)| **baseline > 0.0)
                    .map(|(time, baseline)| time / baseline);
                match ratio {
                    Some(ratio) => row.push(format!("{ratio:.1}x")),
                    None => row.push("-".to_owned()),
                }
            }

            rows.push(row);
        }
    }

    let mut widths = vec![4; rows[0].len()];
    for row in &rows {
        for (width, cell) in widths.iter_mut().zip(row) {
            *width = (*width).max(cell.chars().count());
        }
    }

    let mut out = String::new();
    for (row_index, row) in rows.iter().enumerate() {
        out.push('|');
        for (column, (cell, width)) in row.iter().zip(&widths).enumerate() {
            if column == 0 {
                write!(out, " {cell:<width$} |")?;
            } else {
                write!(out, " {cell:>width$} |")?;
            }
        }

        out.push('\n');

        if row_index == 0 {
            out.push('|');
            for (column, width) in widths.iter().enumerate() {
                if column == 0 {
                    write!(out, " {} |", "-".repeat(*width))?;
                } else {
                    write!(out, " {}: |", "-".repeat(width - 1))?;
                }
            }
            out.push('\n');
        }
    }
    Ok(out)
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\\', "&#92;")
        .replace('|', "&#124;")
        .replace(['\n', '\r'], " ")
}
