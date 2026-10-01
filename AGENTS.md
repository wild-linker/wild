# Repository instructions

## GitHub target

- This checkout's repository is `zackees/wild`, represented by the `origin` remote.
- Treat `wild-linker/wild`, represented by the `upstream` remote, as read-only unless the user explicitly names `wild-linker/wild` or explicitly asks for an upstream mutation in the current request.
- Unqualified requests such as "file an issue", "open a PR", "add a label", "post a comment", or similar GitHub mutations always target `zackees/wild`.
- Before an unqualified GitHub mutation, resolve and verify the target from `origin`; do not infer the target from issue numbers, earlier upstream discussion, PR context, or the repository's upstream relationship.
- Never fall back to `wild-linker/wild` when a requested operation is unavailable on `zackees/wild`. If the fork has Issues disabled, permissions are missing, or another repository setting blocks the operation, stop and report that blocker to the user.
- Read-only upstream research is allowed when useful. It does not authorize writing to upstream.

## Cross-repository links

- **Internal (`zackees/wild`) PRs, issues, comments and commit messages never link to `wild-linker/wild`.** That includes `wild-linker/wild#N`, `wild#N`-style shorthand that resolves upstream, and URLs to upstream PRs, issues or comments. GitHub turns each one into a permanent "mentioned this" entry in the upstream timeline, visible to the maintainers, and editing or deleting the text afterwards doesn't remove it. Refer to upstream items in plain text, e.g. "upstream PR 2634". Never use a bare `#2634` either: in the fork it links to a fork item.
- Internal items may reference other `zackees/wild` items freely (`#N`, `zackees/wild#N`).
- Upstream PRs, comments and commit messages may link other upstream items (`#N`). They never link `zackees/wild` items.
- Squash merges copy the PR title and description into the commit message, so check both before merging.

## Branches

- `upstream` mirrors `wild-linker/wild` `main`. Only fast-forward it to upstream's `main`; never commit to it.
- `main` is the fork: its own commits rebased on `upstream`. To take new upstream work, fast-forward `upstream`, then rebase `main` onto it.
- PR branches come off `main` and target `main`.
- Branches for `wild-linker/wild` PRs are named `up/<topic>`, come off upstream's current `main` (not the fork's), are pushed to `origin`, and must not include fork-only commits.
- The performance guard requires a PR's output to match wild built from the newest `upstream` commit the PR contains, so keep `upstream` exactly equal to what `main` is rebased on.

## Performance PRs

- Every performance PR proposed to `wild-linker/wild` must include the per-benchmark statistics table described in `benchmarks/performance_guard/README.md` ("Upstream performance PRs"), comparing the exact PR head against the exact upstream `main` it is based on.
- Generate the tables with `benchmarks/performance_guard/capture_ab.py` and paste the output of `pr_table.py`; never hand-type or edit the numbers. Fork CI's tables (the `performance` label) compare against the fork's merge base and are not a substitute.

## Upstream pull requests (mandatory)

- **Before authoring, updating or replying on any `wild-linker/wild` pull request, read [`HOW_TO_PR_UPSTREAM.md`](HOW_TO_PR_UPSTREAM.md) in full, and use the `pr-upstream` skill (`.claude/skills/pr-upstream/SKILL.md`).** It holds the required benchmark procedure, the verification steps, the description format (an italic AI-disclosure header, then Summary, Benchmarks, Design), the approval rule, and the lessons from earlier upstream PRs.
- Nothing is posted upstream without Zach Vorhies's explicit approval of the final text.
