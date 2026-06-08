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
3. Relevant files under `.aw/tech-design/projects/cue/`
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
projects/cue/artifact-studio/  Vite + React + TypeScript owner workspace
projects/cue/backend/          temporary Python backend bridge/API contracts
projects/cue/shared/           shared domain/API client/status mapping only
projects/cue/schemas/          artifact and runtime contracts
projects/cue/fe/               legacy transitional scaffold
projects/cue/be/               legacy transitional scaffold
projects/cue/docs/legacy/      historical terminal/TUI notes only
```

Prefer `artifact-studio` and `backend` for new work. Do not revive the old TUI
or make users reason about worktrees, GitLab, CI, branches, or deploy plumbing.

## Local run

Artifact Studio defaults to a fixture API at `http://127.0.0.1:43219` and a
Vite dev server at `http://127.0.0.1:3212`.

```bash
cd projects/cue/artifact-studio
npm run dev:fixture-api
npm run dev
```

Run those as separate long-running processes. If port `43219` or `3212` is in
use, either stop the existing process deliberately or set the matching
environment variable instead of guessing:

```bash
CUE_ARTIFACT_STUDIO_FIXTURE_API_PORT=43220 npm run dev:fixture-api
CUE_ARTIFACT_STUDIO_API_BASE_URL=http://127.0.0.1:43220 npm run dev
```

The backend package also has a dev script, but treat the contract tests as the
reliable validation surface unless the current backend server entrypoint has
been verified in this checkout.

## Validation

Run the narrow checks for the area touched:

```bash
cd projects/cue/artifact-studio
npm run typecheck
npm run build
npm run test:e2e
```

```bash
python -m pytest projects/cue/backend/tests/test_workstream_api.py -q
```

For UI changes, open `http://127.0.0.1:3212` with the browser tool and verify
the first viewport and the affected workflow. Save Playwright screenshots under
`.playwright-mcp/`, not the repo root.

## Implementation rules

- Keep frontend work dense, operational, and artifact/workflow focused; avoid marketing-page composition.
- Preserve the `project list / chat / on-demand right pane` Artifact Studio shape unless the user explicitly changes it.
- Keep chat available for app owners, but make WorkItem/artifact state the durable center of the UI.
- Use same-origin `/api` calls from the frontend and configure Vite proxy via `CUE_ARTIFACT_STUDIO_API_BASE_URL`.
- If Vite works and Jet does not, keep Cue moving on Vite and open/link a `project:jet` or `crate:jet` issue with the exact repro.
- If CPython works and Mamba does not, keep the temporary bridge thin and open/link a `project:mamba` or `crate:mamba` issue before landing broader backend work.
- Before closing substantial Cue work, update the relevant docs, TD files, and issues so the architecture decision is not left only in chat.
