#!/usr/bin/env python3
"""Render paired Wild benchmark vectors as a deterministic self-contained SVG."""

from __future__ import annotations

import argparse
import html
import json
import math
import statistics
from pathlib import Path
from typing import Any


class SchemaError(ValueError):
    """The benchmark report does not match the expected schema."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SchemaError(message)


def load_report(path: Path) -> dict[str, Any]:
    try:
        report = json.loads(path.read_text())
    except (OSError, json.JSONDecodeError) as error:
        raise SchemaError(f"cannot read {path}: {error}") from error
    require(isinstance(report, dict), "report must be a JSON object")
    schema_version = report.get("schema_version")
    require(type(schema_version) is int and schema_version == 1, "schema_version must be integer 1")
    series = report.get("series")
    require(isinstance(series, list) and series, "series must be a non-empty list")
    for index, item in enumerate(series):
        require(isinstance(item, dict), f"series[{index}] must be an object")
        require(isinstance(item.get("mode"), str) and item["mode"], f"series[{index}].mode is required")
        samples = item.get("samples_ms")
        require(isinstance(samples, dict), f"series[{index}].samples_ms must be an object")
        baseline = samples.get("baseline")
        candidate = samples.get("candidate")
        for name, values in (("baseline", baseline), ("candidate", candidate)):
            require(isinstance(values, list) and len(values) >= 2, f"series[{index}] {name} needs at least two samples")
            require(
                all(type(value) in (int, float) and math.isfinite(value) and value > 0 for value in values),
                f"series[{index}] {name} samples must be finite positive numbers",
            )
        require(len(baseline) == len(candidate), f"series[{index}] sample counts differ")
    return report


def esc(value: object) -> str:
    return html.escape(str(value), quote=True)


def render(report: dict[str, Any]) -> str:
    width = 960
    panel_height = 220
    height = 120 + panel_height * len(report["series"])
    colors = {"baseline": "#d97706", "candidate": "#2563eb"}
    lines = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}" role="img">',
        f"<title>{esc(report.get('title', 'Wild performance comparison'))}</title>",
        "<style>text{font-family:ui-monospace,SFMono-Regular,Menlo,monospace;fill:#172033}.title{font-size:24px;font-weight:700}.label{font-size:14px}.small{font-size:12px;fill:#526070}.grid{stroke:#d8dee9;stroke-width:1}.median{stroke:#111827;stroke-width:2}</style>",
        '<rect width="100%" height="100%" fill="#f8fafc"/>',
        f'<text class="title" x="40" y="42">{esc(report.get("title", "Wild performance comparison"))}</text>',
        f'<text class="small" x="40" y="68">baseline {esc(report.get("baseline_sha", "unknown"))} → candidate {esc(report.get("candidate_sha", "unknown"))}</text>',
    ]
    for panel, item in enumerate(report["series"]):
        top = 95 + panel * panel_height
        baseline = [float(value) for value in item["samples_ms"]["baseline"]]
        candidate = [float(value) for value in item["samples_ms"]["candidate"]]
        all_values = baseline + candidate
        low = min(all_values)
        high = max(all_values)
        padding = max((high - low) * 0.08, high * 0.01)
        axis_low = max(0.0, low - padding)
        axis_high = high + padding
        chart_left, chart_right = 230, 910

        def x(value: float) -> float:
            return chart_left + (value - axis_low) / (axis_high - axis_low) * (chart_right - chart_left)

        base_median = statistics.median(baseline)
        candidate_median = statistics.median(candidate)
        improvement = (base_median - candidate_median) / base_median * 100
        lines.extend(
            [
                f'<text class="label" x="40" y="{top + 18}" font-weight="700">--build-id={esc(item["mode"])}</text>',
                f'<text class="small" x="40" y="{top + 42}">{len(baseline)} alternating pairs</text>',
                f'<text class="label" x="40" y="{top + 70}">{improvement:.1f}% faster</text>',
            ]
        )
        for tick in range(6):
            value = axis_low + (axis_high - axis_low) * tick / 5
            tick_x = x(value)
            lines.append(f'<line class="grid" x1="{tick_x:.2f}" y1="{top + 36}" x2="{tick_x:.2f}" y2="{top + 165}"/>')
            lines.append(f'<text class="small" x="{tick_x:.2f}" y="{top + 187}" text-anchor="middle">{value:.0f} ms</text>')
        for row, (name, values) in enumerate((("baseline", baseline), ("candidate", candidate))):
            y = top + 72 + row * 62
            lines.append(f'<text class="label" x="150" y="{y + 5}" text-anchor="end">{name}</text>')
            for sample_index, value in enumerate(values):
                jitter = ((sample_index * 17) % 13 - 6) * 1.2
                lines.append(f'<circle cx="{x(value):.2f}" cy="{y + jitter:.2f}" r="3" fill="{colors[name]}" opacity="0.55"/>')
            median = statistics.median(values)
            lines.append(f'<line class="median" x1="{x(median):.2f}" y1="{y - 18}" x2="{x(median):.2f}" y2="{y + 18}"/>')
            lines.append(f'<text class="small" x="{x(median) + 6:.2f}" y="{y - 22}">median {median:.3f} ms</text>')
    lines.append("</svg>")
    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    try:
        rendered = render(load_report(args.input))
    except SchemaError as error:
        parser.error(str(error))
    args.output.write_text(rendered)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
