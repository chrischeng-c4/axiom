---
name: aw-build
description: Build one project in debug or release mode with cargo and report the outcome. The lumen release pipeline stays with /lumen-build-release.
---

# AW Build

## Goal

Produce one debug or release build of one project and report its exact
outcome, warnings included.

## How

1. Accept one `apps/<name>` or `libs/<name>` project and one mode, `debug` or
   `release`. Default to `debug` when the human names no mode.
2. For `lumen` in `release` mode, follow `/lumen-build-release` instead — the
   release pipeline (version stamp, musl target, image, evidence) is that
   skill's contract, and a bare cargo build is not a lumen release.
3. Otherwise run the build from the repository root:

   ```bash
   cargo build -p <crate>
   ```

   For `release` mode add `--release`. Use the crate name from the project's
   own `Cargo.toml`; when the build needs a feature to be non-vacuous (an
   optional dependency gating the real code), pass the features the project's
   `CONTRIBUTING.md` declares.
4. Report the exact exit code, every warning, and every error verbatim. On
   success, name the produced binary path under `target/debug/` or
   `target/release/`.

## Acceptance

- One build command ran for the named crate and mode, and its exact exit
  code is reported.
- Warnings and errors are reproduced verbatim, not summarized away.
- Nothing outside cargo's `target/` output changed.

## Never

- Never treat a warning-free summary as evidence when the build printed
  warnings.
- Never run a lumen release as a bare cargo build.
- Never edit source, manifests, or lockfiles to make the build pass — a
  build failure is the finding.
- Never write commits, tracker updates, tags, or releases.
