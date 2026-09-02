---
name: aw-test-for
description: Read-only closing regression verification for a scope reference — one delivery issue, one release Milestone, or every open release Milestone of a project. Checks lifecycle evidence trailers, then runs the full unfiltered project gates. Writes nothing.
---

# AW Test for Scope

## Goal

Prove that every delivery issue in the given scope carries the evidence its
flow requires and that the owning project's full declared gates are green,
without changing anything.

## How

1. Resolve the scope from the one argument:
   - `#<iid>` is one delivery issue;
   - `milestone:<number>` or an exact `<project>@<version>` title is one
     release Milestone;
   - a `<project>` name means every open release Milestone of that project,
     listed and processed in version order:

     ```bash
     uv run --project apps/aw aw milestone versions --project <project> --state open --json
     ```

   A bare number never means a Milestone.
2. For a Milestone, read its structure and issue set:

   ```bash
   uv run --project apps/aw aw milestone reconcile milestone:<number>
   uv run --project apps/aw aw milestone children milestone:<number> --json
   ```

   Stop on a structural error. For a direct issue, read it with
   `aw change show <iid> --json`.
3. For each delivery issue in scope, verify its lifecycle evidence against
   the type registry, not against anyone's summary:
   - `type:feat`, `type:fix`, `type:perf` (`flow: behavior`) require a commit
     carrying `E2E-Red:` and a commit carrying `Impl-Red:` and
     `Impl-Contract:`;
   - `type:refactor`, `type:test`, `type:docs`, `type:chore`
     (`flow: maintenance`) require a commit carrying `Maint-Contract:` and
     `Maint-Change-Digest:`.

   Read the issue's recorded lifecycle rows from `aw change show <iid>
   --json`, then confirm each named commit exists and carries its trailer:

   ```bash
   git -c core.fsmonitor=false log --format='%H%n%(trailers)' <commit> -1
   ```

   A closed issue with missing or mismatched evidence is a finding, not a
   pass.
4. Run the owning project's full declared gates from its `README.md` and
   `CONTRIBUTING.md` — for a Rust project at least `cargo test -p <crate>` —
   with no test filter. A filter that matches nothing exits green, so a
   filtered run proves nothing about the rest of the suite.
5. Print one reconciliation table: each issue, its type, its required
   evidence, the commit that carries it or the gap, and each gate command
   with its exact exit code. State plainly whether the scope verifies.

## Acceptance

- Every delivery issue in scope appears in the table with its evidence
  commit or a named gap.
- Every gate run is the project's own declared command, complete and
  unfiltered, with its exit code shown.
- The run leaves the tree byte-identical: no file, tracker, Git ref, or
  release changes.

## Never

- Never use a bare number as a Milestone reference.
- Never write files, commits, tracker updates, tags, or releases.
- Never close, reopen, or relabel an issue — report gaps instead.
- Never pass a test filter to a gate command.
- Never substitute a phase runner's summary for reading the commits.
- Never invent evidence for a gap; a missing trailer is the finding.
