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
- The performance guard requires a PR's output to match wild built from the newest `upstream` commit the PR contains, so keep `upstream` exactly equal to what `main` is rebased on.
