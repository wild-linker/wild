#!/usr/bin/env python3
"""Balanced Hyperfine A/B guard with paired confidence intervals and Poop diagnostics."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import platform
import random
import shlex
import statistics
import subprocess
import sys
from pathlib import Path
from typing import Any


class GuardError(RuntimeError):
    pass


def paired_interval(
    baseline: list[float],
    candidate: list[float],
    confidence: float = 0.95,
    seed: int = 0,
) -> tuple[float, float, float]:
    if len(baseline) != len(candidate) or len(baseline) < 2:
        raise GuardError("paired samples must have equal lengths of at least two")
    ratios = [math.log(c / b) for b, c in zip(baseline, candidate)]
    rng = random.Random(seed)
    boot = []
    for _ in range(10_000):
        boot.append(statistics.mean(ratios[rng.randrange(len(ratios))] for _ in ratios))
    boot.sort()
    tail = (1.0 - confidence) / 2.0
    low = boot[int(tail * len(boot))]
    high = boot[min(len(boot) - 1, int((1.0 - tail) * len(boot)))]

    def convert(value: float) -> float:
        return math.expm1(value) * 100.0

    return convert(statistics.mean(ratios)), convert(low), convert(high)


def classify(
    baseline: list[float],
    candidate: list[float],
    regression: float,
    improvement: float,
    confidence: float = 0.95,
) -> dict[str, Any]:
    delta, low, high = paired_interval(baseline, candidate, confidence)
    if low > regression:
        status = "regression"
    elif high < -improvement:
        status = "improvement"
    elif low >= -improvement and high <= regression:
        status = "no-material-change"
    else:
        status = "inconclusive"
    return {
        "status": status,
        "delta_percent": delta,
        "ci_low_percent": low,
        "ci_high_percent": high,
    }


def counter_status(text: str, returncode: int, perf_paranoid: int | None = None) -> str:
    lowered = text.lower()
    unavailable = (
        "not supported",
        "permission",
        "perf_event_open",
        "access denied",
        "<not counted>",
    )
    if returncode != 0 and perf_paranoid is not None and perf_paranoid > 2:
        return "unavailable"
    if any(token in lowered for token in unavailable):
        return "unavailable"
    return "available" if returncode == 0 else "diagnostic-failed"


def would_fail_policy(cases: list[dict[str, Any]], performance_claim: bool) -> bool:
    if any(row["classification"]["status"] == "regression" for row in cases):
        return True
    return performance_claim and not any(
        row["target"] and row["classification"]["status"] == "improvement"
        for row in cases
    )


def read_hyperfine(path: Path) -> dict[str, list[float]]:
    data = json.loads(path.read_text())
    results = data.get("results")
    if not isinstance(results, list) or len(results) != 2:
        raise GuardError(f"invalid Hyperfine JSON in {path}")
    output: dict[str, list[float]] = {}
    for result in results:
        name = result.get("command")
        times = result.get("times")
        if (
            name not in ("baseline", "candidate")
            or not isinstance(times, list)
            or not times
        ):
            raise GuardError(f"invalid Hyperfine result in {path}")
        output[name] = [float(value) for value in times]
    if set(output) != {"baseline", "candidate"}:
        raise GuardError(f"missing Hyperfine command in {path}")
    return output


def command_for(
    script: Path, manifest: Path, case: dict[str, Any], linker: Path, output: Path
) -> str:
    args = [
        sys.executable,
        str(script),
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
    return shlex.join(args)


def run_checked(command: list[str], **kwargs: Any) -> subprocess.CompletedProcess[str]:
    try:
        return subprocess.run(command, check=True, text=True, **kwargs)
    except subprocess.CalledProcessError as error:
        raise GuardError(
            f"command failed ({error.returncode}): {shlex.join(command)}"
        ) from error


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def verify_case(
    case: dict[str, Any],
    commands: dict[str, str],
    outputs: dict[str, Path],
    evidence: Path,
) -> dict[str, Any]:
    for name in ("baseline", "candidate"):
        run_checked(["bash", "-c", commands[name]])
        run_checked([str(outputs[name])])
    hashes = {name: sha256(path) for name, path in outputs.items()}
    equivalent = hashes["baseline"] == hashes["candidate"]
    if not equivalent and case["build_id"] == "fast":
        stripped = {}
        for name, path in outputs.items():
            stripped[name] = evidence / f"{case['name']}-{name}-without-build-id"
            run_checked(
                [
                    "objcopy",
                    "--remove-section",
                    ".note.gnu.build-id",
                    str(path),
                    str(stripped[name]),
                ]
            )
        equivalent = sha256(stripped["baseline"]) == sha256(stripped["candidate"])
    if not equivalent:
        raise GuardError(
            f"{case['name']}: baseline and candidate outputs are not structurally equivalent"
        )
    return {"sha256": hashes, "structurally_equivalent": True, "executed": True}


def run_poop(
    poop: Path, first: str, second: str, duration_ms: int, destination: Path
) -> dict[str, Any]:
    completed = subprocess.run(
        [str(poop), "--duration", str(duration_ms), "--color", "never", first, second],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )
    destination.write_text(completed.stdout)
    paranoid = None
    try:
        paranoid = int(Path("/proc/sys/kernel/perf_event_paranoid").read_text().strip())
    except (OSError, ValueError):
        pass
    return {
        "returncode": completed.returncode,
        "hardware_counters": counter_status(
            completed.stdout, completed.returncode, paranoid
        ),
    }


def render_svg(report: dict[str, Any], path: Path) -> None:
    rows = report["cases"]
    width, row_height = 900, 82
    height = 92 + row_height * len(rows)
    lines = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}" role="img">',
        "<title>Wild pull-request performance guard</title>",
        "<style>text{font-family:ui-monospace,monospace;fill:#172033}.h{font-size:22px;font-weight:700}.n{font-size:14px}.s{font-size:12px;fill:#526070}</style>",
        '<rect width="100%" height="100%" fill="#f8fafc"/>',
        '<text class="h" x="32" y="38">Wild PR performance guard</text>',
        f'<text class="s" x="32" y="62">paired 95% confidence intervals · {report["policy"]}</text>',
    ]
    for index, row in enumerate(rows):
        y = 100 + index * row_height
        result = row["classification"]
        low, high, delta = (
            result["ci_low_percent"],
            result["ci_high_percent"],
            result["delta_percent"],
        )
        scale, center = 18.0, 565.0
        color = {
            "regression": "#dc2626",
            "improvement": "#16a34a",
            "inconclusive": "#d97706",
        }.get(result["status"], "#64748b")
        lines += [
            f'<text class="n" x="32" y="{y}">{row["name"]}</text>',
            f'<text class="s" x="32" y="{y + 22}">{result["status"]} · {delta:+.2f}% [{low:+.2f}, {high:+.2f}]</text>',
            f'<line x1="385" y1="{y - 5}" x2="745" y2="{y - 5}" stroke="#cbd5e1"/>',
            f'<line x1="{center}" y1="{y - 15}" x2="{center}" y2="{y + 5}" stroke="#475569"/>',
            f'<line x1="{center + low * scale:.1f}" y1="{y - 5}" x2="{center + high * scale:.1f}" y2="{y - 5}" stroke="{color}" stroke-width="7"/>',
            f'<circle cx="{center + delta * scale:.1f}" cy="{y - 5}" r="6" fill="{color}"/>',
        ]
    lines.append("</svg>")
    path.write_text("\n".join(lines) + "\n")


def write_summary(report: dict[str, Any], path: Path) -> None:
    lines = [
        "## Linux performance guard (calibration)",
        "",
        f"Policy: **{report['policy']}**. Negative change is faster.",
        "",
        f"Blocking policy would **{'fail' if report['would_fail_blocking_policy'] else 'pass'}** this result.",
        "",
        "| Case | Base median | Candidate median | Mean base / candidate (σ) | n | Paired change (95% CI) | Classification | Poop counters |",
        "|---|---:|---:|---:|---:|---:|---|---|",
    ]
    for row in report["cases"]:
        result = row["classification"]
        lines.append(
            f"| {row['name']} | {statistics.median(row['samples_seconds']['baseline']) * 1000:.2f} ms | "
            f"{statistics.median(row['samples_seconds']['candidate']) * 1000:.2f} ms | "
            f"{statistics.mean(row['samples_seconds']['baseline']) * 1000:.2f} / "
            f"{statistics.mean(row['samples_seconds']['candidate']) * 1000:.2f} ms "
            f"(σ {statistics.stdev(row['samples_seconds']['baseline']) * 1000:.2f} / "
            f"{statistics.stdev(row['samples_seconds']['candidate']) * 1000:.2f}) | "
            f"{len(row['samples_seconds']['baseline'])} | "
            f"{result['delta_percent']:+.2f}% [{result['ci_low_percent']:+.2f}, {result['ci_high_percent']:+.2f}] | "
            f"{result['status']} | {row['poop']['hardware_counters']} |"
        )
    lines += [
        "",
        "`--build-id=none` is an isolation control only; `--build-id=fast` is measured for both real workloads.",
        "Raw Hyperfine JSON, Poop output, metadata, executable hashes, and `performance.svg` are in the workflow artifact.",
    ]
    if (
        os.environ.get("GITHUB_SERVER_URL")
        and os.environ.get("GITHUB_REPOSITORY")
        and os.environ.get("GITHUB_RUN_ID")
    ):
        lines.append(
            f"[Open this run's artifacts]({os.environ['GITHUB_SERVER_URL']}/{os.environ['GITHUB_REPOSITORY']}/actions/runs/{os.environ['GITHUB_RUN_ID']})."
        )
    path.write_text("\n".join(lines) + "\n")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--policy", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--baseline", type=Path, required=True)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--hyperfine", type=Path, required=True)
    parser.add_argument("--poop", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--baseline-sha", required=True)
    parser.add_argument("--candidate-sha", required=True)
    parser.add_argument("--performance-claim", action="store_true")
    args = parser.parse_args()
    policy = json.loads(args.policy.read_text())
    args.output.mkdir(parents=True, exist_ok=True)
    link_outputs = args.manifest.resolve().parent / "link-outputs"
    link_outputs.mkdir(parents=True, exist_ok=True)
    script = Path(__file__).with_name("run_link.py").resolve()
    cases = []
    for case in policy["cases"]:
        outputs = {
            name: link_outputs / f"{case['name']}-{name}.elf"
            for name in ("baseline", "candidate")
        }
        commands = {
            "baseline": command_for(
                script,
                args.manifest.resolve(),
                case,
                args.baseline.resolve(),
                outputs["baseline"],
            ),
            "candidate": command_for(
                script,
                args.manifest.resolve(),
                case,
                args.candidate.resolve(),
                outputs["candidate"],
            ),
        }
        correctness = verify_case(case, commands, outputs, link_outputs)
        samples = {"baseline": [], "candidate": []}
        block = 0
        while len(samples["baseline"]) < policy["maximum_pairs"]:
            order = (
                ("baseline", "candidate")
                if block % 2 == 0
                else ("candidate", "baseline")
            )
            block_path = args.output / f"{case['name']}-hyperfine-{block:02}.json"
            run_checked(
                [
                    str(args.hyperfine),
                    "--shell=none",
                    "--warmup",
                    str(policy["warmup_count"] if block == 0 else 0),
                    "--runs",
                    str(policy["pairs_per_block"]),
                    "--command-name",
                    order[0],
                    commands[order[0]],
                    "--command-name",
                    order[1],
                    commands[order[1]],
                    "--export-json",
                    str(block_path),
                ],
                stdout=subprocess.DEVNULL,
            )
            block_samples = read_hyperfine(block_path)
            for name in samples:
                samples[name].extend(block_samples[name])
            result = classify(
                samples["baseline"],
                samples["candidate"],
                policy["regression_threshold_percent"],
                policy["improvement_threshold_percent"],
                policy["confidence_level"],
            )
            block += 1
            if (
                len(samples["baseline"]) >= policy["initial_pairs"]
                and result["status"] != "inconclusive"
            ):
                break
        poop_runs = []
        for order_index, order in enumerate(
            (("baseline", "candidate"), ("candidate", "baseline"))
        ):
            poop_runs.append(
                run_poop(
                    args.poop,
                    commands[order[0]],
                    commands[order[1]],
                    policy["poop_duration_ms"],
                    args.output / f"{case['name']}-poop-{order_index}.txt",
                )
            )
        poop_status = (
            "unavailable"
            if any(item["hardware_counters"] == "unavailable" for item in poop_runs)
            else (
                "available"
                if all(item["hardware_counters"] == "available" for item in poop_runs)
                else "diagnostic-failed"
            )
        )
        cases.append(
            {
                "name": case["name"],
                "target": case["target"],
                "samples_seconds": samples,
                "classification": result,
                "correctness": correctness,
                "poop": {"hardware_counters": poop_status, "runs": poop_runs},
            }
        )

    would_fail = would_fail_policy(cases, args.performance_claim)
    calibration = bool(policy["calibration_mode"])
    report = {
        "schema_version": 1,
        "policy": "report-only calibration" if calibration else "blocking",
        "performance_claim": args.performance_claim,
        "would_fail_blocking_policy": would_fail,
        "baseline": {
            "source_sha": args.baseline_sha,
            "path": str(args.baseline.resolve()),
            "sha256": sha256(args.baseline),
        },
        "candidate": {
            "source_sha": args.candidate_sha,
            "path": str(args.candidate.resolve()),
            "sha256": sha256(args.candidate),
        },
        "tools": {"hyperfine": "1.20.0", "poop": "0.5.0"},
        "environment": {
            "platform": platform.platform(),
            "cpu_count": os.cpu_count(),
            "python": platform.python_version(),
        },
        "cases": cases,
    }
    (args.output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    render_svg(report, args.output / "performance.svg")
    write_summary(report, args.output / "summary.md")
    return 0 if calibration else int(would_fail)


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except GuardError as error:
        print(f"performance guard error: {error}", file=sys.stderr)
        raise SystemExit(2)
