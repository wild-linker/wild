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
