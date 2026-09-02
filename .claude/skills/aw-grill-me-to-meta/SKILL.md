---
name: aw-grill-me-to-meta
description: Start in Plan mode, interview the human until one product promise is observable, then update one project's product documents. Use before release Milestone planning.
---

# AW Grill Me to Meta

## Goal

Write one product promise into the owning project's product documents before
release planning starts.

## How

1. Enter Plan mode before reading files, asking product questions, or editing.
   Use the runtime's native Plan mode control. Stop if the runtime cannot
   confirm Plan mode.
2. Resolve one `apps/<name>` or `libs/<name>` project. Read its `README.md` and
   `CONTRIBUTING.md` before editing.
3. Check the dirty paths under the project's `README.md`, `STATUS.md`,
   `ROADMAP.md`, and `docs/**`. Stop if unrelated edits overlap the target
   section.
4. Interview the human for the current problem, user, observable promise,
   limits, non-goals, and the stable STATUS or ROADMAP owner ID.
5. Write only those four document paths. Keep one promise per `##` section.
6. Leave the promise unbound. Its heading must not gain
   `(Milestone #<number>)`. Its tracking field stays `Tracking: Not assigned.`
7. Run from the repository root:

   ```bash
   uv run --project apps/aw aw metadoc check <project>
   uv run --project apps/aw aw meta check --path <project>
   uv run --project apps/aw aw metadoc commit <project> --why <path>
   ```

8. Report the document paths, the promise section, and each gate result.

## Acceptance

- `aw metadoc check` prints a clean result for the four-path allowlist.
- `aw meta check` reports no finding in the edited document set.
- The commit is written by `aw metadoc commit` and carries its document
  trailers.
- No tracker binding appears in the edited section.

## Never

- Never write product source, tests, tracker records, tags, or releases.
- Never invent a STATUS ID, ROADMAP ID, gate, version, or product answer.
- Never bind the section to an issue. Issue-based epics are retired.
- Never bind the section to a Milestone in this skill. That belongs to
  `/aw-grill-meta-to-milestone`.
