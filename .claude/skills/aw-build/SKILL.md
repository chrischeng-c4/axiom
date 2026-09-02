---
name: aw-build
description: Build one project. Debug is a local cargo build; release goes through the GitHub Actions GKE acceptance pipeline (keep, defer, relay, loom) or /lumen-build-release (lumen) and is refused for anything else.
---

# AW Build

## Goal

Produce one build of one project in one mode and report its exact outcome:
a debug build's warnings and errors verbatim, or a release run's URL,
conclusion, park result, and evidence directory.

## How

1. Accept one `apps/<name>` or `libs/<name>` project and one mode, `debug` or
   `release`. Default to `debug` when the human names no mode.
2. `debug` — run from the repository root:

   ```bash
   cargo build -p <crate>
   ```

   Use the crate name from the project's own `Cargo.toml`; when the build
   needs a feature to be non-vacuous (an optional dependency gating the real
   code), pass the features the project's `CONTRIBUTING.md` declares. Report
   the exact exit code, every warning, and every error verbatim; on success
   name the binary under `target/debug/`.
3. `release` — a release is not a compile flag. It is the CI/CD pipeline
   that builds the image, deploys it with terraform + kustomize onto GKE,
   runs the e2e verify, and parks the cluster whatever the result. Route by
   project:

   | Project | Route |
   |---|---|
   | `lumen` | `/lumen-build-release` — version stamp, musl target, image, and evidence are that skill's contract |
   | `keep`, `defer`, `relay`, `loom` | `scripts/gh/gke-acceptance.sh <app>` — dispatches `.github/workflows/gke-acceptance.yml` at the pushed HEAD, watches it, downloads the evidence bundle, and reads back the park step |
   | anything else | refused — the script exits 2 with `refused: release route not wired for <app>`; relay that line and stop, never fall back to a local build |

   Run the script with `run_in_background` and poll its output: the watch
   outlasts the ten-minute foreground limit. It refuses (exit 2) a dirty
   tree, an unpushed HEAD, a workflow file not yet on the default branch,
   and a sha+app that already has a successful run — that last refusal lifts
   only with `--rerun`, and only when the human names what changed.
4. Report a release as the script's five lines, verbatim: `run:` URL,
   `conclusion:`, `acceptance job:`, `park step:`, `evidence:` directory —
   plus its exit code. Exit 0 means the run concluded `success` AND the park
   step concluded `success`. A park step other than `success` means the node
   pool may still be running: say that first, and pass on the script's
   `park.sh` instruction to the human instead of running it.

## Acceptance

| Gate | Observation |
|---|---|
| debug | `cargo build -p <crate>` ran once for the named crate; its exit code, warnings, and errors are reproduced verbatim |
| release, covered app | `scripts/gh/gke-acceptance.sh <app>` exited 0 and its five report lines are quoted, with `park step: success` |
| release, lumen | the run was handed to `/lumen-build-release` — not to cargo, not to the script |
| release, refused | the script's `refused: …` line is quoted verbatim with exit code 2, and nothing else ran |
| negative control | `scripts/gh/gke-acceptance.sh tape` prints `refused: release route not wired for tape (covered: keep defer relay loom)` and exits 2 — free, offline, and it proves the route table is enforced rather than described |
| footprint | nothing outside cargo's `target/` output and the evidence download directory changed |

## Never

- Never treat `cargo build -p <crate> --release` as a release — a release is
  the GKE acceptance run or nothing. The flag alone is a debug build with
  optimizations and proves nothing about deploy, verify, or teardown.
- Never run a lumen release as a bare cargo build.
- Never dispatch from a dirty or unpushed tree, and never work around the
  script's refusal by calling `gh workflow run` yourself.
- Never pass `--rerun` without naming what changed since the last run — a
  paid re-run "to see if it passes this time" is not evidence.
- Never call `acceptance/gke-harness/scripts/*.sh`, `terraform`, `kubectl`,
  or `gcloud` from this skill — the workflow owns deploy, verify, and park.
- Never treat a warning-free summary as evidence when the build printed
  warnings.
- Never edit source, manifests, lockfiles, or workflows to make a build or
  a run pass — the failure is the finding.
- Never write commits, tracker updates, tags, or releases.
