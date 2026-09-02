---
name: aw-grill-meta-to-milestone
description: Start in Plan mode, interview the human about one release, and bind one product promise to one release Milestone. The Milestone title owns the version; the issue set behind it belongs to /aw-grill-milestone-to-issue.
---

# AW Grill Meta to Milestone

## Goal

Make one product promise and one release Milestone describe the same release.

One GitHub Milestone is one versioned epic. Its title is
`<project>@<major>.<minor>.<patch>`. GitHub's native milestone field is the only
parent relation for a delivery issue.

## How

1. Enter Plan mode before reading files, inspecting the tracker, asking release
   questions, or writing. Use the runtime's native Plan mode control. Stop if
   the runtime cannot confirm Plan mode.
2. Accept one `<project>` or one `milestone:<number>` reference. Run the
   read-only gap table:

   ```bash
   uv run --project apps/aw aw wis gap <project>
   ```

   This skill answers G1 and G3-G5: a future promise no release Milestone
   owns, a promise bound to a Milestone that cannot carry it, and ROADMAP or
   STATUS rows no promise claims. G2, G6, and G7 route to
   `/aw-grill-milestone-to-issue`.
3. Read the default target version from all prior open and closed release
   Milestones:

   ```bash
   uv run --project apps/aw aw milestone next-version <project> --json
   ```

   With no override, this applies Axiom's default `minor` bump:
   `M.m.p` becomes `M.(m+1).0`. This is a repository policy, not a human
   question. It has no base-64 ceiling.
4. Ask the human for a version only when no existing release Milestone supplies
   a base — the initial version is the human's choice — or when the release
   needs an explicit exception. Use `--bump patch` for a human-selected
   fix-only release and `--bump major` for a human-selected incompatible
   release. The human may instead select one exact core SemVer title. Then
   resolve with the human whether the promise deserves its own Milestone.
5. Create a Milestone description with:

   ```bash
   uv run --project apps/aw aw milestone skeleton
   ```

   Fill `## Goal` and `## Acceptance` from the interview. Keep the exact
   `## Development Order` draft line printed by the skeleton — the numbered
   issue list that replaces it is `/aw-grill-milestone-to-issue`'s write, not
   this one's.
6. Create the Milestone in draft form with the selected title:

   ```bash
   uv run --project apps/aw aw milestone create --title <project>@<version> --description-file <path> --draft
   ```

7. Bind the product section as `## <title> (Milestone #<number>)`. Set its
   tracking link to `[Milestone #<number>](<milestone-url>)`.
8. Re-run `aw wis gap <project>`. Report the named G1 and G3-G5 rows with
   their populations. The Milestone stays a draft here; finalizing the
   description and running `aw milestone reconcile` happen in
   `/aw-grill-milestone-to-issue` after issues exist.

## Acceptance

- The Milestone title is the release identity and uses core SemVer
  `major.minor.patch`: three non-negative integer fields without leading
  zeroes.
- Without a human-selected exception, the title equals the `next-version`
  command's default minor result and resets patch to zero.
- The Milestone exists in draft form with the skeleton's exact
  `## Development Order` draft line.
- The promise heading carries `(Milestone #<number>)` and its tracking link
  points at that Milestone.
- Each named G1 and G3-G5 row is `0` over a population greater than zero.
- No new `type:epic` issue or `epic:<iid>` label exists.

## Never

- Never use a GitHub issue as an epic.
- Never add, repair, or infer an `epic:<iid>` ownership label.
- Never use a bare number for a Milestone. Use `milestone:<number>` or its
  exact title.
- Never apply the build/release base-64 carry rule to a Milestone version.
- Never choose an initial version, major bump, patch bump, or exact version
  override for the human.
- Never create, assign, or order delivery issues in this skill. That belongs
  to `/aw-grill-milestone-to-issue`.
- Never finalize the draft description or replace the draft order line here.
- Never change product source, tests, Git refs, tags, or releases.
