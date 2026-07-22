---
name: aw-change-review
description: Independently review a concrete code or documentation change set produced by one or more agents, including AGY, Claude Code, and Codex. Use when an agent handoff, dirty worktree, commit, branch diff, or integrated multi-agent batch needs correctness-first review, author-independent reviewer routing, actionable findings, fix verification, and focused re-review before integration.
---

# AW Change Review

Review every cohesive author-owned change set before integration. Review the
patch or commit boundary, not every keystroke. When independently authored
patches interact, review each patch and the final integrated diff.

This is code-change review. It does not replace tests, project architecture/
profile conformance review, or digest-bound `aw ec review` semantic approval.

## Establish the review unit

Preserve the worktree. Do not edit, reset, stash, switch branches, or discard
changes while acting as reviewer.

Select one concrete target in this order:

1. Use the range, commit, paths, or patch supplied by the user/controller.
2. Otherwise review the staged and unstaged worktree diff plus relevant
   untracked files.
3. If the worktree is clean, review the current branch from its merge-base
   with the named base branch.
4. Do not guess a previous commit or broaden the target silently.

Record the target, base, head, included paths, and a stable diff fingerprint.
If the target changes during review, invalidate the verdict and review the new
target.

## Establish authorship and independence

Record the agent identity that authored each patch, path, or overlapping hunk.
Use controller/handoff provenance when available. Git commit author metadata is
not proof of the coding-agent identity. If authorship is unknown, report the
missing provenance and do not claim independent approval.

A reviewer is independent only when all of these hold:

- It is a different agent instance from every author whose hunks it approves.
- It receives the raw target, contract, and verification evidence without the
  author's private reasoning or a leaked expected finding list.
- It operates read-only and does not fix the code it is approving.
- Its identity and engine are recorded in the result.

Prefer a different engine as an additional independence boundary:

| Author | Preferred reviewer |
|---|---|
| Codex | Claude Code |
| Claude Code | Codex |
| AGY | Codex or Claude Code |

Use the installed `claude-review` or `codex-review` helper when it satisfies
the matrix. A fresh native reviewer agent is acceptable when the preferred
engine is unavailable, but it must be a new instance with read-only scope and
minimal task-local context.

For mixed-author changes, either choose one reviewer outside the complete
author set or partition the diff so no reviewer approves its own hunks. If all
available reviewers contributed to overlapping hunks and no independent
instance can be established, return `blocked`, not a self-review verdict.

## Build the review packet

Give the reviewer only the material needed to judge the change:

- exact target/range and author map;
- WI, acceptance criteria, TD/spec, public contract, or user request;
- repository contribution and ownership rules;
- relevant test commands and existing output;
- changed files and necessary surrounding source.

Ask for read-only review. Do not seed suspected bugs, the intended answer, or
the author's explanation unless the review is explicitly evaluating that
explanation.

## Review for material defects

Prioritize:

- incorrect behavior, edge cases, and error handling;
- data loss, security, authorization, and unsafe mutation;
- concurrency, ordering, retry, idempotency, and partial-failure bugs;
- public API, schema, compatibility, and capability/acceptance drift;
- false-green, missing, or non-representative verification;
- ownership, generated-source, mirror, and lifecycle violations;
- cross-patch integration failures that isolated reviews would miss.

Ignore preference-only style comments. Raise maintainability only when it
creates a concrete correctness, ownership, testability, or future-change risk.
Verify every finding against source or a reproducible check before accepting
it.

## Normalize and adjudicate findings

Return findings in descending severity:

- `P0`: immediate catastrophic, security, or irreversible-loss risk.
- `P1`: likely production failure, major regression, or contract break.
- `P2`: bounded correctness, reliability, or verification defect.
- `P3`: non-blocking hardening with a concrete future risk.

Each finding must include severity, file and tight line location, evidence,
impact, and a concrete fix direction. Reject findings that are speculative,
outside the target, already pre-existing without regression, or unsupported by
the contract.

Use this result shape:

```text
Status: pass | needs_fixes | blocked
Target: <range/paths/fingerprint>
Authors: <identity -> scope>
Reviewer: <identity, engine, independence basis>
Findings:
- [P1] <title> — <file:line>; evidence; impact; fix direction
Verification: <commands/results>
Residual risks: <explicitly unverified areas>
```

If there are no findings, say so explicitly and still report target,
independence, verification, and residual risk.

## Fix and re-review

Have the author or integrator fix accepted findings; keep the reviewer
read-only. Then:

1. Recompute the target fingerprint.
2. Recheck each accepted finding against the fix.
3. Review newly changed hunks for regressions.
4. Run the relevant verification gates.
5. Repeat until no blocking finding remains.

Completion requires independent coverage for every author-owned scope, no
unresolved `P0`-`P2` finding unless the user explicitly accepts a documented
residual risk, and passing required gates. `P3` findings may remain only when
recorded as non-blocking follow-up.

Never turn absence of an eligible reviewer, missing provenance, a moving diff,
or unavailable verification into a pass. Report `blocked` with the exact next
action.
