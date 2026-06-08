# Legacy TUI and SDD-Runner Retirement

Status: retired.

Cue's active product path is the browser-based Prompt-to-Governed-Artifact
platform under `projects/cue/artifact-studio`, `projects/cue/admin`,
`projects/cue/backend`, `projects/cue/shared`, and `projects/cue/schemas`.
Terminal TUI, CRRR driver, transport-profile, and SDD app-protocol material is
history only.

## Tracker Status

The legacy tracking issues have already been closed:

- #1243 app protocol over SDD runtime and Jet surfaces
- #1245 session and view model contract
- #1246 app protocol to Jet UI contract
- #1247 transport profiles for terminal desktop and web
- #1248 SDD interface language gaps for Cue specs
- #1226 work-item creation and management interface

## Removal Plan

1. Keep `projects/cue/docs/legacy/` read-only except for retirement notes.
2. Do not add new product features under legacy terminal/TUI docs.
3. Treat `projects/cue/app`, `projects/cue/fe`, and `projects/cue/be` as
   transitional scaffolds only; new owner UI work goes to `artifact-studio`, new
   operator UI work goes to `admin`, and API work goes to `backend`.
4. Delete transitional scaffolds after Artifact Studio, Admin, and backend own
   the corresponding development and build paths end to end.
5. Keep Score/SDD as internal development tooling, not as Cue's end-user
   terminal workflow.
