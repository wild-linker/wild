#!/usr/bin/env python3
"""Paired A/B of two Wild binaries relinking one `WILD_SAVE_DIR` capture.

Runs are interleaved (ABBA order) after warmups. Each run is wrapped in `perf stat` when it is
available, so every sample carries hardware counters alongside wall time. Outputs of the first
pair are compared after normalising the Wild provenance record in `.comment`, so a speedup can't
come from linking something different.
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import shutil
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path

EVENTS = (
    "task-clock",
    "page-faults",
    "context-switches",
    "cycles:u",
    "instructions:u",
    "branches:u",
    "branch-misses:u",
    "L1-dcache-load-misses:u",
    "dTLB-load-misses:u",
)


PERF = "perf"


def perf_available() -> list[str]:
    """Returns the subset of EVENTS this machine can count."""
    if shutil.which(PERF) is None:
        return []
    usable = []
    for event in EVENTS:
        result = subprocess.run(
            [PERF, "stat", "-x", ",", "-e", event, "--", "true"],
            capture_output=True,
            text=True,
            check=False,
        )
        line = next((l for l in result.stderr.splitlines() if event in l), "")
        if result.returncode == 0 and line and not line.startswith("<not"):
            usable.append(event)
    return usable


def parse_perf(path: Path) -> dict[str, float]:
    counters = {}
    for line in path.read_text().splitlines():
        fields = line.split(",")
        if len(fields) < 3 or not fields[0] or fields[0].startswith("<"):
            continue
        try:
            value = float(fields[0])
        except ValueError:
            continue
        # task-clock is msec in older perf and nanoseconds (under various unit names) in newer.
        if fields[2] == "task-clock" and fields[1] != "msec":
            value /= 1e6
        counters[fields[2]] = value
    return counters


def run_once(capture: Path, wild: Path, out: Path, events: list[str], extra: list[str],
             cpus: str | None, workdir: Path) -> dict[str, float]:
    env = dict(os.environ)
    env.pop("BASH_ENV", None)
    env["OUT"] = str(out)
    command = [str(capture / "run-with"), str(wild)]
    if extra:
        command += ["--", *extra]
    if cpus:
        command = ["taskset", "-c", cpus, *command]
    perf_out = workdir / "perf.csv"
    if events:
        command = [PERF, "stat", "-x", ",", "-o", str(perf_out), "-e", ",".join(events), "--",
                   *command]
    start = time.perf_counter()
    subprocess.run(command, env=env, check=True, stdout=subprocess.DEVNULL)
    sample = {"wall_ms": (time.perf_counter() - start) * 1000}
    if events:
        sample.update(parse_perf(perf_out))
    return sample


def normalised(path: Path, workdir: Path) -> bytes:
    """Output bytes without the provenance record and with any build ID zeroed.

    `.comment` names the Wild revision, and a build ID hashes the whole output (including
    `.comment`), so both differ between any two revisions. The build ID is zeroed in place, so a
    different descriptor width (see #23) still shows up as a difference.
    """
    stripped = workdir / (path.name + ".norm")
    subprocess.run(["objcopy", "--remove-section", ".comment", str(path), str(stripped)],
                   check=True)
    data = bytearray(stripped.read_bytes())
    sections = subprocess.run(["readelf", "-SW", str(stripped)], capture_output=True, text=True,
                              check=True).stdout
    for line in sections.splitlines():
        fields = line.replace("[", " ").replace("]", " ").split()
        if ".note.gnu.build-id" in fields:
            index = fields.index(".note.gnu.build-id")
            offset, size = int(fields[index + 3], 16), int(fields[index + 4], 16)
            # Note header (12 bytes) and the "GNU\0" name (4 bytes) precede the descriptor.
            data[offset + 16:offset + size] = bytes(max(0, size - 16))
    return bytes(data)


def summarise(samples: list[dict[str, float]]) -> dict[str, dict[str, float]]:
    keys = sorted({k for s in samples for k in s})
    summary = {}
    for key in keys:
        values = [s[key] for s in samples if key in s]
        summary[key] = {
            "median": statistics.median(values),
            "mean": statistics.fmean(values),
            "stdev": statistics.stdev(values) if len(values) > 1 else 0.0,
            "min": min(values),
        }
    return summary


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--capture", type=Path, required=True,
                        help="WILD_SAVE_DIR directory containing run-with")
    parser.add_argument("--baseline", type=Path, required=True)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--reference", type=Path,
                        help="untimed binary (e.g. upstream wild) whose output the candidate must match")
    parser.add_argument("--pairs", type=int, default=30)
    parser.add_argument("--warmups", type=int, default=4)
    parser.add_argument("--cpus", help="taskset CPU list, e.g. 0-7")
    parser.add_argument("--extra", default="", help="extra linker arguments, space separated")
    parser.add_argument("--no-perf", action="store_true")
    parser.add_argument("--perf", default="perf",
                        help="perf executable, e.g. a private copy granted CAP_PERFMON")
    parser.add_argument("--output", type=Path, required=True, help="JSON report path")
    parser.add_argument("--markdown", type=Path, help="append a markdown table here")
    args = parser.parse_args()

    global PERF
    PERF = args.perf
    events = [] if args.no_perf else perf_available()
    extra = args.extra.split()
    arms = {"baseline": args.baseline.resolve(), "candidate": args.candidate.resolve()}
    samples: dict[str, list[dict[str, float]]] = {name: [] for name in arms}

    with tempfile.TemporaryDirectory() as tmp:
        workdir = Path(tmp)
        outs = {name: workdir / f"{name}.out" for name in arms}
        for name, wild in arms.items():
            for _ in range(args.warmups):
                run_once(args.capture, wild, outs[name], events, extra, args.cpus, workdir)
        candidate_bytes = normalised(outs["candidate"], workdir)
        identical = normalised(outs["baseline"], workdir) == candidate_bytes
        matches_reference = None
        if args.reference:
            reference_out = workdir / "reference.out"
            run_once(args.capture, args.reference.resolve(), reference_out, [], extra, args.cpus,
                     workdir)
            matches_reference = normalised(reference_out, workdir) == candidate_bytes
        for pair in range(args.pairs):
            order = list(arms.items())
            if pair % 2:
                order.reverse()
            for name, wild in order:
                samples[name].append(
                    run_once(args.capture, wild, outs[name], events, extra, args.cpus, workdir))

    summaries = {name: summarise(s) for name, s in samples.items()}
    report = {
        "schema_version": 1,
        "host": {"platform": platform.platform(), "machine": platform.machine(),
                 "logical_cpus": os.cpu_count()},
        "capture": str(args.capture),
        "extra_args": extra,
        "cpus": args.cpus,
        "pairs": args.pairs,
        "events": events,
        "outputs_identical": identical,
        "matches_reference": matches_reference,
        "summary": summaries,
        "samples": samples,
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=1) + "\n")

    base, cand = summaries["baseline"], summaries["candidate"]
    lines = [
        f"| Metric (median of {args.pairs}) | Baseline | Candidate | Change |",
        "| --- | ---: | ---: | ---: |",
    ]
    for key in ["wall_ms", *[e for e in EVENTS if e in base]]:
        if key not in base or key not in cand:
            continue
        b, c = base[key]["median"], cand[key]["median"]
        change = f"{100 * (c - b) / b:+.1f}%" if b else "n/a"
        fmt = "{:,.1f}" if key in ("wall_ms", "task-clock") else "{:,.0f}"
        lines.append(f"| {key} | {fmt.format(b)} | {fmt.format(c)} | {change} |")
    lines.append("")
    lines.append(f"Baseline and candidate outputs identical apart from `.comment`: **{identical}**")
    if matches_reference is not None:
        lines.append(f"Candidate output identical to reference apart from `.comment`: "
                     f"**{matches_reference}**")
    table = "\n".join(lines)
    print(table)
    if args.markdown:
        with args.markdown.open("a") as handle:
            handle.write(table + "\n")
    return 0 if identical and matches_reference is not False else 1


if __name__ == "__main__":
    sys.exit(main())
