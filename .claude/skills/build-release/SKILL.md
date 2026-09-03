---
name: build:release
description: "Release one app through the GitHub Actions GKE acceptance pipeline (keep, defer, relay, loom): image → terraform + kustomize deploy on GKE → verify → park the node pool whatever the result. Lumen routes to /lumen:build:release; anything else is refused. Use when the user asks to release, run release acceptance, or run the GKE pipeline for an app."
user-invocable: true
---

# /build:release

A release is not a compile flag. It is the CI/CD pipeline that builds the
image, deploys it with terraform + kustomize onto GKE, runs the acceptance
verify, and parks the node pool whatever the result. The mechanics live in
`scripts/build/release.sh`; your job is to route the app, run the script, and
relay what it measured. Its local counterpart is `/build:debug`.

## Rules

- Route by project, and only by this table:

  | Project | Route |
  |---|---|
  | `lumen` | `/lumen:build:release` — version stamp, musl target, image, and evidence are that skill's contract |
  | `keep`, `defer`, `relay`, `loom` | `scripts/build/release.sh <app>` — dispatches `.github/workflows/gke-acceptance.yml` at the pushed HEAD, watches it, downloads the evidence bundle, and reads back the park step |
  | anything else | the script exits 2 with `refused: release route not wired for <app> (covered: keep defer relay loom)`; relay that line and stop — never fall back to `cargo build --release`, and never wire a route yourself |

- Never treat `cargo build -p <crate> --release` as a release — that is a
  debug build with optimizations and proves nothing about deploy, verify, or
  teardown. Never run a lumen release as a bare cargo build.
- Never dispatch from a dirty or unpushed tree, and never work around the
  script's refusal by calling `gh workflow run` yourself.
- Never pass `--rerun` without naming what changed since the last run — a
  paid re-run "to see if it passes this time" is not evidence.
- Never call `acceptance/gke-harness/scripts/*.sh`, `terraform`, `kubectl`,
  or `gcloud` from this skill — the workflow owns deploy, verify, and park.
  When the park step did not succeed, relay the script's `park.sh`
  instruction to the human instead of running it.
- Never edit source, manifests, lockfiles, or workflows to make a run pass —
  the failure is the finding.
- Never write commits, tracker updates, tags, or releases.
- This is not the `apps/<name>/build.sh release` contract in
  `CONTRIBUTING.md` (tag + GitHub Release); the four covered apps have no
  `build.sh`.

## Instructions

1. Run the script from the repository root with `run_in_background` and poll
   its output; the watch takes 15–40 minutes and outlasts the ten-minute
   foreground limit:

```bash
scripts/build/release.sh <app>
```

   It refuses (exit 2) a dirty tree, an unpushed HEAD, a workflow file not
   yet on the default branch, and a sha+app that already has a successful
   run — that last refusal lifts only with `--rerun`, and only when the human
   names what changed.

2. Relay its five report lines verbatim plus the exit code: `run:` URL,
   `conclusion:`, `acceptance job:`, `park step:`, `evidence:` directory.

3. Read its exit:
   - `0` — the run concluded `success` AND the park step concluded
     `success`.
   - `1` — the run was red, or the park step was not `success`. A park step
     other than `success` means the node pool may still be running: say that
     first, and pass on the script's `park.sh` instruction. Point at
     `evidence:` for the failing step.
   - `2` — refused. Report the printed reason; do not force past it.
