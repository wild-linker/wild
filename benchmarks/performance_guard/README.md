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
