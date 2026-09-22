#!/usr/bin/env python3
"""Create real, locally compiled ELF link workloads shared by both revisions."""

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path


def run(command: list[str]) -> None:
    subprocess.run(command, check=True)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--clang", default="clang")
    args = parser.parse_args()
    root = args.output.resolve()
    root.mkdir(parents=True, exist_ok=True)

    ordinary = root / "ordinary.c"
    ordinary.write_text(
        "#include <stdio.h>\n"
        "static int sum(int n) { int v=0; for(int i=0;i<n;i++) v += i; return v; }\n"
        "int main(void) { return sum(10) == 45 ? 0 : 1; }\n"
    )
    ordinary_o = root / "ordinary.o"
    run(
        [
            args.clang,
            "-g",
            "-O2",
            "-ffunction-sections",
            "-c",
            str(ordinary),
            "-o",
            str(ordinary_o),
        ]
    )

    # Multiple full-LTO translation units produce a monolithic plugin object at link time.
    # Debug info and retained function pointers ensure that this is a real debug-heavy link,
    # rather than a synthetic sleep or a pre-recorded timing vector.
    lto_objects: list[str] = []
    functions_per_unit = 220
    for unit in range(12):
        source = root / f"lto-{unit}.c"
        lines = ["#include <stdint.h>"]
        for function in range(functions_per_unit):
            index = unit * functions_per_unit + function
            lines.append(
                f"__attribute__((noinline,used)) uint64_t f{index}(uint64_t x) "
                f"{{ return (x * {index + 3}u) ^ {index * 2654435761 % (2**32)}u; }}"
            )
        lines.append("typedef uint64_t (*fn)(uint64_t);")
        lines.append(
            "__attribute__((used)) fn table_%d[] = {%s};"
            % (
                unit,
                ",".join(
                    f"f{unit * functions_per_unit + i}"
                    for i in range(functions_per_unit)
                ),
            )
        )
        source.write_text("\n".join(lines) + "\n")
        obj = root / f"lto-{unit}.o"
        run(
            [
                args.clang,
                "-g",
                "-O1",
                "-flto",
                "-ffunction-sections",
                "-c",
                str(source),
                "-o",
                str(obj),
            ]
        )
        lto_objects.append(str(obj))

    main_source = root / "lto-main.c"
    main_source.write_text(
        "#include <stdint.h>\nextern uint64_t f0(uint64_t);\nint main(void) { return f0(0) == 0 ? 0 : 1; }\n"
    )
    main_obj = root / "lto-main.o"
    run([args.clang, "-g", "-O1", "-flto", "-c", str(main_source), "-o", str(main_obj)])
    lto_objects.append(str(main_obj))

    manifest = {
        "schema_version": 1,
        "ordinary": {"objects": [str(ordinary_o)], "link_flags": []},
        "full-lto": {"objects": lto_objects, "link_flags": ["-flto", "-O1"]},
    }
    (root / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
