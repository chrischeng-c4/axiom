---
name: aw:review
description: Run AW project architecture/profile+rule conformance review. If no project is supplied, infer it from the current project-<name> branch and run aw review --project with the resolved project token.
user-invocable: true
---

# /aw:review

Human-facing entrypoint for Agentic Workflow's architecture/profile+rule
conformance review. Use it when the user asks whether a project's shape
(kind/surface, workload, state ownership, replication/consensus, serving
role) matches one of the reference profile shapes, whether it has
reimplemented a `libs/*`-owned shared-service-kit capability instead of
composing it, or whether its observability/Raft telemetry adoption is
complete. Do not use it for readiness, gates, or production-blocker
questions -- that is `aw:health`.

## Project Resolution

1. If the prompt includes a project token, use it directly:
   ```bash
   aw review --project <project>
   ```
2. If no project is supplied, infer from the current branch:
   ```bash
   git branch --show-current
   ```
   - `project-<token>` -> run `aw review --project <token>`.
   - Example: branch `project-aw` -> `aw review --project aw`.
3. If the inferred token is rejected as an unknown project, read
   `aw.toml` and resolve the token against `[[projects]].name` and
   `[[projects]].aliases`, then rerun with the matching project name.
4. If the branch is not `project-<token>` and the user did not provide a
   project, stop and ask for the project name.

## Command

`aw review` is read-only: it never mutates the target project and never
loops. Do not add `--json`; use `--pretty` only when the user asks for a
human-readable or debug-formatted report.

```bash
aw review --project <project>
aw review --project <project> --pretty
```

Use the stdout envelope (schema `aw.cli.v1`) as authoritative:

- `outcome: "resolved"`: report the resolved `profile` (kind_surface,
  primary_workload, state_ownership, replication, serving_role) plus every
  `findings[]` entry's `severity`, `affected_paths`, and `remediation`.
  Findings combine shared-service-kit adoption, profile negative-assertion,
  structured-observability, and Raft telemetry conformance rules -- report
  them together, they are not separate commands.
- `outcome: "ambiguous"`: report `ambiguous_reason` and the collected
  `evidence[]` array; never guess a profile shape on the agent's own
  judgment when the CLI could not resolve one from evidence.
- `next.kind == "done"`: always the terminal state. `aw review` is a
  read-only report, never a fix-it loop -- do not chain another `aw review`
  invocation expecting different output without a source change in between.

## Rules

- `aw health` owns readiness, gates, and production-blocker status. `aw
  review` owns architecture/profile-shape and shared-service-kit,
  negative-assertion, observability, and Raft-conformance rule findings.
  Never route a readiness/gate question through `aw review`, and never
  route an architecture/profile-shape or rule-conformance question through
  `aw health` -- the two commands do not overlap and do not substitute for
  each other.
- `aw review` is a measurement surface. Do not edit files just because a
  finding fired unless the user asked to fix the project.
- A finding's `remediation` always names either an owning `libs/*` crate or
  a concrete structural fix -- never report a bare "needs review"
  placeholder to the user.
- Prefer the installed `aw` only after it has been built or verified
  recently; when results look stale, build or use `target/debug/aw` from
  the checkout.
