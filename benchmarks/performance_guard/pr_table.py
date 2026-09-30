#!/usr/bin/env python3
"""Per-benchmark statistics table for performance PRs.

Renders the layout wild's maintainer uses when reviewing performance PRs (see
https://github.com/wild-linker/wild/pull/2621#issuecomment-5910436060):

    #1 baseline  upstream-main 7a67b54f
    #2 candidate pr-head d48eb7d2
      metric                       mean        CI ±         SD       n   delta vs #1 (95% CI)
      wall                    81.263 ms       0.09%      1.87%    1771   -6.60% [-6.71%, -6.49%]

`mean`, `CI ±`, `SD` and `n` describe arm #2. `CI ±` is the half-width of the 95% Student-t
interval of the mean and `SD` the sample standard deviation, both as a percentage of the mean.
`delta vs #1` is mean(#2) / mean(#1) - 1 with a 95% interval from the delta method on that ratio
of means (Welch-Satterthwaite degrees of freedom).

`capture_ab.py` writes this table into its JSON report. Running this module on one or more of
those reports prints the Markdown for a PR description, so tables are never typed by hand:

    python3 benchmarks/performance_guard/pr_table.py evidence/*.json
"""

from __future__ import annotations

import argparse
import json
import math
import statistics
import sys
from pathlib import Path

# (sample key, row label, kind). The order is the order of rows in the table.
METRICS = (
    ("wall_ms", "wall", "ms"),
    ("user_ms", "user", "ms"),
    ("sys_ms", "system", "ms"),
    ("max_rss_kib", "peak RSS", "kib"),
    ("cycles", "cycles", "count"),
    ("instructions", "instructions", "count"),
    ("cache-references", "cache-references", "count"),
    ("cache-misses", "cache-misses", "count"),
    ("branches", "branches", "count"),
    ("branch-misses", "branch-misses", "count"),
)

HEADER = ("  metric                       mean        CI ±         SD       n"
          "   delta vs #1 (95% CI)")

_NORMAL_975 = statistics.NormalDist().inv_cdf(0.975)


def t_quantile_975(df: float) -> float:
    """Two-sided 95% critical value of Student's t with `df` degrees of freedom."""
    if df <= 0 or math.isnan(df):
        return math.nan
    exact = {1: 12.706204736174705, 2: 4.302652729749464, 3: 3.182446305284263,
             4: 2.776445105197799}
    if df in exact:
        return exact[df]
    # Cornish-Fisher expansion about the normal quantile; accurate to ~1e-3 from df = 5.
    z = _NORMAL_975
    g1 = (z**3 + z) / 4
    g2 = (5 * z**5 + 16 * z**3 + 3 * z) / 96
    g3 = (3 * z**7 + 19 * z**5 + 17 * z**3 - 15 * z) / 384
    g4 = (79 * z**9 + 776 * z**7 + 1482 * z**5 - 1920 * z**3 - 945 * z) / 92160
    return z + g1 / df + g2 / df**2 + g3 / df**3 + g4 / df**4


def describe(values: list[float]) -> dict[str, float]:
    """Mean, sample SD, n and the 95% CI half-width of the mean."""
    n = len(values)
    mean = statistics.fmean(values) if values else math.nan
    sd = statistics.stdev(values) if n > 1 else math.nan
    half = t_quantile_975(n - 1) * sd / math.sqrt(n) if n > 1 else math.nan
    return {"mean": mean, "sd": sd, "n": n, "ci_half": half}


def delta(base: dict[str, float], cand: dict[str, float]) -> tuple[float, float, float]:
    """Percentage change of the candidate mean over the baseline mean, with its 95% interval."""
    if not base["mean"] or not cand["mean"] or base["n"] < 2 or cand["n"] < 2:
        return math.nan, math.nan, math.nan
    ratio = cand["mean"] / base["mean"]
    rel_c = (cand["sd"] / math.sqrt(cand["n"]) / cand["mean"]) ** 2
    rel_b = (base["sd"] / math.sqrt(base["n"]) / base["mean"]) ** 2
    se = ratio * math.sqrt(rel_c + rel_b)
    if rel_c + rel_b == 0:
        df = cand["n"] + base["n"] - 2
    else:
        df = (rel_c + rel_b) ** 2 / (rel_c**2 / (cand["n"] - 1) + rel_b**2 / (base["n"] - 1))
    half = t_quantile_975(df) * se
    return 100 * (ratio - 1), 100 * (ratio - 1 - half), 100 * (ratio - 1 + half)


def format_mean(value: float, kind: str) -> str:
    if kind == "ms":
        return f"{value:.3f} ms"
    if kind == "kib":
        return f"{value / 1024:.2f} MiB"
    for prefix, scale in (("T", 1e12), ("G", 1e9), ("M", 1e6), ("K", 1e3)):
        if abs(value) >= scale:
            return f"{value / scale:.3f} {prefix}~"
    return f"{value:.3f} ~"


def _percent(value: float) -> str:
    return "n/a" if math.isnan(value) else f"{value:.2f}%"


def format_row(label: str, kind: str, cand: dict[str, float],
               change: tuple[float, float, float]) -> str:
    """One table row from arm #2's `describe()` statistics and the `delta()` against arm #1."""
    rel = (lambda x: 100 * x / cand["mean"]) if cand["mean"] else (lambda x: math.nan)
    pct, lo, hi = change
    text = "n/a" if math.isnan(pct) else f"{pct:+.2f}% [{lo:+.2f}%, {hi:+.2f}%]"
    return (f"  {label:<16}{format_mean(cand['mean'], kind):>17}{_percent(rel(cand['ci_half'])):>12}"
            f"{_percent(rel(cand['sd'])):>11}{cand['n']:>8}   {text}")


def row(label: str, kind: str, base_values: list[float], cand_values: list[float]) -> str:
    base, cand = describe(base_values), describe(cand_values)
    return format_row(label, kind, cand, delta(base, cand))


def render(baseline: dict, candidate: dict, unavailable: dict[str, str] | None = None) -> str:
    """Renders the table for one benchmark.

    `baseline` and `candidate` are `{"label": str, "samples": [ {metric: value} ]}`. Metrics with
    no samples in either arm are listed as not measured, with the reason from `unavailable`.
    """
    unavailable = unavailable or {}
    lines = [f"#1 baseline  {baseline['label']}", f"#2 candidate {candidate['label']}", HEADER]
    for key, label, kind in METRICS:
        b = [s[key] for s in baseline["samples"] if key in s]
        c = [s[key] for s in candidate["samples"] if key in s]
        if b and c:
            lines.append(row(label, kind, b, c))
        else:
            reason = unavailable.get(key, "not measured")
            lines.append(f"  {label:<16}{'n/a':>17}   {reason}")
    return "\n".join(lines)


def markdown(report: dict) -> str:
    """One benchmark section for a PR description, from a `capture_ab.py` JSON report."""
    parts = [f"{report.get('benchmark') or report['capture']}:", "", "```", report["table"], "```"]
    notes = report.get("table_notes")
    if notes:
        parts += ["", notes]
    return "\n".join(parts)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("reports", nargs="+", type=Path, help="capture_ab.py JSON reports")
    args = parser.parse_args()
    sections = []
    for path in args.reports:
        report = json.loads(path.read_text())
        if "table" not in report:
            print(f"{path}: no table (identity-only run?)", file=sys.stderr)
            continue
        sections.append(markdown(report))
    print("\n\n".join(sections))
    return 0 if sections else 1


if __name__ == "__main__":
    sys.exit(main())
