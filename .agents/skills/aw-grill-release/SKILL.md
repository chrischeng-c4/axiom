---
name: aw-grill-release
description: Prepare or reuse release plans in the current runtime mode, then apply approved product promises, Milestones, typed issues, and Development Order through a resumable receipt.
---

# AW Grill Release

## Goal

Turn human intent into one approved release plan, then apply exactly one
project from that plan without choosing new product decisions during writes.

## How

1. Select `plan` or `apply` from the request and the available plan.
   A validated plan with an approved digest goes directly to Apply.
   The public forms are `aw-grill-release plan <project|milestone|intent>` and
   `aw-grill-release apply <approved-plan>`.

### Plan

1. Prepare the plan read-only in any runtime mode. No mode switch is required.
   Reuse an existing approved plan and its settled decisions. Do not restart
   the interview.
2. Resolve the project, Milestone reference, or intent to an ordered list of
   `apps/<name>` or `libs/<name>` projects. Read each project's
   `README.md` and `CONTRIBUTING.md`, its META-docs, its manifest, and its live
   tracker state. Run `uv run --project apps/aw aw wis gap <project>` once.
   Drafts prepared before this skill are inputs, never answers. Read each
   one that exists: a `<project>-pm` draft left uncommitted under the
   project's `README.md`, `STATUS.md`, `ROADMAP.md`, and `docs/**`
   (`git -c core.fsmonitor=false status --short -- <project>`); a
   `project-manager` description draft at the path the session named, checked
   with
   `uv run --project apps/aw aw milestone validate --description-file <path> --title <project>@<version> --draft`;
   and `tech-design` bodies under
   `uv run --project apps/aw aw change bodydir --type <type>`, each checked
   with
   `uv run --project apps/aw aw change validate --type <type> --body-file <path>`.
3. Ask only for product, version, issue-boundary, type, or order decisions that
   the human's input and repository evidence do not already settle.
   A drafted answer the human has not confirmed is not an answer: a draft
   section enters the plan's `after` text, Milestone description, or issue
   body only after the human confirms it, section by section, and an
   unconfirmed section stays out. A `project-manager` draft's skeleton
   `Pending:` line becomes the plan's `{{development_order}}`; a
   `tech-design` body enters with the type, owner label, and `priority` the
   human confirms.
   For a new release Milestone, derive the normal version from every prior
   open and closed release Milestone with `aw milestone next-version`. Use its
   default minor bump without asking. Ask only when there is no prior release
   Milestone or the human requests a major, patch, or exact-version exception.
4. Build one closed `release-plan-v1`. The validator seals its
   `plan_sha256` over the canonical plan with that field omitted. The plan may
   contain several ordered
   projects. The list is also the enforced Apply order. A project can apply
   only after every earlier project has a `COMPLETE` receipt. Put the project
   the human intends to apply first at the start of the list. Each project is
   either `product` mode for META-docs only or `release` mode for META-docs,
   one release Milestone, typed delivery issues, and their complete
   Development Order.
   - Record exact `before_sha256` and approved `after` text for every document.
     Use `null` only for an absent side of an approved create or delete.
   - Record the canonical tracker baseline summary and its SHA-256. The summary
     contains the project's release Milestones and owner-labelled issues. Sort
     both lists by number. Sort each issue's unique labels. A Milestone row has
     `number`, `title`, `state`, and `description_sha256`. An issue row has
     `number`, `title`, `state`, `labels`, `milestone`, and `body_sha256`.
   - In release mode, bind each owned promise as
     `## <title> (Milestone #{{milestone_number}})`. Put the exact link
     `[Milestone #{{milestone_number}}](https://github.com/<owner>/<repo>/milestone/{{milestone_number}})`
     in that same section's `Outcome:` or `Status rows:` Tracking field.
   - Use one `{{development_order}}` in the approved Milestone description.
     The facade replaces it with the issue numbers resolved from stable keys.
   - Use `app:<name>` or `lib:<name>` as every issue's exact owner label.
   - Record each issue's approved `priority` from `p0` through `p5`.
   - Include no command, hook, script, or executable field.
5. Validate the plan without writing a file by sending its JSON to:

   ```bash
   uv run --project apps/aw aw release-plan validate --plan -
   ```

6. The validator prints one sealed canonical `release-plan-v1` JSON document.
   Return that document and its `plan_sha256` separately in the conversation.
   Write no file and make no Git, tracker, tag, release, or cloud change.

### Apply

1. Confirm Default mode and an explicit human approval of one exact validated
   plan digest. This approved digest is the only write authority. Stop if
   either is absent.
2. Set aside any uncommitted draft still in the working tree: its confirmed
   bytes already live in the plan's `after` text, and the facade refuses a
   dirty tree (`working tree is dirty before apply`) and rewrites every
   planned document from the plan itself.
   Materialize only the approved canonical JSON under `.aw/release-plans/`.
   Re-run `release-plan validate` and require the same `plan_sha256`.
3. Apply one project only:

   ```bash
   uv run --project apps/aw aw release-plan apply \
     --plan <path> \
     --project <apps/name-or-libs/name> \
     --approved-digest <sha256>
   ```

4. If the command reports an incomplete receipt, continue only with:

   ```bash
   uv run --project apps/aw aw release-plan resume --receipt <path>
   ```

5. Report the receipt, META commit, Milestone, issue order, reconciliation, and
   final gap evidence. A second project requires a separate `apply` call.

## Acceptance

- `plan` is read-only and asks no question already answered by the input.
- An existing plan needs no runtime mode switch or repeated approval of
  unchanged decisions. Apply still requires approval of the exact digest.
- The validator accepts the closed plan and prints its canonical SHA-256.
- `apply` uses that exact approved digest and changes one project only.
- A complete receipt binds every write and final readback; an incomplete
  receipt resumes without duplicate Milestones, issues, or commits.
- In `release` mode, the final Milestone reconciles, its order is complete,
  and the receipt records every `aw wis gap` row. G1 through G5 must be
  measured and zero. G6 and G7 may name approved delivery work that the new
  issues will close; they remain visible in the receipt and do not deadlock
  the e2e and implementation phases.
- In `product` mode, only the approved META-doc commit is written. Tracker
  state stays byte-for-byte equal to its approved baseline summary.

## Never

- Never run `apply` in Plan mode or run `plan` as permission to write.
- Never edit an approved plan during Apply. On drift or a missing decision,
  return to read-only preparation in the current mode and ask only about
  the unresolved change.
- Never use direct GitHub writers, issue epics, `epic:<iid>`, bare Milestone
  numbers, legacy issue types, or an inferred issue order.
- Never start another apply after a partial write. Resume the same receipt.
- Never delete or roll back remote state to hide an incomplete receipt.
- Never change product source, tests, tags, releases, or cloud resources in
  this skill.
- Never dispatch `<project>-pm`, `cto`, `project-manager`, or `tech-design`
  inside this skill; their drafts are prepared before `plan` and enter it
  only as answers the human confirmed.
- Never treat a draft as confirmed because it validates; `aw milestone
  validate` and `aw change validate` check shape, and the human's
  confirmation supplies the decision.
