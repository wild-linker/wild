# Repository instructions

## GitHub target

- This checkout's repository is `zackees/wild`, represented by the `origin` remote.
- Treat `wild-linker/wild`, represented by the `upstream` remote, as read-only unless the user explicitly names `wild-linker/wild` or explicitly asks for an upstream mutation in the current request.
- Unqualified requests such as "file an issue", "open a PR", "add a label", "post a comment", or similar GitHub mutations always target `zackees/wild`.
- Before an unqualified GitHub mutation, resolve and verify the target from `origin`; do not infer the target from issue numbers, earlier upstream discussion, PR context, or the repository's upstream relationship.
- Never fall back to `wild-linker/wild` when a requested operation is unavailable on `zackees/wild`. If the fork has Issues disabled, permissions are missing, or another repository setting blocks the operation, stop and report that blocker to the user.
- Read-only upstream research is allowed when useful. It does not authorize writing to upstream.

## Branches

- `upstream` mirrors `wild-linker/wild` `main`. Only fast-forward it to upstream's `main`; never commit to it.
- `main` is the fork: its own commits rebased on `upstream`. To take new upstream work, fast-forward `upstream`, then rebase `main` onto it.
- PR branches come off `main` and target `main`.
- Branches for `wild-linker/wild` PRs are named `up/<topic>`, come off upstream's current `main` (not the fork's), are pushed to `origin`, and must not include fork-only commits.
- The performance guard requires a PR's output to match wild built from the newest `upstream` commit the PR contains, so keep `upstream` exactly equal to what `main` is rebased on.

## Performance PRs

- Every performance PR proposed to `wild-linker/wild` must include the per-benchmark statistics table described in `benchmarks/performance_guard/README.md` ("Upstream performance PRs"), comparing the exact PR head against the exact upstream `main` it is based on.
- Generate the tables with `benchmarks/performance_guard/capture_ab.py` and paste the output of `pr_table.py`; never hand-type or edit the numbers. Fork CI's tables (the `performance` label) compare against the fork's merge base and are not a substitute.

## Upstream contributions: lessons learned

These come from getting debug-relocation work into `wild-linker/wild` (#2620 merged; #2621 and #2629 reviewed; #2628 and #2630 open). Follow them to avoid repeat review rounds.

**What the maintainers weigh**

- Every gain is weighed against complexity. #2620 merged because it was "relatively simple" with a large speedup; #2621 drew a tentative reject for adding "a whole new layer of abstraction", despite being faster.
- Keep the generic path unchanged. Put an optimization in one private module with a single `pub(crate)` entry point that falls back to the existing code for anything it doesn't handle. Avoid new `Arch`/`Platform` trait items and edits spread across the arch files.
- Report and minimise the central diff: lines touched outside the new module. Restoring or deleting code in shared paths is a strength; say so.
- One change per PR. Upstream squashes each PR into one commit; separable ideas go in separate PRs, each with its own numbers. Don't open many PRs at once.

**Measuring**

- Ablate before proposing: measure each piece on its own and drop anything inside the noise. In #2621's work, a symbol cache produced most of the gain, and three small standalone ideas were noise on real links.
- Always include a small-link regression check (many small objects, e.g. a 5,000-file C link). Per-call setup cost, such as a heap-allocated cache per debug section, is invisible on big links and dominant on small ones. That is what caused the "small slowdowns" on #2621.
- Measure the exact PR head against the exact current upstream `main`. If `main` moves, rebase and measure again before publishing numbers.
- Local hosts are often loaded by other work. Use instruction counts and paired, interleaved, pinned runs locally; take wall time from a quiet window or from the fork CI's real-link A/B.
- `perf annotate` the hot loop. The biggest wins came from store-forwarding stalls (copying byte-aligned structs such as the raw `Rela`), a runtime division in an alignment check, and a variable-length `copy_from_slice` that became a `memcpy` call.
- Byte-identical output against `main` with `--build-id=none` and `--build-id=fast` is expected in every performance PR description.

**Writing the PR**

- Follow `CONTRIBUTING.md`: short descriptions, in the contributor's own voice. They become the squash commit message. Titles are release-notes friendly with a conventional prefix (`perf:`, `fix:`). Format with nightly rustfmt.
- Structure: Summary (2–3 sentences with the headline numbers), then what changed or why it fails, then the benchmark tables, then the byte-identity statement. For bug fixes, include a minimal repro that you have actually run against upstream's binary, and when it triggers.
- Name the benchmarks you couldn't run (zed, clang, chrome-android and rust-analyzer take hours to build) and invite the maintainer to run their suite.
- Don't mention LLMs, agents or token spend; `CONTRIBUTING.md` asks for contributor-written PRs.

**Review etiquette**

- Answer review concerns with data and a concrete plan, addressed to whoever raised them. Implementing the reviewer's suggested shape and measuring it beats arguing.
- Don't replace code under a PR the maintainer has just validated. Send a follow-up or superseding PR, state "Stacked on #N; review only the last commit" when stacked, and cross-reference both ways.
- Editing a PR description doesn't notify anyone; post a short comment when reviewers need to know.

**Upstream CI gotchas**

- `cargo test -p libwild --target wasm32-wasip1` runs every unit test under WASI, which has no temp dir or process spawning. Guard such tests with `if crate::host::os::SANDBOXED { return; }`.
- On RISC-V and LoongArch some debug relocations read the bytes they patch (add/subtract and ULEB128 pairs). Anything that reorders or parallelises relocation application must keep those serial.
- Check that the PR, its sibling PRs and current `main` merge and compile together before claiming they're independent.

**Fork and GitHub mechanics**

- To benchmark an upstream PR on fork CI, open a draft `[bench]` PR whose base is upstream's `main` plus only the fork's CI/infra commits. `pull_request` workflows run from the PR's merge ref, so a pure-upstream base has no fork workflows.
- `gh pr create --label` applies the label after the `opened` event fires, so label-gated jobs don't start. Add the label, then close and reopen the PR (or push).
- The GitHub API allows 5,000 requests an hour, shared across every tool and agent. Avoid hand-written polling loops; use `clud tool run github/pr_merge_watch.py` with long intervals and check `gh api rate_limit` before bursts.

**Local environment**

- Before blaming a tool, reproduce the failure in a stock container and hash the suspect files. A "soldr cache bug" turned out to be toolchain rlibs zeroed through wild's save-dir hard links (fixed in this fork by #41; proposed upstream as wild-linker/wild#2628); `find ~/.rustup ~/.soldr/rustup -path '*rustlib*' -size 0` finds that damage.
- soldr's `stable` is rustc 1.98.0; this repository pins 1.98.1 (`rust-toolchain.toml`). Build upstream checkouts, which have no toolchain file, with `RUSTUP_TOOLCHAIN=1.98.1 SOLDR_ALLOW_UNPINNED=1`.
