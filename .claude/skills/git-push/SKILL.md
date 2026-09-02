---
name: git:push
description: "Push the current branch to origin, setting the upstream on first push and using --force-with-lease only when a rebase rewrote already-pushed history. Use when the user asks to git push, push this branch, or publish the branch to the remote."
user-invocable: true
---

# /git:push

Push the current branch to `origin`, then stop. The mechanics live in
`scripts/git/push.sh`, which picks the safe variant itself: `-u origin HEAD`
on a first push, a plain push when only ahead, `--force-with-lease` when
history was rewritten, nothing when already in sync — and never bare
`--force`.

## Rules

- Push only the current branch; the script never touches another ref.
- A force push to a persistent ref (`main`, `examples`, `project-mamba`,
  `project-lumen`, `app/*`, `lib/*`) is refused by the script. Pass
  `--force-persistent-ok` only after the user has explicitly confirmed that
  exact overwrite in this conversation — never on your own judgment.
- A rejected push means the remote moved. The script already fetched and
  printed the divergence; report it and stop. Do not override it.

## Instructions

1. Run the script:

```bash
scripts/git/push.sh
```

2. Read its exit:
   - `0` — pushed, or nothing to push; it printed the final `HEAD...@{u}`
     counts. Clean is `0 0`. Report the pushed ref.
   - `2` — refused: either only behind the upstream (nothing local to
     publish), or a force push to a persistent ref that needs the user's
     explicit confirmation first.
   - `4` — rejected by the remote; the divergence was printed. Report it and
     stop.
