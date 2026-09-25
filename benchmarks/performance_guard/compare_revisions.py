#!/usr/bin/env python3
"""Benchmark several already-built Wild binaries against a baseline (issue #14).

The first --build is the baseline (for example upstream/main). Any other build
whose median is slower than the baseline by more than --noise-percent on any
case is a regression and the script exits 1.
"""

from __future__ import annotations

import argparse
import json
import shlex
import statistics
import subprocess
import sys
from pathlib import Path
from typing import Any

SCRIPT_DIRECTORY = Path(__file__).resolve().parent
RUN_LINK = SCRIPT_DIRECTORY / "run_link.py"
MINIMUM_RUNS = 10

CASES: list[dict[str, str]] = [
    {"name": "ordinary-none", "workload": "ordinary", "build_id": "none"},
    {"name": "large-debug-none", "workload": "large-debug", "build_id": "none"},
    {"name": "full-lto-none", "workload": "full-lto", "build_id": "none"},
    {"name": "large-debug-fast", "workload": "large-debug", "build_id": "fast"},
]


def parse_build(value: str) -> dict[str, str]:
    parts = value.split("=", 2)
    if len(parts) != 3 or not all(parts):
        raise argparse.ArgumentTypeError(
            f"expected LABEL=SHA=PATH_TO_WILD, got {value!r}"
        )
    label, sha, path = parts
    return {"label": label, "sha": sha, "path": path}


def runs_argument(value: str) -> int:
    runs = int(value)
    if runs < MINIMUM_RUNS:
        raise argparse.ArgumentTypeError(f"--runs must be at least {MINIMUM_RUNS}")
    return runs


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument(
        "--build",
        type=parse_build,
        action="append",
        required=True,
        help="LABEL=SHA=PATH_TO_WILD; repeatable, the first is the baseline",
    )
    parser.add_argument("--hyperfine", default="hyperfine")
    parser.add_argument("--runs", type=runs_argument, default=MINIMUM_RUNS)
    parser.add_argument("--warmup", type=int, default=3)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--noise-percent", type=float, default=2.0)
    args = parser.parse_args(argv)
    if len(args.build) < 2:
        parser.error("at least two --build values are required")
    labels = [build["label"] for build in args.build]
    if len(set(labels)) != len(labels):
        parser.error("--build labels must be unique")
    return args


def summarize(times: list[float]) -> dict[str, float | int]:
    """Summarize hyperfine times (seconds) in milliseconds."""
    millis = [time * 1000.0 for time in times]
    return {
        "median_ms": statistics.median(millis),
        "stddev_ms": statistics.stdev(millis) if len(millis) > 1 else 0.0,
        "min_ms": min(millis),
        "max_ms": max(millis),
        "runs": len(millis),
    }


def classify(
    baseline_median: float, candidate_median: float, noise_percent: float
) -> str:
    change = (candidate_median - baseline_median) / baseline_median * 100.0
    if change > noise_percent:
        return "slower"
    if change < -noise_percent:
        return "faster"
    return "noise"


def percent_change(baseline_median: float, candidate_median: float) -> float:
    return (candidate_median - baseline_median) / baseline_median * 100.0


def render_markdown(
    builds: list[dict[str, str]],
    results: dict[str, dict[str, dict[str, Any]]],
    noise_percent: float = 2.0,
) -> str:
    """results maps case name -> build label -> summarize() output."""
    baseline = builds[0]["label"]
    lines = [
        "| Workload | Build | SHA | Median ms | Stddev ms | Runs | vs baseline |",
        "| --- | --- | --- | ---: | ---: | ---: | --- |",
    ]
    for case_name, per_build in results.items():
        base_median = per_build[baseline]["median_ms"]
        for build in builds:
            summary = per_build[build["label"]]
            if build["label"] == baseline:
                versus = "baseline"
            else:
                versus = (
                    f"{percent_change(base_median, summary['median_ms']):+.2f}% "
                    f"({classify(base_median, summary['median_ms'], noise_percent)})"
                )
            lines.append(
                f"| {case_name} | {build['label']} | {build['sha'][:12]} "
                f"| {summary['median_ms']:.2f} | {summary['stddev_ms']:.2f} "
                f"| {summary['runs']} | {versus} |"
            )
    lines.append("")
    lines.append("Builds:")
    lines.append("")
    for index, build in enumerate(builds):
        suffix = " (baseline)" if index == 0 else ""
        lines.append(f"- `{build['label']}`: `{build['sha']}`{suffix}")
    return "\n".join(lines) + "\n"


def regressions(
    builds: list[dict[str, str]],
    results: dict[str, dict[str, dict[str, Any]]],
    noise_percent: float,
) -> list[tuple[str, str]]:
    baseline = builds[0]["label"]
    found = []
    for case_name, per_build in results.items():
        base_median = per_build[baseline]["median_ms"]
        for build in builds[1:]:
            median = per_build[build["label"]]["median_ms"]
            if classify(base_median, median, noise_percent) == "slower":
                found.append((case_name, build["label"]))
    return found


def command_for(
    manifest: Path, case: dict[str, str], linker: Path, output: Path
) -> str:
    return shlex.join(
        [
            sys.executable,
            str(RUN_LINK),
            "--manifest",
            str(manifest),
            "--workload",
            case["workload"],
            "--linker",
            str(linker),
            "--build-id",
            case["build_id"],
            "--output",
            str(output),
        ]
    )


def run_case(args: argparse.Namespace, case: dict[str, str]) -> dict[str, Any]:
    export = args.output / f"hyperfine-{case['name']}.json"
    command = [
        args.hyperfine,
        "--runs",
        str(args.runs),
        "--warmup",
        str(args.warmup),
        "--export-json",
        str(export),
    ]
    for build in args.build:
        command += [
            "--command-name",
            build["label"],
            command_for(
                args.manifest.resolve(),
                case,
                Path(build["path"]).resolve(),
                (args.output / f"{build['label']}-{case['name']}.elf").resolve(),
            ),
        ]
    subprocess.run(command, check=True)
    data = json.loads(export.read_text())
    per_build = {}
    for build, result in zip(args.build, data["results"]):
        per_build[build["label"]] = summarize(result["times"])
    return per_build


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    args.output.mkdir(parents=True, exist_ok=True)
    results = {case["name"]: run_case(args, case) for case in CASES}
    markdown = render_markdown(args.build, results, args.noise_percent)
    found = regressions(args.build, results, args.noise_percent)
    (args.output / "results.json").write_text(
        json.dumps(
            {
                "builds": args.build,
                "noise_percent": args.noise_percent,
                "results": results,
                "regressions": [
                    {"case": case, "build": label} for case, label in found
                ],
            },
            indent=2,
        )
        + "\n"
    )
    (args.output / "results.md").write_text(markdown)
    print(markdown)
    if found:
        for case, label in found:
            print(f"regression: {label} slower than baseline on {case}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
