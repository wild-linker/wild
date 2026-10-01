# How to send a PR to upstream Wild

**Mandatory.** Read this whole document before authoring, updating or replying on any pull request
to [`wild-linker/wild`](https://github.com/wild-linker/wild). The `pr-upstream` repo skill
(`.claude/skills/pr-upstream/SKILL.md`) walks through it step by step; use it.

Upstream is a professional project with two maintainers who review every change. A PR that follows
this document should need one review round, not three.

## 1. Before you write code

- Read upstream's [`CONTRIBUTING.md`](CONTRIBUTING.md). In short: short descriptions, one change
  per PR, nightly rustfmt, and the PR description becomes the squash commit message.
- Branch from upstream's current `main`, not the fork's: `git fetch upstream && git worktree add
  -b up/<topic> ../wild-<topic> upstream/main`. Push `up/<topic>` to `origin` and open the PR from
  `zackees:up/<topic>`. Never include fork-only commits.
- Keep the generic path unchanged. Every gain is weighed against complexity: #2620 merged
  because it was "relatively simple" with a large speedup, while #2621 drew a tentative reject for
  adding "a whole new layer of abstraction", even though it was faster. Put an optimization in one
  private module with a single `pub(crate)` or `pub(super)` entry point that falls back to the
  existing code for anything it doesn't handle (see #2634). Avoid new `Arch`/`Platform` trait items
  and edits spread across the arch files. Report the *central diff* (lines touched outside the new
  module) and keep it small.
- Find the cost before designing the fix: `perf record` the link and `perf annotate` the hot loop.
  The biggest wins in the #2621 work were store-forwarding stalls (byte-aligned structs such as the
  raw `Rela` copied to the stack), a runtime division in an alignment check, and a variable-length
  `copy_from_slice` that compiled to a `memcpy` call.
- One idea per PR. Measure each piece on its own before proposing it, and drop anything that is
  noise on real links. In the #2621 work, a symbol cache produced most of the gain, and three small
  standalone ideas were noise.

## 2. Benchmarks (mandatory for every performance PR)

Every performance claim must come from the procedure below. Tables are pasted from the tool's
output, never typed or edited. This is how the numbers in #2634 were produced.

### 2.1 Build the two binaries identically

`#1` is the exact upstream `main` commit the branch is based on; `#2` is the exact PR head.
Upstream has no `rust-toolchain.toml`, so pin the toolchain:

```sh
git fetch upstream
git worktree add --detach ../wild-base upstream/main
for tree in ../wild-base ../wild-<topic>; do
  (cd "$tree" && RUSTUP_TOOLCHAIN=1.98.1 SOLDR_ALLOW_UNPINNED=1 \
    soldr cargo build --release -p wild-linker --bin wild)
done
```

If upstream `main` moves before you publish, rebase, rebuild both and measure again. Old numbers
are context only.

### 2.2 Captures

Benchmarks relink save-dir captures. Capture a link by building the project with Wild as the linker
and `WILD_SAVE_BASE` set, then keep the numbered directory whose `run-with` ends with the main
binary's `# Original output file:`:

```sh
RUSTFLAGS="-Clinker=clang -Clink-arg=--ld-path=$PWD/../wild-base/target/release/wild" \
  WILD_SAVE_BASE=~/save/<name> RUSTUP_TOOLCHAIN=1.98.1 SOLDR_ALLOW_UNPINNED=1 \
  soldr cargo build --release -p wild-linker --bin wild
```

| Benchmark | Link | How to capture | Required |
| --- | --- | --- | --- |
| `wild-lto-debug` | Wild with fat LTO and full debug info | as above plus `CARGO_PROFILE_RELEASE_LTO=fat CARGO_PROFILE_RELEASE_DEBUG=full CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1` | always |
| `wild` | Wild, ordinary release build | as above | always |
| `tinyc` | 5,000 one-function C files plus a main, DWARF 5 | `benchmarks/performance_guard/make_small_captures.sh <wild> <dir>` | always (small-link regression check) |
| `cxxdbg` | 1,500 C++ files using `std::string`/`vector`/`map`, DWARF 5 | the same script | when debug or C++ paths change |
| `rust-analyzer`, `zed-release`, `clang-release`, `clang-debug`, `chrome-android` | the maintainer's larger suite | see `benchmarks/performance_guard/README.md` | optional: hours of build time and up to 25 GiB RAM |

The small-link check is not optional. Per-call setup cost, such as a heap-allocated cache per debug
section, is invisible on big links and dominant on links with thousands of small objects. That is
what caused the "small slowdowns" on #2621.

Name the benchmarks you didn't run in the PR and invite the maintainer to run them; David has the
large captures.

### 2.3 Run the A/B

Use `benchmarks/performance_guard/capture_ab.py` from the fork's `main`. It runs warmups, then
interleaved ABBA pairs until `--pairs`, then keeps going until `--budget-seconds` is spent (capped
by `--max-pairs`). It records wall time, user, system and peak RSS (from `wait4`), plus cycles,
instructions, cache references and misses, and branches and branch misses (from
`perf_event_open`). It also checks that the two outputs are byte-identical apart from `.comment`.

```sh
export TMPDIR=/dev/shm/wild-bench   # outputs on tmpfs, see below
mkdir -p "$TMPDIR"
env -u BASH_ENV python3 benchmarks/performance_guard/capture_ab.py \
  --capture ~/save/wild-lto-debug --benchmark wild-lto-debug \
  --baseline ../wild-base/target/release/wild \
  --baseline-label "upstream-main $(git -C ../wild-base rev-parse --short=12 HEAD)" \
  --candidate ../wild-<topic>/target/release/wild \
  --candidate-label "pr-head $(git -C ../wild-<topic> rev-parse --short=12 HEAD)" \
  --extra=--no-fork --pairs 100 --max-pairs 20000 --budget-seconds 600 \
  --output ~/bench/<topic>/wild-lto-debug.json
python3 benchmarks/performance_guard/pr_table.py ~/bench/<topic>/*.json
```

- Budgets: 600 seconds for `wild-lto-debug` and `wild`, 300 for `tinyc` and `cxxdbg`. That gave
  n = 2,340 and 5,438 pairs for #2634 and confidence intervals of about 0.2%.
- `--no-fork` is required, so peak RSS and CPU time are the linker's own.
- Put outputs on tmpfs (`TMPDIR=/dev/shm/...`). On ext4, page faults followed each arm's output
  file rather than the binary, which once made a candidate look 28% worse in system time.
- `env -u BASH_ENV` is required here; the harness's `BASH_ENV` breaks the generated `run-with`
  scripts.
- The host is shared. Record the 1-minute load (`/proc/loadavg`) with each run. Discard and rerun
  any run that overlapped other heavy work; don't average it in. Add `--cpus` to pin both arms if
  load can't be avoided. Instruction counts are robust to load; wall time is not.
- Remove `/dev/shm/wild-bench` when you're done; it holds hundreds of MB of RAM.

### 2.4 Correctness evidence

- Byte-identical output to `main`, apart from `.comment`, with `--build-id=none` **and**
  `--build-id=fast`, on every capture above. `capture_ab.py` checks this per table; also link each
  capture once with both build-id modes and `cmp` the results.
- If the change has a fast path or a fallback, also prove the fallback: force it, for example with
  a temporary build that makes the fast path always fail, and check output is still identical.
- Changes that reorder or parallelise relocation application must stay correct on RISC-V and
  LoongArch, where some debug relocations read the bytes they patch. Normal-sized tests may never
  reach your path; lower the threshold in a throwaway branch (as in #2634) and run the cross-arch
  CI.

### 2.5 Fork CI (4-vCPU runner numbers)

Also run the real fat-LTO A/B on a GitHub-hosted runner:

1. Build a base branch: upstream `main` plus only the fork's CI and infrastructure commits.
   `pull_request` workflows run from the PR's merge ref, so a pure-upstream base has no fork
   workflows.
2. Push the PR's commits onto it as `bench/<topic>`, and open a draft PR titled
   `[bench] <topic>, do not merge` against the base branch.
3. Add the `performance` label *after* creating the PR, then close and reopen it. A label applied by
   `gh pr create --label` lands after the `opened` event, so label-gated jobs don't start.
4. Read the `capture-ab` job summary and artifact. Hosted runners have no hardware counters, so
   those rows read `n/a`. Close the draft when you're done.

## 3. Verification before pushing

```sh
env -u BASH_ENV RUSTUP_TOOLCHAIN=1.98.1 SOLDR_ALLOW_UNPINNED=1 soldr cargo test --release \
  -p linker-utils -p libwild -p wild-linker -- --skip tidy_tests::check_toml_format
RUSTUP_TOOLCHAIN=1.98.1 SOLDR_ALLOW_UNPINNED=1 soldr cargo clippy --release -p libwild --all-targets
RUSTUP_TOOLCHAIN=nightly SOLDR_ALLOW_UNPINNED=1 soldr cargo fmt --all -- --check
```

- `check_toml_format` fails locally when a worktree contains an untracked `.cargo/registry`;
  CI runs it.
- Upstream CI runs `cargo test --profile ci -p libwild --target wasm32-wasip1` under wasmtime.
  WASI has no temp dir or process spawning; guard such tests with
  `if crate::host::os::SANDBOXED { return; }`. To run it locally, set
  `CARGO_TARGET_WASM32_WASIP1_RUNNER="<wasmtime> --dir . --dir src --dir /"`. Point
  `CC_wasm32_wasip1`/`AR_wasm32_wasip1` at a WASI C compiler
  (`nix build nixpkgs#pkgsCross.wasi32.stdenv.cc`). Add
  `CARGO_TARGET_WASM32_WASIP1_RUSTFLAGS="--cfg local_wasi_cc"` if soldr serves a stale host-built
  `zstd_sys`.
- If you claim the PR is independent of other open PRs, check that they merge and compile together
  with current `main`.

## 4. The PR description

The description is the squash commit message, so keep it short. It starts with the disclosure
header, in italics, then exactly these sections in this order: Summary, Benchmarks, Design.

````markdown
*AI generated description: authored by [AI](https://github.com/zackees/clud), approved by Zach Vorhies*

## Summary

<2–3 sentences: what changes, the headline numbers, and any PRs this supersedes or is stacked on.>

## Benchmarks

`#1` is `main` (<sha>), `#2` is this PR. <machine>, `--no-fork`, interleaved ABBA pairs, output on
tmpfs.

<benchmark name>:

```
<pasted pr_table.py output>
```

<one line per small-link or regression-check result, e.g. tinyc, with numbers.>

On a 4-vCPU GitHub-hosted runner (<n> pairs, fat-LTO link of Wild, vs `main`): <wall> wall and
<CPU> CPU (`--build-id=fast`: <wall> and <CPU>).

Output is byte-identical to `main` with `--build-id=none` and `--build-id=fast`, apart from
`.comment`, on <links>.

I couldn't run <benchmarks>. <Maintainer>, if you have time to run your suite on this branch, that
would cover them.

## Design

- <how it works, as bullets>
- <what stays unchanged in the generic path>
- <correctness on other architectures and the fallback>
- Central diff: <+N/−M in files>; <module> is <N> lines including tests.

````

- For bug fixes, replace Benchmarks with **Repro** (a minimal script you have actually run against
  upstream's binary, with before and after results) and add **When it fails**, as in #2628.
- Stacked PRs start with "Stacked on #N. Please review only the last commit."
- Titles are release-notes friendly with a conventional prefix: `perf:`, `fix:`, `chore:`.
- The first line is always the disclosure header, exactly as above and in italics:
  `*AI generated description: authored by [AI](https://github.com/zackees/clud), approved by Zach Vorhies*`. It is the only place AI authorship is mentioned. Don't mention token counts, CI cost or
  tooling anywhere else.
- **"approved by Zach Vorhies" must be true.** Show Zach the final title and description and get
  explicit approval before creating or editing the PR.

## 5. After you open it

- Editing a description notifies nobody. Post a short comment when reviewers need to know
  something changed.
- Answer review concerns with data and a concrete plan, addressed to whoever raised them.
  Implementing the reviewer's suggested shape and measuring it beats arguing. Draft any reply with
  a sensitive tone for Zach to approve; don't post it unasked.
- Don't replace the code under a PR the maintainer has just validated. Open a follow-up or
  superseding PR and cross-reference both ways. Close the old PR only when the new one is on par.
- The GitHub API allows 5,000 requests an hour, shared by every tool and agent. Don't hand-roll
  polling loops. Use `clud tool run github/pr_merge_watch.py` with long intervals, and check
  `gh api rate_limit` before bursts.

## 6. Local environment

- Before blaming a tool, reproduce the failure in a stock container and hash the suspect files.
  A "soldr cache bug" turned out to be toolchain rlibs zeroed through wild's save-dir hard links
  (fixed in this fork by #41; proposed upstream as wild-linker/wild#2628).
  `find ~/.rustup ~/.soldr/rustup -path '*rustlib*' -size 0` finds that damage.
- soldr's `stable` is rustc 1.98.0. Upstream checkouts have no toolchain file, so always pass
  `RUSTUP_TOOLCHAIN=1.98.1 SOLDR_ALLOW_UNPINNED=1`.
