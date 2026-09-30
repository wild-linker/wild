#!/usr/bin/env python3
"""Paired A/B of two Wild binaries relinking one `WILD_SAVE_DIR` capture.

Runs are interleaved (ABBA order) after warmups. Every sample records wall time, user and system
time and peak RSS (wait4(2) rusage; pass `--extra=--no-fork` so that is the linker's own), plus the
hardware counters perf_event_open(2) allows. Outputs of the warmup runs are compared after
normalising the Wild provenance record in `.comment` and the build ID, so a speedup can't come
from linking something different.

The report embeds the per-benchmark statistics table from `pr_table.py`, the format upstream
performance PRs must carry; see README.md.
"""

from __future__ import annotations

import argparse
import ctypes
import json
import os
import platform
import signal
import statistics
import struct
import subprocess
import sys
import tempfile
import time
from pathlib import Path

import pr_table

PERF_TYPE_HARDWARE = 0
# Hardware counters in table order, with their PERF_COUNT_HW_* config values.
COUNTERS = (
    ("cycles", 0),
    ("instructions", 1),
    ("cache-references", 2),
    ("cache-misses", 3),
    ("branches", 4),
    ("branch-misses", 5),
)
_SYSCALL_PERF_EVENT_OPEN = {"x86_64": 298, "aarch64": 241, "riscv64": 241}
_DISABLED, _INHERIT, _EXCLUDE_KERNEL, _EXCLUDE_HV, _ENABLE_ON_EXEC = 1, 2, 1 << 5, 1 << 6, 1 << 12
_READ_TIMES = 1 | 2  # PERF_FORMAT_TOTAL_TIME_ENABLED | PERF_FORMAT_TOTAL_TIME_RUNNING


class _PerfEventAttr(ctypes.Structure):
    # PERF_ATTR_SIZE_VER0, which every kernel accepts.
    _fields_ = [
        ("type", ctypes.c_uint32),
        ("size", ctypes.c_uint32),
        ("config", ctypes.c_uint64),
        ("sample_period", ctypes.c_uint64),
        ("sample_type", ctypes.c_uint64),
        ("read_format", ctypes.c_uint64),
        ("flags", ctypes.c_uint64),
        ("wakeup_events", ctypes.c_uint32),
        ("bp_type", ctypes.c_uint32),
        ("config1", ctypes.c_uint64),
    ]


class Counters:
    """Hardware counters for one linker run, opened with perf_event_open(2).

    Counters are attached to the forked child before it execs and enabled on exec, so they count
    the linker (and anything it forks) but not this script or a `perf` wrapper. Unavailable
    counters (no PMU on a VM, perf_event_paranoid too strict) leave `available` empty and
    `reason` saying why; those rows are then printed as n/a.
    """

    def __init__(self, enabled: bool = True):
        self.available: list[tuple[str, int]] = []
        self.scope = ""
        self.reason = "disabled with --no-counters"
        self.max_multiplexing = 1.0
        if not enabled:
            return
        number = _SYSCALL_PERF_EVENT_OPEN.get(platform.machine())
        if number is None:
            self.reason = f"perf_event_open not wired up for {platform.machine()}"
            return
        self._syscall = ctypes.CDLL(None, use_errno=True).syscall
        self._number = number
        errors = {}
        for scope, flags in (("user+kernel", 0), ("user only", _EXCLUDE_KERNEL | _EXCLUDE_HV)):
            self._flags = flags
            usable = []
            for name, config in COUNTERS:
                try:
                    os.close(self._open(0, config, probe=True))
                    usable.append((name, config))
                except OSError as error:
                    errors[name] = error.strerror
            if usable:
                self.available, self.scope = usable, scope
                self.reason = ""
                break
        missing = [name for name, _ in COUNTERS if name not in dict(self.available)]
        if missing:
            detail = sorted(set(errors[name] for name in missing if name in errors))
            self.reason = "hardware counter unavailable: " + ", ".join(detail or ["unknown"])

    def _open(self, pid: int, config: int, probe: bool = False) -> int:
        attr = _PerfEventAttr(type=PERF_TYPE_HARDWARE, size=ctypes.sizeof(_PerfEventAttr),
                              config=config, read_format=_READ_TIMES)
        attr.flags = self._flags | _DISABLED | (0 if probe else _INHERIT | _ENABLE_ON_EXEC)
        fd = self._syscall(ctypes.c_long(self._number), ctypes.byref(attr), ctypes.c_int(pid),
                           ctypes.c_int(-1), ctypes.c_int(-1), ctypes.c_ulong(0))
        if fd < 0:
            code = ctypes.get_errno()
            raise OSError(code, os.strerror(code))
        return fd

    def attach(self, pid: int) -> dict[str, int]:
        fds = {}
        try:
            for name, config in self.available:
                fds[name] = self._open(pid, config)
        except OSError:
            for fd in fds.values():
                os.close(fd)
            raise
        return fds

    def read(self, fds: dict[str, int]) -> dict[str, float]:
        values = {}
        for name, fd in fds.items():
            value, enabled, running = struct.unpack("QQQ", os.read(fd, 24))
            os.close(fd)
            if running:
                # Scale up if the kernel had to multiplex more counters than the PMU has.
                self.max_multiplexing = max(self.max_multiplexing, enabled / running)
                values[name] = value * enabled / running
        return values


def run_once(capture: Path, wild: Path, out: Path, counters: Counters | None, extra: list[str],
             cpus: str | None) -> dict[str, float]:
    """One link: wall time, user/system time and peak RSS from wait4(2), plus counters."""
    env = dict(os.environ)
    env.pop("BASH_ENV", None)
    env["OUT"] = str(out)
    command = [str(capture / "run-with"), str(wild)]
    if extra:
        command += ["--", *extra]
    if cpus:
        command = ["taskset", "-c", cpus, *command]
    ready_read, ready_write = os.pipe()
    pid = os.fork()
    if pid == 0:  # pragma: no cover - child
        try:
            os.close(ready_write)
            os.read(ready_read, 1)
            devnull = os.open(os.devnull, os.O_WRONLY)
            os.dup2(devnull, 1)
            os.execvpe(command[0], command, env)
        finally:
            os._exit(127)
    os.close(ready_read)
    try:
        fds = counters.attach(pid) if counters and counters.available else {}
    except BaseException:
        os.kill(pid, signal.SIGKILL)
        os.waitpid(pid, 0)
        raise
    start = time.perf_counter()
    os.write(ready_write, b"x")
    os.close(ready_write)
    _, status, usage = os.wait4(pid, 0)
    wall_ms = (time.perf_counter() - start) * 1000
    sample = counters.read(fds) if fds else {}
    code = os.waitstatus_to_exitcode(status)
    if code != 0:
        raise subprocess.CalledProcessError(code, command)
    sample.update({
        "wall_ms": wall_ms,
        "user_ms": usage.ru_utime * 1000,
        "sys_ms": usage.ru_stime * 1000,
        "max_rss_kib": float(usage.ru_maxrss),
    })
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


def keep_going(pairs: int, elapsed: float, min_pairs: int, max_pairs: int,
               budget_seconds: float) -> bool:
    """Whether to run another pair: at least `min_pairs`, then until the time budget is spent."""
    if pairs < min_pairs:
        return True
    return pairs < max_pairs and elapsed < budget_seconds


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--capture", type=Path, required=True,
                        help="WILD_SAVE_DIR directory containing run-with")
    parser.add_argument("--benchmark", help="benchmark name for the table, e.g. wild-lto-debug")
    parser.add_argument("--baseline", type=Path, required=True, help="arm #1")
    parser.add_argument("--candidate", type=Path, required=True, help="arm #2")
    parser.add_argument("--baseline-label", default="baseline", help="e.g. 'upstream-main <sha>'")
    parser.add_argument("--candidate-label", default="candidate", help="e.g. 'pr-head <sha>'")
    parser.add_argument("--reference", type=Path,
                        help="untimed binary (e.g. upstream wild) whose output the candidate must match")
    parser.add_argument("--pairs", type=int, default=30,
                        help="minimum measured pairs; 0 only checks output identity")
    parser.add_argument("--max-pairs", type=int, help="stop here even within budget (default: --pairs)")
    parser.add_argument("--budget-seconds", type=float, default=0,
                        help="after --pairs, keep adding pairs until this much measuring time is spent")
    parser.add_argument("--warmups", type=int, default=4)
    parser.add_argument("--cpus", help="taskset CPU list, e.g. 0-7")
    parser.add_argument("--extra", default="", help="extra linker arguments, space separated")
    parser.add_argument("--no-counters", action="store_true", help="skip hardware counters")
    parser.add_argument("--output", type=Path, required=True, help="JSON report path")
    parser.add_argument("--markdown", type=Path, help="append the result as markdown here")
    args = parser.parse_args()

    counters = Counters(enabled=not args.no_counters)
    extra = args.extra.split()
    max_pairs = args.max_pairs if args.max_pairs is not None else args.pairs
    arms = {"baseline": args.baseline.resolve(), "candidate": args.candidate.resolve()}
    samples: dict[str, list[dict[str, float]]] = {name: [] for name in arms}

    with tempfile.TemporaryDirectory() as tmp:
        workdir = Path(tmp)
        outs = {name: workdir / f"{name}.out" for name in arms}
        for name, wild in arms.items():
            for _ in range(max(1, args.warmups)):
                run_once(args.capture, wild, outs[name], None, extra, args.cpus)
        candidate_bytes = normalised(outs["candidate"], workdir)
        identical = normalised(outs["baseline"], workdir) == candidate_bytes
        matches_reference = None
        if args.reference:
            reference_out = workdir / "reference.out"
            run_once(args.capture, args.reference.resolve(), reference_out, None, extra, args.cpus)
            matches_reference = normalised(reference_out, workdir) == candidate_bytes
        start = time.monotonic()
        pair = 0
        while keep_going(pair, time.monotonic() - start, args.pairs, max_pairs,
                         args.budget_seconds):
            order = list(arms.items())
            if pair % 2:
                order.reverse()
            for name, wild in order:
                samples[name].append(
                    run_once(args.capture, wild, outs[name], counters, extra, args.cpus))
            pair += 1

    identity = [f"Baseline and candidate outputs identical apart from `.comment` and build ID: "
                f"**{identical}**"]
    if matches_reference is not None:
        identity.append(f"Candidate output identical to reference apart from `.comment` and build "
                        f"ID: **{matches_reference}**")
    report = {
        "schema_version": 2,
        "benchmark": args.benchmark,
        "host": {"platform": platform.platform(), "machine": platform.machine(),
                 "logical_cpus": os.cpu_count()},
        "capture": str(args.capture),
        "extra_args": extra,
        "cpus": args.cpus,
        "labels": {"baseline": args.baseline_label, "candidate": args.candidate_label},
        "pairs": pair,
        "counters": {"available": [name for name, _ in counters.available],
                     "scope": counters.scope, "unavailable_reason": counters.reason,
                     "max_multiplexing": counters.max_multiplexing},
        "outputs_identical": identical,
        "matches_reference": matches_reference,
        "summary": {name: summarise(s) for name, s in samples.items() if s},
        "samples": samples,
    }
    lines = []
    if pair:
        unavailable = {name: counters.reason for name, _ in COUNTERS}
        report["table"] = pr_table.render(
            {"label": args.baseline_label, "samples": samples["baseline"]},
            {"label": args.candidate_label, "samples": samples["candidate"]},
            unavailable)
        notes = [f"Linker args: `{' '.join(extra) or '(as captured)'}`; {pair} interleaved pairs"]
        if counters.available:
            notes.append(f"counters: {counters.scope}"
                         + (f", multiplexed up to {counters.max_multiplexing:.2f}x"
                            if counters.max_multiplexing > 1.001 else ""))
        report["table_notes"] = "; ".join(notes) + "."
        lines += [pr_table.markdown(report), ""]
    lines += identity
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=1) + "\n")

    text = "\n".join(lines)
    print(text)
    if args.markdown:
        with args.markdown.open("a") as handle:
            handle.write(text + "\n")
    return 0 if identical and matches_reference is not False else 1


if __name__ == "__main__":
    sys.exit(main())
