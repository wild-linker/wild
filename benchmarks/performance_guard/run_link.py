#!/usr/bin/env python3
"""Run one prepared link. This is the command timed by Hyperfine and Poop."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--workload", required=True)
    parser.add_argument("--linker", type=Path, required=True)
    parser.add_argument("--build-id", choices=("none", "fast"), required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--clang", default="clang")
    args = parser.parse_args()
    manifest = json.loads(args.manifest.read_text())
    workload = manifest[args.workload]
    try:
        args.output.unlink()
    except FileNotFoundError:
        pass
    command = [
        args.clang,
        f"--ld-path={args.linker.resolve()}",
        *workload["link_flags"],
        *workload["objects"],
        f"-Wl,--build-id={args.build_id}",
        "-o",
        str(args.output),
    ]
    environment = os.environ.copy()
    environment.setdefault("WILD_NUM_THREADS", str(min(os.cpu_count() or 1, 8)))
    return subprocess.run(command, env=environment).returncode


if __name__ == "__main__":
    raise SystemExit(main())
