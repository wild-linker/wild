use crate::BenchmarkResult;
use crate::Result;
use crate::config::Config;
use crate::reporting::ReportMode;
use crate::reporting::confidence_interval;
use crate::reporting::mean;
use std::collections::BTreeMap;
use std::fmt::Write as _;

pub(crate) fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub(crate) fn produce_chart(
    benchmark: &BenchmarkResult,
    mode: ReportMode,
    config: &Config,
) -> Result<String> {
    let mut series = BTreeMap::new();
    for batch in &benchmark.batches {
        if let Some(threads) = batch.threads {
            series
                .entry(batch.bin.index)
                .or_insert_with(Vec::new)
                .push((threads, batch));
        }
    }

    let label = match mode {
        ReportMode::Time => "Link time",
        ReportMode::Memory => "Memory",
    };

    let unit = format!("{label} ({})", mode.unit_name());
    let multiplier = mode.unit_multiplier();

    let max_threads = series
        .values()
        .flatten()
        .map(|(n, _)| *n)
        .max()
        .unwrap_or(1);

    let max_value = series
        .values()
        .flatten()
        .map(|(_, batch)| (mean(batch, mode) + confidence_interval(batch, mode)) * multiplier)
        .fold(0.0_f64, f64::max)
        .max(0.001)
        * 1.1;

    let x =
        |n: u32| 100.0 + f64::from(n - 1) / f64::from(max_threads.saturating_sub(1).max(1)) * 840.0;
    let y = |value: f64| 480.0 - value / max_value * 380.0;
    let height = 560 + series.len() * 26;
    let title = escape(&format!("{} - {}", config.name, benchmark.config.name));
    let mut svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="1000" height="{height}" viewBox="0 0 1000 {height}">
<rect width="100%" height="100%" fill="#000"/>
<g fill="white" font-family="sans-serif" font-size="16">
<text x="500" y="35" text-anchor="middle" font-size="26">{title}</text>
<text x="100" y="75">{unit}</text>
<text x="520" y="535" text-anchor="middle">Threads</text>
"##
    );

    for tick in 0..=5 {
        let value = max_value * f64::from(tick) / 5.0;
        let y = y(value);
        writeln!(
            svg,
            r##"<path d="M100 {y} H940" stroke="#555"/><text x="90" y="{y}" text-anchor="end" dominant-baseline="middle">{value:.2}</text>"##
        )?;
    }

    let step = max_threads.div_ceil(16).max(1);
    let mut ticks: Vec<_> = (1..=max_threads).step_by(step as usize).collect();
    if ticks.last() != Some(&max_threads) {
        ticks.push(max_threads);
    }

    for n in ticks {
        let x = x(n);
        writeln!(
            svg,
            r#"<path d="M{x} 480 v6" stroke="white"/><text x="{x}" y="510" text-anchor="middle">{n}</text>"#
        )?;
    }

    const COLOURS: [&str; 8] = [
        "#00ff00", "#ff60ff", "#60b0ff", "#ffb040", "#40ffff", "#ff6060", "#ffff60", "#c090ff",
    ];

    for (index, points) in series.values_mut().enumerate() {
        points.sort_by_key(|(n, _)| *n);
        let colour = COLOURS[index % COLOURS.len()];
        let dash = if index < COLOURS.len() { "none" } else { "6 4" };
        let mut path = String::new();

        for (i, (n, batch)) in points.iter().enumerate() {
            let value = mean(batch, mode) * multiplier;
            let interval = confidence_interval(batch, mode) * multiplier;
            let x = x(*n);
            let point_y = y(value);
            let upper = y(value + interval);
            let lower = y((value - interval).max(0.0));
            let command = if i == 0 { 'M' } else { 'L' };

            write!(path, "{command}{x:.2} {point_y:.2} ")?;
            writeln!(
                svg,
                r#"<path d="M{x} {upper} V{lower}" stroke="{colour}"/><circle cx="{x}" cy="{point_y}" r="3" fill="{colour}"><title>{n} threads: {value:.3}</title></circle>"#
            )?;
        }

        let label = escape(&points[0].1.bin.to_string());
        let legend_y = 565 + index * 26;

        writeln!(
            svg,
            r#"<path d="{path}" fill="none" stroke="{colour}" stroke-width="2" stroke-dasharray="{dash}"/>
<path d="M100 {legend_y} h35" stroke="{colour}" stroke-width="2" stroke-dasharray="{dash}"/><text x="145" y="{legend_y}" dominant-baseline="middle">{label}</text>"#
        )?;
    }

    svg.push_str("</g></svg>\n");

    Ok(svg)
}
