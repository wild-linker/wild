# Pull-request performance guard

This directory implements the Linux-only PR comparison in issue #2. It builds and
measures the exact merge base and PR head in one job. Hyperfine 1.20.0 is the
authoritative timing source; Poop 0.5.0 is diagnostic only because its output is
human-oriented and Linux hardware counters can be unavailable on hosted runners.
Both are MIT-licensed and downloaded from their upstream GitHub releases with
checked SHA-256 digests.

The policy is intentionally `calibration_mode: true`. Results are reported and raw
evidence is retained, but an unproven 1.5% threshold is not yet a merge gate on
variable GitHub-hosted machines. Infrastructure, command, correctness, and artifact
equivalence failures still fail the workflow. Setting calibration mode to false after
repeat-run calibration activates regression gating and the `performance` label's
required-target-improvement rule.

Correctness checking executes both original linked outputs, then compares
normalized copies byte-for-byte. Normalization replaces only the exact
NUL-delimited `Linker: Wild <revision> (compatible with GNU linkers)` provenance
record in `.comment`; all other comment records are preserved. For
`--build-id=fast`, it also removes `.note.gnu.build-id`. Differences in every
other section, including allocated runtime content and unrelated non-allocated
structure, remain failures.

When the action is given a `reference` binary (upstream wild), the candidate's
normalized output must match the reference's instead of the merge base's. The
fork's output is required to match upstream, so a PR that restores upstream
output passes even though it changes the output relative to the merge base.
Whether the candidate still matches the merge base is recorded in the report
as `baseline_equivalent`. Timing always compares merge base and candidate.

Deployment uses two PRs. The first lands the action, policy, driver, workloads, and
tests without a workflow. After that PR merges, a second PR adds the workflow and
pins this action to the durable merge commit on `origin/main`. The benchmarked PR
tree therefore cannot replace its policy, workload generator, or classifier.

Future action updates use the same two-phase process: land and validate the action
implementation first, then update the workflow's immutable action SHA separately.
Branch protection should require the base branch's named performance check and
review changes to workflow/action paths. GitHub evaluates workflow changes from a
PR ref, so repository review and branch protection remain part of the trust model;
the pinned action prevents such a change from silently replacing the executable
measurement logic.

`full-lto-none-control` isolates non-hashing work. It is never presented as the
realistic result: ordinary and debug-heavy/full-LTO workloads both include
`--build-id=fast`. No output-size scaling claim is made by this suite.

## Comparing fork revisions (issue #14)

`compare_revisions.py` benchmarks already-built Wild binaries against a baseline
with one Hyperfine run per case (`ordinary-none`, `large-debug-none`,
`full-lto-none`, `large-debug-fast`). The first `--build` is the baseline. Any
other build more than `--noise-percent` (default 2%) slower on any case exits 1:
drop that change.

```sh
git fetch upstream
git worktree add ../wild-upstream upstream/main
git worktree add ../wild-pr5 upstream/main
git -C ../wild-pr5 cherry-pick fc159eb4
git worktree add ../wild-tip origin/main
# rust-toolchain.toml pins the toolchain in each worktree.
for tree in ../wild-upstream ../wild-pr5 ../wild-tip; do
  (cd "$tree" && cargo build --release -p wild)
done

python3 benchmarks/performance_guard/prepare_workloads.py --output /tmp/wild-workloads
python3 benchmarks/performance_guard/compare_revisions.py \
  --manifest /tmp/wild-workloads/manifest.json \
  --build "upstream-main=$(git -C ../wild-upstream rev-parse HEAD)=../wild-upstream/target/release/wild" \
  --build "upstream-main+5=$(git -C ../wild-pr5 rev-parse HEAD)=../wild-pr5/target/release/wild" \
  --build "fork-tip=$(git -C ../wild-tip rev-parse HEAD)=../wild-tip/target/release/wild" \
  --runs 10 --output /tmp/wild-compare
```

Results are written to `results.json` and `results.md` in `--output`.

## Upstream performance PRs: the per-benchmark table

Every performance PR we propose to `wild-linker/wild` must include, for each benchmark it
touches, a table in the format the maintainer uses to review performance changes
([wild#2621](https://github.com/wild-linker/wild/pull/2621#issuecomment-5910436060); these are
its `wild-lto-debug` numbers, with this tool's header lines):

```
#1 baseline  upstream-main <upstream-sha>
#2 candidate pr-head <pr-sha>
  metric                       mean        CI ±         SD       n   delta vs #1 (95% CI)
  wall                    81.263 ms       0.09%      1.87%    1771   -6.60% [-6.71%, -6.49%]
  user                   687.344 ms       0.18%      3.83%    1771   -16.37% [-16.56%, -16.18%]
  system                 173.787 ms       0.50%     10.79%    1771   -2.31% [-3.01%, -1.60%]
  peak RSS               467.30 MiB       0.01%      0.16%    1771   +0.07% [+0.06%, +0.08%]
  cycles                   1.313 G~       0.29%      6.31%    1771   -17.14% [-17.42%, -16.87%]
  instructions             1.370 G~       0.22%      4.76%    1771   -34.47% [-34.67%, -34.27%]
  cache-references        94.601 M~       3.42%     73.34%    1771   -23.56% [-27.09%, -20.02%]
  cache-misses            18.960 M~       2.95%     63.36%    1771   -16.94% [-20.43%, -13.44%]
  branches               209.206 M~       0.46%      9.92%    1771   -34.43% [-34.82%, -34.05%]
  branch-misses           63.827 M~       4.76%    102.23%    1771   -16.87% [-22.05%, -11.70%]
```

Rules:

- `#1` is the exact current upstream `main` the PR branch is based on, and `#2` is the exact PR
  head. Rebuild both after every rebase or push; old numbers are context only. Both are built the
  same way (`--release`, same toolchain, no debug info).
- The table comes from `capture_ab.py` / `pr_table.py`. Paste their output; never hand-type or
  edit numbers.
- Include every benchmark below that the change can plausibly affect, and at least `wild` and
  `wild-lto-debug`. For a benchmark you couldn't run, say so rather than leaving it out silently.
- Outputs must be identical apart from the `.comment` provenance record and build ID; the tool
  checks this for every table.

The maintainer's own tool that prints this layout isn't published. It isn't in upstream's
`benchmarks/runner` (whose `report` uses 99% intervals and no hardware counters), and his earlier
reviews used [poop](https://github.com/andrewrk/poop). `capture_ab.py` reproduces the layout:

- Runs are warmed up, then interleaved in ABBA pairs until `--pairs` is reached and, after that,
  until `--budget-seconds` of measuring is spent (capped by `--max-pairs`). Short links need
  thousands of runs for CIs of about 0.1%; the maintainer used n = 10,881 for `wild`.
- wall is timed around the fork and exec of `run-with`. user, system and peak RSS come from
  `wait4(2)` rusage, so pass `--no-fork` (the default in the commands below) to make the linker do
  its own teardown and have peak RSS be the linker's.
- cycles, instructions, cache-references, cache-misses, branches and branch-misses come from
  `perf_event_open(2)` counters attached to the child before exec (no `perf` wrapper overhead),
  counting user and kernel work where `perf_event_paranoid` allows it and user-only otherwise.
  Rows print `n/a` with the reason when the host has no usable PMU, as on GitHub-hosted runners.
- `mean`, `n`, `CI ±` and `SD` describe `#2`. `CI ±` is the 95% Student-t half-width of the mean
  and `SD` the sample standard deviation, both as a percentage of the mean. `delta vs #1` is
  `mean(#2) / mean(#1) - 1`, with a 95% interval from the delta method on that ratio (standard
  error `r * sqrt((se2/m2)^2 + (se1/m1)^2)`, Welch-Satterthwaite degrees of freedom).

### Running it

```sh
# Build both binaries the same way. Upstream has no rust-toolchain.toml, so pin the toolchain.
git worktree add ../wild-base upstream/main
git worktree add ../wild-pr <pr-branch>
for tree in ../wild-base ../wild-pr; do
  (cd "$tree" && RUSTUP_TOOLCHAIN=1.98.1 SOLDR_ALLOW_UNPINNED=1 \
    soldr cargo build --release -p wild-linker --bin wild)
done

# One benchmark: a save-dir captured with WILD_SAVE_BASE (see below).
python3 benchmarks/performance_guard/capture_ab.py \
  --capture ~/save/wild-lto-debug --benchmark wild-lto-debug \
  --baseline ../wild-base/target/release/wild \
  --baseline-label "upstream-main $(git -C ../wild-base rev-parse --short=12 HEAD)" \
  --candidate ../wild-pr/target/release/wild \
  --candidate-label "pr-head $(git -C ../wild-pr rev-parse --short=12 HEAD)" \
  --extra=--no-fork --pairs 100 --max-pairs 20000 --budget-seconds 600 \
  --output /tmp/pr-tables/wild-lto-debug.json

# The PR-description markdown for everything measured.
python3 benchmarks/performance_guard/pr_table.py /tmp/pr-tables/*.json
```

Measure on a quiet machine (check `/proc/loadavg`), and add `--cpus` to pin both arms to the same
cores if other work can't be stopped. The JSON keeps every raw sample; attach it or keep it with
the PR.

### Benchmarks

Captures are made as described in [BENCHMARKING.md](../../BENCHMARKING.md): build the project with
Wild as the linker (`RUSTFLAGS="-Clinker=clang -Clink-arg=--ld-path=$(which wild)"`, or
`-DCMAKE_{EXE,SHARED}_LINKER_FLAGS=--ld-path=$(which wild)` for CMake) and
`WILD_SAVE_BASE=~/save/<name>`, then keep the numbered directory whose `run-with` ends with the
main binary's `# Original output file:`.

| Benchmark | What is linked | How to capture | Peak RSS / link time (maintainer's machine) | Where |
| --- | --- | --- | --- | --- |
| `wild-lto-debug` | Wild, fat LTO + full debug info | `CARGO_PROFILE_RELEASE_LTO=fat CARGO_PROFILE_RELEASE_DEBUG=full CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo build --release -p wild-linker --bin wild` | 0.5 GiB / 80 ms | CI and local |
| `wild` | Wild, ordinary release build | `cargo build --release -p wild-linker --bin wild` | 90 MiB / 30 ms | CI and local |
| `rust-analyzer` | rust-analyzer release | `cargo build --release -p rust-analyzer` in rust-lang/rust-analyzer | 0.65 GiB / 120 ms | local |
| `zed-release` | Zed editor release | `cargo build --release -p zed` in zed-industries/zed | 6.5 GiB / 0.6 s | local, big machine |
| `clang-release` | clang from LLVM, `CMAKE_BUILD_TYPE=Release` | CMake + Ninja build of `clang` with the flags above | 0.8 GiB / 100 ms | local, long build |
| `clang-debug` | clang from LLVM, `CMAKE_BUILD_TYPE=Debug` | as above with `Debug` | **25 GiB** / 2.3 s | local, needs 32+ GiB RAM |
| `chrome-android` | Chromium for Android (`target_os="android"`) | `gn gen` + `autoninja` of the main Android library | 4.2 GiB / 1.3 s | local, very long build and large disk |

Hosted CI (4 CPUs, ~16 GB RAM, 60-minute job) runs only `wild-lto-debug` and `wild`; the others
either can't be built within the job budget or don't fit in memory, so they are measured locally
and pasted from the tool's output.

### What CI produces

Labelling a fork PR `performance` runs the `capture-ab` job of
`.github/workflows/performance-guard.yml` (the label only takes effect on the next push or
reopen). It builds the merge base, the PR head and the newest upstream commit the PR contains,
captures `wild-lto-debug` and `wild`, and for each prints this table (merge base as `#1`, PR head
as `#2`, about four minutes of interleaved pairs each) to the job summary. The JSON reports and
`pr-table.md` are in the `capture-ab-<sha>` artifact. It also checks that the PR head's output is
byte-identical to upstream wild's with `--build-id=none` and `--build-id=fast`.

These CI tables compare against the fork's merge base, so they show what the fork PR changes. For
the upstream PR, rerun the tool with upstream `main` as `#1` and the upstream-bound branch as
`#2`, as above. Hosted runners have no hardware counters, so the counter rows there read `n/a`.
