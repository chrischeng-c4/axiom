---
name: build-debug
description: "Build one app's image from the working tree (cargo debug profile), load it into a local kind cluster, and run the GKE acceptance harness's deploy → verify → teardown against it. Use when the user asks to debug-build, kind-test, or locally verify keep, defer, relay, or loom."
user-invocable: true
---

# build-debug

Prove the working tree deploys and survives on a cluster before it reaches
GitHub: local image, local kind, and the same `run-app.sh` + `verify/`
contract that `build-release` runs on GKE. The mechanics live in
`scripts/build/debug.sh`; your job is to route the app, run the script, and
relay what it measured.

## Rules

- Route by project, and only by this table:

  | Project | Route |
  |---|---|
  | `keep`, `defer`, `relay`, `loom` | `scripts/build/debug.sh <app>` |
  | `lumen` | not this skill — say so and stop; lumen's cluster runs are `build-release`'s contract |
  | anything else | the script exits 2 with `refused: debug route not wired for <app> (covered: keep defer relay loom)`; relay that line and stop — never fall back to `cargo build`, and never wire a route yourself |

- The script runs the working tree as it is: a dirty tree is allowed and the
  image tag carries `-dirty`. Do not commit, stash, or clean up to make the
  tag clean.
- Do not bypass the script with your own `docker build`, `kind load`,
  `kubectl apply`, or `verify/<app>.sh` — the four report lines are the
  evidence, and a hand-driven run leaves none.
- Do not edit source, manifests, Dockerfiles, or the harness to make a run
  pass. A red verify is the finding; report it with the evidence path.
- A `refused:` exit (2) means stop and report the printed reason.
- Never write commits, tags, tracker updates, or releases from this skill.
- This is not the `apps/<name>/build.sh debug` contract in `CONTRIBUTING.md`
  (a versioned local install checkpoint); the four covered apps have no
  `build.sh`.

## Instructions

1. Run the script from the repository root with `run_in_background`; the
   first docker build of a debug image is a cold cargo build and outlasts the
   ten-minute foreground limit:

```bash
scripts/build/debug.sh <app>
```

   Flags, only when the user asks for them: `--keep` leaves the app namespace
   in place for inspection (the script prints the delete command); `--fresh`
   recreates the kind cluster first; `--image <ref>` deploys a prebuilt image
   instead of building — a `linux/amd64` GHCR image on this arm64 host runs
   under emulation and proves that image, not this tree.

2. Relay its four report lines verbatim plus the exit code: `image:`,
   `cluster:`, `verdict:`, `evidence:`.

3. Read its exit:
   - `0` — verify PASS; the namespace is gone (or kept when `--keep` was
     asked for). The kind cluster `axiom-build-debug` stays up between runs;
     pass on the script's `kind delete cluster` line if the user wants it
     gone.
   - `1` — build, load, deploy, or verify failed. Name the failing step from
     the output and point at `evidence:` (`docker-build.log`, or
     `<app>/pods-describe.txt`, `<app>/pod-0.log`, `<app>/port-forward.log`).
     Do not re-run "to see if it passes".
   - `2` — refused. Report the printed reason; do not force past it.
