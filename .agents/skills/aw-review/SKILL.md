---
name: aw-review
description: Read-only comprehensive audit of one project — uncommitted diff, META-doc findings, the wis gap table, and the project's declared gates — reported as findings. Writes nothing.
---

# AW Review

## Goal

Produce one findings report for one project covering its working tree, its
product documents, its tracker alignment, and its declared gates, without
changing anything.

## How

1. Accept one `apps/<name>` or `libs/<name>` project. Read its `README.md`
   and `CONTRIBUTING.md` first — the README owns the promises and gates this
   review measures against.
2. Read the uncommitted state of the project's paths:

   ```bash
   git -c core.fsmonitor=false status --short -- <project-root>
   git -c core.fsmonitor=false diff HEAD -- <project-root>
   ```

   Review the diff against the project's own conventions and against the
   artifact write order: a dirty path outside any phase's write root, an e2e
   case edited alongside the implementation it judges, or a test weakened to
   match its code are findings.
3. Run the META-doc check scoped to the project:

   ```bash
   uv run --project apps/aw aw meta check --path <project>
   ```

   Every finding it prints for the project's documents is a review finding.
4. Run the read-only gap table:

   ```bash
   uv run --project apps/aw aw wis gap <project>
   ```

   Report all seven rows with their populations. A `?` UNMEASURED row is a
   finding about the instrument, not a zero.
5. Run each gate the README declares, complete and unfiltered, and record
   each exact exit code. A gate command that is a strict subset of the
   declared suite is itself a finding.
6. Print one findings report: each finding with its path or command, the
   observed value, and the expected value — most severe first. State plainly
   when an area is clean.

## Acceptance

- The report covers all four areas: uncommitted diff, META-doc findings, the
  seven gap rows, and every declared gate with its exit code.
- Every finding names its path or verbatim command and what was observed.
- The run leaves the tree byte-identical: no file, tracker, Git ref, or
  release changes.

## Never

- Never write files, commits, tracker updates, tags, or releases.
- Never fix a finding in this skill — report it and stop.
- Never pass a test filter to a declared gate.
- Never mark an area clean without running its command in this session.
- Never review a project against another project's conventions or gates.
