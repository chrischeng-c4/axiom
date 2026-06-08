---
name: cue:dev
description: Develop, run, test, or inspect Cue Artifact Studio and the Cue backend in project-cue. Use for Cue frontend/backend dev server work, Artifact Studio UI changes, WorkItem/PRD/API changes, Cue product-slice implementation, and local validation.
user-invocable: true
aliases: [cue-dev, cue:run, cue:artifact-studio]
---

# cue:dev

Use this skill for Cue product development in `projects/cue`.

## Source of truth

Read specs and product docs before source code:

1. `projects/cue/README.md`
2. `.aw/tech-design/projects/cue/README.md`
3. Relevant design files under `.aw/tech-design/projects/cue/`
4. Source code under `projects/cue/` only after the intended behavior is clear

Keep the current product shape intact:

- Cue is a web-based Prompt-to-Governed-Artifact control plane, not the removed terminal/TUI SDD runner.
- Artifact Studio is the project-owner front office.
- Admin is the platform/operator back office and is not the first frontend slice unless explicitly requested.
- WorkItems gate every prompt-to-X flow; PRD and TD are first-class artifacts.
- Hidden GitLab repos, CI, branches, release refs, deploy internals, and runtime tenancy are implementation details hidden from business users.
- `aw` builds Cue itself; generated business apps do not run AW CRRR.

## Active paths

```text
projects/cue/artifact-studio/  Jet + React + TypeScript owner workspace
projects/cue/admin/            Jet + React + TypeScript platform workspace
projects/cue/backend/          Mamba backend target plus temporary mambalibs bridge contracts
projects/cue/shared/           shared domain/API client/status mapping only
projects/cue/schemas/          artifact and runtime contracts
projects/cue/fe/               legacy transitional scaffold
projects/cue/be/               legacy transitional scaffold
projects/cue/docs/legacy/      historical terminal/TUI notes only
```

Prefer `artifact-studio` and `backend` for new work. Do not revive the old TUI
or make users reason about worktrees, GitLab, CI, branches, or deploy plumbing.

## Local run

Use the root Cue dev script when the user asks to start Cue locally. It is
Mamba/Jet-first and starts the current three-process product shape: backend API,
Artifact Studio, and Admin.

```bash
cd projects/cue
npm run dev
```

Default ports:

```text
backend          http://127.0.0.1:43219
Artifact Studio  http://127.0.0.1:3212
Admin            http://127.0.0.1:3216
```

The default backend mode is `CUE_BACKEND_MODE=mamba`. If Mamba backend serving
is blocked by current substrate readiness, use the explicit bridge fallback
only to keep a thin Cue product slice moving:

```bash
CUE_BACKEND_MODE=bridge npm run dev
```

If a port is in use, either stop the existing process deliberately or set the
matching environment variable instead of guessing:

```bash
CUE_BACKEND_PORT=43220 npm run dev
CUE_ARTIFACT_STUDIO_PORT=3213 npm run dev
CUE_ADMIN_PORT=3217 npm run dev
```

Artifact Studio still has a fixture API for deterministic browser e2e tests:

```bash
cd projects/cue/artifact-studio
npm run dev:fixture-api
```

Do not default to Docker Compose for these three dev servers. Prefer the root
Node orchestrator for the hot-reload loop; reserve Compose for external
services such as Postgres, Redis, or NATS when the slice actually depends on
them.

## Validation

Run the narrow checks for the area touched:

```bash
cd projects/cue/artifact-studio
npm run typecheck
npm run build
npm run test:e2e
```

```bash
uv run --with pytest python -m pytest projects/cue/backend/tests/test_workstream_api.py -q
```

For UI changes, open `http://127.0.0.1:3212` with the browser tool and verify
the first viewport and the affected workflow. Save Playwright screenshots under
`.playwright-mcp/`, not the repo root.

## Implementation rules

- Keep frontend work dense, operational, and artifact/workflow focused; avoid marketing-page composition.
- Preserve the `project list / chat / on-demand right pane` Artifact Studio shape unless the user explicitly changes it.
- Keep chat available for app owners, but make WorkItem/artifact state the durable center of the UI.
- Use same-origin `/api` calls from Jet frontends and configure the Jet proxy via `CUE_ARTIFACT_STUDIO_API_BASE_URL`.
- Use Jet as the Cue frontend substrate. Vite is only a minimal repro path when isolating a Jet blocker.
- Use Mamba as the Cue backend target. CPython bridge mode is explicit fallback only, and any bridge-backed product slice needs a linked `project:mamba` or `crate:mamba` issue before it lands.
- Before closing substantial Cue work, update the relevant docs, TD files, and issues so the architecture decision is not left only in chat.
