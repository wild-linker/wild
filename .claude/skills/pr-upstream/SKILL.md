---
name: pr-upstream
description: Author, update or reply on a pull request to upstream wild-linker/wild. Use whenever a PR, PR description, PR comment or review reply for wild-linker/wild is being written or changed, including performance PRs and bug fixes. Enforces HOW_TO_PR_UPSTREAM.md (benchmarks, verification, description format, disclosure header, approval).
---

# PR to upstream Wild

`HOW_TO_PR_UPSTREAM.md` at the repository root is the source of truth. This skill makes sure it is
followed. Do these steps in order and don't skip any.

1. **Read `HOW_TO_PR_UPSTREAM.md` in full**, from the checked-in copy, not from memory. Also read
   upstream's `CONTRIBUTING.md`.
2. **Confirm the target.** Upstream writes need the user's explicit request in this conversation
   (see `AGENTS.md`, "GitHub target"). If the request doesn't name `wild-linker/wild` or ask for an
   upstream change, stop and ask.
3. **Branch** from upstream's current `main` as `up/<topic>` (guide §1). Keep the generic path
   unchanged and the central diff small. Report it.
4. **Benchmarks** for any performance claim: follow guide §2 exactly.
   - Build both binaries the same way, with `RUSTUP_TOOLCHAIN=1.98.1 SOLDR_ALLOW_UNPINNED=1`.
   - Run `capture_ab.py` with `--extra=--no-fork`, `TMPDIR` on tmpfs and the listed budgets.
   - Cover at least `wild-lto-debug`, `wild` and `tinyc`; create the small captures with
     `benchmarks/performance_guard/make_small_captures.sh`.
   - Prove byte-identical output with both build-id modes, and prove any fallback path.
   - Run the fork CI A/B through a draft `[bench]` PR.
   - Paste tables from `pr_table.py` only.
   - Rerun everything if upstream `main` moved.
5. **Verify** (guide §3): tests, clippy, nightly rustfmt, and the `wasm32-wasip1` unit tests. Use
   cross-arch CI when relocation order or parallelism changes.
6. **Draft the description** from the template in guide §4.
   - The first line is the italic disclosure header, exactly:
     `*AI generated description: authored by [AI](https://github.com/zackees/clud), approved by Zach Vorhies*`.
   - Then Summary, Benchmarks (or Repro and When it fails for bug fixes), and Design.
   - Keep it short. Mention AI nowhere else.
7. **Check the links.** The upstream description and comments may link upstream items only, never
   `zackees/wild`. Any internal PR, issue or commit you create along the way, such as a `[bench]`
   draft, must not link upstream; use plain text like "upstream PR 2634".
8. **Get approval.** Show Zach Vorhies the final title, description and any comment text, and wait
   for an explicit OK. The header says "approved by Zach Vorhies", so nothing goes upstream without
   it.
9. **Publish and follow up** (guide §5).
   - Post a short comment when reviewers need to know about a change.
   - Cross-reference superseded or stacked PRs both ways.
   - Check `gh api rate_limit` before bursts, and don't hand-roll polling loops.

Before finishing, report:
- the PR URL;
- the commands and captures used for the tables;
- where the raw JSON is kept;
- anything from the guide that couldn't be done, and why.
